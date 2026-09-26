# Phase 1: Core domain — Summary  (slug: core-domain)

**Status:** complete
**Tasks:** 7/7 complete, 0 blocked

## Executed tasks
- T-1: root virou `[workspace]` puro (`resolver = "3"`, members exatos `crates/ddc-core` + `crates/ddc-adapters`), `[workspace.lints]` (`unsafe_code = "deny"`, clippy `unwrap_used`/`expect_used`/`panic` = warn), `clippy.toml`, `src/main.rs` removido, `Cargo.lock` fechado (só `thiserror` 2.0.21 + deps de proc-macro) — `b21e531` (chore)
- T-2: domínio VCP — `VcpCode` (consts MCCS, `Display` `0x10`), `VcpValue`, `MonitorId`/`MonitorInfo`, `Feature`/`FeatureKind`/`Access`/`Risk` (D-2026-09-25-core-domain-2), `Confirm`, `FeatureReading`, `DdcError` (inclui `ValueNotAllowed`), `risk_for_code` com seed Safe e default `Dangerous` — `243befb`
- T-3: parser próprio de capabilities (`Capabilities::parse`, `declares`, `feature`). Tolerante: hex colado/minúsculo, tags coladas, lista NC colada, tag desconhecida crua (com aninhamento), hex inválido e garbage pós-fechamento ignorados; só parêntese não fechado → `Transport`. Fixture RTK verbatim — `c1915fa`
- T-4: port driven `MonitorBackend` (só assinaturas) + fake `InMemoryMonitorBackend` em `ddc-adapters` (builder `FakeMonitor`: caps cru, valor por código, quirk `ignoring_writes_to`; log ordenado `BackendCall`; `MonitorNotFound`/`UnsupportedFeature`/`Transport` para o que não foi configurado) — `aee8df1`
- T-5: port driving `MonitorControl` (`list_monitors`, `capabilities`, `get_feature`) implementado por `SoftwareOsd<B: MonitorBackend>`; caps sob demanda com cache por monitor, `get_feature` sempre lê o backend e reporta `declared_in_capabilities`, max cacheado — `7ab53e4`
- (fix de T-3) comentário do fixture em `capabilities/tests.rs` citava `/dev/i2c-5`, o que o gate 5.8 bloquearia — `5a419c9` (test)
- T-6: `set_feature` — 1) `Dangerous` + `Confirm::No` → `DangerousWriteNotConfirmed` sem nenhuma chamada ao backend; 2) NC com lista → `ValueNotAllowed`, senão max conhecido (cache, ou leitura se declarado no caps) ou `UnsupportedFeature`, `value > max` → `InvalidValue`; 3) `write_vcp`; 4) um único `read_vcp`, que é o valor retornado e atualiza o cache. Regras puras em `feature.rs` (`authorize_write`, `Feature::requires_known_max`, `Feature::validate_write`); `ReadOnly` nunca é gravável — `da0ba15`
- T-7: README (status, layout real, seção "Monitor-write safety") + `CHANGELOG.md` (Keep a Changelog, `## [Unreleased]`) — `e8de5ba` (docs)

## Blocked tasks
- (nenhuma)

## Desvios do PLAN
- T-5 tocou `crates/ddc-adapters/src/in_memory.rs` (+ `in_memory/tests.rs`), fora dos seus files: fake `Clone` com estado compartilhado (`Arc`), para ler o log depois de mover o backend para `SoftwareOsd`. Descartados: getter `backend()` no `SoftwareOsd` (bypass das regras de escrita) e forwarding `impl MonitorBackend for &T` no core (gate 5.3).
- T-6 tocou `crates/ddc-core/src/domain/mod.rs`, só para re-exportar `authorize_write`.
- Commit extra `5a419c9` (fix de T-3), para não misturar a correção com a T-5.
- `max_for_write` recebe `&Capabilities` em vez de `bool` (convenção). `authorize_write` usa `_ => Ok(())` e o doc do port não escreve `Confirm::Yes` (grep do gate 5.7).
- `Feature::validate_write` recusa `ReadOnly` (hoje todo código é `ReadWrite`, A-3).

## Files modified
- `Cargo.toml`, `Cargo.lock`, `clippy.toml`, `src/main.rs` (removido)
- `crates/ddc-core/Cargo.toml`, `crates/ddc-core/src/lib.rs`
- `crates/ddc-core/src/domain/{mod,error,vcp,monitor,feature,capabilities}.rs`, `crates/ddc-core/src/domain/feature/tests.rs`, `crates/ddc-core/src/domain/capabilities/tests.rs`
- `crates/ddc-core/src/ports/{mod,monitor_backend,monitor_control}.rs`
- `crates/ddc-core/src/app/{mod,software_osd}.rs`
- `crates/ddc-core/tests/monitor_control/{main,support,read_features,write_features}.rs`
- `crates/ddc-adapters/Cargo.toml`, `crates/ddc-adapters/src/{lib,in_memory}.rs`, `crates/ddc-adapters/src/in_memory/tests.rs`
- `README.md`, `CHANGELOG.md`

## Tests
- Total: 52 (ddc-adapters unit 10, ddc-core unit 24, ddc-core integration `monitor_control` 18; doctests 0)
- Passing: 52
- Coverage: 100.00% de linhas (TOTAL de `cargo llvm-cov --workspace --locked --summary-only --fail-under-lines 80 --ignore-filename-regex '(^|[/\\])(main|build)\.rs$'`: 367 linhas, 0 perdidas; regiões 98.29%; exit 0)

## Gates no tree final (e8de5ba)
- build / test (52 passed) / fmt --check / clippy -D warnings / cross-check linux / cargo doc -D warnings: OK
- DoD da CONTEXT #1–#9: PASS; baseline do PROJECT (test, TODO/FIXME): OK
- Greps dos gates 4, 5.1–5.8, 5.11: sem saída (5.7a com hits esperados)
- `cargo audit`: não instalado (WARN 5.10); única dep nova `thiserror` 2.0.21

## Pendências (não bloqueiam)
- Validação no monitor físico (Deferred to PR review da CONTEXT): depende da phase `ddc-backends`.
- Itens manuais do DoD do PROJECT (CHANGELOG/README): no modo autônomo do `/jdi-issue` não há humano para `/jdi-confirm-dod` — ficam para a revisão do PR.
