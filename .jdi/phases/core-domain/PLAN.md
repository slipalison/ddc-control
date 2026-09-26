# Phase 1: Core domain — Plan  (slug: core-domain)

## Goal
Workspace cargo + crate `ddc-core`: port `MonitorBackend`, modelo VCP tipado, parser de capabilities, backend mock (`InMemoryMonitorBackend` em `ddc-adapters`) e testes.

## Locked decisions (from CONTEXT.md)
- D-1: Hexagonal. D-2: core depende só de `thiserror`, parser próprio, fake em `ddc-adapters` como dev-dep.
- D-2026-09-25-core-domain-1: workspace `[workspace]` puro com os 2 crates; `src/main.rs` removido.
- D-2026-09-25-core-domain-2: shape de `Feature`. D-2026-09-25-core-domain-3: seed de risco, fora dela = `Dangerous`.
- D-2026-09-25-core-domain-4: `get_feature` sempre lê o backend; `set_feature` sem max conhecido = `UnsupportedFeature`.
- D-2026-09-25-core-domain-5: shape de `Capabilities`; parser tolerante, só falha (`Transport`) com parênteses desbalanceados.

## Assumptions (cadeia autônoma, sem AskUserQuestion)
- A-1 Ports (`&self`, só tipos do core, retorno `Result<_, DdcError>`): `MonitorBackend { enumerate() -> Vec<MonitorInfo>; read_capabilities(&MonitorId) -> String; read_vcp(&MonitorId, VcpCode) -> VcpValue; write_vcp(&MonitorId, VcpCode, u16) }`; `MonitorControl { list_monitors; capabilities(&MonitorId) -> Capabilities; get_feature(&MonitorId, VcpCode) -> FeatureReading; set_feature(&MonitorId, VcpCode, u16, Confirm) -> VcpValue }`. `VcpValue { current: u16, max: u16 }`.
- A-2 A impl do driving port é `SoftwareOsd<B: MonitorBackend>` em `app/` (sem "Service", anti-padrão do skill hexagonal). Caches (caps por monitor, max por `(MonitorId, VcpCode)`) em `std::sync::Mutex`, lock via `unwrap_or_else(PoisonError::into_inner)`.
- A-3 Sem catálogo ainda (vem em `full-osd-control`): `access = ReadWrite` para todo código; `kind` é `NonContinuous` se o caps traz lista, senão `Continuous`.
- A-4 Max em `set_feature`: cache de leitura anterior; se não houver e o código for declarado no caps, lê do backend; senão `UnsupportedFeature`. NC com `allowed_values` valida pertença à lista, com o novo variant `ValueNotAllowed { code, value }` (max de NC não é confiável em scalers).
- A-5 Caps carregado sob demanda e cacheado por monitor; falha de leitura/parse propaga.
- A-6 Testes de caso de uso em `crates/ddc-core/tests/monitor_control/` (um só binário: `main.rs` + módulos). Unit test em `src/` usando o fake não compila: o ciclo de dev-dep gera duas cópias de `ddc_core`. Binário único também evita `dead_code` nos helpers.
- A-7 T-1 declara todo o grafo de deps: o `Cargo.lock` fecha ali e nenhuma task posterior toca `Cargo.*`.
- A-8 `foo/tests.rs` irmão (`#[cfg(test)] mod tests;`) conta como parte do `files_modified` de `foo.rs`.
- A-9 Nomes de teste do DoD são únicos no workspace (o verify filtra por substring e espera `1 passed`).

## Tasks
Todas as tasks usam o specialist `jdi-doer-ddc-control`. Cada commit exige `cargo fmt --all --check`, `cargo clippy --workspace --all-targets --locked -- -D warnings` e `cargo test --workspace --locked` verdes. O dev é Linux nativo; `cargo` puro basta.

### Wave 1

