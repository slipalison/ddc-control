# Phase 3: Review  (slug: cli)

**Verdict:** APPROVED_PENDING_MANUAL

_Iter 1. Commits revistos: `42e74d7..HEAD` (branch `phase/cli`, empilhada sobre `phase/ddc-backends`). O reviewer não rodou teste de hardware nem o binário real contra o monitor. Os números de hardware citados são evidência do doer (SUMMARY.md § Achados no hardware real). Os arquivos não commitados da phase `full-osd-control` ficaram fora da review: foram só consultados, para checar o carry-over de W-1._

## Gates
| Gate | Status | Details |
|---|---|---|
| Build | PASS | `cargo build --workspace --locked` exit 0. Cross-check Linux `-p ddc-core -p ddc-adapters -p ddc-cli --target x86_64-unknown-linux-gnu` exit 0. Check Windows `-p ddc-cli -p ddc-adapters --target x86_64-pc-windows-msvc` exit 0. `-p ddc-adapters --features ddc-hi --target x86_64-pc-windows-msvc` exit 0. Único aviso: future-incompat de `nom v3.2.1` (transitivo, pré-existente). |
| Tests | PASS | 180 passed, 0 failed, 5 ignored (hardware, `tests/real_monitor.rs`). A phase anterior tinha 91 passed e 5 ignored: são +89 testes, sem nenhum removido. |
| Coverage | PASS | 96.74% lines, threshold 80%. Linha TOTAL: `2046 84 95.89% 257 12 95.33% 1321 43 96.74%` (main.rs/build.rs excluídos). `crates/ddc-cli/src/main.rs` foi lido: é só composition root (parse, escolha do backend, cache dir, `run`), sem lógica fugindo do gate. |
| Lint | PASS | `cargo fmt --all --check` exit 0. `cargo clippy --workspace --all-targets --locked -- -D warnings` exit 0. Nenhum `#[allow(...)]` fora de teste. |
| Hexagonal/Safety/Hygiene | PASS | 5.1–5.11 limpos (detalhe abaixo). |
| Consistency | WARN | Plano e commits consistentes. D-1, D-2, D-2026-09-25-cli-1..5 e D-2026-09-25-core-domain-4 respeitadas. W-1..W-3 tratam do objetivo da phase medido no monitor real. |
| UI Validation | SKIPPED | has_frontend=false |
| DoD | PASS_PENDING_MANUAL | 12/12 auto, 2 manual pending |

### Gate 5 em detalhe
- **5.1 / 5.2 / 5.8:** `ddc-core` `[dependencies]` = só `thiserror`. Sem I/O, cfg de plataforma ou caminho de dispositivo no core. O core não mudou nesta phase (`git diff 42e74d7..HEAD -- crates/ddc-core` vazio).
- **5.3:** nenhum `impl MonitorBackend` no core e nenhum `pub trait` fora dele. `CachingMonitorBackend` é um decorator em `ddc-adapters`, não um port novo (D-2026-09-25-cli-1).
- **5.4:** os adapters só são construídos em `crates/ddc-cli/src/main.rs:23,28,32`. `caching.rs` não importa `in_memory` nem `ddc_hi`. A lib `ddc-cli` só usa `ddc_adapters::FakeMonitor`, como dado, em `fixture.rs`.
- **5.5:** nenhum `unsafe`. `#![forbid(unsafe_code)]` está em `ddc-cli/src/lib.rs` e em `main.rs`. `ddc-adapters` fica coberto por `[workspace.lints.rust] unsafe_code = "deny"`.
- **5.6:** nenhum `unwrap`/`expect`/`panic!` fora de teste.
- **5.7:**
  - `Confirm::Yes` só nasce em `crates/ddc-cli/src/run.rs:86`, a partir de `--yes`.
  - Nenhum `stdin`/`read_line` e nenhum `Risk::`/`risk_for_code`/`authorize_write` em `crates/ddc-cli/src` (D-2026-09-25-cli-3).
  - `DdcHiMonitorBackend` só aparece em teste em `tests/real_monitor.rs`, que é pré-existente, `#[ignore]` e com gate `DDC_HW_TESTS=1`.
