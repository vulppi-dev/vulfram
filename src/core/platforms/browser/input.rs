use wasm_bindgen::JsCast;
use wasm_bindgen::closure::Closure;
use web_sys::{Event, HtmlCanvasElement, KeyboardEvent, PointerEvent, WheelEvent};

use crate::core::cmd::EngineEvent;
use crate::core::input::events::{ElementState, ModifiersState, TouchPhase};
use crate::core::input::events::{
    KeyboardEvent as CoreKeyboardEvent, PointerEvent as CorePointerEvent, ScrollDelta,
};
use crate::core::singleton::with_engine;
use crate::core::window::WindowEvent;

pub fn attach_canvas_listeners(
    window_id: u32,
    canvas: &HtmlCanvasElement,
) -> Vec<Closure<dyn FnMut(Event)>> {
    let mut listeners: Vec<Closure<dyn FnMut(Event)>> = Vec::new();

    let window = match web_sys::window() {
        Some(window) => window,
        None => return listeners,
    };

    let canvas_for_resize = canvas.clone();
    let resize_closure = Closure::wrap(Box::new(move |_event: Event| {
        let width = canvas_for_resize.client_width().max(1) as u32;
        let height = canvas_for_resize.client_height().max(1) as u32;

        let _ = with_engine(|engine| {
            if let Some(window_state) = engine.window.states.get_mut(&window_id) {
                window_state.config.width = width;
                window_state.config.height = height;
                if let Some(device) = engine.device.as_ref() {
                    window_state.surface.configure(device, &window_state.config);
                    window_state.render_state.on_resize(device, width, height);
                }
                window_state.inner_size = glam::UVec2::new(width, height);
                window_state.outer_size = glam::UVec2::new(width, height);
                window_state.is_dirty = true;
            }
            engine
                .event_queue
                .push(EngineEvent::Window(WindowEvent::OnResize {
                    window_id,
                    width,
                    height,
                }));
        });
    }) as Box<dyn FnMut(Event)>);
    let _ =
        window.add_event_listener_with_callback("resize", resize_closure.as_ref().unchecked_ref());
    listeners.push(resize_closure);

    let focus_closure = Closure::wrap(Box::new(move |_event: Event| {
        let _ = with_engine(|engine| {
            engine
                .event_queue
                .push(EngineEvent::Window(WindowEvent::OnFocus {
                    window_id,
                    focused: true,
                }));
        });
    }) as Box<dyn FnMut(Event)>);
    let _ =
        window.add_event_listener_with_callback("focus", focus_closure.as_ref().unchecked_ref());
    listeners.push(focus_closure);

    let blur_closure = Closure::wrap(Box::new(move |_event: Event| {
        let _ = with_engine(|engine| {
            engine
                .event_queue
                .push(EngineEvent::Window(WindowEvent::OnFocus {
                    window_id,
                    focused: false,
                }));
        });
    }) as Box<dyn FnMut(Event)>);
    let _ = window.add_event_listener_with_callback("blur", blur_closure.as_ref().unchecked_ref());
    listeners.push(blur_closure);

    let keydown_closure = Closure::wrap(Box::new(move |event: Event| {
        let event: KeyboardEvent = match event.dyn_into() {
            Ok(ev) => ev,
            Err(_) => return,
        };
        let modifiers = ModifiersState {
            shift: event.shift_key(),
            ctrl: event.ctrl_key(),
            alt: event.alt_key(),
            meta: event.meta_key(),
        };
        let key_code = map_key_code(&event.code());
        let text = event
            .key()
            .chars()
            .next()
            .filter(|_| event.key().len() == 1)
            .map(|_| event.key());

        let _ = with_engine(|engine| {
            engine
                .event_queue
                .push(EngineEvent::Keyboard(CoreKeyboardEvent::OnInput {
                    window_id,
                    key_code,
                    state: ElementState::Pressed,
                    location: event.location() as u32,
                    repeat: event.repeat(),
                    text,
                    modifiers,
                }));
        });
    }) as Box<dyn FnMut(Event)>);
    let _ = window
        .add_event_listener_with_callback("keydown", keydown_closure.as_ref().unchecked_ref());
    listeners.push(keydown_closure);

    let keyup_closure = Closure::wrap(Box::new(move |event: Event| {
        let event: KeyboardEvent = match event.dyn_into() {
            Ok(ev) => ev,
            Err(_) => return,
        };
        let modifiers = ModifiersState {
            shift: event.shift_key(),
            ctrl: event.ctrl_key(),
            alt: event.alt_key(),
            meta: event.meta_key(),
        };
        let key_code = map_key_code(&event.code());
        let text = event
            .key()
            .chars()
            .next()
            .filter(|_| event.key().len() == 1)
            .map(|_| event.key());

        let _ = with_engine(|engine| {
            engine
                .event_queue
                .push(EngineEvent::Keyboard(CoreKeyboardEvent::OnInput {
                    window_id,
                    key_code,
                    state: ElementState::Released,
                    location: event.location() as u32,
                    repeat: event.repeat(),
                    text,
                    modifiers,
                }));
        });
    }) as Box<dyn FnMut(Event)>);
    let _ =
        window.add_event_listener_with_callback("keyup", keyup_closure.as_ref().unchecked_ref());
    listeners.push(keyup_closure);

    let canvas_for_pointer = canvas.clone();
    let pointer_move = Closure::wrap(Box::new(move |event: Event| {
        let event: PointerEvent = match event.dyn_into() {
            Ok(ev) => ev,
            Err(_) => return,
        };
        let position = canvas_relative_pos(&canvas_for_pointer, event.client_x(), event.client_y());
        let pointer_type = map_pointer_type(&event.pointer_type());
        let pointer_id = event.pointer_id() as u64;

        let _ = with_engine(|engine| {
            engine.window.cursor_positions.insert(window_id, position);
            engine
                .event_queue
                .push(EngineEvent::Pointer(CorePointerEvent::OnMove {
                    window_id,
                    pointer_type,
                    pointer_id,
                    position,
                }));
        });
    }) as Box<dyn FnMut(Event)>);
    let _ = canvas
        .add_event_listener_with_callback("pointermove", pointer_move.as_ref().unchecked_ref());
    listeners.push(pointer_move);

    let canvas_for_pointer = canvas.clone();
    let pointer_down = Closure::wrap(Box::new(move |event: Event| {
        let event: PointerEvent = match event.dyn_into() {
            Ok(ev) => ev,
            Err(_) => return,
        };
        let position = canvas_relative_pos(&canvas_for_pointer, event.client_x(), event.client_y());
        let pointer_type = map_pointer_type(&event.pointer_type());
        let pointer_id = event.pointer_id() as u64;
        let button = event.button() as u32;

        let _ = with_engine(|engine| {
            engine
                .event_queue
                .push(EngineEvent::Pointer(CorePointerEvent::OnButton {
                    window_id,
                    pointer_type,
                    pointer_id,
                    button,
                    state: ElementState::Pressed,
                    position,
                }));
        });
    }) as Box<dyn FnMut(Event)>);
    let _ = canvas
        .add_event_listener_with_callback("pointerdown", pointer_down.as_ref().unchecked_ref());
    listeners.push(pointer_down);

    let canvas_for_pointer = canvas.clone();
    let pointer_up = Closure::wrap(Box::new(move |event: Event| {
        let event: PointerEvent = match event.dyn_into() {
            Ok(ev) => ev,
            Err(_) => return,
        };
        let position = canvas_relative_pos(&canvas_for_pointer, event.client_x(), event.client_y());
        let pointer_type = map_pointer_type(&event.pointer_type());
        let pointer_id = event.pointer_id() as u64;
        let button = event.button() as u32;

        let _ = with_engine(|engine| {
            engine
                .event_queue
                .push(EngineEvent::Pointer(CorePointerEvent::OnButton {
                    window_id,
                    pointer_type,
                    pointer_id,
                    button,
                    state: ElementState::Released,
                    position,
                }));
        });
    }) as Box<dyn FnMut(Event)>);
    let _ =
        canvas.add_event_listener_with_callback("pointerup", pointer_up.as_ref().unchecked_ref());
    listeners.push(pointer_up);

    let pointer_enter = Closure::wrap(Box::new(move |event: Event| {
        let event: PointerEvent = match event.dyn_into() {
            Ok(ev) => ev,
            Err(_) => return,
        };
        let pointer_type = map_pointer_type(&event.pointer_type());
        let pointer_id = event.pointer_id() as u64;
        let _ = with_engine(|engine| {
            engine
                .event_queue
                .push(EngineEvent::Pointer(CorePointerEvent::OnEnter {
                    window_id,
                    pointer_type,
                    pointer_id,
                }));
        });
    }) as Box<dyn FnMut(Event)>);
    let _ = canvas
        .add_event_listener_with_callback("pointerenter", pointer_enter.as_ref().unchecked_ref());
    listeners.push(pointer_enter);

    let pointer_leave = Closure::wrap(Box::new(move |event: Event| {
        let event: PointerEvent = match event.dyn_into() {
            Ok(ev) => ev,
            Err(_) => return,
        };
        let pointer_type = map_pointer_type(&event.pointer_type());
        let pointer_id = event.pointer_id() as u64;
        let _ = with_engine(|engine| {
            engine
                .event_queue
                .push(EngineEvent::Pointer(CorePointerEvent::OnLeave {
                    window_id,
                    pointer_type,
                    pointer_id,
                }));
        });
    }) as Box<dyn FnMut(Event)>);
    let _ = canvas
        .add_event_listener_with_callback("pointerleave", pointer_leave.as_ref().unchecked_ref());
    listeners.push(pointer_leave);

    let wheel_canvas = canvas.clone();
    let wheel_closure = Closure::wrap(Box::new(move |event: Event| {
        let event: WheelEvent = match event.dyn_into() {
            Ok(ev) => ev,
            Err(_) => return,
        };
        let delta = glam::Vec2::new(event.delta_x() as f32, event.delta_y() as f32);
        let phase = TouchPhase::Moved;
        let delta = if event.delta_mode() == WheelEvent::DOM_DELTA_PIXEL {
            ScrollDelta::Pixel(delta)
        } else {
            ScrollDelta::Line(delta)
        };

        let _ = with_engine(|engine| {
            let _ = &wheel_canvas;
            engine
                .event_queue
                .push(EngineEvent::Pointer(CorePointerEvent::OnScroll {
                    window_id,
                    delta,
                    phase,
                }));
        });
    }) as Box<dyn FnMut(Event)>);
    let _ =
        canvas.add_event_listener_with_callback("wheel", wheel_closure.as_ref().unchecked_ref());
    listeners.push(wheel_closure);

    listeners
}

