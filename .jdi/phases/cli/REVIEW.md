# Phase 3: Review  (slug: cli)

**Verdict:** APPROVED_PENDING_MANUAL

_Iter 2. Commits revistos: `42e74d7..HEAD` na branch `phase/cli`, com foco no delta `3fdfd61..HEAD` (`3bdb032`, `ba18b7a`, `a44b658`, `e514629`, `a2260dc`, `718ed4b`, `19f9ddf`). O reviewer não rodou teste de hardware nem o binário real. A evidência de hardware citada vem do doer (SUMMARY.md § Iter 2). Os arquivos não commitados da `full-osd-control` ficaram fora da review; só foram consultados para checar o carry-over (W-5)._

## Gates
| Gate | Status | Details |
|---|---|---|
| Build | PASS | `cargo build --workspace --locked`: exit 0. Check Linux (`-p ddc-core -p ddc-adapters -p ddc-cli --target x86_64-unknown-linux-gnu`): exit 0. Check Windows (`-p ddc-cli -p ddc-adapters --target x86_64-pc-windows-msvc`): exit 0. Check Windows com `-p ddc-adapters --features ddc-hi`: exit 0. Único aviso: future-incompat de `nom v3.2.1`, transitivo e pré-existente. |
| Tests | PASS | 189 passed, 0 failed, 5 ignored (hardware, `tests/real_monitor.rs`). Na iter 1 eram 180: são +9 testes (3 do core, 4 do cache, 1 do worker e 1 do binário) e nenhum foi removido. Os 4 testes renomeados mantêm cobertura equivalente ou maior. |
| Coverage | PASS | 96.80% das linhas, threshold 80%. Linha TOTAL: `2064 82 96.03% 260 12 95.38% 1343 43 96.80%` (main.rs/build.rs excluídos). `main.rs` não mudou na iter 2. `software_osd.rs` tem 100% das linhas cobertas e `caching.rs` 99.08%. |
| Lint | PASS | `cargo fmt --all --check`: exit 0. `cargo clippy --workspace --all-targets --locked -- -D warnings`: exit 0, e também com `-p ddc-adapters --features ddc-hi`. Nenhum `#[allow(...)]` fora de teste. |
| Hexagonal/Safety/Hygiene | PASS | 5.1–5.11 limpos (detalhe abaixo). |
| Consistency | WARN | D-1, D-2, D-2026-09-26-cli-1..3 e D-2026-09-25-cli-1..5 respeitadas (itens a–d abaixo). W-4 e W-5 são carry-over para `full-osd-control`. |
| UI Validation | SKIPPED | has_frontend=false |
| DoD | PASS_PENDING_MANUAL | 12/12 auto, 2 manual pending |

### Gate 5 em detalhe
- **5.1 / 5.2 / 5.8:** em `ddc-core`, `[dependencies]` continua só com `thiserror`. Nenhum I/O, cfg de plataforma ou caminho de dispositivo no core. A mudança no core (`software_osd.rs`, doc de `monitor_control.rs`) é lógica pura.
- **5.3:** nenhum `impl MonitorBackend` no core e nenhum `pub trait` fora dele. `RetryPolicies` é um struct `pub(crate)`, não um port.
- **5.4:** nenhum adapter é construído fora da composition root. `caching.rs` só importa `ddc_core` e usa `Capabilities::parse`, na direção permitida (adapter → core), como D-2026-09-26-cli-3 exige.
- **5.5:** nenhum `unsafe`. `#![forbid(unsafe_code)]` está nas raízes. Em `ddc-adapters` vale `[workspace.lints.rust] unsafe_code = "deny"`.
- **5.6:** nenhum `unwrap`/`expect`/`panic!` fora de teste.
- **5.7:** `Confirm::Yes` não aparece fora de `ddc-cli/src` nem de testes. `DdcHiMonitorBackend` só aparece em `tests/real_monitor.rs`, com 5× `#[ignore]` e gate `DDC_HW_TESTS`, sem mudança nesta iteração.
- **5.9:** N/A, ainda não há Tauri.
- **5.10:** `Cargo.toml` e `Cargo.lock` não mudaram na iter 2. `cargo audit` só mostra os já conhecidos RUSTSEC-2018-0005 (serde_yaml 0.7.5) e RUSTSEC-2024-0320 (yaml-rust 0.4.5).
- **5.11:** sem segredos, sem TODO/FIXME.

