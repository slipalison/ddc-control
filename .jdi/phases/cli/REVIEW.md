# Phase 3: Review  (slug: cli)

**Verdict:** APPROVED_WITH_WARNINGS

_Iter 3: re-verify depois do fix round de warnings (`/jdi-issue` Step 6)._
- **Escopo:** commits `42e74d7..HEAD` na branch `phase/cli`, com foco no delta `97561c1..HEAD` (`3b48fce`, `6293e17`, `705e1e3`, `ed1a2af`, `62da5d0`, `f6a8585`).
- **Hardware:** o reviewer não rodou teste de hardware nem o binário real. A evidência de hardware citada vem do doer (SUMMARY.md § Fix round).
- **Fora do escopo:** os arquivos não commitados da `full-osd-control` ficaram de fora; só foram lidos para checar o carry-over.

## Gates
| Gate | Status | Details |
|---|---|---|
| Build | PASS | Todos com exit 0: `cargo build --workspace --locked`; check Linux (`-p ddc-core -p ddc-adapters -p ddc-cli --target x86_64-unknown-linux-gnu`); check Windows (`-p ddc-cli -p ddc-adapters --target x86_64-pc-windows-msvc`); `-p ddc-adapters --features ddc-hi` para Windows, também com `--all-targets` (os dev-deps `ddc`/`ddc-i2c` compilam para msvc). Único aviso: future-incompat de `nom v3.2.1`, transitivo e pré-existente. |
| Tests | PASS | 194 passed, 0 failed, 6 ignored (hardware, `tests/real_monitor.rs`). Na iter 2 eram 189 passed e 5 ignored. Entraram 5 testes (2 de classificação, 1 da cadeia até o port, 1 do worker, 1 do binário) e 1 `#[ignore]` de hardware. Nenhum foi removido. |
| Coverage | PASS | 96.75% das linhas, threshold 80%. Linha TOTAL: `2111 84 96.02% 266 12 95.49% 1386 45 96.75% 0 0 -` (main.rs/build.rs excluídos). Iter 2: 96.80%. Toda linha nova de `6293e17` está coberta. Nos arquivos tocados, só ficam sem cobertura o shell de hardware (`hardware.rs:23-45`) e a falha de spawn da thread (`worker.rs:317-318`), ambos de commits da phase `ddc-backends` (`git blame`: `28ed51a`/`8342338`). `main.rs` não mudou. |
| Lint | PASS | `cargo fmt --all --check`: exit 0. `cargo clippy --workspace --all-targets --locked -- -D warnings`: exit 0. Também exit 0 com `-p ddc-adapters --features ddc-hi --all-targets`, no Linux e para `x86_64-pc-windows-msvc`. Nenhum `#[allow(...)]` novo fora de teste. |
| Hexagonal/Safety/Hygiene | PASS | 5.1–5.11 limpos (detalhe abaixo). |
| Consistency | WARN | D-2026-09-26-cli-4 conforme: o fallback por texto está justificado e documentado. D-1, D-2, D-2026-09-26-ddc-backends-1, D-2026-09-25-cli-5 e D-2026-09-26-cli-1 respeitadas. O WARN vem de 2 commits `chore(repo)` fora do escopo da phase (W-8). |
| UI Validation | SKIPPED | has_frontend=false |
| DoD | PASS_PENDING_MANUAL | 12/12 auto, 2 manual pending |

### Gate 5 em detalhe
- **5.1 / 5.2 / 5.8:** `ddc-core` não foi tocado nesta rodada. `[dependencies]` continua só com `thiserror`, e não há I/O, cfg de plataforma nem caminho de dispositivo.
- **5.3:** nenhum `impl MonitorBackend` no core e nenhum `pub trait` fora dele.
  - `HandleError`, `HandleErrorKind` e `Failure::Unsupported` são `pub(crate)` ou privados: uma seam interna do adapter.
  - Nenhum tipo do `ddc`/`ddc-hi` cruza o port, que continua falando só `DdcError`.
