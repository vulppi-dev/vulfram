use crate::core::cmd::EngineEvent;
use crate::core::input::events::ElementState;
use crate::core::input::events::{KeyboardEvent, PointerEvent, ScrollDelta};
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

    let keyboard_events: Vec<KeyboardEvent> = engine
        .event_queue
        .iter()
        .filter_map(|event| match event {
            EngineEvent::Keyboard(keyboard) => Some(keyboard.clone()),
            _ => None,
        })
        .collect();

    for keyboard in &keyboard_events {
        route_keyboard_event(engine, keyboard);
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
                    let pixels_per_point = ui_pixels_per_point(engine, *window_id);
                    engine
                        .ui
                        .focus_by_window
                        .insert(*window_id, context_id.clone());
                    engine
                        .ui
                        .capture_by_window
                        .insert(*window_id, context_id.clone());
                    push_pointer_event(
                        engine,
                        &context_id,
                        *position,
                        pixels_per_point,
                        Some(*state),
                        Some(*window_id),
                    );
                }
            } else {
                if let Some(context_id) = engine.ui.capture_by_window.get(window_id).cloned() {
                    let pixels_per_point = ui_pixels_per_point(engine, *window_id);
                    push_pointer_event(
                        engine,
                        &context_id,
                        *position,
                        pixels_per_point,
                        Some(*state),
                        Some(*window_id),
                    );
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
                let pixels_per_point = ui_pixels_per_point(engine, *window_id);
                push_pointer_event(
                    engine,
                    &context_id,
                    *position,
                    pixels_per_point,
                    None,
                    Some(*window_id),
                );
                return;
            }
            if let Some(context_id) = pick_context(engine, *window_id, position.x, position.y) {
                let pixels_per_point = ui_pixels_per_point(engine, *window_id);
                push_pointer_event(
                    engine,
                    &context_id,
                    *position,
                    pixels_per_point,
                    None,
                    Some(*window_id),
                );
            }
        }
        PointerEvent::OnLeave { window_id, .. } => {
            engine.ui.capture_by_window.remove(window_id);
            if let Some(context_id) = engine.ui.focus_by_window.get(window_id).cloned() {
                push_pointer_gone(engine, &context_id);
            }
        }
        PointerEvent::OnScroll {
            window_id, delta, ..
        } => {
            if let Some(context_id) = engine.ui.capture_by_window.get(window_id).cloned() {
                let pixels_per_point = ui_pixels_per_point(engine, *window_id);
                push_scroll_event(engine, &context_id, *delta, pixels_per_point);
                return;
            }
            if let Some(context_id) = engine.ui.focus_by_window.get(window_id).cloned() {
                let pixels_per_point = ui_pixels_per_point(engine, *window_id);
                push_scroll_event(engine, &context_id, *delta, pixels_per_point);
                return;
            }
        }
        _ => {}
    }
}

fn route_keyboard_event(engine: &mut EngineState, event: &KeyboardEvent) {
    match event {
        KeyboardEvent::OnInput {
            window_id,
            key_code,
            state,
            repeat,
            text,
            modifiers,
            ..
        } => {
            let Some(context_id) = engine.ui.focus_by_window.get(window_id).cloned() else {
                return;
            };
            let mods = to_egui_modifiers(*modifiers);
            if let Some(key) = map_egui_key(*key_code) {
                push_key_event(
                    engine,
                    &context_id,
                    key,
                    *state == ElementState::Pressed,
                    *repeat,
                    mods,
                );
            }
            if *state == ElementState::Pressed {
                if let Some(text) = text.clone() {
                    if !text.is_empty() {
                        push_text_event(engine, &context_id, text);
                    }
                }
            }
        }
        KeyboardEvent::OnModifiersChange {
            window_id,
            modifiers,
        } => {
            let _ = (window_id, modifiers);
        }
        KeyboardEvent::OnImeEnable { window_id } => {
            if let Some(context_id) = engine.ui.focus_by_window.get(window_id).cloned() {
                push_ime_event(engine, &context_id, egui::ImeEvent::Enabled);
            }
        }
        KeyboardEvent::OnImePreedit {
            window_id, text, ..
        } => {
            if let Some(context_id) = engine.ui.focus_by_window.get(window_id).cloned() {
                push_ime_event(engine, &context_id, egui::ImeEvent::Preedit(text.clone()));
            }
        }
        KeyboardEvent::OnImeCommit { window_id, text } => {
            if let Some(context_id) = engine.ui.focus_by_window.get(window_id).cloned() {
                push_ime_event(engine, &context_id, egui::ImeEvent::Commit(text.clone()));
            }
        }
        KeyboardEvent::OnImeDisable { window_id } => {
            if let Some(context_id) = engine.ui.focus_by_window.get(window_id).cloned() {
                push_ime_event(engine, &context_id, egui::ImeEvent::Disabled);
            }
        }
    }
}

