use gilrs::{Event as GilrsEvent, EventType as GilrsEventType};

use super::cmd::EngineEvent;
use super::cmd::events::{ElementState, GamepadEvent};
use super::state::EngineState;

/// Process a gilrs gamepad event and convert it to engine events
pub fn process_gilrs_event(engine_state: &mut EngineState, event: GilrsEvent) {
    let gamepad_id: u32 = usize::from(event.id) as u32;

    match event.event {
        GilrsEventType::Connected => {
            let name: String = if let Some(gilrs) = &engine_state.gilrs {
                gilrs.gamepad(event.id).name().into()
            } else {
                "Unknown".into()
            };

            // Add to cache for event filtering
            engine_state.gamepad_cache.add_gamepad(gamepad_id);

            engine_state
                .event_queue
                .push(EngineEvent::Gamepad(GamepadEvent::OnConnect {
                    gamepad_id,
                    name,
                }));
        }
        GilrsEventType::Disconnected => {
            // Remove from cache
            engine_state.gamepad_cache.remove_gamepad(gamepad_id);

            engine_state
                .event_queue
                .push(EngineEvent::Gamepad(GamepadEvent::OnDisconnect {
                    gamepad_id,
                }));
        }
        GilrsEventType::ButtonPressed(button, _code) => {
            let button_mapped = super::cmd::events::convert_gilrs_button(button);
            let value = 1.0;
            let state = ElementState::Pressed;

            // Check if button state actually changed
            if let Some(cache) = engine_state.gamepad_cache.get_mut(gamepad_id) {
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
            let button_mapped = super::cmd::events::convert_gilrs_button(button);
            let value = 0.0;
            let state = ElementState::Released;

            // Check if button state actually changed
            if let Some(cache) = engine_state.gamepad_cache.get_mut(gamepad_id) {
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
            let button_mapped = super::cmd::events::convert_gilrs_button(button);
            let state = if value > 0.5 {
                ElementState::Pressed
            } else {
                ElementState::Released
            };

            // Check if button state or value actually changed significantly
            if let Some(cache) = engine_state.gamepad_cache.get_mut(gamepad_id) {
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
            let axis_mapped = super::cmd::events::convert_gilrs_axis(axis);

            // Check if axis value actually changed significantly (with dead zone)
            if let Some(cache) = engine_state.gamepad_cache.get_mut(gamepad_id) {
                if !cache.axis_changed(axis_mapped, value) {
                    return;
                }

                // Get the adjusted value with dead zone applied
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
