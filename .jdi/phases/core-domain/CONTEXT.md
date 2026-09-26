# Phase 1: Core domain — Context (slug: core-domain)

## Goal
Workspace cargo + crate `ddc-core`: port `MonitorBackend`, modelo VCP tipado (códigos MCCS, valores/limites), parser de capabilities string, backend mock e testes unitários.

## Locked decisions
- D-1: Hexagonal (Ports & Adapters) — port `MonitorBackend` no core; adapters (`ddc-hi`, mock) fora.
- D-2: `ddc-core` depende só de `thiserror`; parser de caps próprio; fake `InMemoryMonitorBackend` vive em `ddc-adapters`, entra no core só como `[dev-dependencies]`.
- D-2026-09-25-core-domain-1: Workspace desta phase = apenas `crates/ddc-core` + `crates/ddc-adapters`. `ddc-cli`/`apps/ddc-tray` ficam para suas próprias phases. Root vira `[workspace]` puro; `src/main.rs` é removido.
- D-2026-09-25-core-domain-2: Shape de `Feature` = `{ code: VcpCode, kind: FeatureKind(Continuous|NonContinuous|Table), access: Access(ReadOnly|WriteOnly|ReadWrite), risk: Risk(Safe|Dangerous), allowed_values: Option<Vec<u8>> }` — extensível sem breaking change para o catálogo completo de `full-osd-control`.
- D-2026-09-25-core-domain-3: Tabela seed de `risk_for_code` (Safe = {0x10,0x12,0x14,0x16,0x18,0x1A,0x62,0x87,0xCC}; Dangerous = {0x04,0x05,0x06,0x08,0x60,0xCA,0xD6} ∪ {>=0xE0}); qualquer código fora da tabela seed é `Dangerous` por default (fail-safe).
- D-2026-09-25-core-domain-4: `get_feature` sempre lê o backend, mesmo se o código não está no caps parseado (0x62 responde fora do caps no monitor de dev); retorna `declared_in_capabilities: bool`. `set_feature` sem `max` conhecido (nem caps, nem leitura prévia) retorna `DdcError::UnsupportedFeature` em vez de escrever às cegas.
- D-2026-09-25-core-domain-5: Struct `Capabilities` = `{ protocol, monitor_type, model, commands, vcp: BTreeMap<u8, Option<Vec<u8>>>, mccs_version, unknown_tags }`. Parser tolerante (sem espaço, tags desconhecidas viram `unknown_tags`, nunca erro); só falha (`DdcError::Transport`) com parênteses desbalanceados.

## Canonical refs
- `.jdi/PROJECT.md` § Research notes (caps elidida) e § Definition of Done (baseline herdada pelo Gate 8).
- `.jdi/agents/jdi-doer-ddc-control.md` § Workspace layout / Monitor-write safety — layout de crates e tabela de risco que esta phase implementa.
- Caps string real e completa (RTK QHD HDR, Linux `/dev/i2c-5`, `ddcutil`, sondada 2026-09-25 — usar verbatim no teste do parser, D-2026-09-25-core-domain-5):
  `(prot(monitor)type(LCD)model(RTK)cmds(01 02 03 07 0C E3 F3)vcp(02 04 05 06 08 0B 0C 10 12 14(01 02 04 05 06 08 0B) 16 18 1A 52 60(01 03 04 0F 10 11 12) 87 AC AE B2 B6 C6 C8 CA CC(01 02 03 04 06 0A 0D) D6(01 04 05) DF FD FF)mswhql(1)asset_eep(40)mccs_ver(2.2))`

## Out of scope
- Catálogo MCCS 2.2 completo (nome/tipo/acesso/risco/nomes de valor por código), sondagem read-only de códigos fora do caps, CLI `features` dump → `full-osd-control` (D-2026-09-25-full-osd-control-1).
- Adapter real `ddc-hi` (Windows dxva2 / Linux i2c) → `ddc-backends`.
- Crate `ddc-cli` e `apps/ddc-tray` → phases `cli` e `tray-app` (D-2026-09-25-core-domain-1).

## Definition of Done