- **5.4:** nenhum adapter é construído fora da composition root, e nenhum adapter importa outro.
- **5.5:** nenhum `unsafe`. `#![forbid(unsafe_code)]` está nas raízes. Em `ddc-adapters` vale `[workspace.lints.rust] unsafe_code = "deny"` (`Cargo.toml:15-16`).
- **5.6:** nenhum `unwrap`/`expect`/`panic!` fora de teste. O `strip_prefix(..).unwrap_or(..)` de `hardware.rs:74` é `Option::unwrap_or`, que não entra em panic.
- **5.7:**
  - `Confirm::Yes` não aparece fora de `ddc-cli/src` nem de testes.
  - `DdcHiMonitorBackend` só aparece em `tests/real_monitor.rs`: 6 testes, todos `#[ignore]` + `hardware_enabled()` (`DDC_HW_TESTS=1`) e todos só de leitura.
  - Isso inclui o novo `hardware_code_the_dev_monitor_refuses_is_unsupported` (`:204`), que só lê 0x8D, 0xDC e 0x10.
- **5.9:** N/A, ainda não há Tauri.
- **5.10:**
  - O `Cargo.lock` ganhou só 2 arestas (`ddc` e `ddc-i2c` sob `ddc-adapters`). A lista de pacotes `name`/`version` é idêntica à de `3b48fce`: nenhum pacote novo, removido ou com bump.
  - `cargo audit` 0.22.2 só mostra os já conhecidos RUSTSEC-2018-0005 (serde_yaml 0.7.5) e RUSTSEC-2024-0320 (yaml-rust 0.4.5).
- **5.11:** sem segredos, sem TODO/FIXME.

### Gate 6 em detalhe

**Plano e commits.**
- `6293e17 fix(cli)` é atômico: código, testes, `Cargo.lock` e `Cargo.toml` juntos. Cita D-2026-09-26-cli-4 e D-2026-09-26-ddc-backends-1.
- `705e1e3 docs(cli)` e `f6a8585 docs(cli)` só mexem em docs e `.jdi`. `3b48fce docs(cli)` só traz a decisão. Nenhum commit mistura `.jdi/` com código.
- Os arquivos tocados estão fora do `files_modified` do PLAN: `ddc_hi_backend/{hardware,retry,worker}.rs`, `real_monitor.rs` e `ddc-adapters/Cargo.toml`. D-2026-09-26-cli-4 os autoriza ("a detecção fica toda no adapter"), e o SUMMARY § Fix round os registra.
- `ed1a2af` e `62da5d0` usam scope `repo`: ver W-8.

**(a) D-2026-09-26-cli-4: conforme.**
- **Classificação no adapter:** `transaction_error` (`hardware.rs:60`) marca `Unsupported` quando algum elo da cadeia é exatamente "Unsupported VCP code", puro ou com o prefixo "DDC/CI error: " (`:50-52`, `:72-75`). O resto continua `Transient`.
- **Sem retry:** `RetryPolicy::run` para na 1ª recusa (`retry.rs:118`) e devolve `Failure::Unsupported` (`:141`).
- **Sem checagem de presença:** `explain` responde a recusa antes do bloco de re-enumerate (`worker.rs:259`).
- **Mapeamento por transação:**
  - `read_vcp`/`write_vcp` → `UnsupportedFeature(code)` (`worker.rs:205,219`);
  - caps → `Transport` (`:192`). Esse caso é inalcançável com `ddc-hi`: a resposta de caps não tem result code, e o texto só é gerado em `ddc-0.2.2/src/commands.rs:101-105`, na decodificação do Get VCP Feature.
- **Testes:**
  - `transaction_error_classifies_only_the_unsupported_vcp_code_reply_as_final` (`hardware/tests.rs:184`);
  - `transaction_error_keeps_every_other_ddc_hi_failure_transient` (`:204`): `InvalidOffset`, `InvalidChecksum`, "Unrecognized VCP error code 0x02" e um erro I2C cujo texto é "Unsupported VCP code";
  - `unsupported_vcp_code_reply_reaches_the_port_as_unsupported_feature` (`:226`);
  - `unsupported_reply_is_final_without_retry_or_presence_check` (`worker/tests.rs:671`): custo de 1 tentativa, 1 enumerate (o inicial) e 0 s, com orçamento de 8 s, no qual a checagem de presença caberia.
