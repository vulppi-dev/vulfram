/// Decoded image buffer in RGBA8 format
#[derive(Debug, Clone)]
pub struct ImageBuffer {
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>, // Always RGBA8
}

/// Image decoder that automatically detects format and decodes
pub struct ImageDecoder;

impl ImageDecoder {
    /// Try to decode image data. Returns Some(ImageBuffer) if successful, None otherwise.
    pub fn try_decode(data: &[u8]) -> Option<ImageBuffer> {
        // All formats are now handled by the image crate
        Self::decode_with_image_crate(data)
    }

    /// Decode PNG, JPEG, WebP, and AVIF using the image crate
    fn decode_with_image_crate(data: &[u8]) -> Option<ImageBuffer> {
        let img = image::load_from_memory(data).ok()?;
        let rgba = img.to_rgba8();
        Some(ImageBuffer {
            width: rgba.width(),
            height: rgba.height(),
            data: rgba.into_raw(),
        })
    }
}
