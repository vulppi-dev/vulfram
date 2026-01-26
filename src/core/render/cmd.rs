use serde::{Deserialize, Serialize};

use crate::core::render::graph::{RenderGraphApplyResult, RenderGraphDesc};
use crate::core::state::EngineState;

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CmdRenderGraphSetArgs {
    pub window_id: u32,
    pub graph: RenderGraphDesc,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultRenderGraphSet {
    pub success: bool,
    pub fallback_used: bool,
    pub message: String,
}

pub fn engine_cmd_render_graph_set(
    engine: &mut EngineState,
    args: &CmdRenderGraphSetArgs,
) -> CmdResultRenderGraphSet {
    let window_state = match engine.window.states.get_mut(&args.window_id) {
        Some(state) => state,
        None => {
            return CmdResultRenderGraphSet {
                success: false,
                fallback_used: false,
                message: format!("Window {} not found", args.window_id),
            };
        }
    };

    match window_state
        .render_state
        .render_graph
        .apply_graph(args.graph.clone())
    {
        Ok(RenderGraphApplyResult::Applied) => CmdResultRenderGraphSet {
            success: true,
            fallback_used: false,
            message: "Render graph applied".into(),
        },
        Ok(RenderGraphApplyResult::FallbackUsed(err)) => CmdResultRenderGraphSet {
            success: true,
            fallback_used: true,
            message: format!("Render graph invalid, fallback used: {}", err),
        },
        Err(err) => CmdResultRenderGraphSet {
            success: false,
            fallback_used: false,
            message: err,
        },
    }
}