#### T-1: Converter o crate raiz em workspace cargo
- **Specialist:** jdi-doer-ddc-control
- **Files modified:** `Cargo.toml`, `Cargo.lock`, `clippy.toml`, `src/main.rs` (delete), `crates/ddc-core/Cargo.toml`, `crates/ddc-core/src/lib.rs`, `crates/ddc-adapters/Cargo.toml`, `crates/ddc-adapters/src/lib.rs`
- **Acceptance:**
  - Root: `resolver = "3"`, members exatos, `[workspace.package]` (edition 2024), `[workspace.dependencies]` (`thiserror = "2"` + path deps), `[workspace.lints]` (`unsafe_code = "deny"`; clippy `unwrap_used`/`expect_used`/`panic` = warn); `[lints] workspace = true` em cada crate; `clippy.toml` com `allow-unwrap-in-tests`/`allow-expect-in-tests`.
  - `ddc-core`: deps `thiserror`, dev-deps `ddc-adapters`, e `lib.rs` com `//!`, `#![forbid(unsafe_code)]`, `#![warn(missing_docs)]`. `ddc-adapters` depende de `ddc-core`.
  - Os verifies #1–#3 do DoD da CONTEXT passam. O lock é gerado uma vez sem `--locked` e vai no mesmo commit.
- **Dependencies:** none
- **Test:** `cargo build --workspace --locked && cargo test --workspace --locked`
- **Status:** completed

### Wave 2

#### T-2: Modelo de domínio VCP + tabela de risco seed
- **Specialist:** jdi-doer-ddc-control
- **Files modified:** `crates/ddc-core/src/lib.rs`, `crates/ddc-core/src/domain/mod.rs`, `crates/ddc-core/src/domain/error.rs`, `crates/ddc-core/src/domain/vcp.rs`, `crates/ddc-core/src/domain/monitor.rs`, `crates/ddc-core/src/domain/feature.rs`
- **Acceptance:**
  - `DdcError`: `MonitorNotFound(MonitorId)`, `UnsupportedFeature(VcpCode)`, `InvalidValue { code, value, max }`, `ValueNotAllowed { code, value }`, `DangerousWriteNotConfirmed(VcpCode)`, `Timeout`, `Transport(String)`. `VcpCode(u8)` com consts dos códigos da seed + 0xE0 e `Display` `0x10`. `MonitorId(String)` e `MonitorInfo { id, manufacturer, model, serial }`.
  - `Feature`/`FeatureKind`/`Access`/`Risk` conforme D-2026-09-25-core-domain-2; `Confirm { Yes, No }`; `FeatureReading { feature, value, declared_in_capabilities }`. `risk_for_code` usa as consts (sem número mágico) e cai em `Dangerous` por default.
  - Testes: todos os códigos da seed; `0xE0..=0xFF` e os códigos do caps RTK fora da seed (0x02 0x0B 0x0C 0x52 0xAC 0xAE 0xB2 0xB6 0xC6 0xC8 0xDF) dão `Dangerous`; `Display` coberto. Todo item `pub` com `///`.
- **Dependencies:** T-1
- **Test:** `cargo test -p ddc-core --locked`
- **Status:** completed

### Wave 3 (parallel-eligible)

#### T-3: Parser da capabilities string MCCS
- **Specialist:** jdi-doer-ddc-control
- **Files modified:** `crates/ddc-core/src/domain/mod.rs`, `crates/ddc-core/src/domain/capabilities.rs`
- **Acceptance:**
  - `Capabilities` (`Default`) + `parse(&str) -> Result<Self, DdcError>`. Aceita hex sem espaço/minúsculo (`0c10`), tags coladas e lista NC colada (`14(0102)`). Tag desconhecida, mesmo aninhada, vai crua para `unknown_tags`. Garbage após o fechamento e hex inválido são ignorados. Só parêntese não fechado gera `Transport`. Funções ≤30 linhas.
  - `declares(VcpCode) -> bool`; `feature(VcpCode) -> Feature` (kind/`allowed_values` do caps + `risk_for_code`, A-3).
  - `parses_real_rtk_caps_string` com a string verbatim da CONTEXT: prot `monitor`, type `LCD`, model `RTK`, cmds `01 02 03 07 0C E3 F3`, 28 códigos, listas exatas de 0x14/0x60/0xCC/0xD6, 0x62 ausente, `mccs_version (2,2)`, `unknown_tags` {mswhql: `1`, asset_eep: `40`}.
  - `tolerates_missing_spaces_and_unknown_tags`, mais testes de desbalanceado → `Transport` e de `feature()` para 0x10/0x14/0x62.
