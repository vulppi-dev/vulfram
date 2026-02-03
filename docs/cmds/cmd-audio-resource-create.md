# CmdAudioResourceCreate

Creates an audio resource from raw bytes uploaded to the buffer system.
The upload must be `UploadType::BinaryAsset`.

## Arguments

| Field       | Type        | Description                                   |
| ----------- | ----------- | --------------------------------------------- |
| resourceId  | u32         | Logical ID for the audio asset                |
| bufferId    | u64         | Upload buffer ID containing data              |
| totalBytes  | Option<u64> | Total size of the audio stream (bytes)        |
| offsetBytes | Option<u64> | Chunk offset in bytes (defaults to 0)         |

## Response

Returns `CmdResultAudioResourceCreate`:

| Field   | Type   | Description                                  |
| ------- | ------ | -------------------------------------------- |
| success | bool   | Whether the request was accepted             |
| message | String | Status or error message                      |
| pending | bool   | Whether decoding is happening asynchronously |

## Notes

Use `SystemEvent::AudioReady` to know when decoding finished.
For streams, use `SystemEvent::AudioStreamProgress` to track chunking.