### Auto-verifiable
- [ ] Workspace root é um manifesto `[workspace]` puro com members exatamente `crates/ddc-core` e `crates/ddc-adapters` (sem `ddc-cli`/`ddc-tray`); `src/main.rs` removido.
      **Verify:** `! grep -q '^\[package\]' Cargo.toml && grep -q 'crates/ddc-core' Cargo.toml && grep -q 'crates/ddc-adapters' Cargo.toml && ! grep -qE 'crates/ddc-cli|apps/ddc-tray' Cargo.toml && [ ! -f src/main.rs ] && echo OK`
      **Source:** CONTEXT
- [ ] `crates/ddc-core/Cargo.toml` `[dependencies]` = somente `thiserror` (D-2).
      **Verify:** `test "$(awk '/^\[dependencies\]/{f=1;next}/^\[/{f=0}f' crates/ddc-core/Cargo.toml | sed '/^\s*$/d' | grep -vc '^thiserror')" = 0 && echo OK`
      **Source:** CONTEXT
- [ ] Workspace compila (`ddc-core` + `ddc-adapters`, incluindo dev-dependency cíclica do fake).
      **Verify:** `cargo build --workspace --locked && echo OK`
      **Source:** CONTEXT
- [ ] Teste nomeado prova o parser contra a capabilities string real e completa do RTK QHD HDR (verbatim, ver Canonical refs).
      **Verify:** `cargo test --workspace --locked -- parses_real_rtk_caps_string 2>&1 | grep -qE '1 passed' && echo OK`
      **Source:** CONTEXT
- [ ] Teste nomeado prova tolerância do parser (sem espaço entre tokens, tags top-level desconhecidas, parênteses aninhados em `vcp(...)`) sem erro.
      **Verify:** `cargo test --workspace --locked -- tolerates_missing_spaces_and_unknown_tags 2>&1 | grep -qE '1 passed' && echo OK`
      **Source:** CONTEXT
- [ ] Teste nomeado prova que escrita em feature `Dangerous` sem `Confirm::Yes` retorna `DdcError::DangerousWriteNotConfirmed` e nunca chama o backend.
      **Verify:** `cargo test --workspace --locked -- dangerous_write_without_confirm_is_rejected 2>&1 | grep -qE '1 passed' && echo OK`
      **Source:** CONTEXT
- [ ] Teste nomeado prova que `value > max` é rejeitado (`DdcError::InvalidValue`) antes de qualquer chamada ao backend.
      **Verify:** `cargo test --workspace --locked -- write_value_above_max_is_rejected 2>&1 | grep -qE '1 passed' && echo OK`
      **Source:** CONTEXT
- [ ] Teste nomeado prova read-back após write: `set_feature` bem-sucedido retorna o valor lido de volta do backend (fake) após a escrita.
      **Verify:** `cargo test --workspace --locked -- write_reads_back_value_after_success 2>&1 | grep -qE '1 passed' && echo OK`
      **Source:** CONTEXT
- [ ] Teste nomeado prova que `get_feature` não rejeita um código ausente do caps parseado — lê o backend normalmente (D-2026-09-25-core-domain-4).
      **Verify:** `cargo test --workspace --locked -- get_feature_succeeds_for_code_absent_from_capabilities 2>&1 | grep -qE '1 passed' && echo OK`
      **Source:** CONTEXT

### Manual
- _(none)_

## Deferred to PR review
- Validação no monitor físico "RTK QHD HDR" (Linux `/dev/i2c-5`, MCCS 2.2): confirmar que a tabela de risco seed e o parser batem com leituras reais via `ddc-hi`/`ddcutil`. Sem monitor em CI — só validável depois que `ddc-backends` (phase 2) existir; sinalizar no corpo do PR, não bloqueia esta phase.

## Notes
Baseline de `.jdi/PROJECT.md` (`cargo test --workspace` verde, cobertura ≥80%, sem TODO/FIXME sem issue) é herdada automaticamente pelo Gate 8 do reviewer — não duplicada aqui. Nomes de teste acima são contrato mínimo (substring bate via `cargo test -- <nome>`); planner tem liberdade de módulo/caminho.
