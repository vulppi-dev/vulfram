import init, {
  BufferResult,
  vulfram_init,
  vulfram_receive_events,
  vulfram_receive_queue,
  vulfram_send_queue,
  vulfram_tick,
} from './pkg/vulfram_core.js';
import { Packr, Unpackr } from 'msgpackr';
import { mat4 } from 'gl-matrix';

const logEl = document.getElementById('log');
const packr = new Packr({ useRecords: false });
const unpackr = new Unpackr({ useRecords: false });
const WINDOW_ID = 1;
const CAMERA_ID = 1;
const LIGHT_ID = 2;
const MATERIAL_ID = 10;
const TEXTURE_ID = 20;
const GEOMETRY_CUBE_ID = 1;
const RUN_DURATION_MS = 5_000;

function log(message) {
  logEl.textContent += `\n${message}`;
}

function consumeBufferResult(bufferResult) {
  if (!(bufferResult instanceof BufferResult)) {
    return { result: 1, buffer: new Uint8Array() };
  }
  try {
    return { result: bufferResult.result, buffer: bufferResult.buffer };
  } finally {
    bufferResult.free();
  }
}

function decodeBatch(buffer) {
  if (!buffer || buffer.length === 0) {
    return [];
  }
  try {
    return unpackr.unpack(buffer);
  } catch (err) {
    log(`Decode error: ${err.message}`);
    return [];
  }
}

let nextCmdId = 1;
function sendCommands(commands, label) {
  const envelopes = commands.map((cmd) => ({ id: nextCmdId++, ...cmd }));
  const payload = packr.pack(envelopes);
  const result = vulfram_send_queue(payload);
  if (result !== 0) {
    log(`vulfram_send_queue${label ? ` (${label})` : ''} -> ${result}`);
  }
  return result;
}

let windowCreated = false;
function pollResponses() {
  const responseResult = consumeBufferResult(vulfram_receive_queue());
  if (responseResult.result !== 0) {
    log(`receive_queue error -> ${responseResult.result}`);
    return;
  }
  const batch = decodeBatch(responseResult.buffer);
  if (batch.length > 0) {
    for (const res of batch) {
      if (res.type === 'window-create') {
        if (res.content?.success) {
          windowCreated = true;
          log('Window created.');
        } else {
          log(`Window create failed: ${res.content?.message ?? 'unknown error'}`);
        }
      }
    }
  }
}

function pollEvents() {
  const eventResult = consumeBufferResult(vulfram_receive_events());
  if (eventResult.result !== 0) {
    log(`receive_events error -> ${eventResult.result}`);
    return;
  }
  decodeBatch(eventResult.buffer);
}

function createCameraWorldTransform(eye, center) {
  const view = mat4.create();
  mat4.lookAt(view, eye, center, [0, 1, 0]);
  const world = mat4.create();
  mat4.invert(world, view);
  return Array.from(world);
}

function buildTransform(position, rotation, scale) {
  const transform = mat4.create();
  mat4.fromTranslation(transform, position);
  mat4.rotateX(transform, transform, rotation[0]);
  mat4.rotateY(transform, transform, rotation[1]);
  mat4.rotateZ(transform, transform, rotation[2]);
  mat4.scale(transform, transform, scale);
  return Array.from(transform);
}