- **Prova de mutação reproduzida pelo reviewer.** Numa cópia `git archive HEAD` no scratchpad, com target próprio, removida depois:
  - **M1:** desligar a classificação em `hardware.rs:61` derruba `…classifies_only…` (`hardware/tests.rs:193`) e `…reaches_the_port_as_unsupported_feature` (`:237`);
  - **M2:** desligar a parada em `retry.rs:118` derruba `…reaches_the_port…` (`:237`) e `unsupported_reply_is_final…` (`worker/tests.rs:680`).
- **Detecção por texto em vez de "de preferência por tipo": aceita como o fallback documentado que a decisão prevê.**
  - O reviewer conferiu a premissa do doer na fonte. `ddc-i2c-0.2.2/src/lib.rs:236-250` implementa em `Error<I>` só `description()`/`cause()`, ambos deprecados, sem `source()`. Por isso `anyhow::Error::chain()` para no 1º elo, e `downcast_ref::<ddc::ErrorCode>()` nunca acha o `ErrorCode`. O teste em `hardware/tests.rs:186-190` fixa esse comportamento.
  - Mesmo o caminho tipado terminaria em texto. O `ddc` 0.2.2 não tem variante própria para o result code 0x01, só `ErrorCode::Invalid("Unsupported VCP code".into())` (`commands.rs:103`). O tipo só estreitaria o casamento ao payload de `Invalid`.
  - O casamento é exato (igualdade depois de tirar um prefixo fixo), não um `contains`. As strings são consts nomeadas. O porquê está em `hardware.rs:70-71` e no doc de `transaction_error`.
- **Desvio nos dev-dependencies: aceitável, não pede D-XX.**
  - A decisão autorizava `ddc` como dependência direta, de produção, para o caminho tipado. Como esse caminho não funciona, a produção não precisa de `ddc`.
  - `ddc` e `ddc-i2c` entraram só em `[dev-dependencies]` (`ddc-adapters/Cargo.toml:16-21`), fixados em `=0.2.2`, as versões já travadas.
  - Nenhum pacote novo entrou no lock, nenhuma dependência nova entrou no artefato de produção, e D-2 (que só trata do core) continua intocada.
  - O `ddc-i2c` de dev é o que permite ao teste montar a cadeia real do Linux: `ddc_i2c::Error<io::Error>`, o tipo que o `ddc-hi` 0.4.1 converte com `map_err(From::from)`. Ver a nota sobre o pin exato.

**(b) Demais D-XX relevantes.**
- **D-2026-09-26-ddc-backends-1:** intacta.
  - Uma falha transitória com as tentativas esgotadas continua passando pela checagem de presença quando ela cabe: `failed_capabilities_read_checks_presence_when_it_fits` e `failed_vcp_answers_transport_at_once_when_a_presence_check_cannot_fit` continuam verdes.
  - Um job expirado continua sem tocar o hardware: `Failure::Expired` é decidido antes da 1ª tentativa.
- **D-2025-ddc-backends-3 / D-6** (A-8, "toda falha é transitória"): D-2026-09-26-cli-4 as refina explicitamente. O retry continua dentro do worker.
- **D-2026-09-25-cli-5:** a linha 4 da tabela ("feature não suportada") passa a ser alcançável no hardware Linux, e o README (`README.md:82,86`) diz isso.
- **D-2026-09-26-cli-1:** "só `UnsupportedFeature` quando o monitor responde que não suporta" agora vale também no adapter real do Linux. Um `set` num código recusado falha na leitura do max e nunca escreve (`exit_codes.rs:39`).
- **D-1 / D-2:** a seam é interna ao adapter, e o core não foi tocado.

## Blockers
- (nenhum)

