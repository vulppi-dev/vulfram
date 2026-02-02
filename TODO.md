# 🦊 Vulfram Engine — Roadmap & TODO (Lean Core Architecture)

Plano de evolução focado em um Core minimalista e performático. Lógica complexa e parsing de arquivos são delegados para o **Host**.

## 🟢 Fase 1: Fundamentos & Visibilidade (Core)

_Otimização do fluxo de dados e ferramentas de debug._

- [x] **Geometry Frustum Culling**: Descarte de draw calls no Core para objetos fora da visão (Performance).
- [x] **Visual Debugger (Gizmos)**: Sistema no Core para desenhar linhas/shapes de debug via comandos simples.
- [x] **Instancing**: Suporte a renderização instanciada para otimizar milhares de objetos repetidos.
- [x] **Semantic Labeling**: Labels amigáveis em todos os recursos para facilitar depuração no Host e Profiler.
- [x] **Resource Discovery**: Comandos de `List` para Modelos, Câmeras, Luzes, Materiais, Geometrias e Texturas.

## 🟡 Fase 2: Arquitetura de Renderização (Core)

_Refatoração para suportar efeitos avançados._

- [x] **Render Graph**: Substituir o `Compose` por um grafo real para encadeamento de efeitos.
- [x] **Advanced Profiler**: Exporta métricas detalhadas de GPU para o Host via MessagePack.
- [x] **Skeletal Animation (Skinning)**: Implementar interpolação de ossos via GPU (Shader).

## 🟠 Fase 3: Efeitos & Simulação (Core)

_Features que dependem de acesso a buffers e transformações espaciais._

- [x] **Post-Processing (Fase 3.1)**: Pass de pós-processamento + targets por câmera + demo inicial.
- [x] **Post-Processing (Fase 3.2)**: SSAO com blur bilateral e integração na composição.
- [x] **Post-Processing (Fase 3.3)**: Bloom + Glow (downsample/blur/upsample) e controle de intensidade.
- [x] **Glow baseado em emissive**: Saída emissive no forward + bloom usa emissive quando disponível.
- [x] **Post-Processing (Fase 3.4)**: HDR pipeline avançado (exposure, tone mapping configurável).
- [x] **Post-Processing (Fase 3.5)**: Outline com máscara e cor por modelo (pass outline + pós).
- [x] **Post-Processing (Fase 3.7)**: Efeitos extras (vignette, grain, chromatic aberration, sharpen, posterize).
- [x] **Cell Shading**: Posterize + bandas de luz no pós-processamento inicial.
- [x] **Bloom & HDR**: Pipeline de alta dinâmica com tonemapping.
- [x] **SSAO**: Oclusão de ambiente em screen-space.
- [ ] **Áudio 3D (Core System)**: Integração com a crate `kira`. Suporte a emissores amarrados a `Models` e cálculo de atenuação/doppler sincronizado com as transformações do Core.
  - [x] API base + proxy (desktop/web) e comandos de áudio no Core.
  - [x] Backend desktop (Kira) + backend web (WebAudio).
  - [x] Decodificação async + eventos de ready/erro.
  - [ ] Streaming de áudio (cursor/chunks) para músicas longas.

## 🔴 Opcionais, futuras melhorias

- [ ] **Custom Materials via Graph Nodes**: Sistema no Core que recebe estruturas de "nós" e gera shaders dinâmicos.
- [ ] **Custom Effects via Graph Nodes**: Sistema no Core que recebe estruturas de "nós" e gera efeitos dinâmicos para o render graph.
- [ ] **Projective Spot Lights**: Luzes com projeção de textura.
- [ ] **Occlusion Culling**: Otimização avançada baseada em visibilidade de pixels.
- [ ] **Post-Processing**: Focus/DoF baseado em depth (CoC + blur variável).
- [ ] **Decals (Decalques)**: Projeção de texturas via shader.
- [ ] **Particles (CPU/GPU)**: Sistemas de partículas com dois modos (CPU e GPU).

## 🔵 Responsabilidades do Host (Plugins & Lógica)

_Funcionalidades que serão implementadas como bibliotecas/plugins no lado do Host._

- [ ] **GLTF Loader (Host)**: Crate/Lib no Host para parsear GLTF e enviar `upload_buffer` para o Core.
- [ ] **Physics Engine (Host)**: Integração com motores como Rapier no Host, enviando `ModelUpdate` a cada frame.
- [ ] **Spatial Audio (Host)**: Gerenciamento de áudio 3D direto no Host.
- [ ] **LOD System (Host)**: Lógica de troca de meshes baseada em distância rodando no Host.
- [ ] **Input Mapping (Host)**: Abstração de input bruto para ações complexas.
