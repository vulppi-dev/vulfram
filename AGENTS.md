# AGENTS.md — Vulfram instructions

- Sempre que o usuário comentar sobre um padrão ou regra de desenvolvimento, adicionar ao `AGENTS.md`.
- Variaveis que seguram ownership e não são mais usadas depois ganham sempre o prefixo `_`.
- Se variaveis não forem usadas, devem ser removidas.
- Funções não usadas também são removidas.
- Sempre executar `cargo check --lib` ao final de qualquer implementação para checar erros de compilação.
- Demos devem fechar por padrão ao pressionar a tecla Escape.
- Valores de `outline_threshold` devem ficar no intervalo [0, 1) (clamp).
- Arquivos devem ter como alvo no 300 linhas e no máximo 500 linhas. Se passar disso e for possível, dividir em arquivos menores.
