# CmdAudioResourcePush

Adds a chunk to an existing streaming audio resource.

## Arguments

| Field       | Type | Description                          |
| ----------- | ---- | ------------------------------------ |
| resourceId  | u32  | Logical ID for the audio asset       |
| bufferId    | u64  | Upload buffer ID containing data     |
| offsetBytes | u64  | Chunk offset in bytes                |

## Response

Returns `CmdResultAudioResourcePush`:

| Field         | Type   | Description                        |
| ------------- | ------ | ---------------------------------- |
| success       | bool   | Whether the chunk was accepted     |
| message       | String | Status or error message            |
| receivedBytes | u64    | Total bytes received so far        |
| totalBytes    | u64    | Total expected bytes               |
| complete      | bool   | Whether the stream is complete     |

## Notes

Use `SystemEvent::AudioStreamProgress` to track chunking.
