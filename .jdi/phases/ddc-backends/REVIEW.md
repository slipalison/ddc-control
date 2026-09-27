# Phase 2: Review  (slug: ddc-backends)

**Verdict:** APPROVED_WITH_WARNINGS

_Iter 2: nova verificação depois da rodada de correção de warnings do `/jdi-issue` (Step 6). A review anterior está em `e2b8b70`. Commits revistos: `26c68d6..HEAD`, com foco em `dc167dd..68add8a`. Os testes de hardware não foram rodados; os números de hardware citados abaixo são a evidência do doer (SUMMARY.md § Fix round)._

## Gates
| Gate | Status | Details |
|---|---|---|
| Build | PASS | `cargo build --workspace --locked` exit 0. Cross-check `-p ddc-core -p ddc-adapters --target x86_64-unknown-linux-gnu` exit 0. `cargo check -p ddc-adapters --locked --features ddc-hi --target x86_64-pc-windows-msvc` exit 0, incluindo a asserção estática `Send + Sync`. `--no-default-features` exit 0. Único aviso: future-incompat de `nom v3.2.1` (transitivo, ver W-4). |
| Tests | PASS | 91 passed, 0 failed, 5 ignored (hardware, `tests/real_monitor.rs`). Iter 1 tinha 86 passed e 4 ignored: são +5 testes e +1 ignored, sem nenhum removido. |
| Coverage | PASS | 95.32% lines, threshold 80%. Linha TOTAL: `790 37 95.32%`. main.rs/build.rs estão excluídos, mas não existe nenhum no workspace. |
| Lint | PASS | `fmt --check` exit 0. `clippy --workspace --all-targets -D warnings` exit 0, também limpo com `--no-default-features` e com `--target x86_64-pc-windows-msvc`. Único `#[allow]`: `worker/tests.rs:192`, com `// reason:` na linha 191. |
| Hexagonal/Safety/Hygiene | WARN | 5.1–5.9 e 5.11 sem achado. 5.10 continua WARN pelos mesmos 2 advisories transitivos (W-4); nada novo. `anyhow` entrou só como dev-dependency e não tem advisory. |
| Consistency | PASS | A implementação segue D-2026-09-26-ddc-backends-1 (regras 1–3) e as demais D-XX relevantes (ver § Gate 6). Commits atômicos, escopo `ddc-backends`, tipos corretos, e cada commit cita a D-XX que aplica. |
| UI Validation | SKIPPED | has_frontend=false |
| DoD | PASS_PENDING_MANUAL | 13/13 auto, 2 manual pending |

## Blockers
- _(nenhum)_

## Status dos achados da iter 1

