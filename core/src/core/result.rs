#[derive(Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum VulframResult {
    Success = 0,
    UnknownError = 1,
    NotInitialized,
    AlreadyInitialized,
    WrongThread,
    CmdInvalidMessagePackError,
    BufferNotFound,
    BufferIdCollision,
    InvalidUploadType,
}