### Gate 6 em detalhe

**Plano e commits.**
- Os 7 commits da iter 2 usam scope `cli`. O tipo de cada um bate com o conteúdo (1× `test`, 3× `fix`, 3× `docs`).
- Cada `fix` cita a D-XX que implementa. Nenhum commit mistura `.jdi/` com código.
- `a44b658` leva o README junto com a mudança de comportamento. Isso é aceitável: a doc da regra nova fica atômica com ela.
- Os arquivos do core e de `ddc_hi_backend/` estão fora do `files_modified` do PLAN. Foram autorizados por D-2026-09-26-cli-1/-2 e estão registrados no SUMMARY § Iter 2.
- A linha 7 do PLAN ("Ports e `SoftwareOsd` não mudam") ficou superada pela decisão travada mais nova. Não é pendência do doer.

**(a) Implementação × D-XX novas.**
- **D-2026-09-26-cli-1: conforme.**
  - `SoftwareOsd::set_feature` (`crates/ddc-core/src/app/software_osd.rs:117-133`) segue a ordem da decisão:
    1. `authorize_write` (`:124`);
    2. caps, que dão a lista NC (`:125-126`);
    3. `max_for_write` (`:77-87`): max em cache, senão UMA `read_vcp`, com o código declarado ou não. O `?` propaga o erro da leitura;
    4. `validate_write` (`:128`);
    5. `write_vcp` (`:129`);
    6. read-back (`:130`).
  - Os testes nomeados provam cada ramo: `write_features.rs:97` (0x62 não declarado: `ReadCapabilities, ReadVcp, WriteVcp, ReadVcp`), `:121` (limitado pelo max lido), `:138` (Transport, Timeout e MonitorNotFound, sem `WriteVcp`), `:162` (não respondido → `UnsupportedFeature`), `:175` (reuso do max, sem releitura) e `:264` (caps ilegível → lê o max).
  - Pelo binário: `crates/ddc-cli/tests/cli/writes.rs:64` (`set volume 40` → 0, `set volume 101` → 4).
  - A fixture `--fake` agora espelha o RTK: 0x62 é respondido, mas fica fora do caps (`fixture.rs:14-15`). Continua respondendo os 6 atalhos (D-2026-09-25-cli-4).
  - Ressalva de adapter: ver W-4.
- **D-2026-09-26-cli-2: conforme.**
  - `retry.rs:10-16` define `MAX_ATTEMPTS = 3`, `VCP_BACKOFF = 50 ms` e `CAPABILITIES_BACKOFF = 500 ms`. O worker escolhe a policy pelo tipo de transação (`worker.rs:153,164,175`).
  - Os orçamentos continuam intactos: `ddc_hi_backend.rs:26-30` mantém 1 s, 8 s e 5 s.
  - O teste com relógio virtual (`worker/tests.rs:545`) fixa os valores literais, sem ler as consts: [500, 500] para caps, [50, 50] para VCP, e o corte pelo orçamento (3, 2 e 1 tentativas com 8 s, 700 ms e 500 ms).
  - A eficácia no RTK, 6/6 `caps --refresh` seguidos com exit 0, é evidência do doer. O reviewer não a reproduziu.
- **D-2026-09-26-cli-3: conforme.**
  - (a) `invalidate` (`caching.rs:61`) só marca o id em memória. O arquivo só é trocado por `store` depois de uma releitura OK (`:80-87`), com escrita atômica, e a marca sai.
  - (b) `store` recusa o que `parses` (`:189`) rejeita.
  - Testes: `caching_backend.rs:190` (refresh falho mantém o arquivo para a próxima execução), `:219` (retry dentro da mesma execução), `:238` (truncado nunca persiste, nem com cache frio nem com arquivo bom) e o DoD #8, que continua provando o bypass e o overwrite sem a deleção.

