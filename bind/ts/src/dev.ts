/**
 * Complete Rendering Example
 *
 * This example demonstrates how to use the complete rendering API
 * to create a simple 3D scene with a colored cube.
 */

import { mat4 } from 'gl-matrix';
import {
  // FFI functions
  vulframInit,
  vulframUploadBuffer,
  vulframSendQueue,
  vulframTick,
  vulframReceiveQueue,
  vulframReceiveEvents,
  vulframDispose,
  // Enums
  VulframResult,
  UploadType,
  TextureFormat,
  VertexFormat,
  IndexFormat,
  PrimitiveTopology,
  // Commands
  type CmdShaderCreateArgs,
  type CmdGeometryCreateArgs,
  type CmdMaterialCreateArgs,
  type CmdCameraCreateArgs,
  type CmdModelCreateArgs,
  type EngineBatchCmds,
  // Helpers - Material
  createDefaultPrimitiveState,
  // Helpers - Viewport
  createFullscreenViewport,
  // Helpers - Layers
  LAYER_WORLD,
  // Helpers - Uniforms
  uniformVec4,
  uniformMat4,
  uniformColorHex,
  KeyCode,
} from './index';

// MARK: Initialize Engine

console.log('Initializing Vulfram...');
const initResult = vulframInit();
if (initResult !== VulframResult.Success) {
  console.error('Failed to initialize:', VulframResult[initResult]);
  process.exit(1);
}
console.log('✓ Engine initialized');

// MARK: Create Window

let nextCmdId = 1;
let commands: EngineBatchCmds = [];

// Create window
commands.push({
  id: nextCmdId++,
  type: 'cmd-window-create',
  content: {
    title: 'Vulfram Rendering Example',
    size: [1280, 720],
    position: [100, 100],
    borderless: false,
    resizable: true,
    initialState: 'windowed' as const,
  },
});

// Send window creation command
vulframSendQueue(commands);

// Tick to process the command
vulframTick(Date.now(), 0);

// Get window ID from response
const [responses] = vulframReceiveQueue();
const windowResponse = responses[0];
if (
  !windowResponse ||
  windowResponse.type !== 'window-create' ||
  !windowResponse.content.success
) {
  console.error('Failed to create window:', windowResponse?.content.message);
  vulframDispose();
  process.exit(1);
}
const windowId = windowResponse.content.content; // Window ID
console.log(`✓ Window created with ID: ${windowId}`);

// MARK: Upload Shader Source

const shaderSource = /* wgsl */ `
// Vertex shader
struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec3<f32>,
};

struct CameraUniforms {
    proj: mat4x4<f32>,
    view: mat4x4<f32>,
};

struct ModelUniforms {
    model: mat4x4<f32>,
};

@group(0) @binding(0)
var<uniform> camera: CameraUniforms;

@group(2) @binding(0)
var<uniform> model: ModelUniforms;

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    let world_pos = model.model * vec4<f32>(in.position, 1.0);
    let view_pos = camera.view * world_pos;
    out.position = camera.proj * view_pos;
    out.color = in.color;
    return out;
}

// Fragment shader
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(in.color, 1.0);
}
`;

const shaderBufferId = 1;
vulframUploadBuffer(
  shaderBufferId,
  UploadType.ShaderSource,
  Buffer.from(shaderSource, 'utf-8'),
);
console.log('✓ Shader source uploaded');

// MARK: Create Shader

const shaderId = 1;
const shaderCmd: CmdShaderCreateArgs = {
  shaderId,
  windowId,
  bufferId: shaderBufferId,
  label: 'ColoredCubeShader',
  uniformBuffers: [
    {
      group: 0,
      binding: 0,
      fields: [
        { name: 'proj', uniformType: 'mat4x4' },
        { name: 'view', uniformType: 'mat4x4' },
      ],
    },
    {
      group: 2,
      binding: 0,
      fields: [{ name: 'model', uniformType: 'mat4x4' }],
    },
  ],
  textureBindings: [],
  storageBuffers: [],
  vertexAttributes: [
    { location: 0, semantic: 'position', format: VertexFormat.Float32x3 },
    { location: 1, semantic: 'color0', format: VertexFormat.Float32x3 },
  ],
};

commands = [];
commands.push({
  id: nextCmdId++,
  type: 'cmd-shader-create',
  content: shaderCmd,
});

