# Phase 2: Review  (slug: ddc-backends)

**Verdict:** APPROVED_PENDING_MANUAL

## Gates
| Gate | Status | Details |
|---|---|---|
| Build | PASS | `cargo build --workspace --locked` exit 0. Cross-check `-p ddc-core -p ddc-adapters --target x86_64-unknown-linux-gnu` exit 0. `cargo check -p ddc-adapters --locked --features ddc-hi --target x86_64-pc-windows-msvc` exit 0, incluindo a asserção estática `Send + Sync` em código não-test (`ddc_hi_backend.rs:108-111`). `--no-default-features` exit 0. Único aviso: future-incompat de `nom v3.2.1` (transitivo, ver W-4). |
| Tests | PASS | 86 passed, 0 failed, 4 ignored (hardware, `tests/real_monitor.rs`). Phase anterior: 61 → +25, sem queda. |
| Coverage | PASS | 95.23% lines, threshold 80% (linha TOTAL: `755 36 95.23%`; main.rs/build.rs excluídos, mas não existe nenhum no workspace). |
| Lint | PASS | `fmt --check` exit 0; `clippy --workspace --all-targets -D warnings` exit 0 (também limpo com `--no-default-features` e com `--target x86_64-pc-windows-msvc`). Único `#[allow]` novo: `worker/tests.rs:176`, com `// reason:` na linha acima. |
| Hexagonal/Safety/Hygiene | WARN | 5.1–5.9 e 5.11 sem achado. 5.10: WARN, 2 advisories transitivos inalcançáveis (W-4). Higiene: W-2 e W-5. |
| Consistency | WARN | Nenhuma violação na letra de D-XX. Há conflito suspeito com o rationale de D-7 (W-1) e de D-6 (W-2), e um desvio do PLAN T-5 (W-3). |
| UI Validation | SKIPPED | has_frontend=false |
| DoD | PASS_PENDING_MANUAL | 13/13 auto, 2 manual pending |

## Blockers
- _(nenhum)_

## Warnings

- **W-1 (Gate 6, D-2026-09-25-ddc-backends-7 + PLAN A-5). Ponto 2 do doer: num backend novo, o primeiro `read_vcp`/`write_vcp` sem `enumerate()` antes dá `Timeout` determinístico no hardware de dev.**
  - **Causa.** Em `crates/ddc-adapters/src/ddc_hi_backend/worker.rs:143-144`, um id desconhecido dispara `self.enumerate()`, que leva 1.11–1.20s medidos. Esse enumerate roda dentro do deadline da operação VCP (1s, `ddc_hi_backend.rs:25`), e o caller desiste em `worker.rs:232` antes de ele acabar.
  - **Consequências.**
    - (a) O enumerate sob demanda de A-5 nunca funciona no box com os defaults. O PLAN o justificou justamente com "a CLI pode chamar `read_vcp` sem `enumerate` antes", e `first_request_enumerates_on_demand` (`worker/tests.rs:340`) só o prova com fakes.
    - (b) Id desconhecido (typo, monitor recém-plugado) dá `Timeout` em VCP, não `MonitorNotFound`.
    - (c) Os dois adapters do mesmo port divergem numa pré-condição que o port (`crates/ddc-core/src/ports/monitor_backend.rs`) não declara. O fake responde `read_vcp` sem enumerate; o real não. Um driving adapter testado contra o fake passa nos testes e falha no hardware.
  - **Por que é WARN.** O defeito contraria o rationale de D-7 ("com 1s fixo, todo `enumerate` … daria `Timeout` no monitor de dev"): uma enumeração volta a rodar sob 1s, só que implícita. Mas a letra de D-7 (`read_vcp` = 1s) é cumprida, então é suspeita, não violação.
  - **É BLOCK para o objetivo desta phase ("get/set VCP com retry e timeout")? Não.**
    - Get VCP funciona em hardware pelos caminhos que existem hoje: 0x10 em 43.8ms e 0xDF em 93.8ms depois de `enumerate()`.
    - O único consumidor na árvore, `SoftwareOsd`, chama `read_capabilities` antes de qualquer VCP em `get_feature`/`set_feature`. Esse budget é de 8s e cobre 1.1s + 2.5s = 3.6s.
    - Nenhum item de DoD nem decisão locked é contrariado na letra.
    - O chamador nunca trava, e um write vencido não chega ao monitor (`retry.rs:71`; teste `deadline_spent_finding_the_monitor_sends_nothing`).
  - **Correção da premissa sobre a phase seguinte.** A `cli`, como já decidida, NÃO torna `read_vcp` a primeira chamada ao backend:
    - D-2026-09-25-cli-2 exige `enumerate()` para resolver `--monitor`. Omitido, auto-seleciona sobre a lista. Informado, casa por id exato, índice 1-based ou substring sobre a ordem de `enumerate()`.
    - A CONTEXT da `cli` espera ~1.1–1.3s por `get` com cache ("só o enumerate").
    - Com D-cli-1 + D-cli-2, o fluxo é enumerate → caps do disco → `read_vcp` com a tabela já populada, e funciona.
  - **Onde a armadilha continua latente:**
    - um atalho de id exato na `cli` que pule o `enumerate`;
    - `tray-app`/`profiles-hotkeys` aplicando um `MonitorId` salvo num backend recém-criado (hotkey → `write_vcp` → `Timeout`);
    - um monitor plugado depois do último enumerate.
  - **Recomendação.** Corrigir antes de `tray-app`, de preferência como primeira task da `cli` (ou neste PR, se sair barato). Duas opções:
    - o worker devolve rápido um sinal interno de "id desconhecido", e o `WorkerClient` roda `enumerate()` com `budgets.enumerate` e depois repete a operação com o budget VCP (cada etapa com o budget do seu tipo, como D-7 pede);
    - ou declarar no port/README a pré-condição "chame `enumerate()` antes" e remover o enumerate sob demanda.
    - Em qualquer caso, o planner da `cli` precisa receber esta nota.

