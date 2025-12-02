export enum VulframResult {
  Success = 0,
  UnknownError = 1,
  NotInitialized,
  AlreadyInitialized,
  WrongThread,
  BufferOverflow,
  CmdInvalidCborError,
}
