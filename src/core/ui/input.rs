use crate::core::cmd::EngineEvent;
use crate::core::input::events::PointerEvent;
use crate::core::input::events::ElementState;
use crate::core::render::graph::LogicalId;
use crate::core::state::EngineState;

use super::state::UiContextRecord;

pub fn process_ui_input_routing(engine: &mut EngineState) {
    let pointer_events: Vec<PointerEvent> = engine
        .event_queue
        .iter()
        .filter_map(|event| match event {
            EngineEvent::Pointer(pointer) => Some(pointer.clone()),
            _ => None,
        })
        .collect();

    for pointer in &pointer_events {
        route_pointer_event(engine, pointer);
    }
}

fn route_pointer_event(engine: &mut EngineState, event: &PointerEvent) {
    match event {
        PointerEvent::OnButton {
            window_id,
            state,
            position,
            ..
        } => {
            if *state == ElementState::Pressed {
                if let Some(context_id) = pick_context(engine, *window_id, position.x, position.y) {
                    engine
                        .ui
                        .focus_by_window
                        .insert(*window_id, context_id.clone());
                    engine
                        .ui
                        .capture_by_window
                        .insert(*window_id, context_id.clone());
                    push_pointer_event(engine, &context_id, *position, Some(*state), Some(*window_id));
                }
            } else {
                if let Some(context_id) = engine.ui.capture_by_window.get(window_id).cloned() {
                    push_pointer_event(engine, &context_id, *position, Some(*state), Some(*window_id));
                }
                engine.ui.capture_by_window.remove(window_id);
            }
        }
        PointerEvent::OnMove {
            window_id,
            position,
            ..
        } => {
            if let Some(context_id) = engine.ui.capture_by_window.get(window_id).cloned() {
                push_pointer_event(engine, &context_id, *position, None, Some(*window_id));
                return;
            }
            if let Some(context_id) = pick_context(engine, *window_id, position.x, position.y) {
                push_pointer_event(engine, &context_id, *position, None, Some(*window_id));
            }
        }
        PointerEvent::OnLeave { window_id, .. } => {
            engine.ui.capture_by_window.remove(window_id);
        }
        _ => {}
    }
}

fn pick_context(
    engine: &EngineState,
    window_id: u32,
    x: f32,
    y: f32,
) -> Option<LogicalId> {
    let mut picked: Option<(&UiContextRecord, LogicalId)> = None;
    for (context_id, context) in &engine.ui.contexts {
        if context.window_id != window_id {
            continue;
        }
        if !contains_point(&context.screen_rect, x, y) {
            continue;
        }
        match &picked {
            None => {
                picked = Some((context, context_id.clone()));
            }
            Some((current, _)) => {
                if context.z_index > current.z_index {
                    picked = Some((context, context_id.clone()));
                }
            }
        }
    }
    picked.map(|(_, id)| id)
}

fn contains_point(rect: &super::types::UiRectPx, x: f32, y: f32) -> bool {
    x >= rect.x && y >= rect.y && x <= rect.x + rect.w && y <= rect.y + rect.h
}

fn push_pointer_event(
    engine: &mut EngineState,
    context_id: &LogicalId,
    position: glam::Vec2,
    state: Option<ElementState>,
    window_id: Option<u32>,
) {
    let context = match engine.ui.contexts.get(context_id) {
        Some(ctx) => ctx,
        None => return,
    };
    let local_pos = egui::pos2(position.x - context.screen_rect.x, position.y - context.screen_rect.y);
    let events = engine.ui.pending_events.entry(context_id.clone()).or_default();

    if let Some(button_state) = state {
        let pressed = button_state == ElementState::Pressed;
        events.push(egui::Event::PointerButton {
            pos: local_pos,
            button: egui::PointerButton::Primary,
            pressed,
            modifiers: egui::Modifiers::default(),
        });
    } else {
        events.push(egui::Event::PointerMoved(local_pos));
    }

    if let Some(window_id) = window_id {
        engine
            .ui
            .focus_by_window
            .entry(window_id)
            .or_insert_with(|| context_id.clone());
    }
}