fn canvas_relative_pos(canvas: &HtmlCanvasElement, x: i32, y: i32) -> glam::Vec2 {
    let rect = canvas.get_bounding_client_rect();
    glam::Vec2::new(
        (x as f64 - rect.left()) as f32,
        (y as f64 - rect.top()) as f32,
    )
}

fn map_pointer_type(pointer_type: &str) -> u32 {
    match pointer_type {
        "mouse" => 0,
        "touch" => 1,
        "pen" => 2,
        _ => 0,
    }
}

fn map_key_code(code: &str) -> u32 {
    match code {
        "Backquote" => 0,
        "Backslash" => 1,
        "BracketLeft" => 2,
        "BracketRight" => 3,
        "Comma" => 4,
        "Digit0" => 5,
        "Digit1" => 6,
        "Digit2" => 7,
        "Digit3" => 8,
        "Digit4" => 9,
        "Digit5" => 10,
        "Digit6" => 11,
        "Digit7" => 12,
        "Digit8" => 13,
        "Digit9" => 14,
        "Equal" => 15,
        "IntlBackslash" => 16,
        "IntlRo" => 17,
        "IntlYen" => 18,
        "KeyA" => 19,
        "KeyB" => 20,
        "KeyC" => 21,
        "KeyD" => 22,
        "KeyE" => 23,
        "KeyF" => 24,
        "KeyG" => 25,
        "KeyH" => 26,
        "KeyI" => 27,
        "KeyJ" => 28,
        "KeyK" => 29,
        "KeyL" => 30,
        "KeyM" => 31,
        "KeyN" => 32,
        "KeyO" => 33,
        "KeyP" => 34,
        "KeyQ" => 35,
        "KeyR" => 36,
        "KeyS" => 37,
        "KeyT" => 38,
        "KeyU" => 39,
        "KeyV" => 40,
        "KeyW" => 41,
        "KeyX" => 42,
        "KeyY" => 43,
        "KeyZ" => 44,
        "Minus" => 45,
        "Period" => 46,
        "Quote" => 47,
        "Semicolon" => 48,
        "Slash" => 49,
        "AltLeft" => 50,
        "AltRight" => 51,
        "Backspace" => 52,
        "CapsLock" => 53,
        "ContextMenu" => 54,
        "ControlLeft" => 55,
        "ControlRight" => 56,
        "Enter" => 57,
        "MetaLeft" => 58,
        "MetaRight" => 59,
        "ShiftLeft" => 60,
        "ShiftRight" => 61,
        "Space" => 62,
        "Tab" => 63,
        "Delete" => 64,
        "End" => 65,
        "Help" => 66,
        "Home" => 67,
        "Insert" => 68,
        "PageDown" => 69,
        "PageUp" => 70,
        "ArrowDown" => 71,
        "ArrowLeft" => 72,
        "ArrowRight" => 73,
        "ArrowUp" => 74,
        "NumLock" => 75,
        "Numpad0" => 76,
        "Numpad1" => 77,
        "Numpad2" => 78,
        "Numpad3" => 79,
        "Numpad4" => 80,
        "Numpad5" => 81,
        "Numpad6" => 82,
        "Numpad7" => 83,
        "Numpad8" => 84,
        "Numpad9" => 85,
        "NumpadAdd" => 86,
        "NumpadComma" => 87,
        "NumpadDecimal" => 88,
        "NumpadDivide" => 89,
        "NumpadEnter" => 90,
        "NumpadEqual" => 91,
        "NumpadMultiply" => 92,
        "NumpadSubtract" => 93,
        "Escape" => 94,
        "F1" => 95,
        "F2" => 96,
        "F3" => 97,
        "F4" => 98,
        "F5" => 99,
        "F6" => 100,
        "F7" => 101,
        "F8" => 102,
        "F9" => 103,
        "F10" => 104,
        "F11" => 105,
        "F12" => 106,
        "F13" => 107,
        "F14" => 108,
        "F15" => 109,
        "F16" => 110,
        "F17" => 111,
        "F18" => 112,
        "F19" => 113,
        "F20" => 114,
        "F21" => 115,
        "F22" => 116,
        "F23" => 117,
        "F24" => 118,
        "PrintScreen" => 119,
        "ScrollLock" => 120,
        "Pause" => 121,
        _ => 0,
    }
}
