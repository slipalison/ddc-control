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

## Iter 2 (critic row 10, W-1, W-2, W-3)

**Status:** complete. 5 commits, 0 blocked. Specs travadas: D-2026-09-26-cli-1, -2 e -3.

### Commits
- `ba18b7a test(cli)`: crítico da linha 10 do DoD. O teste nomeado `monitor_flag_matches_by_exact_id_index_or_unique_substring_and_refuses_ambiguous_matches` ganhou os ids `RTK-QHD-1` e `RTK-QHD-10`. `-m RTK-QHD-1` precisa escolher `RTK-QHD-1`, e `-m rtk-qhd-1` (só uma parte do id) dá exit 3, ambíguo, sem tocar o monitor.
  - **Prova de mutação:** apagar localmente `select.rs:69-71` (o ramo de id exato) faz o teste nomeado falhar em `selection.rs:72` (exit 3 em vez de 0) e o Verify deixa de imprimir OK. `select.rs` foi restaurado com `git checkout` e a mutação não foi commitada.
- `a44b658 fix(cli)`: W-1, D-2026-09-26-cli-1. `max_for_write` não recebe mais o caps.
  - Sem max em cache, faz UMA `read_vcp`, com o código declarado ou não. Leitura falha → o erro dela: Transport, Timeout ou MonitorNotFound, e `UnsupportedFeature` quando o monitor não responde o código.
  - A ordem não mudou: `authorize_write` → max → lista NC / `value <= max` → write → read-back. Doc do port `MonitorControl::set_feature` e da struct atualizadas.
  - Testes do core:
    - `write_without_read_or_capabilities_is_unsupported` virou `write_without_capabilities_reads_the_max_first`;
    - `undeclared_code_never_read_is_not_written` virou `safe_write_to_an_answered_but_undeclared_code_reads_its_max_first` (log `ReadCapabilities, ReadVcp(0x62), WriteVcp(0x62), ReadVcp(0x62)`);
    - novos: `undeclared_code_is_bounded_by_the_max_it_reads`, `failed_max_read_is_reported_and_nothing_is_written` (Transport, Timeout e MonitorNotFound, sem WriteVcp) e `code_the_monitor_does_not_answer_is_unsupported_and_not_written`;
    - `undeclared_code_read_earlier_is_written_without_reading_again` fixa o reuso do max.
  - Fixture `--fake`: 0x62 saiu do caps, como no RTK. `reads.rs` foi ajustado: 5 códigos no caps, `declared_in_capabilities: false` para 0x62.
  - Teste de binário novo: `set volume 40` → exit 0; `set volume 101` → exit 4.
  - README: Known limitations e Monitor-write safety.
- `e514629 fix(cli)`: W-2 no adapter, D-2026-09-26-cli-2.
  - `RetryPolicies { vcp, capabilities }` usa as consts `VCP_BACKOFF` (50 ms), `CAPABILITIES_BACKOFF` (500 ms) e `MAX_ATTEMPTS` (3). O worker escolhe a policy pelo tipo de transação, e `Failure::Exhausted` carrega o `max_attempts` da policy.
  - Teste com relógio virtual: `capabilities_retries_wait_500_ms_while_vcp_retries_wait_50_ms_within_budget`.
    - Caps flaky dorme [500, 500]; VCP flaky dorme [50, 50].
    - Caps falhando com o orçamento de 8 s tenta 3 vezes. Com 700 ms, só 2 tentativas e 1 pausa. Com 500 ms, 1 tentativa.
  - `failed_capabilities_read_checks_presence_when_it_fits` foi ajustado para 1000 ms + enumeração.
- `a2260dc fix(cli)`: W-2 no cache + W-3, D-2026-09-26-cli-3.
  - `invalidate` só marca o id em memória e não apaga mais o arquivo; o arquivo só é trocado por uma releitura bem-sucedida (escrita atômica).
  - Só se persiste caps que `Capabilities::parse` aceita. Arquivo que o parse recusa conta como miss (endurecimento no lado da leitura, para arquivos gravados por versões antigas).
  - Testes novos:
    - `failed_refresh_keeps_the_previous_cache_file_for_later_runs`;
    - `a_failed_refresh_is_retried_by_the_next_read_of_the_same_run`;
    - `unparseable_capabilities_are_returned_but_never_persisted` (cache frio e quente);
    - `an_unparseable_cache_file_is_a_miss_and_gets_replaced`.
  - Ajustados: `software_osd_runs_on_a_borrowed_cache_that_can_still_be_invalidated` (o arquivo fica, a próxima leitura vai ao backend) e `invalidation_holds_when_the_stale_file_cannot_be_replaced` (renomeado).
  - Os 3 testes novos de comportamento e o ajustado falham contra o `caching.rs` antigo.
- `718ed4b docs(cli)`: README (Status, cache, backoff, Retries, CachingMonitorBackend), CHANGELOG `[Unreleased]` (bullets reescritos para o comportamento atual) e `docs/linux-ddc-setup.md` (backoff).
  - O Status agora diz que as escritas NÃO foram validadas em hardware.

### Gates (exit codes)
- build 0; test 0 (189 passed, 0 failed, 5 ignored); `fmt --check` 0; clippy `-D warnings` 0.
- Check Linux `-p ddc-core -p ddc-adapters -p ddc-cli --target x86_64-unknown-linux-gnu` 0.
- Check Windows `-p ddc-cli -p ddc-adapters --target x86_64-pc-windows-msvc` 0; `-p ddc-adapters --features ddc-hi` para Windows 0.
- llvm-cov 0: `TOTAL 2064 82 96.03% 260 12 95.38% 1343 43 96.80% 0 0 -`.
- `cargo audit` 1, só com RUSTSEC-2018-0005 (serde_yaml 0.7.5) e RUSTSEC-2024-0320 (yaml-rust 0.4.5).
- DoD da CONTEXT: os 9 Verify imprimem OK.

### Hardware (somente leitura; nenhum `set`/`reset`)
- `list` 1.12 s, os 2 monitores.
- 6 `caps --refresh -m RTK` seguidos: exit 0 em todos, com 6.28 s, 4.66 s, 4.67 s, 4.67 s, 4.67 s e 4.67 s. Numa rodada anterior, logo após o commit do backoff, também foram 6/6 ok (3.64–4.68 s). Antes do fix: 2/4 ok. O pior caso de caps observado foi ~5.2 s sem o enumerate, abaixo do orçamento de 8 s.
- O arquivo de cache é idêntico à fixture `rtk_qhd_hdr_caps.txt`. `caps` a partir do cache: 1.12 s.
- `get volume -m RTK` 1.15 s (30/100, com o aviso de "not declared"); `get brightness --json` 1.14 s (100/100).
- `caps -m LG` (DDC mudo): exit 6 em 3.24 s. Era ~2.3 s antes do backoff de 500 ms.
- `DDC_HW_TESTS=1 ... --test real_monitor`: 5/5 ok.
