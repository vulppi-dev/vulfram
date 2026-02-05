# UI Host-Driven — Commands & Ops

Este documento cobre os comandos da UI host-driven e um exemplo completo de `UiOps`.

## 1. Comandos

### CmdUiThemeDefine

Define ou atualiza um tema de UI.

Payload (MessagePack/JSON):

```json
{
  "cmd": "ui-theme-define",
  "args": {
    "themeId": {"str": "ui_theme_default"},
    "theme": {"fonts": [], "fontFamilies": {}, "textStyles": {}, "debug": null}
  }
}
```

Resposta:

```json
{
  "success": true,
  "message": "Theme stored",
  "themeId": {"str": "ui_theme_default"},
  "themeVersion": 1
}
```

### CmdUiContextCreate

Cria um `UiContext` e vincula a um tema.

```json
{
  "cmd": "ui-context-create",
  "args": {
    "windowId": 1,
    "contextId": {"str": "ui_main"},
    "themeId": {"str": "ui_theme_default"},
    "target": "screen",
    "screenRect": {"x": 0, "y": 0, "w": 1280, "h": 720},
    "zIndex": 10
  }
}
```

### CmdUiContextSetTheme

```json
{
  "cmd": "ui-context-set-theme",
  "args": {
    "contextId": {"str": "ui_main"},
    "themeId": {"str": "ui_theme_dark"}
  }
}
```

### CmdUiContextSetRect

```json
{
  "cmd": "ui-context-set-rect",
  "args": {
    "contextId": {"str": "ui_main"},
    "screenRect": {"x": 0, "y": 0, "w": 1920, "h": 1080}
  }
}
```

### CmdUiContextSetTarget

```json
{
  "cmd": "ui-context-set-target",
  "args": {
    "contextId": {"str": "ui_main"},
    "target": {"textureId": {"int": 900}}
  }
}
```

### CmdUiContextDispose

```json
{
  "cmd": "ui-context-dispose",
  "args": {"contextId": {"str": "ui_main"}}
}
```

### CmdUiApplyOps

Aplica uma lista ordenada de operações na árvore de UI.

```json
{
  "cmd": "ui-apply-ops",
  "args": {
    "contextId": {"str": "ui_main"},
    "baseVersion": 0,
    "ops": [
      {"op": "add", "parent": {"str": "root"}, "id": {"str": "panel"}, "nodeType": "container"}
    ]
  }
}
```

## 2. Operações (UiOps)

### add

```json
{"op": "add", "parent": {"str": "root"}, "id": {"str": "my_node"}, "nodeType": "text"}
```

### remove

```json
{"op": "remove", "id": {"str": "my_node"}}
```

### clear

```json
{"op": "clear", "id": {"str": "panel"}}
```

### move

```json
{"op": "move", "id": {"str": "my_node"}, "parent": {"str": "root"}, "index": 0}
```

### set

```json
{
  "op": "set",
  "id": {"str": "my_node"},
  "mode": "merge",
  "style": {"width": "fill", "padding": 12},
  "props": {"value": "Hello"},
  "listeners": {"onClick": "my_handler"}
}
```

### animate

```json
{
  "op": "animate",
  "id": {"str": "panel"},
  "property": "opacity",
  "from": 0.0,
  "to": 1.0,
  "durationMs": 250,
  "delayMs": 0,
  "easing": "ease-in-out"
}
```

Propriedades suportadas: `opacity`, `translateY`.

## 3. Estilos e Props principais

Estilos (`style`):

- Layout: `layout` (`row`, `reverse-row`, `col`, `reverse-col`, `grid`)
- `gap`, `gapX`, `gapY`, `padding`, `paddingX`, `paddingY`
- Tamanho: `width`, `height` (`auto`, `fill`, `px`)
- Alinhamento: `align` (`start`, `center`, `end`, `stretch`), `justify`
- Visibilidade: `display` (`none`), `visible` (bool), `opacity` (0..1)
- Transformação: `translateY`
- Z-order interno: `zIndex`
- Scroll: `scrollX`, `scrollY`, `scrollbar` (`hidden`, `auto`, `visible`)
- Tipografia: `fontSize`, `textStyle`
- Debug: `debug` (bool no `root`)

Props comuns (`props`):

- `text`: `value`
- `button`: `label`
- `input`: `value`
- `image`: `textureId`, `cameraId`
- `dock`: `activeIndex`, `title` (em tabs)
- `scroll`: `scrollX`, `scrollY` (offsets atuais)

## 4. Eventos

Listeners disponíveis:

- `onClick`, `onChange`, `onChangeCommit`, `onSubmit`, `onFocus`, `onBlur`
- `onViewportHover`, `onViewportClick`, `onViewportDrag`, `onViewportDragEnd`
- `onAnimComplete`

O Core só envia eventos para listeners definidos.

## 5. Exemplo completo de ops

```json
[
  {"op": "add", "parent": {"str": "root"}, "id": {"str": "screen"}, "nodeType": "container"},
  {"op": "set", "id": {"str": "screen"}, "style": {"layout": "col", "gap": 12, "padding": 16, "width": "fill", "height": "fill"}},

  {"op": "add", "parent": {"str": "screen"}, "id": {"str": "title"}, "nodeType": "text", "props": {"value": "Vulfram UI"}},
  {"op": "set", "id": {"str": "title"}, "style": {"fontSize": 20}},

  {"op": "add", "parent": {"str": "screen"}, "id": {"str": "dock"}, "nodeType": "dock", "props": {"activeIndex": 0}},

  {"op": "add", "parent": {"str": "dock"}, "id": {"str": "tab_a"}, "nodeType": "container", "props": {"title": "A"}},
  {"op": "add", "parent": {"str": "dock"}, "id": {"str": "tab_b"}, "nodeType": "container", "props": {"title": "B"}},

  {"op": "add", "parent": {"str": "tab_a"}, "id": {"str": "scroll"}, "nodeType": "scroll"},
  {"op": "set", "id": {"str": "scroll"}, "style": {"height": 240, "scrollY": true, "scrollbar": "auto"}},

  {"op": "add", "parent": {"str": "scroll"}, "id": {"str": "input"}, "nodeType": "input", "props": {"value": ""}, "listeners": {"onChangeCommit": "name_commit"}},
  {"op": "add", "parent": {"str": "scroll"}, "id": {"str": "button"}, "nodeType": "button", "props": {"label": "Salvar"}, "listeners": {"onClick": "save"}},

  {"op": "animate", "id": {"str": "screen"}, "property": "opacity", "from": 0.0, "to": 1.0, "durationMs": 300, "easing": "ease-out"}
]
```
