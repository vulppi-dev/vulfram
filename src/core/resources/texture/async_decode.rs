use std::collections::HashSet;
use std::sync::mpsc::{Receiver, Sender, channel};

use crate::core::image::{ImageBuffer, ImageDecoder};
use crate::core::resources::texture::{ForwardAtlasOptions, TextureCreateMode};

#[derive(Debug, Clone)]
pub struct TextureDecodeJob {
    pub window_id: u32,
    pub texture_id: u32,
    pub label: Option<String>,
    pub srgb: Option<bool>,
    pub mode: TextureCreateMode,
    pub atlas_options: Option<ForwardAtlasOptions>,
    pub bytes: Vec<u8>,
}

#[derive(Debug)]
pub struct TextureDecodeResult {
    pub window_id: u32,
    pub texture_id: u32,
    pub label: Option<String>,
    pub srgb: Option<bool>,
    pub mode: TextureCreateMode,
    pub atlas_options: Option<ForwardAtlasOptions>,
    pub image: Option<ImageBuffer>,
    pub message: String,
}

pub struct TextureAsyncManager {
    sender: Sender<TextureDecodeResult>,
    receiver: Receiver<TextureDecodeResult>,
    pending: HashSet<u32>,
    canceled: HashSet<u32>,
}

impl TextureAsyncManager {
    pub fn new() -> Self {
        let (sender, receiver) = channel();
        Self {
            sender,
            receiver,
            pending: HashSet::new(),
            canceled: HashSet::new(),
        }
    }

    pub fn is_pending(&self, texture_id: u32) -> bool {
        self.pending.contains(&texture_id)
    }

    pub fn enqueue(&mut self, job: TextureDecodeJob) -> Result<(), String> {
        if !self.pending.insert(job.texture_id) {
            return Err(format!("Texture {} is already pending", job.texture_id));
        }
        self.canceled.remove(&job.texture_id);
        let sender = self.sender.clone();
        spawn_decode(job, sender);
        Ok(())
    }

    pub fn cancel(&mut self, texture_id: u32) {
        self.pending.remove(&texture_id);
        self.canceled.insert(texture_id);
    }

    pub fn was_canceled(&mut self, texture_id: u32) -> bool {
        self.canceled.remove(&texture_id)
    }

    pub fn drain_results(&mut self) -> Vec<TextureDecodeResult> {
        let mut results = Vec::new();
        while let Ok(result) = self.receiver.try_recv() {
            self.pending.remove(&result.texture_id);
            results.push(result);
        }
        results
    }
}

#[cfg(not(feature = "wasm"))]
fn spawn_decode(job: TextureDecodeJob, sender: Sender<TextureDecodeResult>) {
    std::thread::spawn(move || {
        let image = ImageDecoder::try_decode(&job.bytes);
        let message = if image.is_some() {
            "Texture decoded".to_string()
        } else {
            "Failed to decode image".to_string()
        };
        let _ = sender.send(TextureDecodeResult {
            window_id: job.window_id,
            texture_id: job.texture_id,
            label: job.label,
            srgb: job.srgb,
            mode: job.mode,
            atlas_options: job.atlas_options,
            image,
            message,
        });
    });
}

#[cfg(feature = "wasm")]
fn spawn_decode(job: TextureDecodeJob, sender: Sender<TextureDecodeResult>) {
    wasm_bindgen_futures::spawn_local(async move {
        let image = ImageDecoder::try_decode(&job.bytes);
        let message = if image.is_some() {
            "Texture decoded".to_string()
        } else {
            "Failed to decode image".to_string()
        };
        let _ = sender.send(TextureDecodeResult {
            window_id: job.window_id,
            texture_id: job.texture_id,
            label: job.label,
            srgb: job.srgb,
            mode: job.mode,
            atlas_options: job.atlas_options,
            image,
            message,
        });
    });
}
