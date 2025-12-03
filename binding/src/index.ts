import * as VULFRAM_CORE from '../napi';
import { decode, encode } from 'cbor2';
import type { VulframResult } from './enums';
import type { EngineBatchCmds, EngineBatchEvents } from './cmds';

export * from './cmds';
export * from './dev';
export * from './enums';
export * from './events';

export function vulframInit(): VulframResult {
  return VULFRAM_CORE.engineInit();
}

export function vulframDispose(): VulframResult {
  return VULFRAM_CORE.engineDispose();
}

export function vulframSendQueue(batch: EngineBatchCmds): VulframResult {
  const buffer = encode(batch);
  return VULFRAM_CORE.engineSendQueue(Buffer.from(buffer));
}

export function vulframReceiveQueue(): [EngineBatchEvents, VulframResult] {
  const { buffer, result } = VULFRAM_CORE.engineReceiveQueue();
  const events = decode(buffer) as EngineBatchEvents;
  return [events, result];
}

export function vulframTick(time: number, deltaTime: number): VulframResult {
  return VULFRAM_CORE.engineTick(time, deltaTime);
}

export function vulframUploadBuffer(
  id: number,
  buffer: Uint8Array,
): VulframResult {
  return VULFRAM_CORE.engineUploadBuffer(id, Buffer.from(buffer));
}

export function vulframDownloadBuffer(id: number): [Uint8Array, VulframResult] {
  const { buffer, result } = VULFRAM_CORE.engineDownloadBuffer(id);
  return [buffer, result];
}

export function vulframClearBuffer(id: number): VulframResult {
  return VULFRAM_CORE.engineClearBuffer(id);
}