- **5.9:** N/A (ainda não há Tauri).
- **5.10:** `cargo audit` só mostra os já conhecidos RUSTSEC-2018-0005 (serde_yaml 0.7.5) e RUSTSEC-2024-0320 (yaml-rust 0.4.5), inalcançáveis (ddc-backends W-4). Nenhum advisory nas 38 entradas novas do `Cargo.lock` (clap, serde_json, assert_cmd, predicates e transitivas).
- **5.11:** sem segredos, sem TODO/FIXME.

### Gate 6 em detalhe
- **Plano:** todos os arquivos de `files_modified` aparecem nos 10 commits da phase. Os 8 tasks têm teste; T-8 é validado pelo grep do plano: README sem `--fake`, `## [Unreleased]` presente.
- **Commits:** todos usam scope `cli` e um tipo que bate com o task. O `fix(cli)` extra (`bf7328a`) começou por um teste falhando. Nenhum commit mistura `.jdi/` com código.
- **Verify #1 da core-domain:** `! grep -qE 'crates/ddc-cli' Cargo.toml` deixa de valer por design do ROADMAP (PLAN A-3). Não é regressão.
- **D-XX:**
  - cli-1: decorator, `cache_dir` explícito, só `Ok` persistido, `invalidate` inerente, `enumerate` sem cache.
  - cli-2: seleção sem adivinhar. A-9, sem fallback de índice para substring, é mais restrita que a cascata e coerente com a decisão.
  - cli-3: `Confirm::Yes` só via `--yes`.
  - cli-4: flag oculta com a fixture especificada.
  - cli-5: parser único, atalhos sobre as consts do core, exit table fechada.
  - core-domain-4: a CLI obedece a letra. Ver W-1.

## Blockers
- (nenhum)

## Warnings

### W-1: `set` num código ausente do caps nunca funciona pela CLI (`set volume` no RTK sempre sai 4)
- **Onde:** `crates/ddc-core/src/app/software_osd.rs:88-90` (`if !capabilities.declares(feature.code) { return Ok(None); }`). Soma-se a isso que cada execução da CLI é um processo novo.
- **O quê:** `SoftwareOsd` só conhece um máximo vindo do caps ou de uma leitura anterior no mesmo processo (D-2026-09-25-core-domain-4). O RTK responde 0x62, mas não o declara. Por isso `ddc-cli set volume N` sai sempre com 4 ("feature 0x62 is not supported"), enquanto `get volume` funciona.
  - Resultado: um dos 6 atalhos do objetivo desta phase não escreve no único monitor real do projeto, e o card é "controlar tudo do monitor".
- **Por que WARN e não BLOCK:**
  - É a consequência literal de uma decisão travada (core-domain-4). O PLAN travou "Ports e SoftwareOsd não mudam", e a CONTEXT (§ Out of scope) manda a sondagem fora do caps para `full-osd-control`.
  - Nenhum gate nem D-XX foi violado. A escrita é recusada sem risco e a limitação está documentada (README § Known limitations).
  - Nos monitores que declaram 0x62, o atalho funciona (o fixture `--fake` prova).
  - Bloquear agora forçaria o doer a mudar o core sem decisão, ou a pôr a regra no lugar errado (ver abaixo).
- **Onde a correção pertence: no use case do core, não na CLI.**
  - `max_for_write` deve aprender o máximo com uma leitura (`read_vcp`, só leitura) também para código não declarado, como já faz para os declarados (`software_osd.rs:91-93`).
  - O ideal é limitar isso aos códigos que o catálogo MCCS da `full-osd-control` classifica como `Continuous`. Hoje `Capabilities::feature` trata como `Continuous` todo código fora do caps. Para um código NC fora do caps (0x14, 0xCC), o `max` lido não significa nada.
- **Por que não um retry na CLI** (`set_feature` → `UnsupportedFeature` → `get_feature` → `set_feature`):
  - A letra de core-domain-4 até permite ("leitura prévia no mesmo processo").
  - Só que o retry põe na driving adapter a política de qual máximo merece confiança para escrever. O tray e qualquer adapter futuro teriam de copiá-la. É a mesma duplicação que D-2026-09-25-cli-3 recusa para a tabela de risco.
  - O retry reagiria às cegas a um `UnsupportedFeature` que também significa feature `ReadOnly` ou caps ilegível.
  - E aplicaria o máximo lido também a código NC.
