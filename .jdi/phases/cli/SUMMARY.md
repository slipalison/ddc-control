# Phase 3: CLI — Summary  (slug: cli)

**Status:** complete
**Tasks:** 8/8 complete, 0 blocked (+1 commit `fix` fora da lista, vindo da validação no hardware)

## Executed tasks
- T-1: `CachingMonitorBackend<B>` + `default_cache_dir()` em `ddc-adapters/src/caching.rs`, sempre compilado; só `Ok` persistido, escrita atômica (tmp único + `rename`), nome de arquivo `[A-Za-z0-9-]` + `_XX` (injetivo, sem `/`, `\`, `.`), `invalidate` com marca em memória, impl também para `&CachingMonitorBackend<B>`. DoD #2/#3 verdes. — `7a8d301 feat(cli)`
- T-2: `FakeMonitor::with_vcp_failure(code, error)`; chamada continua logada, valor intacto, outros códigos inalterados. — `d0f428c test(cli)`
- T-3: crate `ddc-cli` (clap 4.6.7 derive, serde 1.0.229, serde_json 1.0.151; dev: assert_cmd 2.2.2, predicates 3.1.4), `Cli` com `--monitor/-m`, `--json`, `--fake` oculto; `<VCP>` decimal/`0x`/6 nomes via `SHORTCUTS` sobre as consts do core; `<VALUE>` até 65535. Lock sem bump de dep existente (só `nom` passa a citar `memchr 1.0.2`). DoD #1/#8 verdes. — `82fe15f feat(cli)`
- T-4: `output.rs` — `Format {Text, Json}`, `Printer` (um método por comando), DTOs serde só aqui, `monitor_line` pública; avisos/erros sempre em stderr. — `a23e7f9 feat(cli)`
- T-5: `select.rs` (id exato → índice só-dígitos sem fallback → substring única case-insensitive) e `exit.rs` (`Exit` `#[repr(u8)]` 0/2/3/4/5/6, `CliError`, `match` exaustivo sobre `DdcError`, mensagem do 5 pede `--yes`). — `a294b12 feat(cli)`
- T-6: `run`, `report_startup_failure`, fixture `FAKE-CLI-TEST` (+ `SIMULATED_TIMEOUT = VcpCode(0xDF)` → `Timeout`), `main.rs` só composition root, testes ponta a ponta (`tests/cli/`). DoD #4–#7 verdes. — `babaf5e feat(cli)`
- T-7: contrato de exit codes pelo binário `--fake` (3, 4, 5, 6 + 2 do clap + 0). DoD #9 verde. — `14871a7 test(cli)`
- (extra) fix: aviso do `get` dizia "not declared in capabilities" mesmo quando a leitura de caps falhou (o core reporta caps ilegível como "declara nada"); agora cita as duas causas. Começou por teste falhando. — `bf7328a fix(cli)`
- T-8: README (status, install, usage, `--monitor`, `--json`, atalhos, exit codes, cache por SO + R-1, limitações observadas no hardware; saídas reais capturadas só com leitura; sem `--fake`) e CHANGELOG `[Unreleased]`. — `b600d0e docs(cli)`

## Blocked tasks
- (nenhuma)

## Desvios do PLAN (todos dentro das decisões travadas)
- `CliError::Ddc { monitor: Option<MonitorId>, error }` em vez de `Ddc(DdcError)`: a mensagem cita o id do monitor quando já escolhido.
- Renderização como `Printer` com um método por comando (em vez de fns livres); `Exit::Usage = 2` existe só para completar a tabela e é testado contra `clap::Error::exit_code()`.
- A falha de `DdcHiMonitorBackend::new()` é renderizada por `ddc_cli::report_startup_failure` (lib, testada), para `main.rs` ficar sem lógica.
- Fixture exporta também `FIXTURE_ID` e `FIXTURE_CAPS` (consts usadas pelos testes).

## Achados no hardware real (só leitura; nenhum `set` rodado)
- Cache funciona: `get brightness -m RTK` frio 3.74s → quente 1.16s; `get volume` quente 1.14s; `caps` frio 3.64s → do cache 1.11s. O arquivo do cache bate byte a byte com `crates/ddc-core/tests/fixtures/rtk_qhd_hdr_caps.txt`.
- O RTK recusa uma leitura de capabilities feita poucos segundos depois da anterior (`Expected DDC/CI length bit` / `invalid offset returned from DDC/CI`, após 3 tentativas): 4 `caps --refresh` seguidos → 2 ok, 2 exit 6; com 6s de intervalo → 4/4 ok. O teste de hardware in-process do adapter passa. Com o cache quente o `get`/`set` nem lê caps, então só afeta `caps --refresh` e o primeiro uso.
- Consequência para o orquestrador: antes do round-trip de `set brightness`, popular o cache (`-m RTK caps`) e esperar ≥6s após qualquer leitura de caps; com cache frio + caps falhando, `set brightness` sairia 4 ("feature 0x10 is not supported") porque o core não conhece o máximo.
- Limitação de design (não corrigida, decisão do orquestrador): `set` num código não declarado no caps (ex.: `volume` 0x62 no RTK) sempre sai 4, porque o core só escreve contra um máximo vindo do caps ou de uma leitura anterior NO MESMO processo, e cada execução da CLI é um processo novo. Correção possível sem violar D-cli-3: se `set_feature` devolver `UnsupportedFeature`, a CLI faz `get_feature` (leitura) e repete `set_feature` uma vez — a recusa de escrita perigosa continua antes de tocar o backend. Documentado em README § Known limitations.

## Files modified
- `Cargo.toml`, `Cargo.lock`
- `crates/ddc-adapters/src/lib.rs`, `crates/ddc-adapters/src/caching.rs`, `crates/ddc-adapters/src/caching/tests.rs`, `crates/ddc-adapters/tests/caching_backend.rs`
- `crates/ddc-adapters/src/in_memory.rs`, `crates/ddc-adapters/src/in_memory/tests.rs`
- `crates/ddc-cli/Cargo.toml`, `crates/ddc-cli/src/{lib,main,args,output,select,exit,run,fixture}.rs`, `crates/ddc-cli/src/{args,output,select,exit}/tests.rs`
- `crates/ddc-cli/tests/cli/{main,support,selection,reads,writes,exit_codes}.rs`
- `README.md`, `CHANGELOG.md`

## Tests
- Total: 185 (180 passing + 5 `#[ignore]` de hardware, não executados no gate)
- Passing: 180, failing: 0
- Coverage: 96.74% linhas (`cargo llvm-cov --workspace --locked --summary-only --fail-under-lines 80 --ignore-filename-regex '(^|[/\\])(main|build)\.rs$'`, linha TOTAL: `TOTAL 2046 84 95.89% 257 12 95.33% 1321 43 96.74%`)

## Gates (exit codes)
- `cargo build --workspace --locked` = 0; `cargo test --workspace --locked` = 0; `cargo fmt --all --check` = 0; `cargo clippy --workspace --all-targets --locked -- -D warnings` = 0
- `cargo check -p ddc-core -p ddc-adapters -p ddc-cli --locked --target x86_64-unknown-linux-gnu` = 0
- `cargo check -p ddc-cli -p ddc-adapters --locked --target x86_64-pc-windows-msvc` = 0; `cargo check -p ddc-adapters --locked --features ddc-hi --target x86_64-pc-windows-msvc` = 0
- `cargo audit` = 1, só os já conhecidos RUSTSEC-2018-0005 (serde_yaml 0.7.5) e RUSTSEC-2024-0320 (yaml-rust 0.4.5, unmaintained); nenhum advisory nas deps novas (clap, serde, serde_json, assert_cmd, predicates).
- DoD da CONTEXT: #1–#9 todos OK.
