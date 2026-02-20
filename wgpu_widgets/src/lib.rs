pub mod blit;
pub mod texture_to_egui;
mod texture_to_image;
pub mod widget;

// pub use texture_to_image::texture_to_image;
pub use texture_to_image::texture_to_pixel;

#[macro_export]
macro_rules! wgpu_label {
    () => {
        concat!(file!(), ":", line!(), ":", column!())
    };
}