## Status dos achados da iter 2
| Achado | Status | Evidência |
|---|---|---|
| W-4: o adapter real nunca produzia `UnsupportedFeature` (exit 6 em vez de 4) | **RESOLVIDO no Linux** | D-2026-09-26-cli-4 implementada em `6293e17` (ver Gate 6 a), com as mutações M1/M2 reproduzidas. No hardware (evidência do doer, não reproduzida), `get 0x8D/0x9B/0xDC/0xC0 -m RTK` passou de exit 6 em ~1.35 s para exit 4 em ~1.2 s. O resíduo no Windows virou W-6. |
| W-5: o max-por-leitura também vale para NC sem lista | **Aberto, carry-over** | Não foi tocado nesta rodada, por decisão do orquestrador. Ver W-5 abaixo. |
| Nota: "about 1 s longer" no README | **RESOLVIDO** | `README.md:99` diz agora "about 1.6 s longer" (6.28 s contra 4.67 s, medidos pelo doer). |
| Notas `Exit::Usage`, lista de atalhos em `args.rs`, `SIMULATED_TIMEOUT = 0xDF` | Inalteradas | Sem ação obrigatória. |

## Warnings

### W-5 (carry-over da iter 2): o max obtido por leitura também vale para feature NC sem lista conhecida
- **O quê:** nada mudou. `Capabilities::feature` (`crates/ddc-core/src/domain/capabilities.rs:55-68`) trata como `Continuous` todo código sem lista, e `max_for_write` (`crates/ddc-core/src/app/software_osd.rs:77-87`) lê o max de qualquer um deles. Casos afetados:
  - (i) código NC fora do caps, escrito com `--yes`, ou código Safe fora do caps;
  - (ii) NC Safe declarado (0x14, 0xCC) com caps ilegível: um valor fora da lista é aceito sem `--yes`, desde que ≤ max lido.
- **Por que WARN e não BLOCK:**
  - segue a letra de D-2026-09-26-cli-1, cujo "Refinamento previsto" manda a `full-osd-control` restringir o max-por-leitura a `Continuous`;
  - a escrita continua limitada pelo max que o próprio monitor informa;
  - o README documenta os dois casos.
- **Carry-over:** o rascunho não commitado `.jdi/phases/full-osd-control/CONTEXT.md:11` já cita D-2026-09-26-cli-1..4 e o pedido de restringir a `Continuous`. No `/jdi-discuss` e no `/jdi-plan` dessa phase, manter:
  - o teste nomeado de `set` Safe em código respondido mas não declarado;
  - um teste de caps ilegível + NC Safe → recusado;
  - a checagem de `Access::ReadOnly` antes de `max_for_write`.

### W-6: no Windows, um código recusado continua saindo com 6, depois de 3 tentativas
- **Onde:** quando `GetVCPFeatureAndVCPFeatureReply` falha, `ddc-winapi-0.2.2/src/lib.rs:123-124` devolve só `io::Error::last_os_error()`. `transaction_error` (`hardware.rs:60`) não reconhece nada ali, então tudo continua `Transient`.
  - O comportamento está documentado no doc de `transaction_error`, no doc de `DdcHiMonitorBackend`, no README (`:86`, `:113`) e no CHANGELOG.
- **Por que WARN e não BLOCK:**
  - D-2026-09-26-cli-4 descreve só a cadeia do `ddc` 0.2.2 (Linux/i2c), então a letra da decisão está cumprida;
  - no Windows o comportamento é o mesmo de antes;
  - nada é escrito.
- **Ressalva ao texto da doc:** "dxva2 keeps that reply to itself" pode ser forte demais.
  - O Win32 define `ERROR_GRAPHICS_DDCCI_VCP_NOT_SUPPORTED` (`0xC0262584`). Se o dxva2 usar esse código para o result code 0x01, ele chega em `io::Error::raw_os_error()`, e isso daria um caminho tipado, sem texto.
  - Não foi verificado: não há hardware Windows, e a documentação de `GetVCPFeatureAndVCPFeatureReply` só manda chamar `GetLastError`.
