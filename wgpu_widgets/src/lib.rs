pub mod texture_to_egui;
pub mod widget;
pub mod blit;

#[macro_export]
macro_rules! wgpu_label {
    () => {
        concat!(file!(), ":", line!(), ":", column!())
    };
}