- **W-2 (Gate 5 higiene + Gate 6, D-2026-09-25-ddc-backends-6 / PLAN A-5). Ponto 1 do doer: um display com DDC mudo vira `Timeout` e ocupa o worker ~1.2s a cada falha.**
  - **Causa.** Em `worker.rs:154-158`, depois que as tentativas se esgotam (~100ms com NAK), `explain()` re-enumera o barramento inteiro (~1.1s) sem olhar o deadline.
  - **Efeitos com o budget VCP de 1s:**
    - (a) O caller sempre recebe `Timeout` (hardware: LG TV → `Timeout` em 1.000s). Os ramos "`Transport` … (gave up after attempt N of 3)" e "`MonitorNotFound` de monitor desplugado" ficam inalcançáveis para VCP no box. Para caps (8s) eles funcionam.
    - (b) Um NAK rápido passa a ocupar a fila única por ~1.2s. D-6 aceitou o bloqueio da fila só para um monitor "genuinamente travado", com a premissa explícita "(a) NACKs … falham rápido … não travam". A implementação anula essa premissa.
    - (c) Com chamadas concorrentes (tray via `spawn_blocking`, phase `tray-app`), uma leitura do RTK enfileirada atrás de uma falha da TV pode estourar o próprio budget e dar `Timeout`. A medição de 287ms foi sequencial e não mostra isso.
  - **Efeito na `cli`.** D-cli-5 mapeia `Transport`/`Timeout` → exit 6 e `MonitorNotFound` → exit 3. Um monitor desplugado entre o `enumerate` e o `get` sai com 6.
  - **Docs.** `README.md:14` e `CHANGELOG.md:21` descrevem como geral o comportamento de `MonitorNotFound`/`Transport` após falha, mas com os defaults ele não é observável em VCP (ver DoD #4 e #5).
  - **Por que é WARN.** O resultado cabe na letra de D-4 ("`Transport` ou `Timeout` dentro do orçamento") e no critério diferido da CONTEXT.
  - **Recomendação.** Uma de duas:
    - pular a checagem de presença quando ela não cabe no deadline restante: devolver `Transport` direto e marcar a tabela como velha, para que o próximo id desconhecido re-enumere;
    - ou medir o último enumerate e só re-enumerar se `now + custo < deadline`.

- **W-3 (Gate 6, consistência com o PLAN). Ponto 3 do doer: o teste de hardware aceita `Ok` em outros displays** (`crates/ddc-adapters/tests/real_monitor.rs:146`).
  - **O afrouxamento em si é aceitável.** O texto do PLAN T-5 ("com erro só `Transport`/`Timeout`") é ambíguo. Aceitar `Ok` para não quebrar numa máquina com um segundo monitor DDC funcional é razoável, e o mapeamento de erro já está coberto pelos testes unitários (DoD #14).
  - **A fraqueza real é outra.** O loop em `real_monitor.rs:137` passa vacuamente quando não há outro display, e nada garante que D-4 (display mudo listado) foi exercitado. O nome `…_fail_fast_…` também contradiz a medição: a TV "falhou" em 1.000s, o budget inteiro (W-2).
  - **Nesta rodada a evidência vale.** A TV existia no box e deu `Timeout`.
  - **Recomendação.** Afirmar, ou ao menos imprimir, que algum display além do RTK foi exercitado (por exemplo, um `eprintln!` "D-4 não exercitado"). Renomear para `…_fail_within_budget_…`.

- **W-4 (5.10, supply chain). RUSTSEC-2018-0005 (`serde_yaml 0.7.5`, recursão sem limite) e RUSTSEC-2024-0320 (`yaml-rust 0.4.5`, unmaintained).**
  - **Origem.** Transitivos: `ddc-hi 0.4.1 → mccs-db 0.1.3 → serde_yaml 0.7.5 → yaml-rust 0.4.5` (`cargo tree -i`).
  - **São inalcançáveis pelos nossos caminhos:**
    - O YAML só é parseado em `mccs_db::Database::from_version` (usa `include_bytes!("../data/mccs.yml")`, mccs-db `src/lib.rs:301-310`) e em `from_database` (`src/lib.rs:317`).
    - O `ddc-hi` só chega a essas funções por `DisplayInfo::from_capabilities`/`update_from_ddc` e `Display::update_capabilities`/`update_from_ddc` (ddc-hi `src/lib.rs:143-169, 241-249, 482, 500`).
    - `Display::enumerate()` monta `DisplayInfo` por `from_edid`/`new` com `mccs_database: Default::default()`. É `#[derive(Default)]`, sem parse (ddc-hi `src/lib.rs:87-138`).
    - Nosso `impl DdcHandle for Handle` (`hardware.rs:29-44`) usa só os métodos crus de `ddc::Ddc` (`capabilities_string`, `get_vcp_feature`, `set_vcp_feature`) e nunca `Handle::capabilities()`.
    - Mesmo se fosse alcançado, o YAML é o embutido em tempo de compilação, nunca entrada externa.
  - **Risco residual.** Peso morto no grafo (`mccs-caps`/`nom 3` também são inalcançáveis), e o aviso future-incompat de `nom v3.2.1` pode virar erro de compilação num rustc futuro e quebrar o build do adapter.
  - **Status.** Previsto na CONTEXT (Deferred); não bloqueia.
  - **Recomendação.** Acompanhar na `ci-crossbuild`: fixar a toolchain, ou considerar `[patch]`/fork enxuto do `ddc-hi` se o future-incompat escalar.

- **W-5 (DRY, baixo). O fixture de caps do RTK agora está em 3 arquivos:** `crates/ddc-adapters/tests/real_monitor.rs:22`, `crates/ddc-core/src/domain/capabilities/tests.rs:8` e `crates/ddc-core/tests/monitor_control/support.rs:6`.
  - A cópia foi assumida em A-10, porque o core não exporta o fixture.
  - Só que o objetivo do teste de hardware é justamente comparar com o fixture do core, e uma divergência silenciosa entre as cópias o esvaziaria.
  - **Recomendação.** Um único `fixtures/rtk-qhd-hdr.caps`, lido por `include_str!` nos três pontos.

**Notas (não contadas como warning):**
- **Tamanho da T-3.** Saiu com 1060 linhas, contra ~600 estimadas no PLAN (o hook avisa em 800); cerca de 55% são testes. A atomicidade está ok: a task não se divide sem deixar código morto, como o PLAN já previa.
- **Conformidade verificada com as decisões:**
  - D-1: port inalterado; o diff da phase não toca `crates/ddc-core/src`.
  - D-2: o core continua só com `thiserror`.
  - ddc-backends-1: `ddc-hi` numa linha só do `Cargo.toml`, sem `nvapi`/`ddc-macos`.
  - -2: derivação de id em `identity.rs`.
  - -3/-7: números em `retry.rs:10-12` e `ddc_hi_backend.rs:25-29`; `with_budgets` substitui `with_timeout`, como D-7 permite.
  - -4: `hardware.rs:18-27` só traduz.
  - -5: `hardware.rs:48-53`, confirmado em hardware (0xDF = 0x0202).
  - -6: o struct guarda só `Sender` + `JoinHandle`, zero `unsafe`, e a asserção compila no alvo Windows.
- **Monitor-write safety:**
  - Nenhum `Confirm::Yes` fora de fronteira, e o adapter não aplica regra de domínio.
  - Em `tests/`, `DdcHiMonitorBackend` só aparece dentro de corpos `#[ignore]` que exigem `DDC_HW_TESTS=1` (`real_monitor.rs:71,86,103,133`), e o arquivo não tem nenhum `write_vcp`.
  - O teste unitário `backend_starts_and_stops_without_touching_a_monitor` só constrói o backend. A-4 permite isso, porque o worker só enumera na primeira request.
  - Para writes, `Timeout` quer dizer "resultado desconhecido": uma tentativa que começa pouco antes do deadline pode terminar depois de o caller desistir. É inofensivo para Set VCP (valor absoluto), mas a `cli` deve escrever a mensagem de erro com esse sentido.

## DoD Checklist (gate 8)

| # | Criterion | Source | Type | Status | Evidence |
|---|---|---|---|---|---|
| 1 | `cargo test --workspace` exits 0 | PROJECT | Auto | PASS | exit 0: 86 passed, 0 failed, 4 ignored |
| 2 | Coverage >= 80% of lines | PROJECT | Auto | PASS | TOTAL Lines 95.23% (755 linhas, 36 perdidas); `--fail-under-lines 80` exit 0 |
| 3 | No `TODO`/`FIXME` without linked issue reference | PROJECT | Auto | PASS | grep sem saída → OK |
| 4 | CHANGELOG.md updated with entry per release | PROJECT | Manual | MANUAL_REQUIRED | suggested: `## [Unreleased]` ganhou 9 bullets da phase (`CHANGELOG.md:18-26`); nenhum heading `## [version]` novo (sem release nesta phase); `CHANGELOG.md:21` promete `MonitorNotFound` após falha, o que não se observa em VCP com os defaults (W-2) |
| 5 | README accurately describes current behavior | PROJECT | Manual | MANUAL_REQUIRED | suggested: o diff do README atualiza status, layout, dev setup, check Windows, testes de hardware e link do doc Linux. Imprecisões: `README.md:14` (`MonitorNotFound`/`Transport` após falha não acontece em VCP com os defaults, W-2) e nenhuma menção de que o 1º read/write num backend novo sem `enumerate()` dá `Timeout` (W-1) |
| 6 | `ddc-hi` opcional, `default-features = false`, `["ddc-i2c","ddc-winapi"]`, feature `ddc-hi` em `default`, sem `nvapi`/`ddc-macos` | CONTEXT | Auto | PASS | OK |
| 7 | `ddc-adapters` compila para Windows e a asserção estática prova `Send + Sync` | CONTEXT | Auto | PASS | exit 0, OK; asserção em código não-test em `ddc_hi_backend.rs:108-111` |
| 8 | Teste `ddc_hi_backend_is_send_and_sync` | CONTEXT | Auto | PASS | `test result: ok. 1 passed; 0 failed` |
| 9 | Teste `maps_ddc_hi_vcp_reply_to_core_current_and_max` | CONTEXT | Auto | PASS | `test result: ok. 1 passed; 0 failed` |
| 10 | Teste `derives_monitor_id_from_edid_and_disambiguates_duplicate_serials` | CONTEXT | Auto | PASS | `test result: ok. 1 passed; 0 failed` |
| 11 | Teste `derives_monitor_id_from_description_or_index_when_edid_absent` | CONTEXT | Auto | PASS | `test result: ok. 1 passed; 0 failed` |
| 12 | Teste `retries_transient_errors_up_to_three_times_within_timeout_budget` | CONTEXT | Auto | PASS | `test result: ok. 1 passed; 0 failed` (clock virtual: 3 cenários) |
| 13 | Teste `caller_receives_timeout_when_worker_does_not_answer_within_budget` | CONTEXT | Auto | PASS | `test result: ok. 1 passed; 0 failed` (0.02s) |
| 14 | Teste `maps_backend_failures_to_transport_or_timeout_without_leaking_ddc_hi_errors` | CONTEXT | Auto | PASS | `test result: ok. 1 passed; 0 failed` |
| 15 | `docs/linux-ddc-setup.md` cobre `i2c-dev`, udev, `uaccess`/grupo, NVIDIA, `sudo` | CONTEXT | Auto | PASS | OK |

**Totals:** 15 items | Auto: 13 (13 PASS, 0 FAIL) | Manual: 2 pending

**Manual confirmation required:**
Rode `/jdi-confirm-dod ddc-backends` para confirmar cada item manual com evidência. Sem isso, `/jdi-ship` recusa a phase. Ao confirmar o #5 (README), considere as imprecisões apontadas em W-1/W-2.

## Recommendation
- **Sem bloqueios.** Os gates mecânicos estão todos verdes, o DoD auto está 13/13 e a cobertura é de 95.23%. A phase pode seguir para `/jdi-confirm-dod` (CHANGELOG e README) e depois `/jdi-ship`.
- **Prioridade antes de `tray-app`: W-1 e W-2.** São o mesmo defeito de desenho: um enumerate completo (~1.1s) roda dentro do budget VCP de 1s.
  - **W-1** transforma o 1º VCP num backend novo em `Timeout`.
  - **W-2** transforma todo NAK esgotado em `Timeout` e ocupa a fila única por ~1.2s.
  - Nenhum dos dois quebra a `cli` como decidida (D-cli-2 enumera antes), mas ambos quebram o uso concorrente e com id salvo que as phases `tray-app`/`profiles-hotkeys` vão fazer.
  - **Recomendação:** registrar uma D-XX e resolver na primeira task da `cli`, ou num fix curto neste PR se o orquestrador preferir. Levar esta nota ao planner da `cli`.
- **W-3 e W-5** são ajustes baratos de teste.
- **W-4** fica para a `ci-crossbuild`.

## DoD Critic (enhanced)

Forçado pelo `/jdi-issue`.
- DoD row «14 — falhas do backend real nunca vazam tipos `ddc_hi`/`anyhow`» (hollow, não objetivo): o teste nomeado prova o mapeamento dentro do worker só com `FakeHandle` (`HandleError::new(&'static str)`); a única conversão real (`.map_err(HandleError::new)` sobre `anyhow::Error` em `crates/ddc-adapters/src/ddc_hi_backend/hardware.rs:29-43`) não é exercitada fora de `#[ignore]` — a garantia de não-vazamento vem do sistema de tipos, não do teste. O critério "sempre Transport/Timeout" também ignora o `MonitorNotFound` intencional de A-5. Sugestão: teste sem hardware passando um `anyhow::Error` com cadeia de causas por `HandleError::new`.

**Verdict:** APPROVED_WITH_WARNINGS