vulframSendQueue(commands);
vulframTick(Date.now(), 0);

// Check shader creation response
const [shaderResponses] = vulframReceiveQueue();
if (shaderResponses.length > 0) {
  const shaderResp = shaderResponses[0]!;
  console.log('Shader response:', shaderResp);
  if (shaderResp.type === 'shader-create' && !shaderResp.content.success) {
    console.error('Shader creation failed:', shaderResp.content.message);
  }
}
console.log('✓ Shader created');

// MARK: Upload Geometry Data

// Cube vertices (position + color)
const vertices = new Float32Array([
  // Front face (red)
  -1, -1, 1, 1, 0, 0, 1, -1, 1, 1, 0, 0, 1, 1, 1, 1, 0, 0, -1, 1, 1, 1, 0, 0,
  // Back face (green)
  -1, -1, -1, 0, 1, 0, 1, -1, -1, 0, 1, 0, 1, 1, -1, 0, 1, 0, -1, 1, -1, 0, 1,
  0,
  // Top face (blue)
  -1, 1, -1, 0, 0, 1, 1, 1, -1, 0, 0, 1, 1, 1, 1, 0, 0, 1, -1, 1, 1, 0, 0, 1,
  // Bottom face (yellow)
  -1, -1, -1, 1, 1, 0, 1, -1, -1, 1, 1, 0, 1, -1, 1, 1, 1, 0, -1, -1, 1, 1, 1,
  0,
  // Right face (cyan)
  1, -1, -1, 0, 1, 1, 1, 1, -1, 0, 1, 1, 1, 1, 1, 0, 1, 1, 1, -1, 1, 0, 1, 1,
  // Left face (magenta)
  -1, -1, -1, 1, 0, 1, -1, 1, -1, 1, 0, 1, -1, 1, 1, 1, 0, 1, -1, -1, 1, 1, 0,
  1,
]);

// Cube indices
const indices = new Uint16Array([
  0,
  1,
  2,
  0,
  2,
  3, // Front
  4,
  5,
  6,
  4,
  6,
  7, // Back
  8,
  9,
  10,
  8,
  10,
  11, // Top
  12,
  13,
  14,
  12,
  14,
  15, // Bottom
  16,
  17,
  18,
  16,
  18,
  19, // Right
  20,
  21,
  22,
  20,
  22,
  23, // Left
]);

const vertexBufferId = 2;
const indexBufferId = 3;

vulframUploadBuffer(
  vertexBufferId,
  UploadType.VertexData,
  Buffer.from(vertices.buffer),
);
vulframUploadBuffer(
  indexBufferId,
  UploadType.IndexData,
  Buffer.from(indices.buffer),
);
console.log('✓ Geometry data uploaded');

// MARK: Create Geometry

const geometryId = 1;
const geometryCmd: CmdGeometryCreateArgs = {
  geometryId,
  windowId,
  vertexBufferId,
  indexBufferId,
  vertexCount: 24,
  indexCount: 36,
  vertexAttributes: [
    { format: VertexFormat.Float32x3, offset: 0, shaderLocation: 0 }, // position
    { format: VertexFormat.Float32x3, offset: 12, shaderLocation: 1 }, // color
  ],
  indexFormat: IndexFormat.Uint16,
  label: 'ColoredCube',
};

commands = [];
commands.push({
  id: nextCmdId++,
  type: 'cmd-geometry-create',
  content: geometryCmd,
});

vulframSendQueue(commands);
vulframTick(Date.now(), 0);

const [geometryResponses] = vulframReceiveQueue();
if (geometryResponses.length > 0) {
  const geoResp = geometryResponses[0]!;
  console.log('Geometry response:', geoResp);
  if (geoResp.type === 'geometry-create' && !geoResp.content.success) {
    console.error('Geometry creation failed:', geoResp.content.message);
  }
}
console.log('✓ Geometry created');

// MARK: Create Material

const materialId = 1;
const materialCmd: CmdMaterialCreateArgs = {
  materialId,
  windowId,
  shaderId,
  textures: [],
  primitive: createDefaultPrimitiveState(),
  label: 'DefaultMaterial',
};

commands = [];
commands.push({
  id: nextCmdId++,
  type: 'cmd-material-create',
  content: materialCmd,
});

