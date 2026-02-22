use crate::{
    chess_pieces::square::SquareContents,
    icons::{DARK_WOOD, LIGHT_WOOD},
    wormhole::{
        board::{self, MovesLookup, Pos},
        board_render::BoardParams,
    },
};
use eframe::wgpu::{self};
use egui::Color32;
use glam::Quat;
use wgpu_widgets::{
    texture_to_egui, texture_to_pixel,
    widget::{VisiblePart, WgpuEguiRenderPipeline},
};

// Draw the board using depth-peeling for order-independent transparency
#[derive(Clone)]
pub struct Pipeline {
    wgpu_ctx: egui_wgpu::RenderState,
    cube_pipeline_1: super::board_render::Pipeline,
    cube_pipeline_2: super::board_render::Pipeline,
    cube_pipeline_3: super::board_render::Pipeline,
    cube_pipeline_4: super::board_render::Pipeline,
    fill_colour: Color32,
    blit_texture_view: wgpu::TextureView,
    blit_pipeline_1: wgpu_widgets::blit::Pipeline,
    blit_pipeline_2: wgpu_widgets::blit::Pipeline,
    blit_pipeline_3: wgpu_widgets::blit::Pipeline,
    blit_pipeline_4: wgpu_widgets::blit::Pipeline,
    egui_pipeline: texture_to_egui::RenderTexturePipeline,
    click_pipeline: super::board_render::Pipeline,
}

