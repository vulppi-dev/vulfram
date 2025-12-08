use serde::{Deserialize, Serialize};

/// System-level events
#[derive(Debug, Clone, Serialize, Deserialize)]
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
}
