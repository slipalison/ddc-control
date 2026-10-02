D-2026-10-02-usb-switch-follow-14 (2026-10-02): Achados da 2ª rodada do critic.

(a) **Linha 10, objetivo.** O padrão `C` aplicado aos comentários de `en.js`/`pt-BR.js` não pegava a forma `to-do`/`to do`, que o DoD do projeto conta como marcador. O padrão passa a ser `(//|#|<!--).*(\b(todo|fixme)s?\b|\bto[ -]dos?\b[[:space:]]*[:(])`. Só o Verify muda; não há código.

(b) **Linha 3, não objetivo.** O `report` do `Stderr` real passa a escrever no mesmo sink injetável da linha de troca. O teste `follow::stderr_tests::report_prints_without_the_diagnostics_switch` afirma a linha `ddc-tray: could not …` de uma escrita que falha com o diagnóstico DESLIGADO. A linha 3 passa a exigir os 2 testes do módulo `stderr_tests`.
