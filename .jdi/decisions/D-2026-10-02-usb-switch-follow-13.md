D-2026-10-02-usb-switch-follow-13 (2026-10-02): Achados da 1ª rodada do critic. O DoD é apertado em três linhas, e a passada 2 do doer fecha os warnings do reviewer.

(a) **Linha 6, objetivo.** A cláusula da D-12b, "o Playwright prova que o `app.js` usa as funções novas do view-model", era falsa. O `app.js` da base renderiza os mesmos rótulos inline e passa. A linha passa a afirmar o RÓTULO RENDERIZADO, com dois testes:
- um teste e2e nomeado em `tests/e2e/fallback.spec.mjs`, "a silent monitor reads on another input in the picker", que confere o literal `LG TV SSCR2 (em outra entrada)` no seletor do popup, em `light` e `dark`;
- o teste unitário das frases literais, que já existia.
O uso das funções do view-model pelo `app.js` deixa de ser afirmado.

(b) **Linha 3, W3.** O `Stderr` real passa a ter a escrita injetável. O teste `follow::stderr_tests::announce_prints_without_the_diagnostics_switch` prova que a linha de troca sai com o diagnóstico DESLIGADO.

(c) **Linha 8.** O texto dizia que o `~/.config/autostart` e o `~/.config/ddc-control` reais "ficam idênticos", mas o Verify comparava só os nomes. O Verify passa a comparar nomes e conteúdo (sha256 de cada arquivo).

(d) **Passada 2 do doer, fora do DoD:**
- W1: teste de saída escalonada durante o aprender, sem escrita;
- W2: teste da atomicidade do `save`;
- W4: `i18n::input_label` derivado do catálogo do core, sem repetir os nomes das entradas;
- W6: a thread do teste "named thread" é encerrada e juntada.
O orquestrador marca as tasks do PLAN como `completed` (W5).