async function startDemo() {
  if (!navigator.gpu) {
    log('WebGPU not supported in this browser.');
    return;
  }
  logEl.textContent = 'Loading wasm...';
  await init();
  log('WASM loaded.');

  const initResult = vulfram_init();
  log(`vulfram_init -> ${initResult}`);

  sendCommands(
    [
      {
        type: 'cmd-window-create',
        content: {
          windowId: WINDOW_ID,
          title: 'Vulfram WASM',
          size: [960, 540],
          canvasId: 'vulfram-canvas',
          resizable: true,
        },
      },
    ],
    'window-create',
  );

  let sceneSetup = false;
  let startTime = performance.now();
  let lastTime = startTime;

  function setupScene() {
    if (sceneSetup || !windowCreated) {
      return;
    }
    sceneSetup = true;
    log('Setting up scene...');

    sendCommands(
      [
        {
          type: 'cmd-primitive-geometry-create',
          content: {
            windowId: WINDOW_ID,
            geometryId: GEOMETRY_CUBE_ID,
            label: 'Cube',
            shape: 'cube',
            options: {
              type: 'cube',
              content: {
                size: [1.0, 1.0, 1.0],
                subdivisions: 1,
              },
            },
          },
        },
      ],
      'geometry',
    );

    sendCommands(
      [
        {
          type: 'cmd-texture-create-solid-color',
          content: {
            windowId: WINDOW_ID,
            textureId: TEXTURE_ID,
            label: 'Solid White',
            color: [1, 1, 1, 1],
            srgb: true,
          },
        },
      ],
      'texture',
    );

    sendCommands(
      [
        {
          type: 'cmd-material-create',
          content: {
            windowId: WINDOW_ID,
            materialId: MATERIAL_ID,
            label: 'Standard',
            kind: 'standard',
            options: {
              type: 'standard',
              content: {
                baseColor: [1, 1, 1, 1],
                surfaceType: 'Opaque',
                baseTexId: TEXTURE_ID,
                baseSampler: 'pointClamp',
                flags: 0,
              },
            },
          },
        },
      ],
      'material',
    );

    sendCommands(
      [
        {
          type: 'cmd-camera-create',
          content: {
            windowId: WINDOW_ID,
            cameraId: CAMERA_ID,
            label: 'Main Camera',
            transform: createCameraWorldTransform([0, 10, 15], [0, 0, 0]),
            kind: 0,
            flags: 0,
            nearFar: [0.1, 100.0],
            layerMask: 0x7fffffff,
            order: 0,
            viewPosition: null,
            orthoScale: 1.0,
          },
        },
        {
          type: 'cmd-light-create',
          content: {
            windowId: WINDOW_ID,
            lightId: LIGHT_ID,
            label: 'Point Light',
            kind: 1,
            position: [0, 8, 0, 1],
            direction: null,
            color: [1, 1, 1, 1],
            groundColor: null,
            intensity: 20.0,
            range: 30.0,
            spotInnerOuter: null,
            layerMask: 0x7fffffff,
            castShadow: true,
          },
        },
      ],
      'camera-light',
    );

    sendCommands(
      [
        {
          type: 'cmd-model-create',
          content: {
            windowId: WINDOW_ID,
            modelId: 1,
            label: 'Cube',
            geometryId: GEOMETRY_CUBE_ID,
            materialId: MATERIAL_ID,
            transform: buildTransform([0, 0, 0], [0, 0, 0], [1, 1, 1]),
            layerMask: 0x7fffffff,
            castShadow: true,
            receiveShadow: true,
          },
        },
        {
          type: 'cmd-model-create',
          content: {
            windowId: WINDOW_ID,
            modelId: 2,
            label: 'Floor',
            geometryId: GEOMETRY_CUBE_ID,
            materialId: MATERIAL_ID,
            transform: buildTransform([0, -6, 0], [0, 0, 0], [20, 0.1, 20]),
            layerMask: 0x7fffffff,
            castShadow: false,
            receiveShadow: true,
          },
        },
      ],
      'models',
    );

    sendCommands(
      [
        {
          type: 'cmd-shadow-configure',
          content: {
            windowId: WINDOW_ID,
            config: {
              tile_resolution: 512,
              atlas_tiles_w: 16,
              atlas_tiles_h: 16,
              atlas_layers: 2,
              virtual_grid_size: 1,
              smoothing: 2,
            },
          },
        },
      ],
      'shadow',
    );
  }

  function frame(now) {
    const delta = Math.max(1, Math.floor(now - lastTime));
    lastTime = now;
    const tickResult = vulfram_tick(Math.floor(now), delta);
    if (tickResult !== 0) {
      log(`vulfram_tick -> ${tickResult}`);
    }
    pollResponses();
    pollEvents();
    setupScene();

    if (sceneSetup) {
      const updateCmds = [
        {
          type: 'cmd-gizmo-draw-line',
          content: {
            start: [0, 0, 0],
            end: [5, 0, 0],
            color: [1, 0, 0, 1],
          },
        },
        {
          type: 'cmd-gizmo-draw-line',
          content: {
            start: [0, 0, 0],
            end: [0, 5, 0],
            color: [0, 1, 0, 1],
          },
        },
        {
          type: 'cmd-gizmo-draw-line',
          content: {
            start: [0, 0, 0],
            end: [0, 0, 5],
            color: [0, 0, 1, 1],
          },
        },
        {
          type: 'cmd-gizmo-draw-aabb',
          content: {
            min: [-5, -5, -5],
            max: [5, 5, 5],
            color: [1, 1, 1, 0.2],
          },
        },
      ];
      sendCommands(updateCmds);
    }

    if (now - startTime < RUN_DURATION_MS) {
      requestAnimationFrame(frame);
    } else {
      sendCommands(
        [
          {
            type: 'cmd-window-close',
            content: { windowId: WINDOW_ID },
          },
        ],
        'close',
      );
      pollResponses();
      pollEvents();
      log('Demo finished.');
    }
  }

  requestAnimationFrame(frame);
}

startDemo().catch((err) => {
  log(`Error: ${err.message}`);
  console.error(err);
});
