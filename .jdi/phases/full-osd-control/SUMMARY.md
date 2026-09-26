# Phase 4: Full OSD control — Summary  (slug: full-osd-control)

**Status:** complete
**Tasks:** 8/8 complete, 0 blocked (+1 correção extra fora do plano, sinalizada abaixo)

## Executed tasks
- T-1: `ddb22e1` fix — cada tentativa de `Worker::transact` roda em `catch_unwind(AssertUnwindSafe)`; panic vira `HandleError` final (`Panicked`) → `Transport("ddc-hi panicked: <msg>")`, 1 tentativa, sem checagem de presença; worker e tabela intactos. Panic em `enumerate` (fora de D-6) ainda derruba o worker (`worker_gone` coberto por `CrashingDisplays`). Teste `a_panic_inside_one_transaction_fails_only_that_call_and_the_worker_keeps_serving` + `a_panic_is_never_retried_nor_followed_by_a_presence_check` + payload não-texto. Hardware `hardware_reading_0x7e_never_stops_the_worker` (verde no RTK).
- T-2: `c034a94` feat — `domain::mccs_catalog`: tabela única e ordenada de 39 `CatalogEntry { code, name, alias, kind, access, risk, values }` (resets com `(0x01, "Reset")`, D-8), `catalog`, `catalog_entry` (busca binária), `catalog_codes` (derivado por `const`), `value_name`, `value_for_name` (A-3), `code_for_alias`, `hertz`, `version_from_u16`, `interpret` (0xAE Hz, 0xC9/0xDF versão, 0xAC bruto). `risk_for_code` e `Capabilities::feature` leem o catálogo. Consts novas: `VERTICAL_FREQUENCY`, `FIRMWARE_LEVEL`, `VCP_VERSION`. Testes com premissa mudada reescritos (0x52→0x8D; 0xFD/0xFF RO Safe; preset sem caps = NC).
- T-3: `461ddb5` fix — `Feature::ensure_writable` (Table, depois RO), `is_readable` (WO nunca lido), `requires_known_max = Continuous && sem lista`; `validate_write` usa lista do caps, senão valores do catálogo; `set_feature`: authorize → caps → ensure_writable → max (só contínuo e legível) → validate → write → read-back (pulado em WO, devolve o valor enviado). DoD #3/#7/#8/#9 + `reset_writes_one_and_skips_read_back`.
- T-4: `16beaa3` feat — `ProbedFeature` + `MonitorControl::probe_undeclared_features` (só leitura, ordem do catálogo, 1 leitura por código, `read_and_track` compartilhado com `get_feature`, falha por código não interrompe; `MonitorNotFound` antes/durante encerra com `Err`). DoD #6 + extras (max lembrado, caps ilegível sonda tudo, monitor inexistente, monitor some no meio).
- T-5: `114c215` feat — `SHORTCUTS`/`shortcut_name` removidos; `<VCP>` = alias do catálogo ou número; `ValueArg { Number, Name }`; nome de outra feature → `CliError::Usage` (exit 2) antes de qualquer I/O; `get`/`set` mostram significado (`1 (0x01) sRGB`), JSON com `value_name`/`interpreted` opcionais (JSON de `get volume` idêntico); `reset <factory|brightness-contrast|geometry|color> [--yes]` = `set <code> reset --yes` (escreve 0x01), imprime só o envio "(write-only, not read back)". Fixture com 0xCC e resets. DoD #11/#12 + teste D-8 pelo binário.
- T-6: `9b465d5` feat — `ddc-cli features [--probe] [--json]`: tabela com cabeçalho, uma linha por código em ordem (código, alias, C/NC/T, RO/WO/RW, risco, caps/probe, `atual/max` + significado ou `not supported by this monitor`/`not responding`, nome MCCS); JSON com todos os campos sempre presentes e `probe_status`; caps ilegível = aviso; `MonitorNotFound` em qualquer linha = exit 3. Fixture 0x7E → `Transport("ddc-hi panicked: …")`. DoD #10 + extras. `main.rs` intocado.
- **Extra (fora dos `files_modified`)**: `129febd` fix — `hardware.rs` + `hardware/tests.rs`: no Linux, resposta Get VCP cujo código ecoado difere do pedido é recusada como transiente (retry). Motivo: `ddc` 0.2.2 guarda o código ecoado em `ty` e nunca o compara (`// data[2] == vcp code from request`); no RTK, `features --probe` mostrou 0x7E = 80/100 (valor de 0x70). Prioridade 2 (segurança de escrita: max e read-back poderiam vir de outro código). No Windows `ddc-winapi` põe o tipo em `ty` e o `dxva2` valida — nada muda lá (`echoed_code` → `None`). Após a correção, 0x7E aparece `not responding` e os demais com seus valores.
- T-7: `959d7e3` docs — README (status fase 4, uso de `features [--probe]`, nomes de feature/valor + normalização, `reset`, tabela completa do catálogo gerada da tabela Rust, segurança, exit 2 para nome de outra feature, known limitations, exemplos reais só-leitura capturados no box, sem `--fake`) e CHANGELOG `[Unreleased]` (Added/Changed/Fixed).
- T-8: `c824622` docs — `docs/hardware-validation.md`: roteiro exato (build + originais, list/caps/testes de hardware, features/probe/get, 4 escritas SAFE reversíveis, 2 recusas, estado final, lista NUNCA). Doer não executou nenhuma escrita. `set preset 0x03` virou `set preset 3` (mesmo valor): a substring `set 0x` dentro de "pre**set 0x**03" disparava o guard do Test da T-8.

