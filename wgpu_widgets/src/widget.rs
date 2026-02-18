use egui::{Pos2, Rect};
use egui_wgpu::wgpu;
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

    pub fn full() -> Self {
        Self {
            min_x: 0.0,
            min_y: 0.0,
            max_x: 1.0,
            max_y: 1.0,
        }
    }
}

pub trait WgpuEguiRenderPipeline: Send + Sync + 'static {
    fn prepare(&self, device: &wgpu::Device, queue: &wgpu::Queue, visible_part: &VisiblePart);
    fn paint(&self, wgpu_render_pass: &mut wgpu::RenderPass<'_>);
}

struct WgpuRenderCallback<P: WgpuEguiRenderPipeline> {
    visible_part: VisiblePart,
    pipeline: Arc<Mutex<P>>,
}

impl<P: WgpuEguiRenderPipeline> egui_wgpu::CallbackTrait for WgpuRenderCallback<P> {
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
        wgpu_render_pass: &mut wgpu::RenderPass<'static>,
        _resources: &egui_wgpu::CallbackResources,
    ) {
        self.pipeline.lock().unwrap().paint(wgpu_render_pass);
    }
}

// A widget for rendering to part of the UI using WGPU
pub struct WgpuWidget<P: WgpuEguiRenderPipeline> {
    egui_ctx: egui::Context,
    pipeline: Option<Arc<Mutex<P>>>,
    rect: egui::Rect,
    ppp: f32,
    pixels_size: (u32, u32),
    changed: bool,
}

impl<P: WgpuEguiRenderPipeline> WgpuWidget<P> {
    /// Construct the widget once as part of the app state.
    pub fn new(ctx: &egui::Context) -> Self {
        Self {
            egui_ctx: ctx.clone(),
            rect: Rect {
                min: Pos2 { x: 0.0, y: 0.0 },
                max: Pos2 { x: 0.0, y: 0.0 },
            },
            ppp: 0.0,
            pixels_size: (1, 1),
            pipeline: None,
            changed: true,
        }
    }

    /// Call before `.add(..)` and
    ///
    /// Call before preparing the pipeline for this pass via a call to `.pipeline()` if it needs to know how big it will be e.g. for internal textures.
    pub fn set_rect(&mut self, rect: Rect) {
        let ppp = self.egui_ctx.pixels_per_point();
        if self.rect != rect || self.ppp != ppp {
            self.changed = true;
        }
        self.pixels_size = ((rect.width() * ppp) as u32, (rect.height() * ppp) as u32);
        self.rect = rect;
        self.ppp = ppp;
    }

    /// Has something changed since last time such that a pipeline reconstruction is required?
    pub fn set_changed(&mut self) {
        self.changed = true
    }

    /// Has something changed since last time such that a pipeline reconstruction is required?
    pub fn changed(&self) -> bool {
        self.changed
    }

    /// Get the size allocated to this widget in pixels
    pub fn pixels_size(&self) -> (u32, u32) {
        self.pixels_size
    }

    /// Update the pipeline and reset the changed flag
    pub fn set_pipeline(&mut self, pipeline: P) {
        self.changed = false;
        self.pipeline = Some(Arc::new(Mutex::new(pipeline)))
    }

    /// Access the pipeline to set it up for this render pass. e.g. by setting uniform variables or setting up intermediate textures.
    pub fn pipeline(&self) -> Option<std::sync::MutexGuard<'_, P>> {
        Some(self.pipeline.as_ref()?.lock().unwrap())
    }

    /// Add us to the egui UI.
    pub fn add(&self, ui: &egui::Ui) {
        if let Some(pipeline) = &self.pipeline {
            ui.painter().add(egui_wgpu::Callback::new_paint_callback(
                self.rect,
                WgpuRenderCallback {
                    visible_part: VisiblePart::new(self.rect, self.egui_ctx.viewport_rect()),
                    pipeline: pipeline.clone(),
                },
            ));
        }
    }
}
