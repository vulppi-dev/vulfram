use super::ShadowConfig;
use crate::core::state::EngineState;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CmdShadowConfigureArgs {
    pub window_id: u32,
    pub config: ShadowConfig,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultShadowConfigure {
    pub success: bool,
    pub message: String,
}

pub fn engine_cmd_shadow_configure(
    engine: &mut EngineState,
    args: &CmdShadowConfigureArgs,
) -> CmdResultShadowConfigure {
    let window_state = match engine.window.states.get_mut(&args.window_id) {
        Some(ws) => ws,
        None => {
            return CmdResultShadowConfigure {
                success: false,
                message: format!("Window {} not found", args.window_id),
            };
        }
    };

    let device = match engine.device.as_ref() {
        Some(d) => d,
        None => {
            return CmdResultShadowConfigure {
                success: false,
                message: "GPU Device not initialized".into(),
            };
        }
    };

    if let Some(shadow) = window_state.render_state.shadow.as_mut() {
        shadow.configure(device, args.config);
        window_state.is_dirty = true;
        CmdResultShadowConfigure {
            success: true,
            message: "Shadow configuration updated successfully".into(),
        }
    } else {
        CmdResultShadowConfigure {
            success: false,
            message: format!(
                "Shadow manager not initialized for window {}",
                args.window_id
            ),
        }
    }
}
