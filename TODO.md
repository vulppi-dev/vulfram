# TODO — Render Architecture Replace (Realm/Surface/RealmGraph)

> Checklist incremental e detalhado para migrar do modelo window-centric para Realm/Surface/RealmGraph mantendo compatibilidade.

**Fase 0 — Preparacao e alinhamento tecnico**
- [ ] Mapear o fluxo atual de render por window e os pontos de acoplamento (`WindowState.render_state`, `render_frames`, `CmdRenderGraphSet`).
- [ ] Definir o contrato interno de `Realm`, `Surface`, `Present` e `Connector` (campos minimos, defaults e lifecycle).
- [ ] Definir como os IDs logicos novos (`RealmId`, `SurfaceId`, `ConnectorId`, `PresentId`) aparecem no core (tabelas + generation).
- [ ] Definir a politica de buffering do `Surface` (min 2 imagens, prev/current) e como expor `PreviousFrame`.
- [ ] Definir regras de composicao multi-janela: mesma `Surface` em N windows no mesmo frame.
- [ ] Definir regras do `rect` (connector) com comportamento tipo `position: fixed` do CSS.
- [ ] Validar o impacto no profiling e GPU timestamps com execucao multi-realm.
- [ ] Atualizar docs de arquitetura (planejamento) antes de iniciar implementacao pesada.

**Fase A — Infraestrutura base + Compatibilidade (sem mudar o host)**
- [ ] Criar `RealmTable` com generation e estados essenciais (kind, output_surface, render_graph, flags).
- [ ] Criar `SurfaceTable` com generation, `kind` (onscreen/offscreen), size/format/alpha/msaa policies e buffering.
- [ ] Criar `PresentTable` com mapping `windowId -> surfaceId` e defaults.
- [ ] Criar `ConnectorTable` com campos minimos: `connectorId`, `kind`, `sourceSurfaceId`, `rect`, `zIndex`, `blendMode`, `clip`, `inputFlags`.
- [ ] Introduzir `Realm` default por window (criado junto com `CmdWindowCreate`).
- [ ] Criar `Surface` onscreen por window (virtual swapchain) e registrar em `Present`.
- [ ] Ajustar `CmdRenderGraphSet(windowId, graph)` para ser alias do `Realm` default da window.
- [ ] Garantir que o render atual continue funcionando sem host changes.
- [ ] Atualizar docs de `CmdRenderGraphSet` para mencionar o alias do Realm default.
- [ ] Atualizar docs de `RENDER-GRAPH` para esclarecer que e intra-Realm.

**Fase B — RealmGraph minimo (composicao visual basica)**
- [ ] Implementar `RealmGraphPlanner` com build de grafo a partir de `Connectors` + `Presents`.
- [ ] Implementar deteccao de ciclo (SCC) e politica de corte de edges soft.
- [ ] Implementar cache `LastGoodSurface` e `FallbackSurface` para edges cortadas.
- [ ] Definir politica `Hard` vs `Soft` para edges (present roots = hard, demais = soft default).
- [ ] Implementar execucao topologica de Realms (sem input routing).
- [ ] Implementar composicao por `Connector` no Realm consumidor (pass interno) respeitando `zIndex`, `blendMode` e `clip`.
- [ ] Implementar regras de ordenacao: `zIndex` ordena layers quando varios connectors apontam para a mesma `Surface`.
- [ ] Implementar `rect` com regras tipo `position: fixed` (coordenadas relativas a window/viewport + clip por width).
- [ ] Implementar alinhamento por height e clip por width ao desenhar a `Surface` em cada viewport.
- [ ] Implementar resolucao automatica de MSAA para surfaces sampleaveis.
- [ ] Criar conversoes automaticas de formato/alpha/size no compositor do Realm.
- [ ] Garantir que o `Surface` seja sempre renderavel e sampleavel.
- [ ] Atualizar docs de arquitetura com o fluxo `RealmGraph` e ciclo-break.

**Fase C — Input routing (hit-test e foco)**
- [ ] Implementar hit-test de `ViewportConnector` (rect + z-order + clip).
- [ ] Implementar retrace/raycast de `PlaneConnector` (screen -> UV -> local).
- [ ] Implementar pointer capture (down->up) e focus lock por connector.
- [ ] Implementar `eventTrace` e metadata: `Window -> Realm -> Connector -> Realm`.
- [ ] Garantir que o trace suporte multiplas janelas com a mesma `Surface`.
- [ ] Garantir compatibilidade com eventos atuais (nao quebrar payloads existentes).
- [ ] Atualizar docs de eventos para incluir routing opcional.

**Fase D — Qualidade, performance e observabilidade**
- [ ] Implementar guard contra self-sample no frame atual (permitir apenas `PreviousFrame`).
- [ ] Implementar guard de no-progress (dirty ping-pong) com teto de iteracoes por frame.
- [ ] Implementar throttling por Realm (importance, cachePolicy).
- [ ] Consolidar conversoes automaticas no compositor (colorspace/alpha/size/resolve).
- [ ] Implementar `FrameReport` com ordem de execucao, edges cortadas e surfaces em cache/fallback.
- [ ] Garantir que nenhum Realm bloqueia o tick por readback (pipeline N-1).
- [ ] Atualizar docs de profiling para incluir informacoes de RealmGraph/FrameReport.

**Fase E — Cleanups e consolidacao**
- [ ] Remover caminhos window-centric antigos (execucao direta por window) quando RealmGraph estiver estavel.
- [ ] Auditar fallback e caches (evitar leaks, respeitar politicas de dispose).
- [ ] Revisar validacao do RenderGraph intra-Realm para cobrir recursos/inputs/outputs de fato.
- [ ] Atualizar `docs/ARCH.md`, `docs/OVERVIEW.md` e `docs/API.md` com a nova arquitetura.
- [ ] Atualizar exemplos/demos para usar os novos comandos quando existirem.
- [ ] Rodar `scripts/check.sh` antes de finalizar a fase.