| Achado | Status | Evidência |
|---|---|---|
| W-1: 1º VCP num backend novo dava `Timeout` | **RESOLVIDO** | `worker.rs:184`: id desconhecido volta na hora como `TransactError::UnknownMonitor`, sem enumerar dentro do deadline. Em `worker.rs:287-307`, o cliente então enumera sob `budgets.enumerate` (`worker.rs:302` → `:320`) e repete com o budget da operação. Ainda ausente → `MonitorNotFound` (`:76`). Testes: `unknown_id_is_reported_at_once_without_enumerating` (`worker/tests.rs:431`) e `first_request_enumerates_under_the_enumeration_budget` (`:450`, enumerate real de 250ms contra VCP de 100ms → `Ok`; id fantasma → `MonitorNotFound`). Port inalterado, e fake e real voltam a se comportar igual. Hardware (doer): 1º `read_vcp` num backend novo → `Ok` em 1.149s (antes: `Timeout`). |
| W-2: NAK esgotado virava `Timeout` e prendia a fila ~1.2s | **RESOLVIDO** | `worker.rs:198` + `:213-217`: a checagem de presença só roda se `agora + custo medido do último enumerate < deadline` (desigualdade estrita; custo medido em `:136-138`). Senão, `Transport` na hora. Testes: `failed_vcp_answers_transport_at_once_when_a_presence_check_cannot_fit` (`:569`, clock virtual: VCP 1s contra enumerate 1.1s → `Transport` em 100ms, sem enumerate extra; cobre a borda `agora + custo == deadline`) e `failed_capabilities_read_checks_presence_when_it_fits` (`:593`, caps 8s → `MonitorNotFound` se sumiu, `Transport` se presente). Hardware (doer): LG TV → `Transport` em 103ms; RTK logo depois em 43.8ms. A premissa de D-6 ("NAKs falham rápido") voltou a valer. |
| W-3: teste de hardware passava vacuamente, nome enganoso | **RESOLVIDO** | `real_monitor.rs`: renomeado para `hardware_other_displays_fail_within_budget_and_worker_recovers`. Sem outro display, imprime "D-4 not exercised". Agora exige `Ok \| Transport` em < 500ms (`Timeout` deixou de ser aceito) e o RTK em < 250ms logo depois. Novo `hardware_first_read_on_a_fresh_backend_needs_no_enumerate`. O arquivo continua sem `write_vcp`. |
| W-4: RUSTSEC-2018-0005 / RUSTSEC-2024-0320 | **MANTIDO** (deferido por desenho) | `cargo audit` (43 crates): os mesmos 2 advisories, nada novo. A análise de inalcançabilidade da iter 1 continua válida: `impl DdcHandle for Handle` (`hardware.rs:29-43`) ainda usa só `capabilities_string`/`get_vcp_feature`/`set_vcp_feature` crus. Previsto em CONTEXT § Deferred; fica para `ci-crossbuild`. |
| W-5: fixture de caps copiado em 3 lugares | **RESOLVIDO** | Um único `crates/ddc-core/tests/fixtures/rtk_qhd_hdr_caps.txt` (259 bytes, sem newline final), lido por `include_str!(…).trim_ascii_end()` em `capabilities/tests.rs:9`, `monitor_control/support.rs:8` e `real_monitor.rs:24`. O grep não acha nenhuma cópia literal restante; `capabilities.rs:6` é um exemplo encurtado em doc comment, não o fixture. |
| Critic DoD #14 (hollow) | **RESOLVIDO** em substância | A conversão real virou `transaction_error` (`hardware.rs:48`), tipada em `<Handle as DdcHost>::Error` e usada pelos 3 métodos. `ddc_hi_error_chain_reaches_the_port_as_transport_text_only` (`hardware/tests.rs:143`) passa um `anyhow::Error` real com cadeia de contexto por ela e afirma, para caps, read e write, `Transport` com a cadeia inteira em texto e `source() == None`. Se o `ddc-hi` mudar o tipo de erro, o teste deixa de compilar. Resíduo: o `Verify:` da linha #14 ainda roda só o teste nomeado (baseado em fake). O bloco de DoD é locked e o reviewer não o edita; a prova do "backend real" vem do teste companheiro. |

## Gate 6: julgamento contra D-2026-09-26-ddc-backends-1

- **Regra 1 (cada etapa com o orçamento do seu tipo): conforme.**
  - O sinal interno (`TransactError`) nunca chega ao port. `for_caller` o converte em `MonitorNotFound` só depois do enumerate.
  - O port `MonitorBackend` não ganhou pré-condição: o diff da phase em `crates/ddc-core/src` é só `capabilities/tests.rs`, sob `#[cfg(test)]`.
  - **A segurança de escrita foi preservada.** A 1ª tentativa com id desconhecido nunca chega a um handle, então um `write_vcp` é enviado no máximo uma vez. O `serve` continua descartando job vencido.
