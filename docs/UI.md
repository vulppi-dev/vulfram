# 🎨 Vulfram — Decisões Estéticas e Identidade Visual

Game Engine Experimental • Marca: Vulfram

---

## 1. Conceito de Marca

- Engine com estética **moderna** e **tecnológica**.
- **Dark scheme** como base.
- Cores **quentes e vivas**, com foco em:
  - paleta análoga **púrpura → magenta → roxo**.
- **Logo com raposa**:
  - conexão direta com a origem “Vulppi” (Vulpix, raposa).
- Uso de **efeito glitch/frame**:
  - remetendo a frames, renderização, engine gráfica.
- Deve funcionar bem em:
  - UI de ferramentas;
  - ícone de app;
  - material digital.

---

## 2. Paleta de Cores

### 2.1 Paleta Principal (Análoga Quente)

```css
:root {
  --primary-1: oklch(17.8% 0.0203 1.44);
  --primary-2: oklch(20.5% 0.0314 1.44);
  --primary-3: oklch(25.5% 0.0845 1.44);
  --primary-4: oklch(29.3% 0.1341 1.44);
  --primary-5: oklch(33.2% 0.1483 1.44);
  --primary-6: oklch(38.2% 0.1544 1.44);
  --primary-7: oklch(45% 0.1704 1.44);
  --primary-8: oklch(54.2% 0.2104 1.44);
  --primary-9: oklch(54.2% 0.2104 1.44);
  --primary-10: oklch(49.6% 0.2104 1.44);
  --primary-11: oklch(78.7% 0.2104 1.44);
  --primary-12: oklch(90.9% 0.0768 1.44);

  --secondary-1: oklch(17.8% 0.03 329.1);
  --secondary-2: oklch(20.5% 0.0538 329.1);
  --secondary-3: oklch(26.1% 0.1053 329.1);
  --secondary-4: oklch(29.9% 0.1663 329.1);
  --secondary-5: oklch(33.8% 0.1776 329.1);
  --secondary-6: oklch(38.9% 0.1827 329.1);
  --secondary-7: oklch(45.8% 0.2029 329.1);
  --secondary-8: oklch(54.5% 0.2461 329.1);
  --secondary-9: oklch(44.3% 0.2029 329.1);
  --secondary-10: oklch(38.8% 0.1827 329.1);
  --secondary-11: oklch(78.8% 0.2461 329.1);
  --secondary-12: oklch(90.5% 0.1001 329.1);

  --tertiary-1: oklch(17.8% 0.0374 283.9);
  --tertiary-2: oklch(20.6% 0.0506 283.9);
  --tertiary-3: oklch(27.2% 0.1187 283.9);
  --tertiary-4: oklch(31.8% 0.1764 283.9);
  --tertiary-5: oklch(35.8% 0.1891 283.9);
  --tertiary-6: oklch(39.9% 0.1934 283.9);
  --tertiary-7: oklch(44.8% 0.2078 283.9);
  --tertiary-8: oklch(50.7% 0.2389 283.9);
  --tertiary-9: oklch(54% 0.2646 283.9);
  --tertiary-10: oklch(49.3% 0.2389 283.9);
  --tertiary-11: oklch(77.6% 0.2268 283.9);
  --tertiary-12: oklch(91.4% 0.0725 283.9);

  --neutral-1: oklch(17.8% 0.014 307.3);
  --neutral-2: oklch(21.5% 0.0134 307.3);
  --neutral-3: oklch(25.5% 0.0182 307.3);
  --neutral-4: oklch(28.4% 0.025 307.3);
  --neutral-5: oklch(31.4% 0.0295 307.3);
  --neutral-6: oklch(35% 0.0332 307.3);
  --neutral-7: oklch(40.2% 0.0402 307.3);
  --neutral-8: oklch(49.2% 0.0524 307.3);
  --neutral-9: oklch(54% 0.0558 307.3);
  --neutral-10: oklch(58.6% 0.0548 307.3);
  --neutral-11: oklch(77% 0.0461 307.3);
  --neutral-12: oklch(94.9% 0.0086 307.3);
}
```

---

## 3. Neutros para Dark Scheme

Paleta de neutros para fundos, bordas, contornos e texto, uso sugerido:

- tons 1–4: fundos principais (UI, editor, overlay);
- tons 5–8: elementos de destaque e separadores;
- tons 9–12: texto, ícones e destaques sutis.

---

## 4. Tipografia

Todas fontes planejadas via **Fontsource** para fácil integração web/desktop.

### 4.1 Fonte Principal da Marca

**Nunito**

- Logo / títulos: **Nunito 700**
- UI / labels / corpo: **Nunito 400–500**

Características desejadas:

- formato amigável e moderno;
- boa legibilidade em tamanhos pequenos;
- combina com a estética neon/tech.

### 4.2 Fonte Monoespaçada (Debug / Log)

**JetBrains Mono**

Aplicações:

- interfaces técnicas;
- logs e consoles;
- leitura de buffers, hex dumps;
- visualizações de dados estruturados.

---

## 5. Diretrizes do Logo

Decisões principais para o logo da Vulfram:

- **Raposa estilizada**
  - reforça a ligação com a marca Vulppi / Vulpix;
  - silhueta clara e reconhecível.
- **Estética neon quente**
  - uso de magentas, púrpuras e roxos brilhantes;
  - contraste com fundo quase preto.
- **Glitch / scanline / pixel drift**
  - remete a **frames**, renderização e movimento;
  - transmite ideia de engine gráfica e tecnologia.
- **Escalabilidade**
  - legível até em **32×32 px**;
  - sem excesso de detalhes finos.
- **Formato**
  - ícone quadrado com cantos arredondados;
  - funcionamento como app icon, favicon e avatar.

---

## 6. Conceito do Ícone Aprovado

Resumo da última versão conceitual aprovada:

- Raposa estilizada com boa silhueta;
- Glitch horizontal leve, sugerindo troca de frames;
- Cores púrpura/magenta quentes em contraste com fundo quase preto;
- Estilo neon suave, sem agressividade visual;
- Adequado para ícone de app em baixas resoluções.

_(A arte final não está embutida no documento, mas este resumo descreve a versão conceitual atualmente aprovada.)_

---

## 7. Nome Oficial e Mensagem de Marca

**Nome:** **VULFRAM**

Motivações:

- **Vulppi** → raposa → identidade lúdica/visual.
- **Wolfram/Tungstênio** → força, tecnologia, robustez.
- **Frame** → referência direta a frames, motor gráfico, render.
- Nome:
  - forte e memorável;
  - com boa sonoridade internacional;
  - adequado para uso como nome de lib/engine.

Mensagem implícita:

> Uma engine enxuta, forte e moderna, com identidade visual marcante baseada em raposa + neon + glitch/frame, pensada para ferramentas e jogos contemporâneos em dark scheme.