**(b) Extensão do doer: arquivo de cache não parseável conta como miss (`caching.rs:75`). Aceitável, não pede D-XX.**
- É o complemento, no lado da leitura, da intenção de D-2026-09-26-cli-3 (b): "caps truncado nunca vira memória". Sem ela, um arquivo gravado antes de `a2260dc`, ou que um parser futuro mais estrito recuse, ficaria grudado exatamente como W-3 descrevia. Não há outra forma de honrar a decisão para arquivos que já existem.
- D-2026-09-25-cli-1 já tinha o "hit" definido como implementação: arquivo ilegível ou vazio conta como miss desde a iter 1, e isso foi aceito. A validade por parse é da mesma classe.
- O custo é um parse de string por hit, desprezível perto do enumerate de ~1.1 s.
- O caso patológico, um monitor cujo caps nunca parseia, relê o caps a cada execução. D-2026-09-26-cli-3 (b) já implica isso, então a extensão não cria custo novo.
- Está testado (`caching_backend.rs:260`) e documentado (README § Capabilities cache e doc do struct).
- Se o orquestrador quiser a letra alinhada, basta uma linha na próxima D-XX que tocar o cache.

**(c) Mudança no core numa phase chamada `cli`: conforme.**
- D-2026-09-26-cli-1 põe a correção explicitamente no use case do core ("não num retry da CLI"), como a iter 1 recomendou.
- A assinatura do port `MonitorControl::set_feature` não mudou, só o doc. `max_for_write` é privado. A CONTEXT ("assinaturas não mudam nesta phase") continua valendo.
- D-2026-09-25-core-domain-4 não foi editada: a pasta `decisions/` só ganhou arquivos, então o append-only foi respeitado. O commit cita "Refines D-2026-09-25-core-domain-4 as locked by D-2026-09-26-cli-1".
- A CLI continua sem regra de domínio: nenhum `Risk::`, `risk_for_code`, `authorize_write`, `stdin` ou `read_line` em `ddc-cli/src`. `run.rs` não mudou (D-2026-09-25-cli-3).

**(d) Invariante "nunca escrever às cegas": mantido.**
- **Dangerous é recusado antes de qualquer chamada ao backend.** `authorize_write` (`software_osd.rs:124`) roda antes do caps e do max. O DoD #12 prova isso pelo binário: o log da escrita recusada é só `[Enumerate]`.
- **O max é sempre conhecido antes do write.** Para uma feature sem lista, `max_for_write` devolve `Some(max)` ou o erro da leitura, nunca `None`. `validate_write` mantém o `known_max.ok_or(UnsupportedFeature)` defensivo (`feature.rs:84`). `write_vcp` (`:129`) só roda depois de `validate_write` dar Ok.
- **A lista NC é validada quando conhecida.** Um código com lista no caps tem `requires_known_max() == false`, então não há leitura de max e o valor é checado contra a lista (`feature.rs:76-83`).
- **Ressalva, aceita pela letra de D-2026-09-26-cli-1:** quando a lista NÃO é conhecida, o max lido passa a valer. Ver W-5.

## Blockers
- (nenhum)

## Status dos achados da iter 1
| Achado | Status | Evidência |
|---|---|---|
| Crítico DoD linha 10 (hollow, objetivo) | **RESOLVIDO** | `selection.rs:70-80` ganhou os ids `RTK-QHD-1` e `RTK-QHD-10`. A prova de mutação foi **reproduzida pelo reviewer** numa cópia descartável (`git archive HEAD`, no scratchpad, fora do repo): sem `select.rs:69-71`, o teste nomeado falha em `crates/ddc-cli/tests/cli/selection.rs:72` e o Verify não imprime OK. No código real, o Verify imprime OK. |
| W-1: `set volume` sempre saía 4 | **RESOLVIDO** | Corrigido no core por D-2026-09-26-cli-1 (ver a acima). Também cumpre a sugestão opcional da iter 1: a fixture tem agora o formato do RTK, e um teste de binário prova `set volume` → 0. A escrita no hardware não foi validada (nenhum `set` rodou); segue em Deferred. O refinamento para NC está em W-5. |
| W-2: caps em sequência; `--refresh` apagava o cache bom; cache frio virava exit 4 | **RESOLVIDO** | (a) Backoff de 500 ms (D-2026-09-26-cli-2), com teste de relógio virtual e 6/6 no hardware (doer). (b) `invalidate` mantém o arquivo (D-2026-09-26-cli-3 a), com teste em `caching_backend.rs:190`. (c) Cache frio com caps falhando agora lê o max, sem exit 4 espúrio (`write_features.rs:264`). |
| W-3: caps `Ok` não parseável ficava persistido | **RESOLVIDO** | `store` só persiste o que `Capabilities::parse` aceita (D-2026-09-26-cli-3 b), com teste em `caching_backend.rs:238`. A extensão do lado da leitura é aceitável (ver b). |
| Nota: README Status dizia que o binário escreve no hardware | **RESOLVIDO** | `README.md:5` agora diz que as escritas "have not been validated on real hardware yet". |
| Notas `Exit::Usage`, lista de atalhos duplicada em `args.rs`, `SIMULATED_TIMEOUT = 0xDF` | Inalteradas | Sem ação obrigatória; `args.rs` e `exit.rs` não mudaram. |

