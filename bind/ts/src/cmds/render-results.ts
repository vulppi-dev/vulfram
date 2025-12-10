// MARK: Command Results Base

interface CmdResultBase {
  success: boolean;
  message: string;
}

// MARK: Shader Results

export interface CmdResultShaderCreate extends CmdResultBase {}

export interface CmdResultShaderDispose extends CmdResultBase {}

// MARK: Geometry Results

export interface CmdResultGeometryCreate extends CmdResultBase {}

export interface CmdResultGeometryDispose extends CmdResultBase {}

// MARK: Material Results

export interface CmdResultMaterialCreate extends CmdResultBase {}

export interface CmdResultMaterialUpdate extends CmdResultBase {}

export interface CmdResultMaterialDispose extends CmdResultBase {}

// MARK: Texture Results

export interface CmdResultTextureCreate extends CmdResultBase {}

export interface CmdResultTextureUpdate extends CmdResultBase {}

export interface CmdResultTextureDispose extends CmdResultBase {}

// MARK: Sampler Results

export interface CmdResultSamplerCreate extends CmdResultBase {}

export interface CmdResultSamplerUpdate extends CmdResultBase {}

export interface CmdResultSamplerDispose extends CmdResultBase {}

// MARK: Camera Results

export interface CmdResultCameraCreate extends CmdResultBase {}

export interface CmdResultCameraUpdate extends CmdResultBase {}

export interface CmdResultCameraDispose extends CmdResultBase {}

// MARK: Model Results

export interface CmdResultModelCreate extends CmdResultBase {}

export interface CmdResultModelUpdate extends CmdResultBase {}

export interface CmdResultModelDispose extends CmdResultBase {}

// MARK: Render Result Union

export type RenderCmdResult =
  | { type: 'shader-create'; content: CmdResultShaderCreate }
  | { type: 'shader-dispose'; content: CmdResultShaderDispose }
  | { type: 'geometry-create'; content: CmdResultGeometryCreate }
  | { type: 'geometry-dispose'; content: CmdResultGeometryDispose }
  | { type: 'material-create'; content: CmdResultMaterialCreate }
  | { type: 'material-update'; content: CmdResultMaterialUpdate }
  | { type: 'material-dispose'; content: CmdResultMaterialDispose }
  | { type: 'texture-create'; content: CmdResultTextureCreate }
  | { type: 'texture-update'; content: CmdResultTextureUpdate }
  | { type: 'texture-dispose'; content: CmdResultTextureDispose }
  | { type: 'sampler-create'; content: CmdResultSamplerCreate }
  | { type: 'sampler-update'; content: CmdResultSamplerUpdate }
  | { type: 'sampler-dispose'; content: CmdResultSamplerDispose }
  | { type: 'camera-create'; content: CmdResultCameraCreate }
  | { type: 'camera-update'; content: CmdResultCameraUpdate }
  | { type: 'camera-dispose'; content: CmdResultCameraDispose }
  | { type: 'model-create'; content: CmdResultModelCreate }
  | { type: 'model-update'; content: CmdResultModelUpdate }
  | { type: 'model-dispose'; content: CmdResultModelDispose };