- **Carry-over para `full-osd-control`:** no Windows, o `features --probe` não vai distinguir "não suportado" de "não respondeu" e vai pagar 3 tentativas por código recusado.
  - Registrar isso como risco ou limitação na CONTEXT.
  - Opcional: uma D-XX para testar `raw_os_error() == 0xC0262584` num Windows com o RTK antes de mapear.

### W-7: `get 0x7E` faz o `ddc-i2c` 0.2.2 entrar em panic e derruba o worker (defeito de terceiro)
- **Onde:** `ddc-i2c-0.2.2/src/lib.rs:194` faz `if out[2 + len] != checksum` logo depois de `if full_len < len + 2` (`:185`).
  - Quando `full_len == len + 2`, o índice `len + 2` sai do buffer. É um off-by-one: a checagem deveria ser `< len + 3`.
  - O panic acontece dentro de `get_vcp_feature`, antes de qualquer `Result` voltar, então `transaction_error` nunca roda.
  - A thread `ddc-hi-worker` morre, e toda chamada seguinte do mesmo processo responde `Transport("the DDC worker thread is not running")` (`worker.rs:410`).
  - Pela evidência de hardware do doer, o comando sai com 6 antes e depois do fix. O reviewer não reproduziu.
- **O fix de W-4 (`6293e17`) não piora nada:** o caminho do panic não passa pela classificação nem pelo retry.
- **Uma mudança desta phase amplia o alcance do panic, sem risco de escrita.** Desde a iter 2 (`a44b658`), D-2026-09-26-cli-1 faz o `set` num código sem max conhecido ler esse código antes.
  - 0x7E é `Dangerous` (`feature.rs:122`). Sem `--yes`, o `set` sai com 5 sem tocar o barramento, como antes.
  - Com `--yes`, antes da iter 2 o `set` saía com 4 sem I/O. Agora ele lê 0x7E, entra no panic e sai com 6.
  - O write nunca é enviado, porque o panic acontece na leitura do max. Numa CLI de um processo só, o estrago fica nesse comando.
- **Por que WARN e não BLOCK:**
  - é defeito de terceiro, e já existia para `get`;
  - nenhuma escrita acontece às cegas;
  - o exit 6 e a mensagem são honestos (o hook padrão imprime o panic no stderr).
- **Carry-over:**
  - O `D-2026-09-26-full-osd-control-6.md`, ainda não commitado, já prevê um `catch_unwind` por transação no worker. Nenhum `[profile]` do workspace usa `panic = "abort"`, então o `catch_unwind` funciona.
  - A premissa em `.jdi/phases/full-osd-control/CONTEXT.md:14` ("0x7E… falha por retries esgotados") está errada. Sem o `catch_unwind`, a sondagem morre em 0x7E. Com a D-6 como está (panic → `Transport`, sem retry), 0x7E vira "unresponsive", o que bate com a linha 80 do rascunho. Alinhar a linha 14 com a D-6.
  - Também vale reportar o off-by-one upstream ou avaliar um `[patch]`.

### W-8: dois commits fora do escopo da phase na branch `phase/cli`
- **Os commits:**
  - `ed1a2af chore(repo)`: adiciona `.claude/settings.local.json` ao `.gitignore`. Inofensivo.
  - `62da5d0 chore(repo)`: remove de `.claude/settings.json` (versionado e compartilhado) todas as regras `ask`: `git push`, `git reset --hard`, `git rebase`, `git clean`, `rm`, `chmod`, `sudo`, `curl`, `wget`, `docker run`, `kubectl delete/apply`, entre outras. O commit não tem trailer de co-autoria; parece mudança manual do mantenedor.
- **Por que WARN e não BLOCK:** não é código, não há D-XX sobre o assunto, e o reviewer não julga a intenção. Mesmo assim, o `62da5d0` foge do escopo `cli` e tira as confirmações de comandos destrutivos para quem usa Claude Code neste repo. Isso muda a postura de segurança, prioridade 1 do projeto, sem nenhum registro.
- **Recomendação:** explicitar a mudança no PR, ou movê-la para um PR próprio, para que seja uma decisão consciente.

