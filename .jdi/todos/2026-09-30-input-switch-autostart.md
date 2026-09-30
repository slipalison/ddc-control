# Todos — input-switch-autostart (2026-09-30)

Fora de escopo da phase `input-switch-autostart`, registrados pelo asker.

- Esconder ou marcar, na lista de entradas, as que estão sem sinal: não há como detectar sinal por DDC sem escrever o input, e escrever o input em monitor real é vedado (docs/hardware-validation.md).
- Assentamento (polling até o monitor confirmar) para outros códigos — energia (`0xD6`), resets: YAGNI, só o `0x60` tem a reversão relatada.
- Releitura tardia no popup: se o monitor reverter a entrada DEPOIS da janela de assentamento de 3 s (auto source), o chip só se corrige na próxima abertura do popup.
- Perfis e hotkeys globais: phase `profiles-hotkeys`.
- Autostart no macOS (o plugin compila lá, mas a phase só cobre Linux e Windows).
- Flag de autostart no `ddc-cli`.
- Teste automatizado da chave `HKCU\...\Run` no runner Windows do CI (hoje o Windows só é provado por compilação e à mão).
