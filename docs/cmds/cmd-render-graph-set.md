# CmdRenderGraphSet

Sets a host-defined render graph for a window. The core validates the graph and compiles an execution plan. If invalid and `fallback=true`, the core uses the default fallback graph.

## Arguments

| Field    | Type            | Description       |
| -------- | --------------- | ----------------- |
| windowId | u32             | ID of the window  |
| graph    | RenderGraphDesc | Graph description |

## Response

Returns `CmdResultRenderGraphSet`:

| Field        | Type   | Description                            |
| ------------ | ------ | -------------------------------------- |
| success      | bool   | Whether the graph was accepted         |
| fallbackUsed | bool   | Whether the fallback graph was applied |
| message      | String | Status or error message                |
