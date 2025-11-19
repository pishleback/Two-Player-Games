use eframe::egui_wgpu::wgpu;
use pollster::FutureExt;

use crate::demo::texture;

struct HeadlessContext {
    instance: wgpu::Instance,
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
}

impl HeadlessContext {
    pub async fn new() -> Self {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: None,
                force_fallback_adapter: false,
            })
            .await
            .unwrap();
        let (device, queue) = adapter.request_device(&Default::default()).await.unwrap();

        Self {
            instance,
            adapter,
            device,
            queue,
        }
    }
}

pub fn render_to_texture(
    ctx: &egui_wgpu::RenderState,
    texture_view: &wgpu::TextureView,
    render: impl FnOnce(&mut wgpu::RenderPass),
) {
    let mut encoder = ctx
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

    let render_pass_desc = wgpu::RenderPassDescriptor {
        label: Some("Render Pass"),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view: &texture_view,
            resolve_target: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Clear(wgpu::Color {
                    r: 0.5,
                    g: 0.5,
                    b: 0.5,
                    a: 1.0,
                }),
                store: wgpu::StoreOp::Store,
            },
            depth_slice: None,
        })],
        depth_stencil_attachment: None,
        occlusion_query_set: None,
        timestamp_writes: None,
    };

    {
        let mut render_pass = encoder.begin_render_pass(&render_pass_desc);

        render(&mut render_pass);
    }

    if false {
        let cube_renderer = super::cube::CubeRenderer::new(ctx, texture_view.texture().format());
        let mut render_pass = encoder.begin_render_pass(&render_pass_desc);

        cube_renderer.prepare(&ctx.device, &ctx.queue, glam::Quat::IDENTITY);
        cube_renderer.paint(&mut render_pass);
    }

    if false {
        let mut render_pass = encoder.begin_render_pass(&render_pass_desc);

        let shader = ctx
            .device
            .create_shader_module(wgpu::include_wgsl!("headless_shader.wgsl"));

        let render_pipeline_layout =
            ctx.device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Render Pipeline Layout"),
                    bind_group_layouts: &[],
                    push_constant_ranges: &[],
                });

        let render_pipeline = ctx
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Render Pipeline"),
                layout: Some(&render_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: None,
                    buffers: &[],
                    compilation_options: Default::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: None,
                    targets: &[Some(wgpu::ColorTargetState {
                        format: texture_view.texture().format(),
                        blend: Some(wgpu::BlendState {
                            alpha: wgpu::BlendComponent::REPLACE,
                            color: wgpu::BlendComponent::REPLACE,
                        }),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                    compilation_options: Default::default(),
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: Some(wgpu::Face::Back),
                    // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                    polygon_mode: wgpu::PolygonMode::Fill,
                    // Requires Features::DEPTH_CLIP_CONTROL
                    unclipped_depth: false,
                    // Requires Features::CONSERVATIVE_RASTERIZATION
                    conservative: false,
                },
                depth_stencil: None,
                multisample: wgpu::MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                // If the pipeline will be used with a multiview render pass, this
                // indicates how many array layers the attachments will have.
                multiview: None,
                cache: None,
            });

        render_pass.set_pipeline(&render_pipeline);
        render_pass.draw(0..3, 0..1);
    }

    ctx.queue.submit(Some(encoder.finish()));
}

pub async fn save_texture(ctx: &egui_wgpu::RenderState, texture: &wgpu::Texture) {
    let u32_size = std::mem::size_of::<u32>() as u32;

    let texture_size = texture.size();
    assert_eq!(texture_size.depth_or_array_layers, 1);
    assert_eq!(texture.format(), wgpu::TextureFormat::Rgba8UnormSrgb);

    let output_buffer_size =
        (u32_size * texture_size.width * texture_size.height) as wgpu::BufferAddress;
    let output_buffer_desc = wgpu::BufferDescriptor {
        size: output_buffer_size,
        usage: wgpu::BufferUsages::COPY_DST
            // this tells wpgu that we want to read this buffer from the cpu
            | wgpu::BufferUsages::MAP_READ,
        label: None,
        mapped_at_creation: false,
    };
    let output_buffer = ctx.device.create_buffer(&output_buffer_desc);

    let mut encoder = ctx
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

    encoder.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo {
            aspect: wgpu::TextureAspect::All,
            texture: &texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
        },
        wgpu::TexelCopyBufferInfo {
            buffer: &output_buffer,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(u32_size * texture_size.width),
                rows_per_image: Some(texture_size.height),
            },
        },
        texture_size,
    );

    ctx.queue.submit(Some(encoder.finish()));

    // We need to scope the mapping variables so that we can
    // unmap the buffer
    {
        let buffer_slice = output_buffer.slice(..);

        // NOTE: We have to create the mapping THEN device.poll() before await
        // the future. Otherwise the application will freeze.
        let (tx, rx) = futures_intrusive::channel::shared::oneshot_channel();
        buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
            tx.send(result).unwrap();
        });
        ctx.device
            .poll(wgpu::PollType::wait_indefinitely())
            .unwrap();
        rx.receive().await.unwrap().unwrap();

        let data = buffer_slice.get_mapped_range();

        use image::{ImageBuffer, Rgba};
        let buffer =
            ImageBuffer::<Rgba<u8>, _>::from_raw(texture_size.width, texture_size.height, data)
                .unwrap();
        buffer.save("image.png").unwrap();
    }
    output_buffer.unmap();
}
