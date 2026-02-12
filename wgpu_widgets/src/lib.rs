use eframe::wgpu::{self};
use egui::{Pos2, Rect};
use std::sync::{Arc, Mutex};

#[derive(Debug)]
pub struct VisiblePart {
    pub min_x: f32,
    pub min_y: f32,
    pub max_x: f32,
    pub max_y: f32,
}

impl VisiblePart {
    fn new(rect: Rect, viewport: Rect) -> Self {
        let intersection_rect = rect.intersect(viewport);
        fn frac(range: (f32, f32), value: f32) -> f32 {
            (value - range.0) / (range.1 - range.0)
        }
        Self {
            min_x: frac((rect.min.x, rect.max.x), intersection_rect.min.x),
            max_x: frac((rect.min.x, rect.max.x), intersection_rect.max.x),
            min_y: frac((rect.min.y, rect.max.y), intersection_rect.min.y),
            max_y: frac((rect.min.y, rect.max.y), intersection_rect.max.y),
        }
    }
}

pub trait WgpuRenderPipeline: Send + Sync + 'static {
    fn new(ctx: &egui::Context, wgpu_ctx: &egui_wgpu::RenderState) -> Self;

    fn set_rect(&mut self, rect: Rect);

    fn prepare(&self, _device: &wgpu::Device, queue: &wgpu::Queue, visible_part: &VisiblePart);

    fn paint(&self, render_pass: &mut wgpu::RenderPass<'_>);
}

pub struct WgpuRenderCallback<P: WgpuRenderPipeline> {
    visible_part: VisiblePart,
    pipeline: Arc<Mutex<P>>,
}

impl<P: WgpuRenderPipeline> egui_wgpu::CallbackTrait for WgpuRenderCallback<P> {
    fn prepare(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        _screen_descriptor: &egui_wgpu::ScreenDescriptor,
        _egui_encoder: &mut wgpu::CommandEncoder,
        _resources: &mut egui_wgpu::CallbackResources,
    ) -> Vec<wgpu::CommandBuffer> {
        self.pipeline
            .lock()
            .unwrap()
            .prepare(device, queue, &self.visible_part);
        Vec::new()
    }

    fn paint(
        &self,
        _info: egui::PaintCallbackInfo,
        render_pass: &mut wgpu::RenderPass<'static>,
        _resources: &egui_wgpu::CallbackResources,
    ) {
        self.pipeline.lock().unwrap().paint(render_pass);
    }
}

// A widget for rendering to part of the UI using WGPU
pub struct WgpuWidget<P: WgpuRenderPipeline> {
    egui_ctx: egui::Context,
    rect: egui::Rect,
    pipeline: Arc<Mutex<P>>,
}

impl<P: WgpuRenderPipeline> WgpuWidget<P> {
    /// Construct the widget once as part of the app state.
    pub fn new(ctx: &egui::Context, frame: &eframe::Frame) -> Self {
        let wgpu_ctx: &egui_wgpu::RenderState = frame.wgpu_render_state.as_ref().unwrap();
        Self {
            egui_ctx: ctx.clone(),
            rect: Rect {
                min: Pos2 { x: 0.0, y: 0.0 },
                max: Pos2 { x: 0.0, y: 0.0 },
            },
            pipeline: Arc::new(Mutex::new(P::new(ctx, wgpu_ctx))),
        }
    }

    /// Call before `.add(..)` and
    ///
    /// Call before preparing the pipeline for this pass via a call to `.pipeline()` if it needs to know how big it will be e.g. for internal textures.
    pub fn set_rect(&mut self, rect: Rect) {
        self.rect = rect;
        self.pipeline.lock().unwrap().set_rect(rect);
    }

    /// Access the pipeline to set it up for this render pass. e.g. by setting uniform variables or setting up intermediate textures.
    pub fn pipeline(&self) -> Arc<Mutex<P>> {
        self.pipeline.clone()
    }

    /// Add us to the egui UI.
    pub fn add(&self, ui: &egui::Ui) {
        ui.painter().add(egui_wgpu::Callback::new_paint_callback(
            self.rect,
            WgpuRenderCallback {
                visible_part: VisiblePart::new(self.rect, self.egui_ctx.viewport_rect()),
                pipeline: self.pipeline.clone(),
            },
        ));
    }
}
