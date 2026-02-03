# CmdUploadBufferDiscardAll

Discards all uploaded buffers that are still pending (not consumed by any
`Create*` command).

This is a global maintenance command. It is not tied to any window.

## Arguments

None.

## Response

Returns `CmdResultUploadBufferDiscardAll`:

| Field          | Type   | Description                   |
| -------------- | ------ | ----------------------------- |
| success        | bool   | Whether the discard completed |
| discardedCount | u32    | Number of uploads discarded   |
| message        | String | Status or error message       |
