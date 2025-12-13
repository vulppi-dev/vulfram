import { createRequire } from 'module';

const require = createRequire(import.meta.url);
const VULFRAM_CORE = require('./libvulfram_core.node');

export interface BufferResult {
  buffer: Buffer;
  result: number;
}

export function vulframDispose(): number {
  return VULFRAM_CORE.vulframDispose();
}

export function vulframDownloadBuffer(id: number): BufferResult {
  return VULFRAM_CORE.vulframDownloadBuffer(id);
}

export function vulframInit(): number {
  return VULFRAM_CORE.vulframInit();
}

export function vulframReceiveQueue(): BufferResult {
  return VULFRAM_CORE.vulframReceiveQueue();
}

export function vulframReceiveEvents(): BufferResult {
  return VULFRAM_CORE.vulframReceiveEvents();
}

export function vulframSendQueue(data: Buffer): number {
  return VULFRAM_CORE.vulframSendQueue(data);
}

export function vulframTick(time: number, deltaTime: number): number {
  return VULFRAM_CORE.vulframTick(time, deltaTime);
}

export function vulframUploadBuffer(
  id: number,
  uploadType: number,
  data: Buffer,
): number {
  return VULFRAM_CORE.vulframUploadBuffer(id, uploadType, data);
}

export function vulframGetProfiling(): BufferResult {
  return VULFRAM_CORE.vulframGetProfiling();
}