### Notas (sem ação obrigatória)
- **Pin exato dos dev-deps.** `ddc = "=0.2.2"` e `ddc-i2c = "=0.2.2"` só continuam fiéis enquanto o `ddc-hi` puxar essas mesmas versões.
  - Num bump do `ddc-hi`, o lock teria duas versões de `ddc-i2c`. O teste montaria a cadeia com um tipo diferente do de produção, e a asserção "o downcast não acha o `ErrorCode`" passaria trivialmente.
  - O comentário em `Cargo.toml:17-18` avisa. Ao subir o `ddc-hi`, subir os dois juntos.
- **Teste de CLI calibrado.** `a_code_the_monitor_refuses_exits_4_for_get_and_set_without_writing` (`exit_codes.rs:39`) fixa o contrato da CLI, mas não prova a regressão de W-4.
  - O fake sempre respondeu `UnsupportedFeature` para código desconhecido, então o teste passaria sem `6293e17`.
  - A prova de regressão são os 3 testes do adapter (mutações M1/M2). Nada a corrigir; é só para calibrar o SUMMARY, que chama esse teste de "ponta a ponta".
- **`write_vcp` → `UnsupportedFeature`** é, na prática, inalcançável no Linux: o Set VCP Feature não tem resposta com result code. É simétrico e inofensivo.
- **Deferred to PR review (inalterado):**
  - round-trip reversível de `set brightness` e de `set volume` no RTK;
  - opcional: `get 0x8D` → exit 4 no RTK;
  - nunca `set` em `input`, `power`, 0x7E ou código desconhecido.

## DoD Checklist (gate 8)

| # | Criterion | Source | Type | Status | Evidence |
|---|---|---|---|---|---|
| 1 | `cargo test --workspace` exits 0 | PROJECT | Auto | PASS | exit 0: 194 passed, 0 failed, 6 ignored (hardware). Saída do Gate 2 reaproveitada |
| 2 | Coverage >= 80% of lines | PROJECT | Auto | PASS | TOTAL Lines 96.75% (1386 linhas, 45 perdidas). `--fail-under-lines 80` com exit 0 (saída do Gate 3 reaproveitada) |
| 3 | No `TODO`/`FIXME` without linked issue reference | PROJECT | Auto | PASS | Verify exit 0, nenhum hit |
| 4 | CHANGELOG.md updated with entry per release | PROJECT | Manual | CONFIRMED | suggested: em `705e1e3`, `## [Unreleased]` (`CHANGELOG.md:8`) ganhou a recusa 0x01 → `UnsupportedFeature` sem retry (no Linux; no Windows não) e a frase "A code the monitor refuses as unsupported exits 4, not 6". Ainda não há heading `## [x.y.z]`: nenhuma release foi cortada |
| 5 | README accurately describes current behavior | PROJECT | Manual | CONFIRMED | suggested: linha 4 da tabela de exit codes (`README.md:82`); parágrafo novo "exit 4 vs 6", com os códigos que o RTK recusa e a ressalva do Windows (`:86`); bullet Retries (`:113`); "about 1.6 s longer" (`:99`). Conferir a frase "dxva2 keeps that reply to itself" contra W-6, e o panic em 0x7E (W-7), que não está em Known limitations |
| 6 | `crates/ddc-cli` no workspace com clap derive; serde só em `ddc-cli` (D-2) | CONTEXT | Auto | PASS | Verify imprimiu `OK`. Os novos dev-deps `ddc`/`ddc-i2c` não trazem serde para `ddc-adapters/Cargo.toml` |
| 7 | Teste `caching_backend_reads_capabilities_from_disk_without_calling_wrapped_backend_after_first_fetch` | CONTEXT | Auto | PASS | `test result: ok. 1 passed; 0 failed` |
| 8 | Teste `caching_backend_never_persists_failures_and_refresh_forces_a_fresh_overwrite` | CONTEXT | Auto | PASS | `test result: ok. 1 passed; 0 failed` |
| 9 | Teste `resolving_monitor_without_the_flag_auto_selects_the_only_one_or_refuses_when_zero_or_many` | CONTEXT | Auto | PASS | `test result: ok. 1 passed; 0 failed` |
| 10 | Teste `monitor_flag_matches_by_exact_id_index_or_unique_substring_and_refuses_ambiguous_matches` | CONTEXT | Auto | PASS | `test result: ok. 1 passed; 0 failed`. `select.rs` e `selection.rs` não mudaram desde a prova de mutação da iter 2 |
| 11 | Teste `fake_flag_selects_a_deterministic_in_memory_backend_and_stays_hidden_from_help` | CONTEXT | Auto | PASS | `test result: ok. 1 passed; 0 failed` |
| 12 | Teste `dangerous_write_without_yes_is_refused_before_touching_the_backend_and_applied_with_yes` | CONTEXT | Auto | PASS | `test result: ok. 1 passed; 0 failed` |
| 13 | Teste `vcp_argument_accepts_decimal_hex_and_named_shortcuts_and_rejects_unknown_names` | CONTEXT | Auto | PASS | `test result: ok. 1 passed; 0 failed` |
| 14 | Teste `exit_codes_are_stable_for_not_found_invalid_value_unconfirmed_and_transport_failures` | CONTEXT | Auto | PASS | `test result: ok. 1 passed; 0 failed` |

