# Phase 2: DDC backends — Summary  (slug: ddc-backends)

**Status:** complete
**Tasks:** 6/6 complete, 0 blocked
**Modo:** ralph_loop, iter 1 (sem REVIEW.md prévio)

## Executed tasks
- T-1: `ddc-hi 0.4.1` opcional (`default-features = false`, `["ddc-i2c","ddc-winapi"]`) atrás da feature `ddc-hi` default-on; `Cargo.lock` +35 pacotes; verify #1 OK; check Windows verde; `cargo audit`: RUSTSEC-2018-0005 (`serde_yaml 0.7.5`, vuln) + RUSTSEC-2024-0320 (`yaml-rust`, unmaintained), ambos transitivos via `mccs-db` (WARN, previsto na CONTEXT). Commit `b0bac90`.
- T-2: `FakeMonitor::with_transient_capabilities_failures(n)` + unit test no fake; `explicit_capabilities_request_recovers_from_transient_failure` no core (log exato `[ReadCapabilities, ReadVcp, ReadCapabilities, ReadVcp]`). Commit `bb16639`.
- T-3: `DdcHiMonitorBackend` (newtype sobre `WorkerClient<DdcHiDisplays>`) + `DdcHiBudgets` (D-7: VCP 1s, caps 8s, enumerate 5s, `with_budgets`); worker único dono de todo `Display`/`Handle` (D-6), `recv_timeout` → `Timeout`, worker morto → `Transport`; seam `DisplaySource`/`DdcHandle`/`HandleError` (`pub(crate)`); identidade A-6 em `identity.rs`; traduções D-5/A-7/`DisplayInfo` em `hardware.rs`; asserção estática `Send + Sync` compila no check Windows. 6 testes do DoD + 13 extras. Casca não coberta = 36 linhas (≤ 40). Hook avisou 1060 linhas (WARN, não bloqueia; ~55% são testes). Commit `28ed51a`.
- T-4: `retry.rs` com `RetryPolicy` (3 tentativas / 50ms em consts nomeadas, `Default`) + `Clock`/`SystemClock`; worker aplica A-8 em caps/read/write com deadline do caller; job vencido não toca o hardware (`serve`); falha final → re-enumerate → `MonitorNotFound` se sumiu, senão `Transport` "… (gave up after attempt N of 3)". DoD #7 + `expired_write_is_never_sent_to_the_monitor`, `monitor_gone_after_a_failure_is_not_found`, `deadline_spent_finding_the_monitor_sends_nothing`. Testes spawnados usam política sem backoff; os diretos usam clock virtual. Cobertura TOTAL 95.23%. Commit `575f3b1`.
- T-5: `crates/ddc-adapters/tests/real_monitor.rs` com 4 testes `#[ignore]` + `DDC_HW_TESTS=1`, só leitura (`grep -n write_vcp` vazio; hits do grep 5.7 só dentro dos corpos `#[ignore]`). `cargo test --workspace --locked` → 4 ignored, 0 failed. Rodada real (read-only) no box: 4/4 ok — ver seção Hardware. No outro display, `Ok` também é aceito (só `MonitorNotFound`/outros erros reprovam), para o teste não quebrar numa máquina com um segundo monitor DDC funcional. Commit `3bbf778`.
- T-6: `docs/linux-ddc-setup.md` (inglês: libudev, `i2c-dev`, regra udev `uaccess` ou grupo `i2c` + verificação, nunca `sudo`, lista vazia = permissão, display mudo listado (D-4), ressalva NVIDIA); README (status phases 1–2, layout do `DdcHiMonitorBackend`, check Windows, libudev, comando dos testes de hardware, link do doc); CHANGELOG `[Unreleased]`. Verify #10 OK. Commit `1ffaab4`.

## Blocked tasks
- _(nenhuma)_

## Files modified
- `crates/ddc-adapters/Cargo.toml`, `Cargo.lock` (T-1)
- `crates/ddc-adapters/src/in_memory.rs`, `crates/ddc-adapters/src/in_memory/tests.rs`, `crates/ddc-core/tests/monitor_control/read_features.rs` (T-2)
- `crates/ddc-adapters/src/lib.rs`, `crates/ddc-adapters/src/ddc_hi_backend.rs`, `crates/ddc-adapters/src/ddc_hi_backend/{tests,identity,worker,hardware}.rs`, `crates/ddc-adapters/src/ddc_hi_backend/{identity,worker,hardware}/tests.rs` (T-3)
- `crates/ddc-adapters/src/ddc_hi_backend.rs`, `crates/ddc-adapters/src/ddc_hi_backend/retry.rs`, `crates/ddc-adapters/src/ddc_hi_backend/worker.rs`, `crates/ddc-adapters/src/ddc_hi_backend/worker/tests.rs` (T-4)
- `crates/ddc-adapters/tests/real_monitor.rs` (T-5)
- `docs/linux-ddc-setup.md`, `README.md`, `CHANGELOG.md` (T-6)
- Nenhum arquivo fora dos `files_modified` do PLAN. Nenhum `Cargo.*` tocado depois da T-1 (A-11).