## Warnings

### W-4: o adapter real nunca produz `UnsupportedFeature`. No hardware, um código que o monitor declara "não suportado" sai 6, não 4
- **Onde:**
  - `ddc-0.2.2/src/commands.rs:103` transforma o result code DDC/CI `0x01` em `ErrorCode::Invalid("Unsupported VCP code")`;
  - `crates/ddc-adapters/src/ddc_hi_backend/hardware.rs:34-37,48-49` converte isso em `HandleError` (string);
  - `retry.rs:102` retenta qualquer `HandleError` até 3 vezes;
  - `worker.rs:198-216` responde `DdcError::Transport("... Unsupported VCP code (gave up after attempt 3 of 3)")`.
- **O quê:** D-2026-09-26-cli-1 diz "só `UnsupportedFeature` quando o monitor responde que não suporta". O core cumpre: propaga o erro que o port devolver, provado com o fake em `write_features.rs:162`. Mas o adapter `ddc-hi` não distingue essa resposta de uma falha de transporte.
  - Consequência: no hardware real, `get`/`set` num código que o monitor recusa explicitamente sai **6** (transporte). Pela tabela de D-2026-09-25-cli-5 e pelo README (`README.md:82`), seria **4**.
  - A resposta é determinística, então os 2 retries não servem para nada.
  - Antes desta iteração, um `set` num código não declarado saía 4 sem tocar o barramento. Agora vai ao monitor e cai nisso.
  - Esta análise vem da leitura do código. O reviewer não observou no RTK. Os 9 códigos `Safe` são todos declarados ou respondidos no RTK, então um `set` Safe no monitor de dev não cai nesse caminho.
- **Por que WARN e não BLOCK:**
  - É comportamento pré-existente do adapter, da phase `ddc-backends`. Já valia para `get`.
  - D-2026-09-26-cli-1 escopa a correção ao core.
  - Nenhuma escrita acontece: a leitura falha antes.
  - O exit 6 vem com uma mensagem honesta, que cita "Unsupported VCP code".
- **Carry-over obrigatório para `full-osd-control`:**
  - `features --probe` vai varrer códigos que o monitor não suporta. Cada um custaria 3 tentativas e viraria "Transport".
  - Com isso, o probe não consegue separar "não suportado" de "não respondeu": `.jdi/phases/full-osd-control/CONTEXT.md:70`, ainda não commitado, espera 0x7E marcado como "not responding".
  - É preciso uma D-XX que refine o mapeamento de erro de `ddc-backends`: resposta `0x01` → `DdcError::UnsupportedFeature`, sem retry. Mais um teste no adapter com um `HandleError` tipado.

### W-5: o max obtido por leitura também vale para feature NC sem lista conhecida. O refinamento de D-2026-09-26-cli-1 ainda não está no rascunho da `full-osd-control`
- **O quê:** `Capabilities::feature` (`crates/ddc-core/src/domain/capabilities.rs:55-68`) trata como `Continuous` todo código sem lista. Com D-2026-09-26-cli-1, esses códigos passam a ser escritos contra o max lido. Isso vale em dois casos:
  - (i) **Código NC fora do caps.** Por exemplo, input 0x60 num monitor que não lista entradas, com `--yes`. Ou qualquer código não classificado, inclusive da faixa 0xE0–0xFF, com `--yes`.
  - (ii) **Código NC declarado quando o caps está ilegível ou não parseia.** A lista se perde, e `preset` 0x14 e `OSD language` 0xCC, que são `Safe`, aceitam um valor fora da lista **sem `--yes`**, desde que ≤ max lido. Antes da iter 2, esse caso saía 4.
