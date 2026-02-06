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

- [x] **Post-Processing**: Pass de pós-processamento + targets por câmera + demo inicial.
- [x] **Post-Processing**: SSAO com blur bilateral e integração na composição.
- [x] **Post-Processing**: Bloom + Glow (downsample/blur/upsample) e controle de intensidade.
- [x] **Glow baseado em emissive**: Saída emissive no forward + bloom usa emissive quando disponível.
- [x] **Post-Processing**: HDR pipeline avançado (exposure, tone mapping configurável).
- [x] **Post-Processing**: Outline com máscara e cor por modelo (pass outline + pós).
- [x] **Post-Processing**: Efeitos extras (vignette, grain, chromatic aberration, sharpen, posterize).
- [x] **Cell Shading**: Posterize + bandas de luz no pós-processamento inicial.
- [x] **Bloom & HDR**: Pipeline de alta dinâmica com tonemapping.
- [x] **SSAO**: Oclusão de ambiente em screen-space.
- [x] **Áudio 3D (Core System)**: Integração com a crate `kira`. Suporte a emissores amarrados a `Models` e cálculo de atenuação/doppler sincronizado com as transformações do Core.
  - [x] API base + proxy (desktop/web) e comandos de áudio no Core.
  - [x] Backend desktop (Kira) + backend web (WebAudio).
  - [x] Decodificação async + eventos de ready/erro.
  - [x] Streaming de áudio (cursor/chunks) para músicas longas.

## 🟣 Fase 4: UI Host-Driven (egui)

_Sistema de UI renderizado no core, definido pelo Host via ops._

- [x] **Fundação**: adicionar dependências e scaffolding do subsistema de UI (egui + wgpu), sem render ainda.
- [x] **ThemeResource**: `CmdUiThemeDefine` (cache + versionamento) e resposta `UiThemeDefined`.
- [x] **UiContext (lifecycle)**: `CmdUiContextCreate/Dispose/SetRect/SetTheme/SetTarget` com `screenRect` e `zIndex`.
- [x] **Ops & Árvore**: `CmdUiApplyOps` com versionamento e ops `add/remove/clear/set/move` + validação de IDs.
- [x] **Widgets MVP**: `container`, `text`, `button`, `input`, `image`, `separator`, `spacer`.
- [x] **Layout MVP**: `row/col/grid`, `gaps`, `padding`, `size` (`auto/fill/px`) e `align/justify` básicos.
- [x] **Listeners MVP**: `onClick` e `onChangeCommit`, emitindo `UiEvent` com label + nodeId.
- [x] **Render Target**: cada `UiContext` renderiza na `targetTexture` indicada.
- [x] **Input Routing**: roteamento por `screenRect` + `zIndex` e foco por último input.
- [x] **Docs & Exemplo**: documentação de comandos e exemplo completo de ops no host.
- [x] **Composição UI/3D**: definir camadas e regras de target lógico para câmera/UI.
- [x] **Camadas (prioridade)**: composição por layers com ordem explícita (ex.: `layer: 0` 3D base, `layer: 10` UI, `layer: 20` debug).
- [x] **Target lógico (câmera)**: se a câmera renderiza para texture target (LogicalId), essa textura pode ser aplicada em qualquer superfície; se não, vai para o layer da câmera.
- [x] **Target lógico (UI)**: se o UiContext renderiza para texture target (LogicalId), essa textura pode ser aplicada em qualquer superfície; se não, vai para o layer de UI.
- [x] **UI em superfície 3D (prioridade)**: garantir que UI pode sempre renderizar para textura e ser aplicada em um plane no 3D.
- [x] **Viewport POC**: render de câmera para textura sRGB e exibição no egui como imagem.
- [x] **Viewport Resize**: adaptar target ao tamanho do widget + resolver MSAA quando aplicável.
- [x] **Viewport Input**: mapear input do retângulo do widget para a câmera correspondente.
- [x] **Multi-Viewports**: suportar 2+ viewports simultâneos com IDs e targets independentes.
- [x] **Wrap**: `wrap` para `row/reverse-row` e `col/reverse-col` com height limitada.
- [x] **Animate**: `animate` para `opacity` e `translateY` com easing e `animComplete`.
- [x] **Editor Docking**: layout de painéis e docking para o editor host-driven.
- [x] **Clipping/Scissor**: clipping consistente para scrolls, listas e painéis.
- [x] **Scroll Real**: containers scrolláveis com offsets e barras.
- [x] **Text/Fonts**: fallback de fontes, tamanho por estilo e atlas de glyphs.
- [x] **Hit-Testing**: regras de input respeitando `display/visible/opacity`.
- [x] **Z-Order Interno**: overlays/menu/contexto dentro do mesmo `UiContext`.
- [x] **Focus & Keyboard**: tab/focus, navegação básica por teclado em inputs.
- [x] **Hot-Reload Theme**: atualização de theme sem recriar context.
- [x] **Debug UI**: overlay de bounds/ids e profiling básico.
- [x] **Performance**: cache de layout e invalidation por dirty flags.
- [x] **Demo 5 (UI)**: criar um demo para testar e demonstrar o sistema de UI.
- [x] **Ajuste de Demos**: atualizar demos existentes para continuarem funcionando após a integração da UI.
- [x] **Refactor Demos**: dividir os demos de `main.rs` em subarquivos para reduzir o tamanho e melhorar organização.

## 🔵 Fase 5: Grafo Recursivo UI ↔ Render 3D (Swapchain Virtual)

_Grafo ordenado e recursivo para múltiplos níveis de composição UI/3D._

- [x] **Modelo Conceitual**: definir “Swapchain Virtual” por janela como nó raiz.
- [x] **Níveis**: definir identidade única por nível (UI → 3D → UI), com `level_id`.
- [x] **Nós de Grafo**: mapear `UiContext`, `CameraViewport`, `PanelPlane`, `ComposeTarget` como nós explícitos.
- [x] **Ordenação Determinística**: regras para `layer`, `z_index`, `depth_level`, `order`.
- [x] **Texturas por Nível**: política de IDs únicos e resolução de conflitos.
- [x] **Dependências Topológicas**: execução baseada em dependências de textura.
- [x] **Recursão N-vezes**: permitir encadeamento `UI → 3D → UI → 3D` no mesmo frame.
- [x] **Quebra de Ciclos**: detectar ciclo real e resolver com `frame-lag` controlado.
- [x] **UI como Fonte**: `UiContext` renderizando para target do nível.
- [x] **UI como Destino**: UI consumindo `camera_target` do nível anterior.
- [x] **3D como Fonte**: câmeras com targets por nível.
- [x] **3D como Destino**: `PanelPlane` exibindo UI do nível anterior.
- [x] **Depth/Layer/Viewport**: normalizar regras de profundidade e visibilidade entre níveis.
- [x] **Roteamento de Input por Nível**: input da janela deve atingir nível top.
- [x] **Picking de Panel**: resolução correta de `UiContext`/nível via retrace.
- [x] **Captura de Input**: preservar contexto e nível até release.
- [x] **Limites de Profundidade**: `max_depth` configurável e reutilização de targets.
- [x] **Dirty Flags por Nível**: evitar recomputação sem mudanças.
- [x] **Debug Overlay**: `level_id`, `target_id`, `layer` visíveis em runtime.
- [x] **Assert de Consistência**: validação de targets e dependências.

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
