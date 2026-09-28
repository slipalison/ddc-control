D-2026-09-27-ci-crossbuild-5 (2026-09-27): O pitfall tauri-apps/tauri#13419 (binário de TESTE de crate Tauri 2 morre no Windows com `STATUS_ENTRYPOINT_NOT_FOUND`, porque o manifesto comctl32 v6 só é embutido no bin) é corrigido no `apps/ddc-tray/src-tauri/build.rs`, sem excluir o `ddc-tray` dos testes no Windows:
- `tauri_build::WindowsAttributes::new_without_app_manifest()`;
- manifesto embutido em TODOS os alvos por `cargo:rustc-link-arg=/MANIFEST:EMBED` e `cargo:rustc-link-arg=/MANIFESTINPUT:<arquivo versionado>`;
- decisão feita lendo `CARGO_CFG_TARGET_OS == "windows"` e `CARGO_CFG_TARGET_ENV == "msvc"` em runtime do build script, nunca `#[cfg(windows)]`, que descreve o HOST.

A prova é o log do `rust-windows`: os testes do `ddc-tray` rodam e passam, e o número de binários de teste é igual ao do Linux.