- **Dependencies:** T-2
- **Test:** `cargo test -p ddc-core --locked capabilities`
- **Status:** completed

#### T-4: Port `MonitorBackend` + fake `InMemoryMonitorBackend`
- **Specialist:** jdi-doer-ddc-control
- **Files modified:** `crates/ddc-core/src/lib.rs`, `crates/ddc-core/src/ports/mod.rs`, `crates/ddc-core/src/ports/monitor_backend.rs`, `crates/ddc-adapters/src/lib.rs`, `crates/ddc-adapters/src/in_memory.rs`
- **Acceptance:**
  - `MonitorBackend` (A-1): só assinaturas + `///`, sem impl no core.
  - O fake tem builder (monitor + caps cru, valor por código, quirk que descarta writes de um código), log ordenado `BackendCall` e erros `MonitorNotFound`/`UnsupportedFeature` para o que não foi configurado. Nenhuma regra de negócio (hexagonal, regra 14), nenhum `pub trait` e nenhum `unwrap`/`expect`.
  - Unit tests para cada comportamento acima.
- **Dependencies:** T-2
- **Test:** `cargo test -p ddc-adapters --locked`
- **Status:** completed

### Wave 4

#### T-5: Driving port `MonitorControl` (leitura) + `SoftwareOsd`
- **Specialist:** jdi-doer-ddc-control
- **Files modified:** `crates/ddc-core/src/lib.rs`, `crates/ddc-core/src/ports/mod.rs`, `crates/ddc-core/src/ports/monitor_control.rs`, `crates/ddc-core/src/app/mod.rs`, `crates/ddc-core/src/app/software_osd.rs`, `crates/ddc-core/tests/monitor_control/main.rs`, `crates/ddc-core/tests/monitor_control/support.rs`, `crates/ddc-core/tests/monitor_control/read_features.rs`
- **Acceptance:**
  - `MonitorControl` com `list_monitors`/`capabilities`/`get_feature`, implementado por `SoftwareOsd` (A-2, A-5). `get_feature` sempre chama `read_vcp`, preenche `feature`/`declared_in_capabilities` e guarda o max no cache.
  - `get_feature_succeeds_for_code_absent_from_capabilities`: caps RTK + 0x62 no fake → `Ok`, `declared_in_capabilities == false`, o valor do fake, `ReadVcp(0x62)` no log.
  - Testes extras: 0x10 declarado/Continuous, `allowed_values` de 0x60, caps lido uma vez só, `MonitorNotFound`, caps desbalanceado → `Transport`, `list_monitors`. Os greps dos gates 5.3/5.4 não retornam nada.
- **Dependencies:** T-3, T-4
- **Test:** `cargo test -p ddc-core --locked --test monitor_control`
- **Status:** completed

### Wave 5

