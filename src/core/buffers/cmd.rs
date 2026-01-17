use serde::{Deserialize, Serialize};

use crate::core::state::EngineState;

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdUploadBufferDiscardAllArgs {}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultUploadBufferDiscardAll {
    pub success: bool,
    pub discarded_count: u32,
    pub message: String,
}

pub fn engine_cmd_upload_buffer_discard_all(
    engine: &mut EngineState,
    _args: &CmdUploadBufferDiscardAllArgs,
) -> CmdResultUploadBufferDiscardAll {
    let discarded_count = engine.buffers.uploads.len() as u32;
    engine.buffers.uploads.clear();

    CmdResultUploadBufferDiscardAll {
        success: true,
        discarded_count,
        message: format!("Discarded {} upload(s)", discarded_count),
    }
}
