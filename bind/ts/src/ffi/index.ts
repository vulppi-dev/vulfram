import dll from './vulfram_core.dll' with { type: 'file' };
import { dlopen, ptr, toBuffer, toArrayBuffer, type Pointer } from 'bun:ffi';

const { symbols: VULFRAM_CORE, close } = dlopen(dll, {
  vulfram_init: { args: [], returns: 'u32' },
  vulfram_dispose: { args: [], returns: 'u32' },
  vulfram_send_queue: { args: ['ptr', 'usize'], returns: 'u32' },
  vulfram_receive_queue: { args: ['ptr', 'ptr'], returns: 'u32' },
  vulfram_receive_events: { args: ['ptr', 'ptr'], returns: 'u32' },
  vulfram_upload_buffer: {
    args: ['u64', 'u32', 'ptr', 'usize'],
    returns: 'u32',
  },
  vulfram_download_buffer: { args: ['u64', 'ptr', 'ptr'], returns: 'u32' },
  vulfram_tick: { args: ['u64', 'u32'], returns: 'u32' },
  vulfram_get_profiling: { args: ['ptr', 'ptr'], returns: 'u32' },
});

process.once('beforeExit', () => {
  close();
});

export interface BufferResult {
  buffer: Buffer;
  result: number;
}

export function vulframDispose(): number {
  return VULFRAM_CORE.vulfram_dispose();
}

export function vulframDownloadBuffer(id: number): BufferResult {
  const ptrHolder = new BigUint64Array(1);
  const sizeHolder = new BigUint64Array(1);
  const result = VULFRAM_CORE.vulfram_download_buffer(
    BigInt(id),
    ptr(ptrHolder),
    ptr(sizeHolder),
  );
  if (!sizeHolder[0]) {
    return { buffer: Buffer.alloc(0), result };
  }
  const srcPtr = Number(ptrHolder[0]) as Pointer;
  if (!srcPtr) {
    return { buffer: Buffer.alloc(0), result };
  }
  const buffer = Buffer.from(toArrayBuffer(srcPtr, 0, Number(sizeHolder[0])));

  return { buffer, result };
}

export function vulframInit(): number {
  return VULFRAM_CORE.vulfram_init();
}

export function vulframReceiveQueue(): BufferResult {
  const ptrHolder = new BigUint64Array(1);
  const sizeHolder = new BigUint64Array(1);
  const result = VULFRAM_CORE.vulfram_receive_queue(
    ptr(ptrHolder),
    ptr(sizeHolder),
  );
  if (!sizeHolder[0]) {
    return { buffer: Buffer.alloc(0), result };
  }
  const srcPtr = Number(ptrHolder[0]) as Pointer;
  if (!srcPtr) {
    return { buffer: Buffer.alloc(0), result };
  }
  const buffer = Buffer.from(toArrayBuffer(srcPtr, 0, Number(sizeHolder[0])));

  return { buffer, result };
}

export function vulframReceiveEvents(): BufferResult {
  const ptrHolder = new BigUint64Array(1);
  const sizeHolder = new BigUint64Array(1);
  const result = VULFRAM_CORE.vulfram_receive_events(
    ptr(ptrHolder),
    ptr(sizeHolder),
  );
  if (!sizeHolder[0]) {
    return { buffer: Buffer.alloc(0), result };
  }
  const srcPtr = Number(ptrHolder[0]) as Pointer;
  if (!srcPtr) {
    return { buffer: Buffer.alloc(0), result };
  }
  const buffer = Buffer.from(toArrayBuffer(srcPtr, 0, Number(sizeHolder[0])));

  return { buffer, result };
}

export function vulframSendQueue(data: Buffer): number {
  return VULFRAM_CORE.vulfram_send_queue(ptr(data), data.length);
}

export function vulframTick(time: number, deltaTime: number): number {
  return VULFRAM_CORE.vulfram_tick(time, deltaTime);
}

export function vulframUploadBuffer(
  id: number,
  uploadType: number,
  data: Buffer,
): number {
  return VULFRAM_CORE.vulfram_upload_buffer(
    BigInt(id),
    uploadType,
    ptr(data),
    data.length,
  );
}

export function vulframGetProfiling(): BufferResult {
  const ptrHolder = new BigUint64Array(1);
  const sizeHolder = new BigUint64Array(1);
  const result = VULFRAM_CORE.vulfram_get_profiling(
    ptr(ptrHolder),
    ptr(sizeHolder),
  );
  if (!sizeHolder[0]) {
    return { buffer: Buffer.alloc(0), result };
  }
  const srcPtr = Number(ptrHolder[0]) as Pointer;
  if (!srcPtr) {
    return { buffer: Buffer.alloc(0), result };
  }
  const buffer = Buffer.from(toArrayBuffer(srcPtr, 0, Number(sizeHolder[0])));

  return { buffer, result };
}
