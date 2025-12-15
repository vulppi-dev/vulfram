pub mod cache;
pub mod converters;
pub mod events;
pub mod state;

use gilrs::{Event as GilrsEvent, EventType as GilrsEventType};

use crate::core::cmd::EngineEvent;
use crate::core::gamepad::converters::{convert_gilrs_axis, convert_gilrs_button};
use crate::core::gamepad::events::GamepadEvent;
use crate::core::input::events::ElementState;
use crate::core::state::EngineState;

use self::cache::GamepadCacheManager;

/// Process a gilrs gamepad event and convert it to engine events
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
