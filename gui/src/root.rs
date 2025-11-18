/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
// Add #[serde(skip)] to fields to opt-out of serialization of a field
pub struct RootState {
    #[serde(skip)]
    state: Box<dyn AppState>,

    // pixels per point i.e. zoom level
    ppp: f32,
}

pub trait AppState {
    fn update(
        &mut self,
        ctx: &egui::Context,
        frame: &mut eframe::Frame,
    ) -> Option<Box<dyn AppState>>;
}

impl Default for RootState {
    fn default() -> Self {
        Self {
            state: Box::new(crate::menu::State::default()),
            ppp: 2.5,
        }
    }
}

impl RootState {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        setup_custom_fonts(&cc.egui_ctx);

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        if let Some(storage) = cc.storage {
            eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default()
        } else {
            Default::default()
        }
    }
}

fn setup_custom_fonts(_ctx: &egui::Context) {}

impl eframe::App for RootState {
    /// Called by the framework to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        // Allow changing the zoom with ctrl + scroll
        ctx.set_pixels_per_point(self.ppp);

        ctx.input(|input| {
            let scroll_y = input.raw_scroll_delta.y;
            if input.modifiers.ctrl && scroll_y != 0.0 {
                let step = 1.003f32;
                let mut new_scale = self.ppp * step.powf(scroll_y);
                new_scale = new_scale.clamp(0.2, 12.0);
                self.ppp = new_scale;
            }
        });

        // Global Settings
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                egui::widgets::global_theme_preference_buttons(ui);
            });
        });

        if let Some(new_state) = self.state.update(ctx, frame) {
            self.state = new_state;
            ctx.request_discard("Changed State");
        }
    }
}
