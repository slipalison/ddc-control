D-2026-10-02-usb-switch-follow-12 (2026-10-02): O orquestrador acolheu os achados da 1ª passada do doer.

(a) **Raiz USB na simulação.** Com `DDC_TRAY_FAKE=1` e sem `DDC_TRAY_USB_ROOT`, o laço do follow NÃO sobe: não há raiz. O `SharedFollow` é registrado, o menu funciona e um diagnóstico diz que não há árvore USB na simulação. Fora da simulação, a raiz é `/sys/bus/usb/devices`. A letra do plano ("senão `/sys/bus/usb/devices`") faria os smokes antigos, que rodam com `FAKE=1` e sem raiz, lerem o `/sys` real como raiz do follow, o que a D-9 veda. Teste: `switch_tests::the_follow_reads_the_tree_of_ddc_tray_usb_root_only_in_simulation`.

(b) **Linha 6 do DoD.** Com o `app.js` sem usar as funções novas do view-model, o Verify 6 dava `OK`; só o Playwright caía (`fallback.spec`, `pseudo-locale.spec`). A linha passa a exigir também a suíte Playwright inteira verde, sem `failed` nem `flaky`. O texto deixa de prometer o popup só pelo view-model.

(c) **Linha 8 do DoD.** O smoke remove da raiz fake o teclado, o mouse e o hub do switch, e um root hub fica, como num switch real. O texto da linha passa a dizer isso. As 10 linhas impressas não mudam.

(d) **Congelamento.** O marcador `<FREEZE>` foi trocado pelos ids reais do HEAD `df69902`.

(e) **Uso real (para o PR).** Depois de uma troca bem-sucedida, o monitor deixa de responder a esta máquina. Por isso a releitura do core falha e o `report` mostra que não conseguiu confirmar a troca. Isso é esperado e o guia explica. A confirmação humana fica no teste PC + notebook.
