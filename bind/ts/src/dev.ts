/**
 * Complete Rendering Example
 *
 * This example demonstrates how to use the complete rendering API
 * to create a simple 3D scene with a colored cube.
 */

import { mat4 } from 'gl-matrix';
import { WindowState } from './enums';
import {
  // FFI functions
  vulframInit,
  vulframUploadBuffer,
  vulframSendQueue,
  vulframTick,
  vulframReceiveQueue,
  vulframReceiveEvents,
  vulframDispose,
  startLoop,
  // Enums
  VulframResult,
  UploadType,
  TextureFormat,
  VertexFormat,
  IndexFormat,
  PrimitiveTopology,
  UniformType,
  VertexSemantic,
  // Commands
  type EngineCmdEnvelope,
  type CmdShaderCreateArgs,
  type CmdGeometryCreateArgs,
  type CmdMaterialCreateArgs,
  type CmdCameraCreateArgs,
  type CmdModelCreateArgs,
  type EngineBatchCmds,
  // Helpers - Material
  createDefaultPrimitiveState,
  createDoubleSidedPrimitiveState,
  // Helpers - Viewport
  createFullscreenViewport,
  // Helpers - Layers
  LAYER_WORLD,
  // Helpers - Uniforms
  uniformVec4,
  uniformMat4,
  uniformColorHex,
  // Helpers - Matrix conversion
  mat4ToArray,
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
    initialState: WindowState.Windowed,
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
    camera_view_projection: mat4x4<f32>,
};

struct ModelUniforms {
    model_transform: mat4x4<f32>,
};

@group(0) @binding(0)
var<uniform> camera: CameraUniforms;

@group(1) @binding(0)
var<uniform> model: ModelUniforms;

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    let world_pos = model.model_transform * vec4<f32>(in.position, 1.0);
    out.position = camera.camera_view_projection * world_pos;
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
      fields: [{ name: 'camera_view_projection', type: UniformType.Mat4x4 }],
    },
    {
      group: 1,
      binding: 0,
      fields: [{ name: 'model_transform', type: UniformType.Mat4x4 }],
    },
  ],
  textureBindings: [],
  samplerBindings: [],
  storageBuffers: [],
  vertexAttributes: [
    {
      location: 0,
      semantic: VertexSemantic.Position,
      format: VertexFormat.Float32x3,
    },
    {
      location: 1,
      semantic: VertexSemantic.Color0,
      format: VertexFormat.Float32x3,
    },
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
  if (shaderResp.type === 'shader-create' && !shaderResp.content.success) {
    console.error('Shader creation failed:', shaderResp.content.message);
  }
}
console.log('✓ Shader created');

// MARK: Upload Geometry Data

// Cube vertices (position + color)
const vertices = new Float32Array([
  // Front face (Red)
  -0.5,
  -0.5,
  0.5,
  1.0,
  0.0,
  0.0, // 0
  0.5,
  -0.5,
  0.5,
  1.0,
  0.0,
  0.0, // 1
  0.5,
  0.5,
  0.5,
  1.0,
  0.0,
  0.0, // 2
  -0.5,
  0.5,
  0.5,
  1.0,
  0.0,
  0.0, // 3

  // Back face (Green)
  -0.5,
  -0.5,
  -0.5,
  0.0,
  1.0,
  0.0, // 4
  0.5,
  -0.5,
  -0.5,
  0.0,
  1.0,
  0.0, // 5
  0.5,
  0.5,
  -0.5,
  0.0,
  1.0,
  0.0, // 6
  -0.5,
  0.5,
  -0.5,
  0.0,
  1.0,
  0.0, // 7

  // Top face (Blue)
  -0.5,
  0.5,
  -0.5,
  0.0,
  0.0,
  1.0, // 8
  -0.5,
  0.5,
  0.5,
  0.0,
  0.0,
  1.0, // 9
  0.5,
  0.5,
  0.5,
  0.0,
  0.0,
  1.0, // 10
  0.5,
  0.5,
  -0.5,
  0.0,
  0.0,
  1.0, // 11

  // Bottom face (Yellow)
  -0.5,
  -0.5,
  -0.5,
  1.0,
  1.0,
  0.0, // 12
  0.5,
  -0.5,
  -0.5,
  1.0,
  1.0,
  0.0, // 13
  0.5,
  -0.5,
  0.5,
  1.0,
  1.0,
  0.0, // 14
  -0.5,
  -0.5,
  0.5,
  1.0,
  1.0,
  0.0, // 15

  // Right face (Magenta)
  0.5,
  -0.5,
  -0.5,
  1.0,
  0.0,
  1.0, // 16
  0.5,
  0.5,
  -0.5,
  1.0,
  0.0,
  1.0, // 17
  0.5,
  0.5,
  0.5,
  1.0,
  0.0,
  1.0, // 18
  0.5,
  -0.5,
  0.5,
  1.0,
  0.0,
  1.0, // 19

  // Left face (Cyan)
  -0.5,
  -0.5,
  -0.5,
  0.0,
  1.0,
  1.0, // 20
  -0.5,
  -0.5,
  0.5,
  0.0,
  1.0,
  1.0, // 21
  -0.5,
  0.5,
  0.5,
  0.0,
  1.0,
  1.0, // 22
  -0.5,
  0.5,
  -0.5,
  0.0,
  1.0,
  1.0, // 23
]);

