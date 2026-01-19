pub mod cache;
#[cfg(not(feature = "wasm"))]
pub mod converters;
pub mod events;
pub mod state;

#[cfg(not(feature = "wasm"))]
use crate::core::platform::gilrs::{Event as GilrsEvent, EventType as GilrsEventType};

use crate::core::cmd::EngineEvent;
#[cfg(not(feature = "wasm"))]
use crate::core::gamepad::converters::{convert_gilrs_axis, convert_gilrs_button};
use crate::core::gamepad::events::GamepadEvent;
use crate::core::input::events::ElementState;
use crate::core::state::EngineState;

use self::cache::GamepadCacheManager;

/// Process a gilrs gamepad event and convert it to engine events
#[cfg(not(feature = "wasm"))]
pub fn process_gilrs_event(engine_state: &mut EngineState, event: GilrsEvent) {
    let gamepad_id: u32 = usize::from(event.id) as u32;
    let manager: &mut GamepadCacheManager = &mut engine_state.gamepad.cache;

    match event.event {
        GilrsEventType::Connected => {
            let name: String = if let Some(gilrs) = &engine_state.gamepad.gilrs {
                gilrs.gamepad(event.id).name().into()
            } else {
                "Unknown".into()
            };

            manager.add_gamepad(gamepad_id);

            engine_state
                .event_queue
                .push(EngineEvent::Gamepad(GamepadEvent::OnConnect {
                    gamepad_id,
                    name,
                }));
        }
        GilrsEventType::Disconnected => {
            manager.remove_gamepad(gamepad_id);

            engine_state
                .event_queue
                .push(EngineEvent::Gamepad(GamepadEvent::OnDisconnect {
                    gamepad_id,
                }));
        }
        GilrsEventType::ButtonPressed(button, _code) => {
            let button_mapped = convert_gilrs_button(button);
            let value = 1.0;
            let state = ElementState::Pressed;

            if let Some(cache) = manager.get_mut(gamepad_id) {
                if !cache.button_changed(button_mapped, state, value) {
                    return;
                }
                cache.update_button(button_mapped, state, value);
            }

            engine_state
                .event_queue
                .push(EngineEvent::Gamepad(GamepadEvent::OnButton {
                    gamepad_id,
                    button: button_mapped,
                    state,
                    value,
                }));
        }
        GilrsEventType::ButtonReleased(button, _code) => {
            let button_mapped = convert_gilrs_button(button);
            let value = 0.0;
            let state = ElementState::Released;

            if let Some(cache) = manager.get_mut(gamepad_id) {
                if !cache.button_changed(button_mapped, state, value) {
                    return;
                }
                cache.update_button(button_mapped, state, value);
            }

            engine_state
                .event_queue
                .push(EngineEvent::Gamepad(GamepadEvent::OnButton {
                    gamepad_id,
                    button: button_mapped,
                    state,
                    value,
                }));
        }
        GilrsEventType::ButtonChanged(button, value, _code) => {
            let button_mapped = convert_gilrs_button(button);
            let state = if value > 0.5 {
                ElementState::Pressed
            } else {
                ElementState::Released
            };

            if let Some(cache) = manager.get_mut(gamepad_id) {
                if !cache.button_changed(button_mapped, state, value) {
                    return;
                }
                cache.update_button(button_mapped, state, value);
            }

            engine_state
                .event_queue
                .push(EngineEvent::Gamepad(GamepadEvent::OnButton {
                    gamepad_id,
                    button: button_mapped,
                    state,
                    value,
                }));
        }
        GilrsEventType::AxisChanged(axis, value, _code) => {
            let axis_mapped = convert_gilrs_axis(axis);

            if let Some(cache) = manager.get_mut(gamepad_id) {
                if !cache.axis_changed(axis_mapped, value) {
                    return;
                }

                let adjusted_value = cache.get_axis_value(axis_mapped);
                cache.update_axis(axis_mapped, value);

                engine_state
                    .event_queue
                    .push(EngineEvent::Gamepad(GamepadEvent::OnAxis {
                        gamepad_id,
                        axis: axis_mapped,
                        value: adjusted_value,
                    }));
            }
        }
        _ => {}
    }
}

#[cfg(feature = "wasm")]
pub fn process_web_gamepads(engine_state: &mut EngineState) {
    use wasm_bindgen::JsCast;

    let window = match web_sys::window() {
        Some(window) => window,
        None => return,
    };
    let navigator = window.navigator();
    let pads = match navigator.get_gamepads() {
        Ok(list) => list,
        Err(_) => return,
    };
    let array = js_sys::Array::from(&pads);
    let manager: &mut GamepadCacheManager = &mut engine_state.gamepad.cache;

    for (index, pad_value) in array.iter().enumerate() {
        let pad = match pad_value.dyn_into::<web_sys::Gamepad>() {
            Ok(pad) => pad,
            Err(_) => continue,
        };
        if !pad.connected() {
            continue;
        }

        let gamepad_id = index as u32;
        if manager.get_mut(gamepad_id).is_none() {
            manager.add_gamepad(gamepad_id);
            engine_state
                .event_queue
                .push(EngineEvent::Gamepad(GamepadEvent::OnConnect {
                    gamepad_id,
                    name: pad.id(),
                }));
        }

        if let Some(cache) = manager.get_mut(gamepad_id) {
            let buttons = pad.buttons();
            for (button_idx, button_val) in buttons.iter().enumerate() {
                let button = match button_val.dyn_into::<web_sys::GamepadButton>() {
                    Ok(button) => button,
                    Err(_) => continue,
                };
                let value = button.value() as f32;
                let state = if button.pressed() || value > 0.5 {
                    ElementState::Pressed
                } else {
                    ElementState::Released
                };
                let button_id = button_idx as u32;

                if !cache.button_changed(button_id, state, value) {
                    continue;
                }
                cache.update_button(button_id, state, value);

                engine_state
                    .event_queue
                    .push(EngineEvent::Gamepad(GamepadEvent::OnButton {
                        gamepad_id,
                        button: button_id,
                        state,
                        value,
                    }));
            }

            let axes = pad.axes();
            for (axis_idx, axis_val) in axes.iter().enumerate() {
                let value = axis_val.as_f64().unwrap_or(0.0) as f32;
                let axis_id = axis_idx as u32;
                if !cache.axis_changed(axis_id, value) {
                    continue;
                }
                let adjusted_value = cache.get_axis_value(axis_id);
                cache.update_axis(axis_id, value);
                engine_state
                    .event_queue
                    .push(EngineEvent::Gamepad(GamepadEvent::OnAxis {
                        gamepad_id,
                        axis: axis_id,
                        value: adjusted_value,
                    }));
            }
        }
    }

    let connected_ids: std::collections::HashSet<u32> = array
        .iter()
        .enumerate()
        .filter_map(|(index, pad_value)| {
            let pad = pad_value.dyn_into::<web_sys::Gamepad>().ok()?;
            if pad.connected() {
                Some(index as u32)
            } else {
                None
            }
        })
        .collect();

    let known_ids: Vec<u32> = manager.gamepads.keys().copied().collect();
    for gamepad_id in known_ids {
        if !connected_ids.contains(&gamepad_id) {
            manager.remove_gamepad(gamepad_id);
            engine_state
                .event_queue
                .push(EngineEvent::Gamepad(GamepadEvent::OnDisconnect {
                    gamepad_id,
                }));
        }
    }
}