**Totals:** 14 items | Auto: 12 (12 PASS, 0 FAIL) | Manual: 2 pending

**Manual confirmation required:**
Run `/jdi-confirm-dod cli` to confirm each manual item with evidence. Without that, `/jdi-ship` will refuse the phase.

## Recommendation
- **A phase passa.**
  - Os gates 1–8 estão limpos, sem blockers.
  - W-4 foi resolvido no Linux, conforme D-2026-09-26-cli-4. O fallback por texto é inevitável com `ddc` 0.2.2 e está documentado e fixado por teste.
  - O reviewer reproduziu as mutações que provam a classificação e a parada do retry.
  - O desvio para dev-dependencies é mais restrito que o autorizado e não trouxe pacote novo.
- **Falta confirmar os 2 itens Manual do PROJECT** com `/jdi-confirm-dod cli`. No README, avaliar se entram:
  - o panic em 0x7E (W-7) em Known limitations;
  - uma frase mais cautelosa sobre o Windows (W-6).
- **No PR:** deixar visíveis W-7 (defeito do `ddc-i2c` 0.2.2 em 0x7E) e W-8 (`62da5d0` remove as regras `ask` compartilhadas). Para W-8, confirmar a intenção ou mover o commit para um PR próprio.
- **Carry-over para `full-osd-control`:**
  - W-5: restringir o max-por-leitura a `Continuous`, com os testes nomeados;
  - W-6: probe no Windows, com a hipótese de `ERROR_GRAPHICS_DDCCI_VCP_NOT_SUPPORTED`;
  - W-7: `catch_unwind` por D-6, e corrigir a premissa da linha 14 da CONTEXT rascunho.
- **Hardware (Deferred, do orquestrador):**
  - round-trip reversível de `set brightness` e de `set volume` no RTK;
  - nunca `input`, `power`, 0x7E ou código desconhecido.

## DoD Critic (enhanced)

Forçado pelo `/jdi-issue` na re-verificação pós fix round W-4: 0 linhas hollow (critic retornou `[]`).

**Verdict:** APPROVED

## DoD Manual Confirmations

- [x] CHANGELOG.md updated with entry per release
      **Confirmed at:** 2026-09-27T15:34:08Z
      **By:** alison amorim
      **Evidence:** PR #5, landed via PR #6 (2628f8c) revisado e mergeado pelo mantenedor; a entrada da phase está em `## [Unreleased]` do CHANGELOG.md. Nenhuma versão foi lançada ainda: o heading `## [versão]` passa a ser cortado pelo fluxo de release (phase `release-packaging`). Confirmado pelo usuário em 2026-09-27.
- [x] README accurately describes current behavior
      **Confirmed at:** 2026-09-27T15:34:08Z
      **By:** alison amorim
      **Evidence:** diff do README revisado no PR #5, landed via PR #6 (2628f8c) e mergeado na `main`. Confirmado pelo usuário em 2026-09-27.