vulframSendQueue(commands);
vulframTick(Date.now(), 0);

const [materialResponses] = vulframReceiveQueue();
if (materialResponses.length > 0) {
  const matResp = materialResponses[0]!;
  console.log('Material response:', matResp);
  if (matResp.type === 'material-create' && !matResp.content.success) {
    console.error('Material creation failed:', matResp.content.message);
  }
}
console.log('✓ Material created');

// MARK: Create Camera

const cameraId = 100;
const aspect = 1280 / 720;
const projMat = mat4.perspective(
  mat4.create(),
  (60 * Math.PI) / 180, // FOV em radianos
  aspect,
  0.1,
  1000,
);
const viewMat = mat4.lookAt(
  mat4.create(),
  [0, 2, 5], // eye
  [0, 0, 0], // target
  [0, 1, 0], // up
);

const cameraCmd: CmdCameraCreateArgs = {
  componentId: cameraId,
  windowId,
  projMat,
  viewMat,
  viewport: createFullscreenViewport(),
  layerMask: LAYER_WORLD,
};

commands = [];
commands.push({
  id: nextCmdId++,
  type: 'cmd-camera-create',
  content: cameraCmd,
});

vulframSendQueue(commands);
vulframTick(Date.now(), 0);

const [cameraResponses] = vulframReceiveQueue();
if (cameraResponses.length > 0) {
  const camResp = cameraResponses[0]!;
  console.log('Camera response:', camResp);
  if (camResp.type === 'camera-create' && !camResp.content.success) {
    console.error('Camera creation failed:', camResp.content.message);
  }
}
console.log('✓ Camera created');

// MARK: Create Model (rotating cube)

const modelId = 200;
let rotation = 0;

const modelCmd: CmdModelCreateArgs = {
  componentId: modelId,
  windowId,
  geometryId,
  materialId,
  modelMat: mat4.create(), // Identity matrix
  layerMask: LAYER_WORLD,
};

commands = [];
commands.push({
  id: nextCmdId++,
  type: 'cmd-model-create',
  content: modelCmd,
});

vulframSendQueue(commands);
vulframTick(Date.now(), 0);

const [modelResponses] = vulframReceiveQueue();
if (modelResponses.length > 0) {
  const modelResp = modelResponses[0]!;
  console.log('Model response:', modelResp);
  if (modelResp.type === 'model-create' && !modelResp.content.success) {
    console.error('Model creation failed:', modelResp.content.message);
  }
}
console.log('✓ Model created');

// MARK: Main Loop

console.log('\n🎮 Starting render loop... (Press Ctrl+C to exit)\n');

let running = true;
let lastTime = Date.now();

process.on('SIGINT', () => {
  console.log('\n\n👋 Shutting down...');
  running = false;
  setTimeout(() => {
    vulframDispose();
    process.exit(0);
  }, 100);
});

while (running) {
  const currentTime = Date.now();
  const deltaTime = (currentTime - lastTime) / 1000;
  lastTime = currentTime;

  // Update rotation
  rotation += deltaTime;

  // Update model matrix (rotation around Y axis)
  const modelMat = mat4.create();
  mat4.rotateY(modelMat, modelMat, rotation);

  commands = [];
  commands.push({
    id: nextCmdId++,
    type: 'cmd-model-update',
    content: {
      componentId: modelId,
      windowId,
      modelMat,
    },
  });

  vulframSendQueue(commands);

  // Tick the engine
  vulframTick(currentTime, Math.floor(deltaTime * 1000));

  // Process events
  const [events] = vulframReceiveEvents();
  if (events.length > 0) {
    console.log(
      `Received ${events.length} events:`,
      events.map((e) => e.type),
    );
    for (const event of events) {
      if (
        event.type === 'window' &&
        event.content.event === 'on-close-request'
      ) {
        console.log('Window close requested');
        running = false;
      }

      if (event.type === 'keyboard' && event.content.event === 'on-input') {
        const keyEvent = event.content.data;
        if (keyEvent.keyCode === KeyCode.KeyC && keyEvent.modifiers.ctrl) {
          running = false;
        }
      }
    }
  }

  // Sleep to maintain ~60 FPS
  await Bun.sleep(16);
}

// MARK: Cleanup

console.log('Cleaning up...');
vulframDispose();
console.log('✓ Done');