- **Por que WARN e não BLOCK:**
  - Segue a letra de D-2026-09-26-cli-1 ("declarado no caps ou não"), cujo "Refinamento previsto" manda a `full-osd-control` restringir isso a `Continuous` pelo catálogo.
  - O caso (ii) ficou mais raro com D-2026-09-26-cli-2/-3, que dão menos cache frio e nenhum caps ruim persistido.
  - A escrita continua limitada pelo máximo que o próprio monitor informa. Os códigos Safe afetados são de baixo dano.
  - O README documenta (i) em Known limitations e (ii) em Monitor-write safety.
- **Carry-over:**
  - O rascunho não commitado de `.jdi/phases/full-osd-control/CONTEXT.md` (09:11) é anterior a D-2026-09-26-cli-1..3 (09:36) e não as cita.
  - `D-2026-09-26-full-osd-control-5.md`, também não commitado, ainda parte da premissa de que o exit 4 `UnsupportedFeature` "já existe para qualquer feature sem `max` conhecido". Isso deixou de ser verdade.
  - No `/jdi-discuss full-osd-control`, incorporar:
    - o teste nomeado previsto por D-2026-09-26-cli-1: `set` Safe em código respondido mas não declarado;
    - a restrição do max-por-leitura a `Continuous`;
    - um teste para (ii): caps ilegível + NC Safe → recusado.
  - Quando o catálogo introduzir `Access::ReadOnly`, checar o acesso ANTES de `max_for_write`. Hoje a leitura do max (`software_osd.rs:127`) vem antes de `validate_write` checar o acesso (`feature.rs:73`), então um `set` num código RO leria o monitor à toa antes de recusar.

### Notas (sem ação obrigatória)
- README § Capabilities cache diz que o refresh leva "about 1 s longer" quando a primeira tentativa é recusada. Pelos números do doer (6.28 s contra 4.67 s), são ~1.6 s. Detalhe para a confirmação manual do README.
- Deferred to PR review: além do round-trip reversível de `set brightness`, vale incluir um `set volume` reversível no RTK. É Safe e prova D-2026-09-26-cli-1 no hardware. Nunca `input`, `power` ou código desconhecido.

## DoD Checklist (gate 8)

