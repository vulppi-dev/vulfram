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
      fields: [{ name: 'camera_view_projection', uniformType: 'mat4x4' }],
    },
    {
      group: 1,
      binding: 0,
      fields: [{ name: 'model_transform', uniformType: 'mat4x4' }],
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

// Simple triangle vertices (position + color) - matching main.rs
const vertices = new Float32Array([
  // position        color
  0.0,
  0.5,
  0.0,
  1.0,
  0.0,
  0.0, // Top - Red
  -0.5,
  -0.5,
  0.0,
  0.0,
  1.0,
  0.0, // Bottom Left - Green
  0.5,
  -0.5,
  0.0,
  0.0,
  0.0,
  1.0, // Bottom Right - Blue
]);

// Triangle indices
const indices = new Uint16Array([0, 1, 2]);

console.log('\n🔍 Geometry data:');
console.log('Vertices:', vertices);
console.log('Indices:', indices);
console.log('Vertex buffer size:', vertices.byteLength, 'bytes');
console.log('Index buffer size:', indices.byteLength, 'bytes');

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
  vertexCount: 3,
  indexCount: 3,
  vertexAttributes: [
    { format: VertexFormat.Float32x3, offset: 0, shaderLocation: 0 }, // position
    { format: VertexFormat.Float32x3, offset: 12, shaderLocation: 1 }, // color
  ],
  indexFormat: IndexFormat.Uint16,
  label: 'Triangle',
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
  primitive: createDoubleSidedPrimitiveState(), // No culling to ensure triangle is visible
  label: 'DefaultMaterial',
};

console.log('\n🔍 Material command:', JSON.stringify(materialCmd, null, 2));

commands = [];
const materialCommand = {
  id: nextCmdId++,
  type: 'cmd-material-create',
  content: materialCmd,
};
commands.push(materialCommand);

console.log(
  '🔍 Full material command:',
  JSON.stringify(materialCommand, null, 2),
);

vulframSendQueue(commands);
vulframTick(Date.now(), 0);

const [materialResponses] = vulframReceiveQueue();
console.log('Material responses count:', materialResponses.length);
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

// Debug: print matrices
console.log('\n🔍 Camera matrices:');
console.log('Projection matrix:', projMatArray);
console.log('View matrix:', viewMatArray);

const cameraCmd: CmdCameraCreateArgs = {
  componentId: cameraId,
  windowId,
  projMat: projMatArray as any, // Convert to plain array
  viewMat: viewMatArray as any, // Convert to plain array
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

const initialModelMat = mat4.create();
const initialModelMatArray = mat4ToArray(initialModelMat);
console.log('\n🔍 Initial model matrix (identity):', initialModelMatArray);

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
