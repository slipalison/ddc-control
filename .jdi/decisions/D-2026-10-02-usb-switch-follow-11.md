D-2026-10-02-usb-switch-follow-11 (2026-10-02): O orquestrador do `/jdi-issue` ajusta o DoD escrito pelo asker antes do plano, aplicando as lições da phase `input-switch-autostart`, que levou 10 iterações porque o DoD critic achava linhas ocas. São sete ajustes:

(a) Cada `Verify:` é AUTOCONTIDO. A função de conferência de conjunto (`sc`) fica definida dentro de cada comando, e não nas Notes, porque o reviewer roda cada Verify sozinho, num `bash` limpo.

(b) As conferências de conjunto usam `--lib` e um prefixo exclusivo dos testes novos (`app::usb_follow::tests::`, `usb_sysfs::tests::`, `follow::tests::`, `follow_config::tests::`, `docs::tests::`, `menu::tests::follow_`, `i18n::tests::follow_`). Em `menu::tests` e `i18n::tests`, que já têm testes antigos, o conjunto exato só faz sentido com prefixo próprio. Os testes antigos de `menu::`/`i18n::` continuam obrigatórios, com `0 failed` e `0 ignored`.

(c) TODA linha de comportamento inclui o CONGELAMENTO do código revisado, porque uma linha separada não protege as outras diante do critic:
- árvores git de `crates/ddc-core`, `crates/ddc-adapters`, `apps/ddc-tray/src-tauri`, `apps/ddc-tray/src`, `apps/ddc-tray/tests`, `apps/ddc-tray/scripts` e `docs`;
- blobs do `Cargo.toml` e do `Cargo.lock` da raiz;
- nenhuma diferença do working tree contra o HEAD;
- nenhum arquivo não rastreado.

O orquestrador troca o marcador `<FREEZE>` pelo fragmento real, com os ids, depois de cada passada do doer. Enquanto o marcador estiver lá, quem roda o Verify o troca por `:;`.

(d) Entra uma linha de SMOKE DE PONTA A PONTA do binário release, em sessão D-Bus privada, com `HOME` e `XDG_CONFIG_HOME` em tempdir, `DDC_TRAY_FAKE=1` e `DDC_TRAY_USB_ROOT` numa raiz USB fake. É o que prova que o `run()` liga o laço do follow, que o menu aprende, configura e liga, e que a saída dos dispositivos troca o monitor simulado exatamente uma vez. O script imprime linhas exatas, e o app imprime no stderr, a cada disparo, `ddc-tray: follow: switching <monitor-id> to input 0x<hh>`.

(e) A linha de `bwrap` passa a afirmar só o que prova: a suíte passa sem o `/sys` real e sem nós `/dev/i2c-*`, conferido dentro do próprio sandbox. Não afirma que nenhum teste tenta tocá-los.

(f) A linha de TODO/FIXME reusa a versão endurecida da phase anterior: une caminhos fixos e arquivos não-Rust do diff, e só olha comentários em `en.js`/`pt-BR.js`, onde "Todos" é palavra comum.

(g) Arquivos temporários usam `mktemp`, nunca `$TMPDIR`, que não existe no ambiente limpo do Verify.
