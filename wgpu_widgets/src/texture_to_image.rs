use eframe::wgpu::{self};

pub fn texture_to_buffer_view(
    ctx: &egui_wgpu::RenderState,
    texture: &wgpu::Texture,
) -> (u32, u32, wgpu::Buffer) {
    let device = &ctx.device;
    let queue = &ctx.queue;

    let bytes_per_pixel = 4u32;
    let padded_bytes_per_row = wgpu::util::align_to(
        texture.width() * bytes_per_pixel,
        wgpu::COPY_BYTES_PER_ROW_ALIGNMENT,
    );

    let output_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("click_readback_buffer"),
        size: (padded_bytes_per_row * texture.height()) as u64,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });

    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("click_readback_encoder"),
    });

    encoder.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo {
            texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::TexelCopyBufferInfo {
            buffer: &output_buffer,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(padded_bytes_per_row),
                rows_per_image: None,
            },
        },
        texture.size(),
    );

    let submission_index = queue.submit(Some(encoder.finish()));

    device
        .poll(wgpu::PollType::Wait {
            submission_index: Some(submission_index),
            timeout: None,
        })
        .unwrap();

    (bytes_per_pixel, padded_bytes_per_row, output_buffer)
}

// pub fn texture_to_image(
//     ctx: &egui_wgpu::RenderState,
//     texture: &wgpu::Texture,
// ) -> ImageBuffer<image::Rgba<u8>, Vec<u8>> {
//     let (bytes_per_pixel, padded_bytes_per_row, output_buffer) =
//         texture_to_buffer_view(ctx, texture);

//     let buffer_slice = output_buffer.slice(..);
//     buffer_slice.map_async(wgpu::MapMode::Read, |result| {
//         result.unwrap();
//     });
//     ctx.device
//         .poll(wgpu::PollType::Wait {
//             submission_index: None,
//             timeout: None,
//         })
//         .unwrap();
//     let data = buffer_slice.get_mapped_range();

//     let mut pixels = vec![0u8; (texture.width() * texture.height() * bytes_per_pixel) as usize];
//     for row in 0..texture.height() as usize {
//         let src_offset = row * padded_bytes_per_row as usize;
//         let dst_offset = row * (texture.width() * bytes_per_pixel) as usize;
//         let row_bytes = (texture.width() * bytes_per_pixel) as usize;
//         pixels[dst_offset..dst_offset + row_bytes]
//             .copy_from_slice(&data[src_offset..src_offset + row_bytes]);
//     }

//     drop(data);
//     output_buffer.unmap();

//     ImageBuffer::<image::Rgba<u8>, _>::from_raw(texture.width(), texture.height(), pixels)
//         .expect("Failed to create image buffer")
// }

pub async fn texture_to_pixel(
    ctx: &egui_wgpu::RenderState,
    texture: &wgpu::Texture,
    pixel: (u32, u32),
) -> (u8, u8, u8, u8) {
    let width = texture.width();
    let height = texture.height();

    let (bytes_per_pixel, padded_bytes_per_row, output_buffer) =
        texture_to_buffer_view(ctx, texture);

    let (tx, rx) = futures::channel::oneshot::channel();

    output_buffer.map_async(wgpu::MapMode::Read, .., move |result| {
        tx.send(result).ok();
    });

    ctx.device.poll(wgpu::PollType::Poll).unwrap();

    rx.await.unwrap().unwrap();

    let data = output_buffer.get_mapped_range(..);

    let (x, y) = pixel;
    assert!(x < width && y < height, "Pixel out of bounds");

    let row_start = y as usize * padded_bytes_per_row as usize;
    let pixel_start = row_start + (x as usize * bytes_per_pixel as usize);

    let r = data[pixel_start];
    let g = data[pixel_start + 1];
    let b = data[pixel_start + 2];
    let a = if bytes_per_pixel == 4 {
        data[pixel_start + 3]
    } else {
        255
    };

    drop(data);
    output_buffer.unmap();

    (r, g, b, a)
}