- **Regra 2 (checagem de presença só quando cabe): conforme.** Os dois ramos e a borda de igualdade estão testados com clock virtual.
- **Regra 3 (provas em hardware): conforme.** Os testes `#[ignore]` com `DDC_HW_TESTS=1` existem e cobrem os dois pontos. Evidência do doer: 5/5, read-only.
- **Nota do doer ("com os budgets VCP default, um monitor que some dá `Transport`"): é ACEITÁVEL.**
  1. **Está conforme também ao rationale, não só à letra.** A própria D-26 descreve essa consequência dos defaults ("VCP em display mudo → `Transport` em ~100ms … caps (8s) mantém a distinção").
  2. **Nenhum cache fica envenenado.** O core só memoriza o resultado de caps, e `MonitorNotFound` fica fora (`software_osd.rs:40-47`). Erro de VCP não é memorizado, então um `Transport` para um monitor sumido só muda o tipo de erro daquela chamada. No caminho de caps a distinção continua, e está testada.
  3. **Na `cli` o efeito é uma janela estreita.** D-cli-2 enumera para resolver `--monitor`, então um monitor ausente falha na resolução (`MonitorNotFound`, exit 3). Só um monitor que some entre o enumerate e o VCP sai como `Transport` (exit 6).
  4. **A alternativa era pior.** Enumerar dentro de 1s dava `Timeout` sempre e prendia a fila ~1.2s.
  - **Resíduo, só para `tray-app` (nota N-4, não conta como warning).** Num backend de vida longa o handle velho fica na tabela. Todo VCP num monitor sumido, ou re-plugado no Windows (handle novo), responde `Transport` em ~100ms até alguém chamar `enumerate()`. Nada marca a tabela como velha: a sugestão da iter 1 não entrou na D-26, o que é legítimo.
- **Ponto de interpretação: conforme.** Se o próprio enumerate implícito estourar `budgets.enumerate` (5s), o caller recebe `Timeout` (`worker.rs:302`, via `?`).
  - A D-26 diz "(nunca `Timeout` por causa do enumerate)". Leio o parêntese como: o enumerate nunca consome o budget da operação, e um enumerate concluído sem achar o id dá `MonitorNotFound`.
  - Uma etapa que estoura o próprio budget → `Timeout` é exatamente "cada etapa usa o orçamento do seu tipo" (D-6/D-7). Responder `MonitorNotFound` sem saber se o monitor existe seria incorreto.
  - Esse ramo não tem teste (ver N-2).
- **Demais D-XX relevantes: todas conformes.**
  - D-1: port intocado.
  - D-2: core só com `thiserror`; `anyhow` só em `[dev-dependencies]` do `ddc-adapters`.
  - ddc-backends-3: `retry.rs` intocado na rodada.
  - -4: `DdcHiDisplays::enumerate` intocado.
  - -5: `vcp_value` intocado.
  - -6: struct = `Sender` + `JoinHandle`; o passo duplo do cliente não toca handles; zero `unsafe`.
  - -7: budgets inalterados; o enumerate implícito usa `budgets.enumerate`.

## Warnings
- **W-4 (5.10, supply chain; herdado da iter 1, não resolvido de propósito).**
  - RUSTSEC-2018-0005 (`serde_yaml 0.7.5`) e RUSTSEC-2024-0320 (`yaml-rust 0.4.5`), via `ddc-hi 0.4.1 → mccs-db 0.1.3`.
  - Continuam inalcançáveis pelos nossos caminhos. O aviso future-incompat de `nom v3.2.1` segue sendo o risco real de build num rustc futuro.
  - **Recomendação.** Acompanhar na `ci-crossbuild`: fixar a toolchain, ou fazer `[patch]`/fork enxuto do `ddc-hi` se escalar.

**Notas (não contadas como warning):**
- **N-1 (clean-code, nome de teste desatualizado).** `deadline_spent_finding_the_monitor_sends_nothing` (`worker/tests.rs:559`).
  - Depois do fix, `read_on_worker` enumera antes e o worker não procura mais o monitor dentro do deadline.
  - O teste agora prova "transação com deadline já vencido não envia nada". Sugestão: `spent_deadline_sends_nothing`.
- **N-2 (teste faltando, barato).** Nenhum teste cobre "enumerate implícito passa de `budgets.enumerate` → `Timeout`" (`worker.rs:302`, ramo de erro do `?`).
  - Dá para fazer com `SlowDisplays` + `SystemClock`, custo de 50ms contra enumerate de 20ms.