#### T-6: `set_feature` com guarda de escrita + read-back
- **Specialist:** jdi-doer-ddc-control
- **Files modified:** `crates/ddc-core/src/domain/feature.rs`, `crates/ddc-core/src/ports/monitor_control.rs`, `crates/ddc-core/src/app/software_osd.rs`, `crates/ddc-core/tests/monitor_control/main.rs`, `crates/ddc-core/tests/monitor_control/support.rs`, `crates/ddc-core/tests/monitor_control/write_features.rs`
- **Acceptance:**
  - Ordem:
    1. `Dangerous` + `Confirm::No` → `DangerousWriteNotConfirmed`, sem nenhuma chamada ao backend (nem caps).
    2. NC com lista → `ValueNotAllowed` se o valor não estiver nela. Senão: max (A-4) ou `UnsupportedFeature`; `value > max` → `InvalidValue`.
    3. `write_vcp`.
    4. Um único `read_vcp`, que é o valor retornado e atualiza o cache.
  - Regras puras em `Feature`; orquestração em `SoftwareOsd`.
  - DoD `dangerous_write_without_confirm_is_rejected`: 0x60 + um código fora da seed, log do fake vazio.
  - DoD `write_value_above_max_is_rejected`: após `get_feature` de 0x10 (max 100), `set` de 101 → `InvalidValue`, sem chamada nova.
  - DoD `write_reads_back_value_after_success`: retorna a leitura pós-write; com o quirk, retorna o valor antigo; log termina em `WriteVcp` → `ReadVcp`.
  - Testes extras: código ausente e nunca lido → `UnsupportedFeature`; aceito depois de `get_feature`; NC fora da lista; Dangerous confirmado escreve. `grep -RnE 'Confirm::Yes' crates/*/src | grep -v 'tests\.rs:'` sem saída (gate 5.7). Cobertura ≥ 80%.
- **Dependencies:** T-5
- **Test:** `cargo test --workspace --locked` + cobertura (abaixo)
- **Status:** completed

### Wave 6

#### T-7: README + CHANGELOG (DoD manual do PROJECT)
- **Specialist:** jdi-doer-ddc-control
- **Files modified:** `README.md`, `CHANGELOG.md`
- **Acceptance:**
  - README (em inglês): status com `core-domain` concluída (2 crates, ainda sem binário), layout real e regra de escrita segura (`Risk`/`Confirm`, read-back).
  - `CHANGELOG.md` (Keep a Changelog) com `## [Unreleased]` listando o que a phase adicionou.
- **Dependencies:** T-6
- **Test:** `grep -q '^## \[Unreleased\]' CHANGELOG.md && ! grep -q 'not started' README.md`
- **Status:** completed

## Execution
- Total tasks: 7 | Waves: 6 (só a W3 é paralelizável) | Speedup estimado: 1.2x
- Commits com scope `core-domain`: T-1 `chore`, T-2 a T-6 `feat`, T-7 `docs`. D-XX citada no corpo; nunca junto com `.jdi/`.

## Files modified (all tasks)
- `Cargo.toml`, `Cargo.lock`, `clippy.toml`, `src/main.rs` (removido)
- `crates/ddc-core/Cargo.toml`, `crates/ddc-core/src/lib.rs`
- `crates/ddc-core/src/domain/{mod,error,vcp,monitor,feature,capabilities}.rs`
- `crates/ddc-core/src/ports/{mod,monitor_backend,monitor_control}.rs`
- `crates/ddc-core/src/app/{mod,software_osd}.rs`
- `crates/ddc-core/tests/monitor_control/{main,support,read_features,write_features}.rs`
- `crates/ddc-adapters/Cargo.toml`, `crates/ddc-adapters/src/{lib,in_memory}.rs`
- `README.md`, `CHANGELOG.md`

## Test requirements
- `cargo test --workspace --locked`; `cargo fmt --all --check && cargo clippy --workspace --all-targets --locked -- -D warnings`
- Cobertura ≥ 80% linhas: `cargo llvm-cov --workspace --locked --summary-only --fail-under-lines 80 --ignore-filename-regex '(^|[/\\])(main|build)\.rs$'`
- DoD da CONTEXT: #1–#3 → T-1 (seguem verdes depois); parser → T-3; `get_feature` → T-5; escrita → T-6.