- **Precisa de novo D-XX?** Sim. A correção no core muda a letra de core-domain-4 ("ausente do caps, sem max conhecido → `UnsupportedFeature`"). As decisões são append-only, então é preciso registrar um D-XX que refine core-domain-4. Sugestão de texto: "sem max conhecido e código `Continuous` pelo catálogo → ler uma vez antes de escrever". A regra "nunca escrever às cegas" continua valendo.
- **Carry-over obrigatório:** o rascunho não commitado de `.jdi/phases/full-osd-control/CONTEXT.md` não cobre isso. Lá, `features --probe` roda num processo diferente do `set`, então o gap sobreviveria à phase 4 como está desenhada. Antes do `/jdi-ship` desta phase, o orquestrador deve registrar na `full-osd-control`:
  - o D-XX acima;
  - um item de DoD Auto, por exemplo um teste nomeado: `SoftwareOsd` novo, 0x62 respondido mas não declarado, `set_feature` Safe → `Ok` com read-back;
  - `set volume` reversível no RTK, no Deferred da phase.
- **Opcional agora:** um teste de caracterização na CLI com um fake no formato do RTK (código respondido e não declarado). Hoje o fixture em `fixture.rs:13-14` declara 0x62 e esconde esse caso, por isso a suíte fica verde enquanto o monitor de dev falha. O teste fixaria o exit 4 atual e depois viraria o teste da correção.

### W-2: o RTK recusa leituras de caps em sequência; `caps --refresh` apaga o cache bom antes de reler, e o cache frio vira exit 4
- **Evidência do doer, no hardware:** 4 `caps --refresh` seguidos deram 2 ok e 2 exit 6 (`Expected DDC/CI length bit` / `invalid offset`). Com 6 s de intervalo, 4/4 ok.
- **Consequência 1:** `CachingMonitorBackend::invalidate` (`crates/ddc-adapters/src/caching.rs:54-57`) apaga o arquivo antes da releitura. Um `--refresh` que falha (50% no RTK logo depois de outra leitura) transforma um cache quente e válido num cache frio.
- **Consequência 2:** com cache frio e caps falhando, `SoftwareOsd::capabilities_or_empty` (`software_osd.rs:56-63`) trata o caps como vazio, e `set brightness` sai 4 ("feature 0x10 is not supported").
  - D-2026-09-25-cli-5 reserva o 4 para feature ou valor inválido. A rationale da decisão é justamente o script conseguir distinguir "valor inválido" de "hardware não respondeu".
  - Aqui uma falha transitória de transporte aparece como erro permanente. Não viola a letra: o core devolve `UnsupportedFeature` e `exit.rs:57-59` mapeia corretamente. Mas contraria a intenção.
- **Classificação:** WARN.
  - É quirk do monitor, num caminho raro (primeiro uso ou `--refresh`).
  - O `caps` sai com exit 6, que é honesto.
  - Está documentado no README § Capabilities cache.
- **(a) Onde mitigar: no adapter `ddc-hi`, não no core nem na CLI.**
  - Hoje um único `RetryPolicy` (3 tentativas, 50 ms, `retry.rs:10-12`, aplicado em `worker.rs:96,185`) vale para toda operação.
  - Uma política própria para `read_capabilities`, com backoff na escala de segundos dentro do orçamento de 8 s (`ddc_hi_backend.rs:27`), absorveria o quirk.
  - Antes, medir quanto dura uma tentativa que falha. Se falha rápido, cabem 1–2 retries com 2–3 s de pausa. Se leva os ~2.6 s inteiros, o orçamento de 8 s também precisa ser revisto.
  - Precisa de um D-XX que refine D-2026-09-25-ddc-backends-3/-7, cujo backoff hoje é uniforme de 50 ms.
- **(b)** `invalidate` pode manter o arquivo até uma releitura bem-sucedida sobrescrevê-lo. A marca em memória (`caching.rs:55`) já garante a releitura neste processo, e a escrita atômica já faz o overwrite. A letra de D-2026-09-25-cli-1 diz "apaga o arquivo", então registre isso no mesmo D-XX.
- **(c)** A consequência 2 some com a correção de W-1 para códigos `Continuous`: sem caps, o core leria o máximo de 0x10 direto.

