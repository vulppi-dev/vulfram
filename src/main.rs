mod core;
mod demos;

use crate::core::VulframResult;
use crate::core::cmd::EngineCmd;
use crate::core::window::CmdWindowCloseArgs;
use std::sync::Mutex;
use std::time::Duration;

static ENGINE_GUARD: Mutex<()> = Mutex::new(());

fn main() {
    let _lock = ENGINE_GUARD.lock().unwrap();

    assert_eq!(core::vulfram_init(), VulframResult::Success);

    let demo = demos::select_demo();
    let window_id: u32 = 1;

    demos::common::create_window(window_id, demo.title());
    demos::common::pump_for(Duration::from_millis(200));
    demos::common::wait_for_confirmation(window_id);

    let close_sent = demos::run_demo(demo, window_id);

    if !close_sent {
        let close_cmd = EngineCmd::CmdWindowClose(CmdWindowCloseArgs { window_id });
        let _ = demos::common::send_commands(vec![close_cmd]);
    }
    demos::common::pump_for(Duration::from_millis(100));

    assert_eq!(core::vulfram_dispose(), VulframResult::Success);
}