## Provas de mutação (não commitadas)
- T-1: `isolated` sem `catch_unwind` → `a_panic_inside_one_transaction_fails_only_that_call_and_the_worker_keeps_serving` FAILED.
- T-3 (#8): `requires_known_max` sem a restrição a `Continuous` → `set_feature_never_reads_a_max_for_a_non_continuous_code_…` FAILED.
- T-3 (#9): sem `feature.ensure_writable()?` em `set_feature` → `set_feature_rejects_a_read_only_or_table_code_…` FAILED.
- T-4 (#6): sondagem abortando no 1º erro → `probe_undeclared_features_reads_only_catalog_codes_…` FAILED.

## Blocked tasks
- (nenhuma)

## Achados de hardware para o orquestrador (só leitura, RTK em /dev/i2c-5, 2026-09-26)
1. **Falhas intermitentes em leituras seguidas** (anterior a esta phase): `Expected DDC/CI length bit` nas 3 tentativas (≈281 ms), em código aleatório, ~1 a cada 36 leituras; em `features --probe`, 0x70 falhou em 4 de 6 execuções (e 0x10/0x12 uma vez cada). Experimento local revertido (4 rodadas × 36 leituras na sequência exata do `features --probe`): backoff VCP 50 ms (D-2026-09-25-ddc-backends-3) → 3/4 e 3/4 rodadas com falha; 100 ms → 1/4; **200 ms → 0/4**. Espaçar transações (0/50/100/150 ms extras) NÃO resolve. Proposta: D-XX emendando D-3 para backoff VCP de 200 ms (cabe no orçamento de 1 s: 3 tentativas + 2×200 ms). Não alterei (decisão travada).
2. **Valor errado raro com checksum válido**: `get input` leu 16/17 em vez de 15 (5 de 12 processos numa amostra; in-process ~1/72 leituras); `ddcutil` leu `0x0F` em 8/8 e 3/3. Não há como detectar no adapter (código ecoado correto). Registrado nas Known limitations e no roteiro T-8.
3. **0xAE**: o RTK responde `44818 (0xAF12)` → `448.18 Hz` rodando a 143,96 Hz (KDE, 2560x1600); `ddcutil --verbose` lê os mesmos bytes ("448.18 hz"). O DoD de PR "get v-frequency → 144.00 Hz" não vale hoje; o roteiro manda comparar com o ddcutil. 0xAC = 3/41092 (bytes `sl:mh:ml` = 0x03A084 = 237 700 Hz ≈ 237,7 kHz, plausível — mantido bruto, D-8).
4. **Resets respondem a leitura** (0x04/05/06/08 → `0/1`, exit 0, ~1,15 s por processo) — `features` os lê como os demais; `set_feature` não os lê (D-8).
5. Itens em aberto herdados do PLAN para o PR: A-4 (0xAC bruto, aceito por D-8), semântica de 0xE6/0xF1 (nunca gravar), R-2 (Windows mostra `unresponsive`), R-3 (mensagem de panic no stderr). R-1 resolvido por D-8(b).

## Evidência de hardware (só leitura)
- `DDC_HW_TESTS=1 cargo test -p ddc-adapters --locked --test real_monitor -- --ignored --test-threads=1 --nocapture`: 7 passed (antes e depois da correção extra). `read_vcp 0x7E: Err(Transport("ddc-hi panicked: index out of bounds: the len is 11 but the index is 11")) in 1.14s` (inclui a enumeração de ~1,1 s), depois `read_vcp 0x10: Ok(100/100) in 137ms` no mesmo backend.
- `ddc-cli -m RTK features`: exit 0, 28 linhas + cabeçalho, 3,68 s.
- `ddc-cli -m RTK features --probe`: exit 0, 39 linhas, ~5,0 s; 0x1E 0x20 0x30 0x62 0x6C 0x6E 0x70 0xC9 0xE6 0xF1 `ok` e 0x7E `not responding` (execução limpa; ver achado 1 para as intermitentes).
- `get preset` → `0x14 preset: 1 (0x01) sRGB, max 11 (0x0B)`; `get input` → `0x60 input: 15 (0x0F) DisplayPort-1, max 3 (0x03)`; `get vcp-version` → `514 (0x202) 2.2`; `get firmware-level` → `1 (0x01) 0.1`; `get v-frequency` → `44818 (0xAF12) 448.18 Hz`.
- `ddc-cli -m LG features --probe`: exit 0, aviso de caps ilegível, 39 `not responding`, 8,95 s.
- Nenhum `set`/`reset` executado em hardware.

## Files modified
- crates/ddc-adapters/src/ddc_hi_backend.rs
- crates/ddc-adapters/src/ddc_hi_backend/worker.rs
- crates/ddc-adapters/src/ddc_hi_backend/worker/tests.rs
- crates/ddc-adapters/src/ddc_hi_backend/retry.rs
- crates/ddc-adapters/src/ddc_hi_backend/hardware.rs (extra, fora do plano)
- crates/ddc-adapters/src/ddc_hi_backend/hardware/tests.rs (extra, fora do plano)
- crates/ddc-adapters/tests/real_monitor.rs
- crates/ddc-core/src/domain/{mod,vcp,mccs_catalog,feature,capabilities}.rs
- crates/ddc-core/src/domain/{mccs_catalog,feature,capabilities}/tests.rs
- crates/ddc-core/src/app/software_osd.rs
- crates/ddc-core/src/ports/monitor_control.rs
- crates/ddc-core/tests/monitor_control/{main,support,read_features,write_features,probe_features}.rs
- crates/ddc-cli/src/{args,run,exit,output,fixture}.rs
- crates/ddc-cli/src/{args,exit,output}/tests.rs
- crates/ddc-cli/tests/cli/{main,reads,writes,reset,features}.rs
- README.md, CHANGELOG.md, docs/hardware-validation.md
- `Cargo.*` e `crates/ddc-cli/src/main.rs` intocados; `ddc-core` segue só com `thiserror`.

## Tests
- Total: 250 (243 passando + 7 de hardware `#[ignore]`, 7/7 verdes com `DDC_HW_TESTS=1`)
- Passing: 243/243 (`cargo test --workspace --locked`)
- Coverage: 95.92% linhas (TOTAL `cargo llvm-cov --workspace --locked --summary-only --fail-under-lines 80 --ignore-filename-regex '(^|[/\\])(main|build)\.rs$'`); `mccs_catalog.rs` 76.92% porque `row`/`with_values` e a tabela são avaliados em tempo de compilação.
- Gates: build 0, test 0, fmt --check 0, clippy -D warnings 0, check Linux (core+adapters+cli) 0, check Windows (`-p ddc-cli -p ddc-adapters`) 0, `cargo audit` só RUSTSEC-2018-0005 / RUSTSEC-2024-0320.
- DoD: 12/12 Verify da CONTEXT OK + `reset_writes_one_and_skips_read_back` OK + `reset_factory_with_yes_through_the_binary_sends_one_and_says_it_is_write_only` OK.

## Fix (D-9)
- `25c0f22` fix — D-2026-09-26-full-osd-control-9: `VCP_BACKOFF` 50 → 200 ms em `retry.rs` (caps 500 ms, 3 tentativas e orçamentos inalterados); doc do módulo `ddc_hi_backend.rs` atualizada.
- Testes de relógio virtual: `retries_transient_errors_up_to_three_times_within_timeout_budget` fixa 200 ms e corta para 2 tentativas com orçamento de 300 ms; `capabilities_retries_wait_500_ms_while_vcp_retries_wait_50_ms_within_budget` renomeado para `…_vcp_retries_wait_200_ms_within_budget`, com VCP 1 s → 3 tentativas, 300 ms → 2 ("gave up after attempt 2 of 3"), 200 ms → 1; `failed_vcp_answers_transport_at_once_when_a_presence_check_cannot_fit` passa a `took = 400 ms` e o caso limite a `400 ms + ENUMERATION` (mantém o fim da enumeração exatamente no prazo).
- `real_monitor.rs` (fora da lista do coordenador, mas fixava o timing): `FAST_FAILURE` 500 → 800 ms (LG falha em ~0,4 s; ainda < orçamento de 1 s e < enumeração de ~1,1 s).
- Docs: README (cache de caps, Retries, Known limitations: valor errado com checksum válido e 0xAE = 448.18 Hz como comportamento de firmware, "rode `get` de novo"), CHANGELOG (Added 200 ms/~0,4 s + entrada em Changed), `docs/linux-ddc-setup.md` (200 ms, ~0,4 s), `docs/hardware-validation.md` (4 probes seguidos sem falha além de 0x7E, `get input` com a nota do firmware e comparação com ddcutil, 0xAE comparado com `ddcutil getvcp ae` sem afirmar 144 Hz, tempos da LG).
- Hardware (só leitura, 2026-09-26, após D-9):
  - 4× `features --probe` seguidos no RTK: **0/4 rodadas com código falhando** (fora 0x7E `unresponsive` esperado); 10 sondados `ok` em todas; 0xFD/0xFF `unsupported`; ~5,13 s cada; nenhum valor divergiu entre as 4 rodadas.
  - `get input` 10×: `15 15 16 15 15 17 15 15 15 15` → 8× 15, 1× 16, 1× 17 (firmware; ddcutil lê 0x0F).
  - LG: `read_vcp` no teste de hardware = 403 ms (3 tentativas, 2×200 ms); `ddc-cli -m LG get brightness` = exit 6 em 3,61–3,63 s (enumeração ~1,1 s + caps da LG falhando 3× a 500 ms + checagem de presença ~2,1 s + VCP ~0,4 s); `-m LG features --probe` = exit 0 em 20,6 s.
  - Testes de hardware: 7/7 verdes.
- Gates: build 0, test 0 (243 + 7 ignorados), fmt --check 0, clippy -D warnings 0, Linux check 0, Windows check 0, llvm-cov TOTAL 95.92% linhas, `cargo audit` só RUSTSEC-2018-0005 / RUSTSEC-2024-0320, DoD 14/14 OK.