### W-3: um caps `Ok` que o core não consegue parsear fica persistido para sempre
- **Onde:** `crates/ddc-adapters/src/caching.rs:88-89` persiste qualquer `Ok` do backend. Já `Capabilities::parse` recusa parênteses desbalanceados (`crates/ddc-core/src/domain/capabilities.rs:135`), como os de um caps truncado.
- **Efeito:**
  - Uma leitura truncada que o `ddc-hi` aceite fica gravada em disco.
  - A partir daí, todo `caps` sai 6 e todo `set` de código contínuo sai 4, até um `caps --refresh` manual.
  - Também anula a recuperação do core: `SoftwareOsd::capabilities` (`software_osd.rs:106-111`) relê depois de uma falha lembrada, mas a "releitura" cai no mesmo arquivo ruim.
  - Não foi observado (o arquivo do cache bate byte a byte com a fixture), mas é plausível no RTK, que já mostrou leituras de caps instáveis.
- **Sugestão:** persistir só se `Capabilities::parse(&raw).is_ok()`. O adapter já depende de `ddc_core::domain`, e isso segue a filosofia de D-2026-09-25-cli-1 ("falha não é memória"). Cobrir com um teste em que o fake devolve um caps truncado.

### Notas (sem ação obrigatória)
- `Exit::Usage` (`exit.rs:17`) nunca é construído pela CLI, porque o clap sai com 2 sozinho. Existe para completar a tabela e é testado contra `clap::Error::exit_code()`. Aceitável como documentação do contrato.
- A lista dos 6 atalhos aparece literal duas vezes nos doc comments do clap (`args.rs:52-53`, `59-60`), além de em `SHORTCUTS`. Se um atalho for adicionado, o `--help` pode divergir; a mensagem de erro já deriva de `SHORTCUTS`.
- `SIMULATED_TIMEOUT = VcpCode(0xDF)` (`fixture.rs:18`) usa o código MCCS real de "VCP version", que o RTK responde. No fake não faz mal, mas um código da faixa de fabricante deixaria a intenção mais óbvia.
- O Status do README (linha 5) diz que o binário "reads and writes their features on real hardware". Nenhum `set` rodou em hardware nesta phase (SUMMARY); o round-trip de `set brightness` continua em Deferred to PR review.

## DoD Checklist (gate 8)

| # | Criterion | Source | Type | Status | Evidence |
|---|---|---|---|---|---|
| 1 | `cargo test --workspace` exits 0 | PROJECT | Auto | PASS | exit 0: 180 passed, 0 failed, 5 ignored (hardware) |
| 2 | Coverage >= 80% of lines | PROJECT | Auto | PASS | TOTAL Lines 96.74% (1321 linhas, 43 perdidas); `--fail-under-lines 80` exit 0 (saída do Gate 3 reaproveitada) |
| 3 | No `TODO`/`FIXME` without linked issue reference | PROJECT | Auto | PASS | grep sem hits, exit 0 |
| 4 | CHANGELOG.md updated with entry per release | PROJECT | Manual | MANUAL_REQUIRED | suggested: `## [Unreleased]` ganhou 3 entradas (`ddc-cli`, `CachingMonitorBackend`/`default_cache_dir()`, `FakeMonitor::with_vcp_failure`); ainda sem heading `## [x.y.z]`, nenhuma release cortada |
| 5 | README accurately describes current behavior | PROJECT | Manual | MANUAL_REQUIRED | suggested: README +104/-3 (Install, Usage, `--monitor`, `--json`, atalhos, Exit codes, Capabilities cache com a recusa de caps em sequência, Known limitations com `set volume` → 4), sem `--fake`. Conferir a frase do Status "reads and writes ... on real hardware": ainda não rodou nenhum `set` em hardware |
| 6 | `crates/ddc-cli` no workspace com clap derive; serde só em `ddc-cli` (D-2) | CONTEXT | Auto | PASS | o Verify imprimiu `OK` |
| 7 | Teste `caching_backend_reads_capabilities_from_disk_without_calling_wrapped_backend_after_first_fetch` | CONTEXT | Auto | PASS | `test result: ok. 1 passed; 0 failed` (instância nova, fake novo, log vazio) |
| 8 | Teste `caching_backend_never_persists_failures_and_refresh_forces_a_fresh_overwrite` | CONTEXT | Auto | PASS | `test result: ok. 1 passed; 0 failed` |
| 9 | Teste `resolving_monitor_without_the_flag_auto_selects_the_only_one_or_refuses_when_zero_or_many` | CONTEXT | Auto | PASS | `test result: ok. 1 passed; 0 failed` |
| 10 | Teste `monitor_flag_matches_by_exact_id_index_or_unique_substring_and_refuses_ambiguous_matches` | CONTEXT | Auto | PASS | `test result: ok. 1 passed; 0 failed` |
| 11 | Teste `fake_flag_selects_a_deterministic_in_memory_backend_and_stays_hidden_from_help` | CONTEXT | Auto | PASS | `test result: ok. 1 passed; 0 failed` |
| 12 | Teste `dangerous_write_without_yes_is_refused_before_touching_the_backend_and_applied_with_yes` | CONTEXT | Auto | PASS | `test result: ok. 1 passed; 0 failed`; o log da escrita recusada é só `[Enumerate]` (`tests/cli/writes.rs:20`) |
| 13 | Teste `vcp_argument_accepts_decimal_hex_and_named_shortcuts_and_rejects_unknown_names` | CONTEXT | Auto | PASS | `test result: ok. 1 passed; 0 failed` |
| 14 | Teste `exit_codes_are_stable_for_not_found_invalid_value_unconfirmed_and_transport_failures` | CONTEXT | Auto | PASS | `test result: ok. 1 passed; 0 failed` (3/3/4/4/4/5/6 pelo binário `--fake`) |