// Cube indices (2 triangles per face, 6 faces)
const indices = new Uint16Array([
  // Front face
  0, 1, 2, 2, 3, 0,
  // Back face
  4, 6, 5, 6, 4, 7,
  // Top face
  8, 9, 10, 10, 11, 8,
  // Bottom face
  12, 13, 14, 14, 15, 12,
  // Right face
  16, 17, 18, 18, 19, 16,
  // Left face
  20, 21, 22, 22, 23, 20,
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
  label: 'Cube',
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
  samplers: [],
  primitive: createDoubleSidedPrimitiveState(),
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
  (45 * Math.PI) / 180, // 45° FOV (matching Rust)
  aspect,
  0.1,
  100.0, // Far plane matching Rust
);
const viewMat = mat4.lookAt(
  mat4.create(),
  [0, 0, 2], // eye - matching main.rs distance
  [0, 0, 0], // target
  [0, 1, 0], // up
);

// Convert Float32Array to plain arrays for MessagePack serialization
const projMatArray = mat4ToArray(projMat);
const viewMatArray = mat4ToArray(viewMat);

const cameraCmd: CmdCameraCreateArgs = {
  componentId: cameraId,
  windowId,
  projMat: projMatArray as any,
  viewMat: viewMatArray as any,
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
  if (camResp.type === 'camera-create' && !camResp.content.success) {
    console.error('Camera creation failed:', camResp.content.message);
  }
}
console.log('✓ Camera created');
const modelId = 200;
let rotation = 0;

const initialModelMat = mat4.create();
const initialModelMatArray = mat4ToArray(initialModelMat);

const modelCmd: CmdModelCreateArgs = {
  componentId: modelId,
  windowId,
  geometryId,
  materialId,
  modelMat: initialModelMatArray as any, // Identity matrix
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
  if (modelResp.type === 'model-create' && !modelResp.content.success) {
    console.error('Model creation failed:', modelResp.content.message);
  }
}
console.log('✓ Model created');

// MARK: Main Loop

console.log('\n🎮 Starting render loop... (Ctrl+C to exit)\n');

let lastTime = Date.now();
const startTime = Date.now();
const AUTO_CLOSE_MS = 50000; // 50 seconds for auto-close

let stopLoop: (() => void) | null = null;

process.on('SIGINT', () => {
  console.log('\n\n👋 Shutting down...');
  if (stopLoop) stopLoop();
  setTimeout(() => {
    vulframDispose();
    process.exit(0);
  }, 100);
});

stopLoop = startLoop(() => {
  const currentTime = Date.now();
  const deltaTime = (currentTime - lastTime) / 1000;
  lastTime = currentTime;

  // Auto-close after 5 seconds
  if (currentTime - startTime >= AUTO_CLOSE_MS) {
    console.log('\n⏱️ 5 seconds elapsed, closing...');
    if (stopLoop) stopLoop();
    vulframDispose();
    process.exit(0);
    return;
  }

  // Update rotation
  rotation += deltaTime;

  // Update model matrix (rotation around Y axis)
  const modelMat = mat4.create();
  mat4.rotateY(modelMat, modelMat, rotation);
  const modelMatArray = mat4ToArray(modelMat);

  commands = [];
  commands.push({
    id: nextCmdId++,
    type: 'cmd-model-update',
    content: {
      componentId: modelId,
      windowId,
      modelMat: modelMatArray as any,
    },
  });

  vulframSendQueue(commands);

  // Tick the engine
  vulframTick(currentTime, Math.floor(deltaTime * 1000));

  // Process events
  const [events] = vulframReceiveEvents();
  if (events.length > 0) {
    for (const event of events) {
      if (
        event.type === 'window' &&
        event.content.event === 'on-close-request'
      ) {
        if (stopLoop) stopLoop();
        vulframDispose();
        process.exit(0);
      }

      if (event.type === 'keyboard' && event.content.event === 'on-input') {
        const keyEvent = event.content.data;
        if (keyEvent.keyCode === KeyCode.KeyC && keyEvent.modifiers.ctrl) {
          if (stopLoop) stopLoop();
          vulframDispose();
          process.exit(0);
        }
      }
    }
  }
});