| # | Criterion | Source | Type | Status | Evidence |
|---|---|---|---|---|---|
| 1 | `cargo test --workspace` exits 0 | PROJECT | Auto | PASS | exit 0: 189 passed, 0 failed, 5 ignored (hardware); saída do Gate 2 reaproveitada |
| 2 | Coverage >= 80% of lines | PROJECT | Auto | PASS | TOTAL Lines 96.80% (1343 linhas, 43 perdidas); `--fail-under-lines 80` exit 0 (saída do Gate 3 reaproveitada) |
| 3 | No `TODO`/`FIXME` without linked issue reference | PROJECT | Auto | PASS | Verify exit 0 (nenhum hit) |
| 4 | CHANGELOG.md updated with entry per release | PROJECT | Manual | MANUAL_REQUIRED | suggested: `## [Unreleased]` (`CHANGELOG.md:8`) teve os bullets reescritos na iter 2 (max lido antes do write, retries de 500 ms para caps, cache só com caps parseável e refresh que mantém o arquivo, `set` em código não declarado). Ainda não há heading `## [x.y.z]`, nenhuma release foi cortada |
| 5 | README accurately describes current behavior | PROJECT | Manual | MANUAL_REQUIRED | suggested: Status corrigido (escritas não validadas no hardware); Capabilities cache (filtro por parse, refresh que mantém o arquivo, backoff de 500 ms); Known limitations reescrita (`volume` agora é gravável; ressalva para NC fora do caps); Monitor-write safety, passo 2, reescrito; sem `--fake`. Conferir: a linha 4 da tabela de exit codes contra W-4, e "about 1 s longer" contra ~1.6 s medido |
| 6 | `crates/ddc-cli` no workspace com clap derive; serde só em `ddc-cli` (D-2) | CONTEXT | Auto | PASS | Verify imprimiu `OK` |
| 7 | Teste `caching_backend_reads_capabilities_from_disk_without_calling_wrapped_backend_after_first_fetch` | CONTEXT | Auto | PASS | `test result: ok. 1 passed; 0 failed` |
| 8 | Teste `caching_backend_never_persists_failures_and_refresh_forces_a_fresh_overwrite` | CONTEXT | Auto | PASS | `test result: ok. 1 passed; 0 failed`. Sem a deleção em `invalidate`, o teste continua provando o bypass do arquivo velho e o overwrite (`caching_backend.rs:128-134`) |
| 9 | Teste `resolving_monitor_without_the_flag_auto_selects_the_only_one_or_refuses_when_zero_or_many` | CONTEXT | Auto | PASS | `test result: ok. 1 passed; 0 failed` |
| 10 | Teste `monitor_flag_matches_by_exact_id_index_or_unique_substring_and_refuses_ambiguous_matches` | CONTEXT | Auto | PASS | `test result: ok. 1 passed; 0 failed`. Deixou de ser oco: a mutação de `select.rs:69-71` faz o teste falhar em `selection.rs:72` (reproduzido pelo reviewer) |
| 11 | Teste `fake_flag_selects_a_deterministic_in_memory_backend_and_stays_hidden_from_help` | CONTEXT | Auto | PASS | `test result: ok. 1 passed; 0 failed` |
| 12 | Teste `dangerous_write_without_yes_is_refused_before_touching_the_backend_and_applied_with_yes` | CONTEXT | Auto | PASS | `test result: ok. 1 passed; 0 failed`. O log da escrita recusada continua só `[Enumerate]` (`writes.rs:20`) |
| 13 | Teste `vcp_argument_accepts_decimal_hex_and_named_shortcuts_and_rejects_unknown_names` | CONTEXT | Auto | PASS | `test result: ok. 1 passed; 0 failed` |
| 14 | Teste `exit_codes_are_stable_for_not_found_invalid_value_unconfirmed_and_transport_failures` | CONTEXT | Auto | PASS | `test result: ok. 1 passed; 0 failed` |

**Totals:** 14 items | Auto: 12 (12 PASS, 0 FAIL) | Manual: 2 pending

**Manual confirmation required:**
Run `/jdi-confirm-dod cli` to confirm each manual item with evidence. Without that, `/jdi-ship` will refuse the phase.

## Recommendation
- **A phase passa:**
  - gates 1–8 limpos, sem blockers;
  - os achados da iter 1 estão todos resolvidos: crítico da linha 10, W-1, W-2 e W-3;
  - as 3 decisões novas foram implementadas conforme a letra e provadas por testes nomeados;
  - o invariante "nunca escrever às cegas" está preservado.
- **Falta:** confirmar os 2 itens Manual do PROJECT com `/jdi-confirm-dod cli`. No README, ajustar ou aceitar a linha do exit 4 (W-4) e o "about 1 s longer".
- **Antes ou durante o `/jdi-discuss full-osd-control`, registre o carry-over de W-4 e W-5, porque o rascunho atual não cobre nenhum dos dois:**
  - W-4: D-XX para mapear a resposta "Unsupported VCP code" para `UnsupportedFeature`, sem retry. O `--probe` depende disso.
  - W-5: incorporar D-2026-09-26-cli-1..3 à CONTEXT e revisar a premissa de `D-2026-09-26-full-osd-control-5`. Levar também o teste nomeado previsto, a restrição do max-por-leitura a `Continuous`, o caso de caps ilegível + NC Safe e a ordem de acesso antes do max.
- **Hardware (Deferred, do orquestrador):**
  - round-trip reversível de `set brightness` e de `set volume` no RTK, lendo antes, escrevendo, conferindo o read-back e restaurando;
  - nunca `input`, `power` ou código desconhecido.

## DoD Critic (enhanced)

Forçado pelo `/jdi-issue` na iteração 2: 0 linhas hollow (critic retornou `[]`). A linha 10 deixou de ser hollow — o teste nomeado agora falha se o ramo de id exato de `select.rs` for removido.

**Verdict:** APPROVED