- **N-3 (PLAN A-11).** A-11 diz "nenhuma task posterior toca `Cargo.*`", mas o commit `8342338` adicionou `anyhow` como dev-dependency e 1 aresta no `Cargo.lock`. Não conta como inconsistência:
  - fica fora das tasks do PLAN (rodada orquestrada);
  - o lock foi commitado junto com a mudança;
  - a versão já estava travada (1.0.104);
  - D-2 restringe só o core.
- **N-4 (para `tray-app` e `profiles-hotkeys`).**
  - **Handle velho.** Ver § Gate 6. A tray precisa re-enumerar por conta própria: ao abrir o popup, num evento de mudança de display, ou depois de um `Transport`.
  - **Pior caso de tempo.** Uma chamada com id desconhecido leva cerca de fila + 5s + 1s, algo como 7s. O port não tem contrato de tempo, então isso só importa para o `spawn_blocking` da tray.
  - **Id salvo de monitor ausente.** Cada hotkey paga um enumerate completo (~1.1s) e ocupa a fila por esse tempo.
- **N-5 (herdada, ainda válida).** Para writes, `Timeout` quer dizer "resultado desconhecido", e a `cli` deve escrever a mensagem de erro com esse sentido.
- **N-6.** `real_monitor.rs:24` lê o fixture de `../../ddc-core/tests/fixtures/` por caminho relativo. Funciona no workspace; só quebraria num `cargo package` do `ddc-adapters`, que não está planejado.

## DoD Checklist (gate 8)

