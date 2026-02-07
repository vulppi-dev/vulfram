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