fn pick_context(engine: &EngineState, window_id: u32, x: f32, y: f32) -> Option<LogicalId> {
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
    pixels_per_point: f32,
    state: Option<ElementState>,
    window_id: Option<u32>,
) {
    let context = match engine.ui.contexts.get(context_id) {
        Some(ctx) => ctx,
        None => return,
    };
    let local_pos = egui::pos2(
        (position.x - context.screen_rect.x) / pixels_per_point,
        (position.y - context.screen_rect.y) / pixels_per_point,
    );
    let events = engine
        .ui
        .pending_events
        .entry(context_id.clone())
        .or_default();

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

fn push_pointer_gone(engine: &mut EngineState, context_id: &LogicalId) {
    let events = engine
        .ui
        .pending_events
        .entry(context_id.clone())
        .or_default();
    events.push(egui::Event::PointerGone);
}

fn push_scroll_event(
    engine: &mut EngineState,
    context_id: &LogicalId,
    delta: ScrollDelta,
    pixels_per_point: f32,
) {
    let events = engine
        .ui
        .pending_events
        .entry(context_id.clone())
        .or_default();
    let (unit, delta) = match delta {
        ScrollDelta::Line(value) => (egui::MouseWheelUnit::Line, egui::vec2(value.x, value.y)),
        ScrollDelta::Pixel(value) => (
            egui::MouseWheelUnit::Point,
            egui::vec2(value.x / pixels_per_point, value.y / pixels_per_point),
        ),
    };
    events.push(egui::Event::MouseWheel {
        unit,
        delta,
        modifiers: egui::Modifiers::default(),
    });
}

fn push_key_event(
    engine: &mut EngineState,
    context_id: &LogicalId,
    key: egui::Key,
    pressed: bool,
    repeat: bool,
    modifiers: egui::Modifiers,
) {
    let events = engine
        .ui
        .pending_events
        .entry(context_id.clone())
        .or_default();
    events.push(egui::Event::Key {
        key,
        physical_key: None,
        pressed,
        repeat,
        modifiers,
    });
}

fn push_text_event(engine: &mut EngineState, context_id: &LogicalId, text: String) {
    let events = engine
        .ui
        .pending_events
        .entry(context_id.clone())
        .or_default();
    events.push(egui::Event::Text(text));
}

fn push_ime_event(engine: &mut EngineState, context_id: &LogicalId, event: egui::ImeEvent) {
    let events = engine
        .ui
        .pending_events
        .entry(context_id.clone())
        .or_default();
    events.push(egui::Event::Ime(event));
}

fn to_egui_modifiers(mods: crate::core::input::events::ModifiersState) -> egui::Modifiers {
    egui::Modifiers {
        alt: mods.alt,
        ctrl: mods.ctrl,
        shift: mods.shift,
        mac_cmd: mods.meta,
        command: mods.meta || mods.ctrl,
    }
}

fn map_egui_key(key_code: u32) -> Option<egui::Key> {
    match key_code {
        19 => Some(egui::Key::A),
        20 => Some(egui::Key::B),
        21 => Some(egui::Key::C),
        22 => Some(egui::Key::D),
        23 => Some(egui::Key::E),
        24 => Some(egui::Key::F),
        25 => Some(egui::Key::G),
        26 => Some(egui::Key::H),
        27 => Some(egui::Key::I),
        28 => Some(egui::Key::J),
        29 => Some(egui::Key::K),
        30 => Some(egui::Key::L),
        31 => Some(egui::Key::M),
        32 => Some(egui::Key::N),
        33 => Some(egui::Key::O),
        34 => Some(egui::Key::P),
        35 => Some(egui::Key::Q),
        36 => Some(egui::Key::R),
        37 => Some(egui::Key::S),
        38 => Some(egui::Key::T),
        39 => Some(egui::Key::U),
        40 => Some(egui::Key::V),
        41 => Some(egui::Key::W),
        42 => Some(egui::Key::X),
        43 => Some(egui::Key::Y),
        44 => Some(egui::Key::Z),
        52 => Some(egui::Key::Backspace),
        57 => Some(egui::Key::Enter),
        62 => Some(egui::Key::Space),
        63 => Some(egui::Key::Tab),
        64 => Some(egui::Key::Delete),
        65 => Some(egui::Key::End),
        67 => Some(egui::Key::Home),
        69 => Some(egui::Key::PageDown),
        70 => Some(egui::Key::PageUp),
        71 => Some(egui::Key::ArrowDown),
        72 => Some(egui::Key::ArrowLeft),
        73 => Some(egui::Key::ArrowRight),
        74 => Some(egui::Key::ArrowUp),
        106 => Some(egui::Key::Escape),
        _ => None,
    }
}

fn ui_pixels_per_point(engine: &EngineState, window_id: u32) -> f32 {
    #[cfg(not(feature = "wasm"))]
    {
        engine
            .window
            .states
            .get(&window_id)
            .map(|state| state.scale_factor)
            .unwrap_or(1.0)
            .max(0.1)
    }
    #[cfg(feature = "wasm")]
    {
        let _ = (engine, window_id);
        1.0
    }
}
