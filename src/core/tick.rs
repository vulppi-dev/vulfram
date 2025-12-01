use gilrs::{Event as GilrsEvent, EventType as GilrsEventType};
use winit::event_loop::ControlFlow;
use winit::platform::pump_events::EventLoopExtPumpEvents;

use super::cmd::events::{ElementState, GamepadEvent};
use super::cmd::{EngineEvent, EngineEventEnvelope};
use super::result::EngineResult;
use super::singleton::with_engine_singleton;
use super::state::EngineState;

/// Main engine tick - processes events and updates state
pub fn engine_tick(time: u64, delta_time: u32) -> EngineResult {
    match with_engine_singleton(|engine| {
        engine.state.time = time;
        engine.state.delta_time = delta_time;

        // Process gamepad events
        let mut gilrs_events = Vec::new();
        if let Some(gilrs) = &mut engine.state.gilrs {
            while let Some(event) = gilrs.next_event() {
                gilrs_events.push(event);
            }
        }

        for event in gilrs_events {
            process_gilrs_event(&mut engine.state, event);
        }

        if let Some(mut event_loop) = engine.event_loop.take() {
            event_loop.set_control_flow(ControlFlow::Poll);
            event_loop.pump_app_events(None, &mut engine.state);
            engine.event_loop = Some(event_loop);
        }

        engine.state.request_redraw();
    }) {
        Err(e) => e,
        Ok(_) => EngineResult::Success,
    }
}

/// Process a gilrs gamepad event and convert it to engine events
fn process_gilrs_event(engine_state: &mut EngineState, event: GilrsEvent) {
    let gamepad_id: u32 = usize::from(event.id) as u32;

    match event.event {
        GilrsEventType::Connected => {
            let name = if let Some(gilrs) = &engine_state.gilrs {
                gilrs.gamepad(event.id).name().to_string()
            } else {
                "Unknown".to_string()
            };

            engine_state.event_queue.push(EngineEventEnvelope {
                id: 0,
                event: EngineEvent::Gamepad(GamepadEvent::OnConnect { gamepad_id, name }),
            });
        }
        GilrsEventType::Disconnected => {
            engine_state.event_queue.push(EngineEventEnvelope {
                id: 0,
                event: EngineEvent::Gamepad(GamepadEvent::OnDisconnect { gamepad_id }),
            });
        }
        GilrsEventType::ButtonPressed(button, _code) => {
            let button_mapped = super::cmd::events::convert_gilrs_button(button);
            engine_state.event_queue.push(EngineEventEnvelope {
                id: 0,
                event: EngineEvent::Gamepad(GamepadEvent::OnButton {
                    gamepad_id,
                    button: button_mapped,
                    state: ElementState::Pressed,
                    value: 1.0,
                }),
            });
        }
        GilrsEventType::ButtonReleased(button, _code) => {
            let button_mapped = super::cmd::events::convert_gilrs_button(button);
            engine_state.event_queue.push(EngineEventEnvelope {
                id: 0,
                event: EngineEvent::Gamepad(GamepadEvent::OnButton {
                    gamepad_id,
                    button: button_mapped,
                    state: ElementState::Released,
                    value: 0.0,
                }),
            });
        }
        GilrsEventType::ButtonChanged(button, value, _code) => {
            let button_mapped = super::cmd::events::convert_gilrs_button(button);
            let state = if value > 0.5 {
                ElementState::Pressed
            } else {
                ElementState::Released
            };
            engine_state.event_queue.push(EngineEventEnvelope {
                id: 0,
                event: EngineEvent::Gamepad(GamepadEvent::OnButton {
                    gamepad_id,
                    button: button_mapped,
                    state,
                    value,
                }),
            });
        }
        GilrsEventType::AxisChanged(axis, value, _code) => {
            let axis_mapped = super::cmd::events::convert_gilrs_axis(axis);
            engine_state.event_queue.push(EngineEventEnvelope {
                id: 0,
                event: EngineEvent::Gamepad(GamepadEvent::OnAxis {
                    gamepad_id,
                    axis: axis_mapped,
                    value,
                }),
            });
        }
        _ => {}
    }
}