**Totals:** 14 items | Auto: 12 (12 PASS, 0 FAIL) | Manual: 2 pending

**Manual confirmation required:**
Run `/jdi-confirm-dod cli` to confirm each manual item with evidence. Without that, `/jdi-ship` will refuse the phase.

## Recommendation
- **A phase passa no mérito:** gates 1–8 limpos, core intocado, hexágono respeitado, contrato de exit codes e de confirmação provado pelo binário. Faltam os 2 itens Manual do PROJECT (`/jdi-confirm-dod cli`). Na confirmação do README, ajustar ou aceitar conscientemente a frase de Status sobre escrita em hardware.
- **W-1 não pode se perder:** é o único atalho do objetivo que não escreve no monitor de dev, e o rascunho da `full-osd-control` não o cobre. Antes do ship, registre o D-XX que refina D-2026-09-25-core-domain-4 e o item de DoD correspondente na `full-osd-control`.
  - A correção vai no core (`max_for_write`), condicionada ao `kind` do catálogo, e não num retry na CLI.
  - Se esse carry-over não for registrado, trate W-1 como bloqueante no ship: o atalho ficaria sem dono.
- **W-2 e W-3 são robustez do cache e do adapter diante do RTK.** Nenhum dos dois exige mudar a CLI. Cabem num fix round curto antes do ship ou como itens da próxima phase:
  - W-3: persistir só caps parseável. São ~3 linhas + teste, dentro da letra de D-2026-09-25-cli-1.
  - W-2(b): `invalidate` sem apagar o arquivo. Precisa de D-XX.
  - W-2(a): retry de caps com backoff em segundos. Precisa de D-XX e de medição no hardware.
- **Round-trip de `set brightness` no hardware** (Deferred to PR review, do orquestrador): popular o cache antes com `-m RTK caps` e esperar ≥6 s depois de qualquer leitura de caps, senão cai em W-2 e sai 4. Nunca `input`/`power`/código desconhecido.

## DoD Critic (enhanced)

Forçado pelo `/jdi-issue`.
- DoD row «10 — `--monitor` casa por id exato → índice → substring única» (hollow, OBJETIVO): todo caso de id exato do teste nomeado também é substring única do mesmo id (`crates/ddc-cli/tests/cli/selection.rs:42`, `:54`). Mutação que apaga o ramo de id exato em `crates/ddc-cli/src/select.rs:69-71` mantém o teste nomeado verde e o Verify imprimindo OK. A precedência do id exato só é provada por unit tests que o Verify não roda (`select/tests.rs:57`, `:87`). Correção: acrescentar ao teste nomeado um caso em que o id exato também é substring de outro id (ex.: ids `RTK-QHD-1` e `RTK-QHD-10`, `-m RTK-QHD-1` → `RTK-QHD-1`).

**Verdict:** BLOCKED
