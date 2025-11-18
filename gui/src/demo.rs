use eframe::egui;
use std::borrow::Cow;
use eframe::wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex { pos: [f32; 2], color: [f32; 3] }

struct TriangleRenderer {
    vertex_buffer: Option<eframe::wgpu::Buffer>,
    pipeline: Option<eframe::wgpu::RenderPipeline>,
}

impl TriangleRenderer {
    fn new() -> Self {
        Self { vertex_buffer: None, pipeline: None }
    }

    fn init(&mut self, device: &eframe::wgpu::Device, config: eframe::wgpu::TextureFormat) {
        let verts = [
            Vertex { pos: [ 0.0,  0.7], color: [1.0, 0.0, 0.0] },
            Vertex { pos: [-0.7, -0.7], color: [0.0, 1.0, 0.0] },
            Vertex { pos: [ 0.7, -0.7], color: [0.0, 0.0, 1.0] },
        ];

        self.vertex_buffer = Some(device.create_buffer_init(&eframe::wgpu::util::BufferInitDescriptor {
            label: Some("triangle_vertex_buffer"),
            contents: bytemuck::bytes_of(&verts),
            usage: eframe::wgpu::BufferUsages::VERTEX,
        }));
    }
}

struct App { renderer: TriangleRenderer }

impl Default for App {
    fn default() -> Self { Self { renderer: TriangleRenderer::new() } }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("WGPU Triangle via PaintCallback");

            let (rect, _response) = ui.allocate_exact_size(ui.available_size(), egui::Sense::hover());

            let paint_callback = egui::PaintCallback {
                rect,
                callback: std::sync::Arc::new(|info: &egui::PaintCallbackInfo, render_pass: &mut eframe::Renderer| {
                    // Here you would use info.render_device and render_pass to draw the triangle
                    // For full WGPU integration, initialize pipeline and buffers using info.render_device
                    // and issue draw calls using render_pass.
                }),
            };

            ui.painter().add(paint_callback);
        });
    }
}