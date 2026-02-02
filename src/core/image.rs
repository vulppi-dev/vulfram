use image::imageops::FilterType;
use image::{GenericImageView, ImageFormat};
use half::f16;

/// Decoded image buffer in RGBA format
#[derive(Debug, Clone)]
pub struct ImageBuffer {
    pub width: u32,
    pub height: u32,
    pub pixels: ImagePixels,
}

#[derive(Debug, Clone)]
pub enum ImagePixels {
    Rgba8(Vec<u8>),
    Rgba16F(Vec<u16>),
}

/// Image decoder that automatically detects format and decodes
pub struct ImageDecoder;

impl ImageDecoder {
    /// Try to decode image data. Returns Some(ImageBuffer) if successful, None otherwise.
    pub fn try_decode(data: &[u8]) -> Option<ImageBuffer> {
        let format = image::guess_format(data).ok();
        let start = std::time::Instant::now();
        let result = Self::decode_with_image_crate(data, format);
        if let Some(fmt) = format {
            println!(
                "ImageDecoder: format={:?} bytes={} decoded={} in {:?}",
                fmt,
                data.len(),
                result.is_some(),
                start.elapsed()
            );
        } else {
            println!(
                "ImageDecoder: format=unknown bytes={} decoded={} in {:?}",
                data.len(),
                result.is_some(),
                start.elapsed()
            );
        }
        result
    }

    /// Decode PNG, JPEG, WebP, and AVIF using the image crate
    fn decode_with_image_crate(data: &[u8], format: Option<ImageFormat>) -> Option<ImageBuffer> {
        let mut img = image::load_from_memory(data).ok()?;
        let max_dim: u32 = 2048;

        if matches!(format, Some(ImageFormat::OpenExr | ImageFormat::Hdr)) {
            let decode_start = std::time::Instant::now();
            let (width, height) = img.dimensions();
            let largest = width.max(height);
            if largest > max_dim {
                let scale = max_dim as f32 / largest as f32;
                let new_width = (width as f32 * scale).round().max(1.0) as u32;
                let new_height = (height as f32 * scale).round().max(1.0) as u32;
                img = img.resize(new_width, new_height, FilterType::Lanczos3);
            }

            let rgba = img.to_rgba32f();
            let mut out = Vec::with_capacity(rgba.len());
            for value in rgba.into_raw() {
                out.push(f16::from_f32(value).to_bits());
            }
            println!(
                "ImageDecoder: EXR/HDR size={}x{} downscale={} elapsed={:?}",
                img.width(),
                img.height(),
                largest > max_dim,
                decode_start.elapsed()
            );
            return Some(ImageBuffer {
                width: img.width(),
                height: img.height(),
                pixels: ImagePixels::Rgba16F(out),
            });
        }

        let rgba = img.to_rgba8();
        Some(ImageBuffer {
            width: rgba.width(),
            height: rgba.height(),
            pixels: ImagePixels::Rgba8(rgba.into_raw()),
        })
    }
}
