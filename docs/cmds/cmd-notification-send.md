# CmdNotificationSend

Sends a system notification and captures interaction events.

## Platform Notes

- **WASM:** Not supported (returns `success=false`).

## Arguments

| Field   | Type              | Description                                                              |
| ------- | ----------------- | ------------------------------------------------------------------------ |
| id      | String (Optional) | User-defined ID to identify this notification in interaction events      |
| title   | String            | Title of the notification                                                |
| body    | String            | Body text of the notification                                            |
| level   | NotificationLevel | Level (info, warning, error, success)                                    |
| timeout | u32 (Optional)    | Timeout in milliseconds (0 for persistent, if supported by the platform) |

### NotificationLevel (Enum)

- `info`
- `warning`
- `error`
- `success`

## Response

Returns `CmdResultNotificationSend`:

| Field   | Type | Description                         |
| ------- | ---- | ----------------------------------- |
| success | bool | Whether the notification was queued |

## Events Dispatched

Depending on user interaction, the following events may be dispatched to the event pool:

### OnNotificationClicked

Dispatched when the user clicks the notification.

| Field | Type   | Description                            |
| ----- | ------ | -------------------------------------- |
| id    | String | The ID passed in `CmdNotificationSend` |

### OnNotificationDismissed

Dispatched when the notification is closed or expires.

| Field | Type   | Description                            |
| ----- | ------ | -------------------------------------- |
| id    | String | The ID passed in `CmdNotificationSend` |