| # | Criterion | Source | Type | Status | Evidence |
|---|---|---|---|---|---|
| 1 | `cargo test --workspace` exits 0 | PROJECT | Auto | PASS | exit 0: 91 passed, 0 failed, 5 ignored |
| 2 | Coverage >= 80% of lines | PROJECT | Auto | PASS | TOTAL Lines 95.32% (790 linhas, 37 perdidas); `--fail-under-lines 80` exit 0 |
| 3 | No `TODO`/`FIXME` without linked issue reference | PROJECT | Auto | PASS | grep sem saída; exit 0 |
| 4 | CHANGELOG.md updated with entry per release | PROJECT | Manual | CONFIRMED | suggested: só existe `## [Unreleased]` (`CHANGELOG.md:8`), sem heading `## [version]` (nenhuma release nesta phase). `CHANGELOG.md:21-22` agora descreve o comportamento correto: sem `enumerate()` prévio; `Transport` rápido em VCP; distinção `MonitorNotFound` só em caps. A imprecisão da iter 1 foi corrigida. |
| 5 | README accurately describes current behavior | PROJECT | Manual | CONFIRMED | suggested: `README.md:14` (novo bullet "No `enumerate()` needed first") e `README.md:15` (Retries reescrito) batem com `worker.rs`; as imprecisões de W-1/W-2 foram corrigidas. Não menciona explicitamente que um monitor sumido continua respondendo `Transport` até um novo `enumerate()` (N-4); isso é implícito, não impreciso. |
| 6 | `ddc-hi` opcional, `default-features = false`, `["ddc-i2c","ddc-winapi"]`, feature `ddc-hi` em `default`, sem `nvapi`/`ddc-macos` | CONTEXT | Auto | PASS | OK |
| 7 | `ddc-adapters` compila para Windows e a asserção estática prova `Send + Sync` | CONTEXT | Auto | PASS | exit 0, OK |
| 8 | Teste `ddc_hi_backend_is_send_and_sync` | CONTEXT | Auto | PASS | `test result: ok. 1 passed; 0 failed` |
| 9 | Teste `maps_ddc_hi_vcp_reply_to_core_current_and_max` | CONTEXT | Auto | PASS | `test result: ok. 1 passed; 0 failed` |
| 10 | Teste `derives_monitor_id_from_edid_and_disambiguates_duplicate_serials` | CONTEXT | Auto | PASS | `test result: ok. 1 passed; 0 failed` |
| 11 | Teste `derives_monitor_id_from_description_or_index_when_edid_absent` | CONTEXT | Auto | PASS | `test result: ok. 1 passed; 0 failed` |
| 12 | Teste `retries_transient_errors_up_to_three_times_within_timeout_budget` | CONTEXT | Auto | PASS | `test result: ok. 1 passed; 0 failed` (clock virtual, 3 cenários) |
| 13 | Teste `caller_receives_timeout_when_worker_does_not_answer_within_budget` | CONTEXT | Auto | PASS | `test result: ok. 1 passed; 0 failed` (0.02s) |
| 14 | Teste `maps_backend_failures_to_transport_or_timeout_without_leaking_ddc_hi_errors` | CONTEXT | Auto | PASS | `test result: ok. 1 passed; 0 failed`. A conversão real é provada pelo teste companheiro `ddc_hi_error_chain_reaches_the_port_as_transport_text_only` (`hardware/tests.rs:143`), ver § Status (critic #14). |
| 15 | `docs/linux-ddc-setup.md` cobre `i2c-dev`, udev, `uaccess`/grupo, NVIDIA, `sudo` | CONTEXT | Auto | PASS | OK |

**Totals:** 15 items | Auto: 13 (13 PASS, 0 FAIL) | Manual: 2 pending

**Manual confirmation required:**
Rode `/jdi-confirm-dod ddc-backends` para confirmar cada item manual com evidência. Sem isso, `/jdi-ship` recusa a phase.

## Recommendation
- **Sem bloqueios.** Os gates mecânicos estão verdes, o DoD auto está 13/13 e a cobertura é de 95.32%.
- **Rodada de correção.** W-1, W-2, W-3, W-5 e o critic #14 foram resolvidos. W-4 continua deferido por desenho para `ci-crossbuild`.
- **D-2026-09-26-ddc-backends-1.** A implementação segue as 3 regras, e o `Transport` para um monitor sumido com os budgets VCP default é aceitável (ver § Gate 6).
- **Próximo passo.** `/jdi-confirm-dod ddc-backends` (CHANGELOG e README), depois `/jdi-ship ddc-backends`.
- **Levar ao planner da `tray-app`: N-4.** Re-enumerar ao abrir o popup, num evento de mudança de display ou depois de um `Transport`, e considerar o pior caso de ~7s de uma chamada com id desconhecido.
- **N-1 e N-2 são baratos.** Cabem na primeira task que tocar `worker/tests.rs`.

## DoD Critic (enhanced)

Forçado pelo `/jdi-issue` na re-verificação pós fix round: 0 linhas hollow. A linha #14 deixou de ser hollow — a garantia de não-vazamento está provada pelo tipo (`DdcError` sem `#[source]`, `Clone + Eq`), pela conversão de produção `transaction_error` e pelo teste `ddc_hi_error_chain_reaches_the_port_as_transport_text_only` (roda no `cargo test --workspace`). Resíduos não-hollow: o `Verify:` travado da #14 ainda aponta o teste baseado em fake; `source().is_none()` é decorativo; o critério "sempre Transport/Timeout" ficou desatualizado pelo `MonitorNotFound` de D-2026-09-26-ddc-backends-1.

**Verdict:** APPROVED

## DoD Manual Confirmations

- [x] CHANGELOG.md updated with entry per release
      **Confirmed at:** 2026-09-27T15:34:08Z
      **By:** alison amorim
      **Evidence:** PR #2, re-landed as PR #6 (2628f8c) revisado e mergeado pelo mantenedor; a entrada da phase está em `## [Unreleased]` do CHANGELOG.md. Nenhuma versão foi lançada ainda: o heading `## [versão]` passa a ser cortado pelo fluxo de release (phase `release-packaging`). Confirmado pelo usuário em 2026-09-27.
- [x] README accurately describes current behavior
      **Confirmed at:** 2026-09-27T15:34:08Z
      **By:** alison amorim
      **Evidence:** diff do README revisado no PR #2, re-landed as PR #6 (2628f8c) e mergeado na `main`. Confirmado pelo usuário em 2026-09-27.
