mod core;

use crate::core::VulframResult;
use crate::core::cmd::{CommandResponse, CommandResponseEnvelope, EngineCmd, EngineCmdEnvelope};
use crate::core::window::cmd::{CmdWindowCloseArgs, CmdWindowCreateArgs};
use rmp_serde::{from_slice, to_vec_named};
use std::sync::Mutex;
use std::time::{Duration, Instant};

static ENGINE_GUARD: Mutex<()> = Mutex::new(());

fn main() {
    let _lock = ENGINE_GUARD.lock().unwrap();
    assert_eq!(core::vulfram_init(), VulframResult::Success);

    let create_cmd = EngineCmd::CmdWindowCreate(CmdWindowCreateArgs {
        title: "Vulfram Test Window".into(),
        ..Default::default()
    });
    assert_eq!(send_commands(vec![create_cmd]), VulframResult::Success);

    pump_for(Duration::from_millis(100));
    let window_id = wait_for_window_id();

    pump_for(Duration::from_secs(5));

    let close_cmd = EngineCmd::CmdWindowClose(CmdWindowCloseArgs { window_id });
    assert_eq!(send_commands(vec![close_cmd]), VulframResult::Success);
    pump_for(Duration::from_millis(100));

    assert_eq!(core::vulfram_dispose(), VulframResult::Success);
}

fn pump_for(duration: Duration) {
    let start = Instant::now();
    let mut frame: u64 = 0;
    while start.elapsed() < duration {
        assert_eq!(core::vulfram_tick(frame * 16, 16), VulframResult::Success);
        frame += 1;
        std::thread::sleep(Duration::from_millis(50));
    }
}

fn wait_for_window_id() -> u32 {
    for _ in 0..40 {
        let responses = receive_responses();
        for response in responses {
            if let CommandResponse::WindowCreate(result) = response.response {
                assert!(result.success, "window creation failed: {}", result.message);
                return result.content;
            }
        }
        pump_for(Duration::from_millis(50));
    }
    panic!("window creation response not received");
}

fn send_commands(cmds: Vec<EngineCmd>) -> VulframResult {
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

fn receive_responses() -> Vec<CommandResponseEnvelope> {
    let mut ptr = std::ptr::null();
    let mut len: usize = 0;
    assert_eq!(
        core::vulfram_receive_queue(&mut ptr, &mut len),
        VulframResult::Success
    );
    if len == 0 {
        return Vec::new();
    }
    let bytes = unsafe { Vec::from_raw_parts(ptr as *mut u8, len, len) };
    from_slice(&bytes).expect("failed to deserialize responses")
}