## Tests
- Total: 90 (86 executados + 4 `#[ignore]` de hardware)
- Passing: 86/86, 0 failed, 4 ignored (`cargo test --workspace --locked`); antes da phase: 61
- Coverage: 95.23% de linhas. Linha TOTAL do `cargo llvm-cov --workspace --locked --summary-only --fail-under-lines 80 --ignore-filename-regex '(^|[/\\])(main|build)\.rs$'`, exit 0:
  `TOTAL 1118 67 94.01% 152 11 92.76% 755 36 95.23% 0 0 -`
- Casca não coberta (sem monitor): 36 linhas (≤ 40). São as delegações do newtype e a const-fn da asserção estática (13), `DdcHiDisplays::enumerate` + `impl DdcHandle for ddc_hi::Handle` (19), e o ramo de falha do `thread::Builder::spawn` (4).

## Gates na árvore final (exit codes)
- `cargo build --workspace --locked` = 0
- `cargo test --workspace --locked` = 0
- `cargo fmt --all --check` = 0
- `cargo clippy --workspace --all-targets --locked -- -D warnings` = 0
- `cargo check -p ddc-core -p ddc-adapters --locked --target x86_64-unknown-linux-gnu` = 0
- `cargo check -p ddc-adapters --locked --features ddc-hi --target x86_64-pc-windows-msvc` = 0 (inclui a asserção estática `Send + Sync`)
- `cargo check -p ddc-adapters --locked --no-default-features` = 0
- `cargo llvm-cov ...` = 0
- `cargo audit` = 1 (WARN previsto na CONTEXT, não bloqueia): RUSTSEC-2018-0005 (`serde_yaml 0.7.5`, recursão sem limite) e RUSTSEC-2024-0320 (`yaml-rust 0.4.5`, unmaintained). Os dois vêm de `mccs-db`, transitivo do `ddc-hi`. O único YAML que ele desserializa é o banco MCCS embutido no próprio crate, nunca entrada externa.
- Greps 5.1–5.8 sem saída. O `#[allow]` novo fica só em `worker/tests.rs`, com `// reason:`, no `clippy::panic` do fake que simula um bug de transporte.
- DoD da CONTEXT: #1–#10 = OK. DoD do projeto (TODO/FIXME sem issue): OK.

## Hardware (rodada read-only no box, `DDC_HW_TESTS=1 ... --ignored --test-threads=1 --nocapture`)
- 4/4 passed em 8.55s. Nenhuma escrita: o arquivo não contém `write_vcp`.
- enumerate: 1.16s / 1.20s / 1.13s / 1.11s (< 5s). Achou `GSM-LG-TV-SSCR2-01010101` (LG TV) e `RTK-RTK-QHD-HDR-01010101` (id de D-2 confirmado).
- read_capabilities: 2.53s (< 8s), igual ao fixture do core.
- read_vcp 0x10: 43.8ms → `{current: 100, max: 100}`. read_vcp 0xDF: 93.8ms → `current = 0x0202` (MCCS 2.2, confirma D-5; `max = 0xFFFF`).
- LG TV (DDC mudo): listada sem sondagem (D-4 confirmado). `read_vcp 0x10` → `Err(Timeout)` em 1.000s. Depois dela, o RTK respondeu em 287ms.

## Notas para o orquestrador / próximas phases
- **Display com DDC mudo vira `Timeout`, não `Transport`, com os budgets default.** As 3 tentativas falham em cerca de 100ms, e a checagem de presença de A-5 re-enumera o barramento (~1.1s). Isso passa do budget de 1s de VCP. O resultado está dentro de D-4 ("Transport ou Timeout"), mas o worker fica ocupado ~1.2s a cada falha esgotada, e o próximo caller espera por isso (o RTK levou 287ms em vez de ~45ms).
- **Primeira leitura sem `enumerate()` prévio.** Num backend novo, `read_vcp` paga o enumerate implícito (~1.1s) dentro do budget de VCP (1s) e dá `Timeout` no hardware de dev. O `SoftwareOsd` não enumera antes. A phase `cli` deve chamar `enumerate()`/`list_monitors()` primeiro ou ajustar `DdcHiBudgets`. Isso se soma à nota de cache de caps de D-7.
- **`hardware_mute_displays_fail_fast_and_worker_recovers` aceita `Ok` em outros displays.** O PLAN diz "com erro só Transport/Timeout". O teste reprova `MonitorNotFound`, outros erros e estouro de budget+250ms, mas aceita `Ok`, para não quebrar numa máquina com um segundo monitor DDC funcional. No box a TV deu `Timeout`.
- **Hook: WARN de tamanho na T-3** (9 arquivos / 1060 linhas; cerca de 55% são testes). Não bloqueou. O PLAN estimava ~600.
- Validação funcional no Windows real continua diferida (CONTEXT § Deferred). Aqui só rodaram o cross-check e a asserção estática.
