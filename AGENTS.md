# AGENTS.md — Vulfram instructions

- Sempre que o usuário comentar sobre um padrão ou regra de desenvolvimento, adicionar ao `AGENTS.md`.
- Variaveis que seguram ownership e não são mais usadas depois ganham sempre o prefixo `_`.
- Se variaveis não forem usadas, devem ser removidas.
- Funções não usadas também são removidas.
- Sempre executar `cargo check --lib` ao final de qualquer implementação para checar erros de compilação.
- Configurações de filtros do pós-processamento devem usar o prefixo `filter_` e a flag `filter_enabled`.
- Demos devem fechar por padrão ao pressionar a tecla Escape.
- Valores de `outline_threshold` devem ficar no intervalo [0, 1) (clamp).
- Arquivos devem ter como alvo no 300 linhas e no máximo 500 linhas. Se passar disso e for possível, dividir em arquivos menores.
- Texturas de cor devem usar formato float (ex.: `rgba16f`) por padrão; depth também pode ser float quando aplicável.
- A árvore do render graph não deve declarar formatos de textura; o core define padrões.
- Sempre atualizar a documentação relacionada ao terminar uma fase.
- Em auditorias futuras, ignorar retenção por recursos host-side sem dispose.
- `scripts/check.sh` roda `cargo check --lib` + valida WGSL.
- Decode de imagens/texturas deve ser assíncrono por padrão; no browser tentar Worker e, se indisponível, usar Promises/async com divisão em chunks para não travar o loop.
