use serde::{Deserialize, Serialize};

use crate::core::resources::EnvironmentConfig;
use crate::core::state::EngineState;

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CmdEnvironmentCreateArgs {
    pub window_id: u32,
    pub config: EnvironmentConfig,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CmdEnvironmentUpdateArgs {
    pub window_id: u32,
    pub config: EnvironmentConfig,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdEnvironmentDisposeArgs {
    pub window_id: u32,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultEnvironment {
    pub success: bool,
    pub message: String,
}

pub fn engine_cmd_environment_create(
    engine: &mut EngineState,
    args: &CmdEnvironmentCreateArgs,
) -> CmdResultEnvironment {
    let window_state = match engine.window.states.get_mut(&args.window_id) {
        Some(state) => state,
        None => {
            return CmdResultEnvironment {
                success: false,
                message: format!("Window {} not found", args.window_id),
            };
        }
    };

    if window_state.render_state.environment_is_configured {
        return CmdResultEnvironment {
            success: false,
            message: "Environment already configured for this window".into(),
        };
    }

    window_state.render_state.environment = args.config.clone();
    window_state.render_state.environment_is_configured = true;

    CmdResultEnvironment {
        success: true,
        message: "Environment created".into(),
    }
}

pub fn engine_cmd_environment_update(
    engine: &mut EngineState,
    args: &CmdEnvironmentUpdateArgs,
) -> CmdResultEnvironment {
    let window_state = match engine.window.states.get_mut(&args.window_id) {
        Some(state) => state,
        None => {
            return CmdResultEnvironment {
                success: false,
                message: format!("Window {} not found", args.window_id),
            };
        }
    };

    window_state.render_state.environment = args.config.clone();
    window_state.render_state.environment_is_configured = true;

    CmdResultEnvironment {
        success: true,
        message: "Environment updated".into(),
    }
}

pub fn engine_cmd_environment_dispose(
    engine: &mut EngineState,
    args: &CmdEnvironmentDisposeArgs,
) -> CmdResultEnvironment {
    let window_state = match engine.window.states.get_mut(&args.window_id) {
        Some(state) => state,
        None => {
            return CmdResultEnvironment {
                success: false,
                message: format!("Window {} not found", args.window_id),
            };
        }
    };

    window_state.render_state.environment = EnvironmentConfig::default();
    window_state.render_state.environment_is_configured = false;

    CmdResultEnvironment {
        success: true,
        message: "Environment disposed".into(),
    }
}
