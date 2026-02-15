pub mod widget;
pub mod texture_to_egui;

#[macro_export]
macro_rules! wgpu_label {
    () => {
        concat!(file!(), ":", line!(), ":", column!())
    };
}
