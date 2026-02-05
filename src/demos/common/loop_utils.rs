use crate::core;
use crate::core::VulframResult;
use crate::core::cmd::{
    CommandResponse, CommandResponseEnvelope, EngineCmd, EngineCmdEnvelope, EngineEvent,
};
use crate::core::input::events::{ElementState, KeyboardEvent};
use crate::core::window::{CmdWindowCloseArgs, WindowEvent};
use rmp_serde::{from_slice, to_vec_named};
use std::time::{Duration, Instant};

pub fn run_loop<F>(window_id: u32, max_duration: Option<Duration>, on_frame: F) -> bool
where
    F: FnMut(u64, u32) -> Vec<EngineCmd>,
{
    run_loop_with_events(window_id, max_duration, on_frame, |_| false)
}

pub fn run_loop_with_events<F, G>(
    window_id: u32,
    max_duration: Option<Duration>,
    mut on_frame: F,
    mut on_event: G,
) -> bool
where
    F: FnMut(u64, u32) -> Vec<EngineCmd>,
    G: FnMut(EngineEvent) -> bool,
{
    let start_time = Instant::now();
    let mut last_frame_time = Instant::now();
    let mut total_ms: u64 = 0;
    let target_frame_time = Duration::from_millis(16);

    loop {
        if let Some(max_duration) = max_duration {
            if start_time.elapsed() >= max_duration {
                return false;
            }
        }

        let now = Instant::now();
        let delta_ms = now.duration_since(last_frame_time).as_millis() as u32;
        last_frame_time = now;
        total_ms += delta_ms as u64;

        let frame_cmds = on_frame(total_ms, delta_ms);
        if !frame_cmds.is_empty() {
            let _ = send_commands(frame_cmds);
        }

        let tick_start = Instant::now();
        assert_eq!(
            core::vulfram_tick(total_ms, delta_ms),
            VulframResult::Success
        );

        let _ = receive_responses();
        if total_ms % 1000 == 0 {
            if let Some(profiling) = get_profiling() {
                println!("Profiling: {:?}", profiling);
            }
        }

        if handle_events(window_id, &mut on_event) {
            let _ = send_commands(vec![EngineCmd::CmdWindowClose(CmdWindowCloseArgs {
                window_id,
            })]);
            return true;
        }

        let elapsed = tick_start.elapsed();
        if elapsed < target_frame_time {
            std::thread::sleep(target_frame_time - elapsed);
        }
    }
}

pub fn is_close_event(window_id: u32, event: &EngineEvent) -> bool {
    match event {
        EngineEvent::Window(WindowEvent::OnCloseRequest { window_id: id }) if *id == window_id => {
            true
        }
        EngineEvent::Keyboard(KeyboardEvent::OnInput {
            window_id: id,
            key_code,
            state: ElementState::Pressed,
            ..
        }) if *id == window_id && *key_code == 106 => true,
        _ => false,
    }
}

pub fn handle_events<F>(window_id: u32, on_event: &mut F) -> bool
where
    F: FnMut(EngineEvent) -> bool,
{
    let events = receive_events();
    for event in events {
        if is_close_event(window_id, &event) {
            return true;
        }
        if on_event(event) {
            return true;
        }
    }
    false
}

pub fn get_profiling() -> Option<core::profiling::cmd::ProfilingData> {
    let mut ptr = std::ptr::null();
    let mut len: usize = 0;
    let result = core::vulfram_get_profiling(&mut ptr, &mut len);
    if result != VulframResult::Success || len == 0 {
        return None;
    }
    let bytes = unsafe { Box::from_raw(std::slice::from_raw_parts_mut(ptr as *mut u8, len)) };
    let profiling = from_slice(&bytes).ok()?;
    Some(profiling)
}

pub fn pump_for(duration: Duration) {
    let start = Instant::now();
    let mut total_ms: u64 = 0;
    while start.elapsed() < duration {
        assert_eq!(core::vulfram_tick(total_ms, 16), VulframResult::Success);
        total_ms += 16;
        std::thread::sleep(Duration::from_millis(16));
    }
}

pub fn wait_for_confirmation(_window_id: u32) {
    for _ in 0..100 {
        let responses = receive_responses();
        for response in responses {
            match response.response {
                CommandResponse::WindowCreate(res) => {
                    if res.success {
                        return;
                    } else {
                        panic!("Window creation failed: {}", res.message);
                    }
                }
                _ => {}
            }
        }
        std::thread::sleep(Duration::from_millis(10));
        assert_eq!(core::vulfram_tick(0, 0), VulframResult::Success);
    }
}

pub fn send_commands(cmds: Vec<EngineCmd>) -> VulframResult {
    let envelopes: Vec<EngineCmdEnvelope> = cmds
        .into_iter()
        .enumerate()
        .map(|(idx, cmd)| EngineCmdEnvelope {
            id: idx as u64,
            cmd,
        })
        .collect();
    let data = to_vec_named(&envelopes).expect("failed to serialize commands");
    core::vulfram_send_queue(data.as_ptr(), data.len())
}

pub fn receive_responses() -> Vec<CommandResponseEnvelope> {
    let mut ptr = std::ptr::null();
    let mut len: usize = 0;
    let result = core::vulfram_receive_queue(&mut ptr, &mut len);

    if result != VulframResult::Success || len == 0 {
        return Vec::new();
    }

    let bytes = unsafe { Box::from_raw(std::slice::from_raw_parts_mut(ptr as *mut u8, len)) };
    let responses = from_slice(&bytes).expect("failed to deserialize responses");
    responses
}

pub fn receive_events() -> Vec<EngineEvent> {
    let mut ptr = std::ptr::null();
    let mut len: usize = 0;
    let result = core::vulfram_receive_events(&mut ptr, &mut len);

    if result != VulframResult::Success || len == 0 {
        return Vec::new();
    }

    let bytes = unsafe { Box::from_raw(std::slice::from_raw_parts_mut(ptr as *mut u8, len)) };
    let events = from_slice(&bytes).expect("failed to deserialize events");
    events
}
