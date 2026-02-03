use serde::{Deserialize, Serialize};

/// System-level events
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "event", content = "data", rename_all = "kebab-case")]
pub enum SystemEvent {
    /// Application was resumed (from suspended state)
    OnResume,

    /// Application was suspended
    OnSuspend,

    /// Low memory warning
    OnMemoryWarning,

    /// Application is about to exit
    OnExit,

    /// Notification was clicked
    OnNotificationClicked { id: String },

    /// Notification was dismissed or expired
    OnNotificationDismissed { id: String },

    /// Async texture decode finished
    TextureReady {
        window_id: u32,
        texture_id: u32,
        success: bool,
        message: String,
    },

    /// Async audio decode finished
    AudioReady {
        resource_id: u32,
        success: bool,
        message: String,
    },
}
