# ddc-control

## Visão
Controlar pelo software tudo que o OSD físico do monitor oferece (brilho, contraste, entrada, cor, volume, energia) via DDC/CI, com o mesmo binário rodando em Windows e Linux.

## Tipo
cli + desktop app (ícone na bandeja). Sem backend, sem rede.

## Stack
- Language: Rust stable (1.98, edition 2024), cargo workspace
- Framework: Tauri 2.11 (tray + popup), clap 4.6 (CLI)
- Version: ddc-hi 0.4.1 (backends `ddc-winapi` Windows / `ddc-i2c` Linux); parser de capabilities MCCS próprio em `ddc-core` (D-2 — sem `mccs`/`mccs-caps`)
- Key dependencies: tauri-plugin-global-shortcut 2.3, tauri-plugin-autostart, serde/serde_json, thiserror, tracing
- Layout alvo: `crates/ddc-core` (hexágono, deps = thiserror), `crates/ddc-adapters` (driven: ddc-hi + fake), `crates/ddc-cli` (driving, clap), `apps/ddc-tray` (driving, Tauri). O crate raiz atual (`src/main.rs`) vira workspace na phase 1.
- Testes: `cargo test --workspace` + backend mock; cobertura via `cargo llvm-cov`

## Code Design
**LOCKED:** Hexagonal (Ports & Adapters)

Decided in /jdi-new. Do not change. Port `MonitorBackend` no core; adapters `ddc-hi` (real) e mock (testes); CLI e comandos Tauri são adapters de entrada.

## Slug
ddc-control

## Research notes
- Monitor alvo sondado em 2026-09-11 (dxva2): "RTK QHD HDR" via USB-C DP Alt Mode, MCCS 2.2. Caps: `vcp(02 04 05 06 08 0B 0C 10 12 14(01 02 04 05 06 08 0B) 16 18 1A 52 60(01 03 04 0F 10 11 12) 87 AC AE B2 B6 C6 C8 CA CC(...) D6(01 04 05) DF FD FF)`. Input atual 0x0F (DP1). Respondeu 0x62 (volume) fora do caps — validar por efeito.
- LG TV (HDMI) não responde DDC/CI (I2C 0xC0262582) — fora de escopo; TVs precisam CEC/API própria.
- Linux: precisa módulo `i2c-dev` + permissão em `/dev/i2c-*` (grupo `i2c` ou regra udev). NVIDIA proprietário pode bloquear DDC em algumas saídas. Tauri no Linux exige `webkit2gtk-4.1`; tray via StatusNotifierItem (GNOME precisa extensão AppIndicator).
- Latência DDC ~50-200 ms por comando: sliders precisam debounce e leitura assíncrona fora da thread de UI.
- Dev principal roda Windows; backend Linux só é validado via CI (ubuntu-latest) até haver máquina Linux.

## Restrições globais
- Cobertura mínima 80% (linhas) em `crates/` e `apps/`
- Conventional commits, commits atômicos por task
- Idioma: código, commits e PRs em inglês; discussão e artefatos em `.jdi/` em pt-BR
- `cargo clippy -- -D warnings` e `cargo fmt --check` limpos antes de qualquer PR

## Definition of Done

**LOCKED — project-wide baseline.** Inherited by every phase's reviewer (Gate 8). Change requires a new D-XX in DECISIONS.md plus manual edit here.

### Auto-verifiable
- [ ] `cargo test --workspace` exits 0
      **Verify:** `cargo test --workspace --locked && echo OK`
      **Source:** PROJECT
- [ ] Coverage >= 80% of lines
      **Verify:** `cargo llvm-cov --workspace --summary-only` → coluna Lines >= 80%
      **Source:** PROJECT
- [ ] No `TODO`/`FIXME` without linked issue reference
      **Verify:** `! grep -RInE '\b(TODO|FIXME)S?\b|\b(todo|unimplemented)!\(' --include='*.rs' src/ crates/ apps/ 2>/dev/null | grep -vE '\b(TODO|FIXME)S?\b[[:space:]]*[(:]?[[:space:]]*\(?#[0-9]+' | grep -q . && ! grep -RInEi '(//[/!]?|/\*[*!]?|^[[:space:]]*\*|@)[[:space:]]*(todo|fixme)s?\b' --include='*.rs' src/ crates/ apps/ 2>/dev/null | grep -vEi '\b(todo|fixme)s?\b[[:space:]]*[(:]?[[:space:]]*\(?#[0-9]+' | grep -q . && ! grep -RInEi '\b(to[ -]?do|fixme)s?\b[[:space:]]*[:(]' --include='*.rs' src/ crates/ apps/ 2>/dev/null | grep -vEi '\b(to[ -]?do|fixme)s?\b[[:space:]]*[(:]?[[:space:]]*\(?#[0-9]+' | grep -q . && echo OK` (D-2026-09-27-tray-app-4/-8/-9/-12: referência de issue colada ao marcador; `todo!`/`unimplemented!`, plural e marcadores de comentário `//`, `//!`, `///`, `/*`, `/**`, `*`, `@` em qualquer caixa; e `todo:`/`To do:`/`fixme(` em qualquer ponto da linha)
      **Source:** PROJECT

### Manual
- [ ] CHANGELOG.md updated with entry per release
      **Verify:** human confirmation required
      **Evidence:** new `## [version]` heading in CHANGELOG.md for current release
      **Source:** PROJECT
- [ ] README accurately describes current behavior
      **Verify:** human confirmation required
      **Evidence:** README diff reviewed in PR
      **Source:** PROJECT

## LLM config

```yaml
llm_config:
  default_model_opencode: anthropic   # (a) Anthropic Claude — default do JDI; runtime = Claude Code, OpenCode não usado
```

Applied by `/jdi-bootstrap` to `.opencode/opencode.jsonc`. Other runtimes ignore.

## Frontend

```yaml
frontend:
  # D-2026-09-26-tray-app-8: o popup do tray roda num navegador comum em modo demo (bridge JS
  # fake quando `window.__TAURI__` não existe, cenário escolhido por `?demo=`). O Gate 7 roda a
  # suíte Playwright do próprio app em `apps/ddc-tray`: `npm ci --ignore-scripts && npx playwright test`
  # (zero erro de console/`pageerror` e zero violação axe critical/serious em cada caminho abaixo).
  has_frontend: true
  frontend_url: http://localhost:1420
  dev_command: python3 -m http.server 1420 --directory apps/ddc-tray/src
  critical_paths:
    - /
    - /?demo=rtk
    - /?demo=two-monitors
    - /?demo=empty
    - /?demo=error
```