impl Pipeline {
    pub fn new(
        wgpu_ctx: &egui_wgpu::RenderState,
        pixels_size: (u32, u32),
        content: &[SquareContents; 144],
        board_params: &BoardParams,
        icons_texture_array: &wgpu::Texture,
    ) -> Self {
        let cube_pipeline_1 = super::board_render::Pipeline::new(
            &wgpu_ctx,
            pixels_size,
            Color32::TRANSPARENT,
            content,
            &board_params,
            wgpu::TextureFormat::Rgba8UnormSrgb,
            wgpu::TextureFormat::Depth32Float,
            None,
            icons_texture_array,
            Some(wgpu::BlendState::REPLACE),
            true,
        );

        let cube_pipeline_2 = super::board_render::Pipeline::new(
            &wgpu_ctx,
            pixels_size,
            Color32::TRANSPARENT,
            content,
            &board_params,
            wgpu::TextureFormat::Rgba8UnormSrgb,
            wgpu::TextureFormat::Depth32Float,
            Some(cube_pipeline_1.depth_texture_view()),
            icons_texture_array,
            Some(wgpu::BlendState::REPLACE),
            true,
        );

        let cube_pipeline_3 = super::board_render::Pipeline::new(
            &wgpu_ctx,
            pixels_size,
            Color32::TRANSPARENT,
            content,
            &board_params,
            wgpu::TextureFormat::Rgba8UnormSrgb,
            wgpu::TextureFormat::Depth32Float,
            Some(cube_pipeline_2.depth_texture_view()),
            icons_texture_array,
            Some(wgpu::BlendState::REPLACE),
            true,
        );

        let cube_pipeline_4 = super::board_render::Pipeline::new(
            &wgpu_ctx,
            pixels_size,
            Color32::TRANSPARENT,
            content,
            &board_params,
            wgpu::TextureFormat::Rgba8UnormSrgb,
            wgpu::TextureFormat::Depth32Float,
            Some(cube_pipeline_3.depth_texture_view()),
            icons_texture_array,
            Some(wgpu::BlendState::REPLACE),
            true,
        );

        let blit_texture = wgpu_ctx.device.create_texture(&wgpu::TextureDescriptor {
            label: Some(wgpu_widgets::wgpu_label!()),
            size: wgpu::Extent3d {
                width: pixels_size.0,
                height: pixels_size.1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::COPY_SRC
                | wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::TEXTURE_BINDING,

            view_formats: &[],
        });
        let blit_texture_view = blit_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let blit_pipeline_1 = wgpu_widgets::blit::Pipeline::new(
            wgpu_ctx,
            pixels_size,
            cube_pipeline_4.colour_texture_view(),
            blit_texture_view.clone(),
        );

        let blit_pipeline_2 = wgpu_widgets::blit::Pipeline::new(
            wgpu_ctx,
            pixels_size,
            cube_pipeline_3.colour_texture_view(),
            blit_texture_view.clone(),
        );

        let blit_pipeline_3 = wgpu_widgets::blit::Pipeline::new(
            wgpu_ctx,
            pixels_size,
            cube_pipeline_2.colour_texture_view(),
            blit_texture_view.clone(),
        );

        let blit_pipeline_4 = wgpu_widgets::blit::Pipeline::new(
            wgpu_ctx,
            pixels_size,
            cube_pipeline_1.colour_texture_view(),
            blit_texture_view.clone(),
        );

        let click_pipeline = super::board_render::Pipeline::new(
            &wgpu_ctx,
            pixels_size,
            Color32::from_rgb(255, 0, 0),
            &[SquareContents::empty(); 144],
            &board_params,
            wgpu::TextureFormat::Rgba8UnormSrgb,
            wgpu::TextureFormat::Depth32Float,
            None,
            icons_texture_array,
            Some(wgpu::BlendState::REPLACE),
            false,
        );

        let egui_pipeline =
            texture_to_egui::RenderTexturePipeline::new(&wgpu_ctx, blit_texture_view.clone());

        Self {
            wgpu_ctx: wgpu_ctx.clone(),
            cube_pipeline_1,
            cube_pipeline_2,
            cube_pipeline_3,
            cube_pipeline_4,
            fill_colour: Color32::GREEN,
            blit_texture_view,
            blit_pipeline_1,
            blit_pipeline_2,
            blit_pipeline_3,
            blit_pipeline_4,
            egui_pipeline,
            click_pipeline,
        }
    }

    pub async fn pixels_to_square(self, pos_frac: (f32, f32)) -> Option<Pos> {
        // Determine which square was clicked by doing serveral renders.
        // In each render, the background is red and squares are coloured according to whether different bits are set in their binary expansions.
        // Then we can reconstruct the clicked square using the combination of bits.

        let width = self.click_pipeline.colour_texture().width();
        let height = self.click_pipeline.colour_texture().height();

        let pos_pixels = (
            (pos_frac.0 * width as f32) as u32,
            (pos_frac.1 * height as f32) as u32,
        );

        // Return None for inputs outside the texture.
        if !(pos_pixels.0 < width && pos_pixels.1 < height) {
            return None;
        }

        // Create a list of async tasks, one for each bit.
        let mut tasks = Vec::with_capacity(8);
        for b in 0..8 {
            let mut pipeline_clone = self.click_pipeline.clone(); // clone pipeline handle if necessary
            let wgpu_ctx = &self.wgpu_ctx;
            let pos_pixels = pos_pixels;

            tasks.push(async move {
                pipeline_clone.set_colours(std::array::from_fn(|i| {
                    let i = i as u8;
                    let v = if i & (1 << b) != 0 { 1.0 } else { 0.0 };
                    [0.0, v, 0.0, 1.0]
                }));

                pipeline_clone.prepare(&wgpu_ctx.device, &wgpu_ctx.queue);
                pipeline_clone.render();

                let (r, g, _b, _a) =
                    texture_to_pixel(&wgpu_ctx, pipeline_clone.colour_texture(), pos_pixels).await;

                if r > 128 {
                    return None;
                }
                Some(g > 128)
            });
        }

        // Get the results.
        let results = futures::future::join_all(tasks).await;

        // Convert the returned 8 bits into a u8.
        let mut n = 0u8;
        for (b, bit) in results.into_iter().enumerate() {
            if let Some(bit) = bit {
                if bit {
                    n |= 1 << b;
                }
            } else {
                return None;
            }
        }

        if n < 144 {
            Some(Pos::new(n))
        } else {
            #[cfg(debug_assertions)]
            unreachable!();

            #[cfg(not(debug_assertions))]
            None
        }
    }

    pub fn set_selected(&mut self, pos: Option<board::Pos>) {
        let mut colours = std::array::from_fn(|idx| {
            let pos = board::Pos::new(idx as u8);
            let (sym, orb) = pos.full_symmetry_and_orbit();
            let state = match orb {
                board::OrbitFull::P0
                | board::OrbitFull::P2
                | board::OrbitFull::P9
                | board::OrbitFull::P11
                | board::OrbitFull::P44
                | board::OrbitFull::P140 => false,
                board::OrbitFull::P1
                | board::OrbitFull::P3
                | board::OrbitFull::P10
                | board::OrbitFull::P42
                | board::OrbitFull::P142 => true,
            } ^ sym.flip_x
                ^ sym.flip_y
                ^ sym.flip_z;
            if state {
                [
                    DARK_WOOD.r() as f32 / 255.0,
                    DARK_WOOD.g() as f32 / 255.0,
                    DARK_WOOD.b() as f32 / 255.0,
                    0.7,
                ]
            } else {
                [
                    LIGHT_WOOD.r() as f32 / 255.0,
                    LIGHT_WOOD.g() as f32 / 255.0,
                    LIGHT_WOOD.b() as f32 / 255.0,
                    0.7,
                ]
            }
        });
        if let Some(pos) = pos {
            colours[pos.idx()] = [0.0, 0.5, 1.0, 0.8];
            let moves = MovesLookup::new();
            for p in moves
                .cardinal_adjacent(pos)
                .iter()
                .chain(moves.diagonal_adjacent(pos))
            {
                for c in moves.continuations(*p, pos) {
                    if colours[c.idx()] != [1.0, 0.0, 1.0, 0.8] {
                        colours[c.idx()] = [1.0, 0.0, 1.0, 0.8];
                    } else {
                        colours[c.idx()] = [0.0, 1.0, 1.0, 0.8];
                    }
                }
            }
        }
        println!("{:?}", pos);

        self.cube_pipeline_1.set_colours(colours.clone());
        self.cube_pipeline_2.set_colours(colours.clone());
        self.cube_pipeline_3.set_colours(colours.clone());
        self.cube_pipeline_4.set_colours(colours);
    }

    pub fn set_rotation(&mut self, rotation: Quat) {
        self.cube_pipeline_1.set_rotation(rotation);
        self.cube_pipeline_2.set_rotation(rotation);
        self.cube_pipeline_3.set_rotation(rotation);
        self.cube_pipeline_4.set_rotation(rotation);
        self.click_pipeline.set_rotation(rotation);
    }

    pub fn set_fill_colour(&mut self, fill_colour: Color32) {
        self.fill_colour = fill_colour;
    }
}

impl WgpuEguiRenderPipeline for Pipeline {
    fn prepare(&self, device: &wgpu::Device, queue: &wgpu::Queue, visible_part: &VisiblePart) {
        self.cube_pipeline_1.prepare(device, queue);
        self.cube_pipeline_2.prepare(device, queue);
        self.cube_pipeline_3.prepare(device, queue);
        self.cube_pipeline_4.prepare(device, queue);
        self.blit_pipeline_1.prepare(device, queue);
        self.blit_pipeline_2.prepare(device, queue);
        self.egui_pipeline.prepare(device, queue, visible_part);
    }

    fn paint(&self, wgpu_render_pass: &mut wgpu::RenderPass<'_>) {
        self.cube_pipeline_1.render();
        self.cube_pipeline_2.render();
        self.cube_pipeline_3.render();
        self.cube_pipeline_4.render();

        let mut encoder =
            self.wgpu_ctx
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some(wgpu_widgets::wgpu_label!()),
                });
        encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some(wgpu_widgets::wgpu_label!()),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &self.blit_texture_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: (self.fill_colour.r() as f64) / 255.0,
                        g: (self.fill_colour.g() as f64) / 255.0,
                        b: (self.fill_colour.b() as f64) / 255.0,
                        a: 0.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
                depth_slice: None,
            })],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
        });
        self.wgpu_ctx.queue.submit(Some(encoder.finish()));

        self.blit_pipeline_1.render();
        self.blit_pipeline_2.render();
        self.blit_pipeline_3.render();
        self.blit_pipeline_4.render();

        self.egui_pipeline.paint(wgpu_render_pass);
    }
}
