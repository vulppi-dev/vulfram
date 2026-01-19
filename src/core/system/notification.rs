#[cfg(target_os = "linux")]
use notify_rust::Urgency;
use notify_rust::{Notification, Timeout};
use serde::{Deserialize, Serialize};
use crate::core::platform::EventLoopProxy;

use crate::core::singleton::EngineCustomEvents;
use crate::core::state::EngineState;
use crate::core::system::SystemEvent;

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub enum NotificationLevel {
    Info,
    Warning,
    Error,
    Success,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct CmdNotificationSendArgs {
    pub id: Option<String>,
    pub title: String,
    pub body: String,
    pub level: NotificationLevel,
    pub timeout: Option<u32>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct CmdResultNotificationSend {
    pub success: bool,
}

pub fn engine_cmd_notification_send(
    _engine: &mut EngineState,
    loop_proxy: &EventLoopProxy<EngineCustomEvents>,
    args: &CmdNotificationSendArgs,
) -> CmdResultNotificationSend {
    let mut notification = Notification::new();
    notification.summary(&args.title).body(&args.body);

    #[cfg(target_os = "linux")]
    match args.level {
        NotificationLevel::Info | NotificationLevel::Success => {
            notification.urgency(Urgency::Low);
        }
        NotificationLevel::Warning => {
            notification.urgency(Urgency::Normal);
        }
        NotificationLevel::Error => {
            notification.urgency(Urgency::Critical);
        }
    }

    #[cfg(not(target_os = "linux"))]
    let _ = &args.level;

    if let Some(ms) = args.timeout {
        notification.timeout(Timeout::Milliseconds(ms));
    } else {
        notification.timeout(Timeout::Default);
    }

    // Add a default action to capture clicks on platforms that support it
    #[cfg(target_os = "linux")]
    notification.action("default", "Clicked");

    let proxy = loop_proxy.clone();
    let id = args.id.clone().unwrap_or_default();

    std::thread::spawn(move || match notification.show() {
        Ok(handle) => {
            #[cfg(target_os = "linux")]
            handle.wait_for_action(|action| match action {
                "default" => {
                    let _ = proxy.send_event(EngineCustomEvents::NotificationInteraction(
                        SystemEvent::OnNotificationClicked { id: id.clone() },
                    ));
                }
                "__closed" => {
                    let _ = proxy.send_event(EngineCustomEvents::NotificationInteraction(
                        SystemEvent::OnNotificationDismissed { id: id.clone() },
                    ));
                }
                _ => {}
            });

            #[cfg(not(target_os = "linux"))]
            let _ = handle; // Handle is () or different on other platforms
        }
        Err(e) => {
            log::error!("Failed to show notification: {}", e);
        }
    });

    CmdResultNotificationSend { success: true }
}
