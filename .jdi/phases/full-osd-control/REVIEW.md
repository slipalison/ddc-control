# Phase 4: Review  (slug: full-osd-control)

**Verdict:** APPROVED_PENDING_MANUAL

Iteração 2 do loop. Esta review substitui a da iter 1 (`4ed9b02`, agregado BLOCKED pelo DoD critic) e cobre `18f9494..HEAD` (`2363bc7`), com foco no delta `4ed9b02..HEAD`: `a2f5fda`, `f3dc39d`, `2def274`, `6df4c07`, `8a09f06` e `2363bc7`, julgados contra a D-2026-09-26-full-osd-control-10 (`8fb334c`).

## Gates
| Gate | Status | Details |
|---|---|---|
| Build | PASS | `cargo build --workspace --locked` 0. Check Linux (`-p ddc-core -p ddc-adapters -p ddc-cli --target x86_64-unknown-linux-gnu`) 0. Check Windows (`-p ddc-cli -p ddc-adapters --target x86_64-pc-windows-msvc`) 0, e também com `--tests` (compila `a_reply_off_linux_carries_no_echo`). Único aviso: future-incompat de `nom` 3.2.1, transitivo e anterior à phase. |
| Tests | PASS | 247 passed, 0 failed, 7 ignored (hardware, `crates/ddc-adapters/tests/real_monitor.rs`). Na iter 1 eram 243: entraram 5 (`auto_setup_and_osd_control_accept_only_their_catalog_values`, `auto_setup_and_osd_lock_take_only_their_catalog_values_and_only_with_yes`, `a_reply_that_echoes_another_vcp_code_is_retried_until_the_reply_to_the_code_asked_arrives`, `a_reply_for_another_code_is_a_transient_failure` e `a_linux_reply_carries_the_code_the_monitor_echoed`) e saiu 1 (`a_reply_that_answers_another_vcp_code_is_a_transient_failure`, cujo helper foi movido para o worker e hoje é coberto pelos novos). Não houve queda sem substituto. |
| Coverage | PASS | 96.10% lines (TOTAL row, main.rs/build.rs excluded), threshold 80%. `crates/ddc-cli/src/main.rs` não mudou na phase. `hardware.rs` 69.35% (era 65.62%), `ddc_hi_backend.rs` 60.61% e `mccs_catalog.rs` 76.92%: cola de hardware e tabela avaliada em `const`. |
| Lint | PASS | `cargo fmt --all --check` 0. `cargo clippy --workspace --all-targets --locked -- -D warnings` 0, e também no alvo msvc (`-p ddc-adapters -p ddc-cli`) 0. Nenhum `#[allow]` fora de teste; os 2 em `worker/tests.rs:221` e `:228` têm `// reason:`. |
| Hexagonal/Safety/Hygiene | PASS | 5.1–5.11 limpos (detalhe abaixo). |
| Consistency | PASS | D-1, D-2 e D-2026-09-26-full-osd-control-1..10 conformes. O código da iter 2 segue a D-10 ponto a ponto (ver abaixo). Commits com scope `full-osd-control`, tipo adequado e D-XX citada no corpo. |
| UI Validation | SKIPPED | has_frontend=false |
| DoD | PASS_PENDING_MANUAL | 15/15 auto, 2 manual pending |

### Gate 5 em detalhe
- **5.1:** `ddc-core` `[dependencies]` só tem `thiserror`. Nenhum `Cargo.toml` ou `Cargo.lock` mudou na phase.
- **5.2 / 5.8:** nenhuma saída. Os valores novos do catálogo (`AUTO_SETUP`, `OSD_CONTROL` em `mccs_catalog.rs:107` e `:111`) são dado estático puro.
- **5.3:**
  - nenhum `impl MonitorBackend` no core;
  - nenhum `pub trait` fora dele. Os `pub(crate) trait` `Clock`, `DisplaySource` e `DdcHandle` são seams internas do adapter, e é aí que a D-10 manda pôr o eco;
  - `VcpReply` é `pub(crate)` em `worker.rs:58`: não cruza para o core e não muda o port.
- **5.4:** nenhuma saída.
- **5.5:**
  - nenhum `unsafe`;
  - `#![forbid(unsafe_code)]` presente em `ddc-core/src/lib.rs` e `ddc-cli/src/main.rs`;
  - `ddc-adapters/src/lib.rs` não tem o atributo, mas herda `[workspace.lints.rust] unsafe_code = "deny"` (`Cargo.toml:15-16`, `[lints] workspace = true`), a alternativa aceita.
- **5.6:** o único hit é um doc comment (`worker.rs:141`).
- **5.7:**
  - `Dangerous` é classificado no catálogo (16 ocorrências em `mccs_catalog.rs`). 0x1E e 0xCA continuam `Dangerous`, fixados em `mccs_catalog/tests.rs:38` e `:56`;
  - `Confirm::Yes` só aparece em `crates/ddc-cli/src/run.rs:166`, a partir de `--yes`;
  - `DdcHiMonitorBackend` só aparece nos 7 testes de `real_monitor.rs`, todos `#[ignore]`, gated por `DDC_HW_TESTS=1` e sem `write_vcp`. O arquivo não mudou na iter 2.
- **5.9:** n/a (`apps/` não existe).
- **5.10:** `cargo audit` só mostra as conhecidas RUSTSEC-2018-0005 (`serde_yaml` 0.7.5) e RUSTSEC-2024-0320 (`yaml-rust` 0.4.5). Nenhuma dependência nova.
- **5.11:** nenhuma saída.
- **Skills (dry/kiss/yagni/clean-code):**
  - `VcpReply` + `answering` é o mínimo para a seam da D-10: um struct, um método, um único ponto de chamada;
  - `VALUE_LISTS` no teste repete as listas do catálogo de propósito, porque é o oráculo que as fixa.

### Pendências da iter 1: status

- **DoD critic, linha 7 (oco, OBJETIVO): RESOLVIDO** (`a2f5fda`, ampliado em `f3dc39d`).
  - O teste nomeado agora compara `catalog_entry(code).unwrap().values` por igualdade com a lista inteira (bytes e nomes) de 10 códigos: 0x04/05/06/08, 0x14, 0x1E, 0x60, 0xCA, 0xCC e 0xD6 (`mccs_catalog/tests.rs:115-166` e `:175`).
  - Também fixa QUAIS códigos do catálogo têm lista (`named == VALUE_LISTS.map(..)`, `:180-185`), então uma lista nova em outro código derruba o teste.
  - **Mutação reproduzida pelo reviewer** (cópia via `git archive` no scratchpad, target separado, repositório intocado): acrescentar `(0x0E, "Italian")` a `OSD_LANGUAGES` dá `FAILED`, com `assertion left == right failed: 0xCC` em `tests.rs:175`. É exatamente a mutação que o critic usou para provar que o teste era oco.
- **W-1 (0xCA/0x1E documentados como graváveis, mas sempre recusados): RESOLVIDO** (`f3dc39d`, `6df4c07`), no caminho que a D-10(a) escolheu: tornar os códigos graváveis, em vez de só corrigir o texto.
  - **Catálogo** (`mccs_catalog.rs:130` e `:148`):
    - 0x1E recebe `.with_values(AUTO_SETUP)` = `00 Off, 01 Run, 02 Continuous`;
    - 0xCA recebe `.with_values(OSD_CONTROL)` = `01 OSD disabled, 02 OSD enabled`;
    - os dois continuam `NC, RW, Dangerous`.
  - **Byte de botões (SH) sempre zero:** `Feature::lists` (`feature.rs:113`) faz `u8::try_from(value)`, então `0x0102` sai como `ValueNotAllowed`. Isso está provado no unitário `auto_setup_and_osd_control_accept_only_their_catalog_values` e na CLI (`set osd-lock 0x0102 --yes`, exit 4, sem `WriteVcp`).
  - **Sem `--yes`:** exit 5, e o log fica só com `Enumerate` (in-process e pelo binário, `crates/ddc-cli/tests/cli/writes.rs:191-260`).
  - **DoD #8:** o caso "sem lista em lugar nenhum" passou de 0x1E para 0x02, e entraram 0x1E (caps ilegível, aceita 0x01 e recusa 0x03) e 0xCA (declarado sem lista, aceita 0x02 e recusa 0x00).
  - **Documentação:**
    - README tem as linhas do catálogo `:216` e `:234` com os valores, o parágrafo `:243` e as Known limitations `:252-254`: só 0x02 é recusado, 0xCA cobre só o byte OSD, e 0x1E/0xCA nunca foram escritos em hardware;
    - CHANGELOG tem `:44` (só `0x02`) e a entrada nova `:45`;
    - `grep` em README, CHANGELOG e `docs/` não encontra nenhum código documentado como gravável sem estar, nem o contrário.
  - **UX:** a armadilha anotada na iter 1 (pedir `--yes` para depois recusar) não existe mais para 0x1E e 0xCA. Ela continua só para 0x02, e o README (`:203`, `:252`) diz que 0x02 é recusado.
- **W-2 (a checagem de eco só era testada num helper): RESOLVIDO** (`2def274`), conforme a D-10(b).
  - `DdcHandle::read_vcp` devolve `VcpReply { value, echoed: Option<VcpCode> }` (`worker.rs:51`, `:58-82`).
  - A casca só traduz (`hardware.rs:36-40`, `vcp_reply` em `:81-86`): `echoed_code` devolve `Some` no Linux (`:92-95`) e `None` fora dele (`:97-100`).
  - O worker valida em `Worker::read_vcp` (`worker.rs:274`: `handle.read_vcp(code)?.answering(code)`).
  - **Ponto único:** um `grep` mostra que essa é a ÚNICA chamada de `DdcHandle::read_vcp` fora de testes. Leitura normal, max aprendido, read-back e probe passam todos por ela.
  - **Teste nomeado** `a_reply_that_echoes_another_vcp_code_is_retried_until_the_reply_to_the_code_asked_arrives`: o fake deixa respostas "no barramento" e o teste cobre três casos.
    - 1 resposta de 0x70: 2 tentativas, 1×200 ms, `Ok(50/100)`;
    - 3 respostas: `Transport("reply answers VCP code 0x70, not 0x7E (gave up after attempt 3 of 3)")`;
    - sem eco: aceita na 1ª tentativa.
  - **Mutação reproduzida pelo reviewer** (mesma cópia descartável): trocar a linha 274 por `handle.read_vcp(code).map(|reply| reply.value)` dá `FAILED` (left `Ok(VcpValue { current: 80, max: 100 })`, right `Ok(… 50 …)`).
  - **Resíduo aceitável:** o corpo de `Handle::read_vcp` (`self.get_vcp_feature(code.0).map(vcp_reply)`) continua exercitado só pelo hardware. As duas peças dele (`vcp_reply` com o eco Linux e a validação no worker) são testadas sem hardware, que é o que a D-10 pede.
- **WARN de plano da iter 1 (`129febd` fora de `files_modified`, sem teste): RESOLVIDO.** A D-10(b) travou a seam em `hardware.rs`/`worker.rs`, e o teste existe.

### Gate 6: conformidade com a D-2026-09-26-full-osd-control-10
- **(a) Valores:**
  - os bytes e nomes de 0x1E e de 0xCA batem literalmente com a D-10;
  - só o byte SL de 0xCA é aceito, e o SH é sempre 0;
  - os dois continuam `Dangerous`.
- **(a) Docs:** README e CHANGELOG dizem exatamente o que é gravável.
- **(a) Hardware:** o roteiro proíbe os dois "with any value" (`docs/hardware-validation.md:130-131`, `:136`).
- **(b) Seam:** a seam devolve o código ecoado, o worker genérico valida, o teste nomeado usa fake e prova falha transitória + retry, e a mutação derruba o teste. No Windows vem `None` e a validação é pulada. Conforme.
- **D-1 / D-2:** o eco fica no adapter, o port não muda, e o core só ganha dado estático.
- **D-8:** os resets continuam `(0x01, "Reset")`, agora fixados por igualdade.
- **D-9:** o backoff de 200 ms não mudou. O retry do eco usa a mesma política VCP.
- **CONTEXT "Out of scope":** a linha 32 ("0x1E fica `UnsupportedFeature`") foi superada pela D-10, que é posterior. Não é violação.

### Roteiro de hardware (`docs/hardware-validation.md`): conferido
- **Escritas do roteiro:** só `set brightness`, `set contrast`, `set preset 6500k`/`srgb` e `set volume`, cada uma desfeita em seguida (§3). As recusas de §4 não escrevem nada:
  - `set preset 3` → exit 4, porque 0x03 não está na lista do caps nem na do catálogo;
  - `set brightness srgb` → exit 2, antes de qualquer I/O.
- **Nenhum `--yes` no roteiro inteiro.**
- **"Never run":**
  - `input` 0x60 e `power` 0xD6;
  - os 4 resets, por nome e por número;
  - 0xCA e 0x1E com qualquer valor;
  - 0x20, 0x30 e 0x7E;
  - 0xE6 e 0xF1;
  - qualquer código fora do catálogo ou escrito por número.
- **Testes de hardware (§1):** os 7 de `real_monitor.rs` são só de leitura.

## Blockers (if any)
- (nenhum)

## Warnings (if any)
- (nenhum)

### Notas (sem ação obrigatória)
- **CHANGELOG `:45`, nit:**
  - "Before, `0xCA` took any value up to the maximum the monitor reported" também valia para 0x1E (pela D-2026-09-26-cli-1, por número com `--yes`). A frase não é falsa, só incompleta.
  - Vale ajustar na confirmação manual do item 4.
- **Semântica de 0xCA no RTK não confirmada:**
  - o RTK lê 0x01 ("OSD disabled", que é também o nome do `ddcutil`), e o doer não conseguiu confirmar só lendo se o OSD dele está mesmo desabilitado;
  - o rótulo segue a D-10 e o MCCS, e o README registra isso de forma neutra (`:253`);
  - como a escrita é proibida no roteiro e `Dangerous` exige `--yes`, fica para o PR.
- **Texto da CONTEXT defasado:** o DoD #1 diz "33 códigos", o DoD #2 diz "`00 Reset`", e o Out of scope diz que 0x1E é recusado.
  - Os testes seguem a D-8 e a D-10, então não são ocos.
  - O reviewer não edita o DoD.
- **`.jdi/DECISIONS.md` defasado:** é um rollup gitignored (`.gitignore:31`) e para na D-8. A D-9 e a D-10 existem só em `.jdi/decisions/`. Convém regenerá-lo antes da próxima phase, porque os reviewers leem o rollup.
- **Teste Windows só compilado:** `a_reply_off_linux_carries_no_echo` é compilado (`cargo check --tests` e clippy msvc, 0), mas nunca executado. Não há runner Windows até a `ci-crossbuild`.
- **DRY menor da iter 1, ainda presente:** `RESET_VALUE_NAME = "Reset"` (`crates/ddc-cli/src/args.rs:11`) repete `mccs_catalog.rs:68`. Os testes de `reset` protegem contra divergência.
- **Adiados para o PR:**
  - as escritas SAFE de §3 do roteiro (pendentes desde a `cli`);
  - a semântica de 0xE6/0xF1/0xCA;
  - 0xAE, que agora lê 144.00 Hz a 2560x1600@144 e leu 448.18 Hz uma vez; o README (`:259`) e o roteiro (`:66`) já não afirmam um número;
  - R-2/W-6 (Windows `unresponsive`);
  - R-3 (mensagem do panic no stderr e off-by-one do `ddc-i2c` 0.2.2);
  - W-8 da `cli`;
  - os 2 itens manuais pendentes da `cli`.

## DoD Checklist (gate 8)

| # | Criterion | Source | Type | Status | Evidence |
|---|---|---|---|---|---|
| 1 | `cargo test --workspace` exits 0 | PROJECT | Auto | PASS | exit 0: 247 passed, 0 failed, 7 ignored (Gate 2) |
| 2 | Coverage >= 80% of lines | PROJECT | Auto | PASS | TOTAL 96.10% lines (Gate 3, main.rs/build.rs excluded) |
| 3 | No `TODO`/`FIXME` without linked issue reference | PROJECT | Auto | PASS | Verify OK: grep vazio em `src/ crates/ apps/` |
| 4 | CHANGELOG.md updated with entry per release | PROJECT | Manual | MANUAL_REQUIRED | suggested: só `## [Unreleased]` (`:8`), sem heading versionado `## [x.y.z]`. Changed `:44` (recusa só `0x02`) e `:45` (0x1E/0xCA com valores MCCS, `--yes`, byte de botões zero); Fixed `:54` (resposta de outro código). Nit: `:45` omite que 0x1E também aceitava qualquer valor ≤ max antes |
| 5 | README accurately describes current behavior | PROJECT | Manual | MANUAL_REQUIRED | suggested: status fase 4 (`:5`); exemplos com `1/2 OSD disabled` e `0/1 Off` (`:115`, `:141`, `:159`); catálogo 0x1E/0xCA com valores (`:216`, `:234`); regra das listas (`:243`); Known limitations 0x02/0xCA/0x1E (`:252-254`) e 0xAE dependente do modo (`:259`); "Replies for another code" com a validação no worker |
| 6 | Catálogo cobre os códigos observados com kind/access/risk travados + invariante RO≠Dangerous + novos Dangerous | CONTEXT | Auto | PASS | OK: `test result: ok. 1 passed; 0 failed` (39 códigos, D-8d; 0x1E/0xCA seguem `Dangerous`) |
| 7 | `value_name` para 0x14/0x60/0xCC/0xD6 e os 4 resets | CONTEXT | Auto | PASS | OK: agora fixa as 10 listas por igualdade e quais códigos têm lista; mutação `(0x0E,"Italian")` reproduzida pelo reviewer derruba o teste |
| 8 | `validate_write` rejeita `Table` antes de access/lista/max | CONTEXT | Auto | PASS | OK |
| 9 | `Capabilities::feature` usa kind/access do catálogo para código fora do caps | CONTEXT | Auto | PASS | OK |
| 10 | `hertz`/`version_from_u16` convertem 14400, 0x0202 e 0x0001 | CONTEXT | Auto | PASS | OK |
| 11 | `probe_undeclared_features` só lê catálogo − caps, distingue unsupported/unresponsive, não aborta | CONTEXT | Auto | PASS | OK |
| 12 | `set_feature` aprende o max com 1 leitura para `Continuous` não declarado | CONTEXT | Auto | PASS | OK |
| 13 | `set_feature` nunca lê max de NC sem lista; valida contra o catálogo ou recusa | CONTEXT | Auto | PASS | OK: inclui 0x1E (catálogo), 0xCA (declarado sem lista) e 0x02 (sem lista, recusado) |
| 14 | `set_feature` recusa RO/`Table` antes de ler o backend | CONTEXT | Auto | PASS | OK |
| 15 | `ddc-cli features` (padrão / `--probe`) | CONTEXT | Auto | PASS | OK |
| 16 | `ddc-cli set <nome> <valor-por-nome>` resolve ou sai com 4 | CONTEXT | Auto | PASS | OK |
| 17 | `ddc-cli reset <target>`: exit 5 sem `--yes`, aplicado com `--yes` | CONTEXT | Auto | PASS | OK; os extras da D-8 também estão verdes: `reset_writes_one_and_skips_read_back` e `reset_factory_with_yes_through_the_binary_sends_one_and_says_it_is_write_only` |

**Totals:** 17 items | Auto: 15 (15 PASS, 0 FAIL) | Manual: 2 pending

**Manual confirmation required** (only if any MANUAL_REQUIRED item exists):
Run `/jdi-confirm-dod full-osd-control` to confirm each manual item with evidence. Without that, `/jdi-ship` will refuse the phase.

## Recommendation
Os três pontos que bloquearam a iter 1 foram resolvidos no código e nos testes, com as mutações reproduzidas de forma independente:
- **Linha 7 do critic:** as listas do catálogo estão fixadas por igualdade.
- **W-1:** 0x1E e 0xCA são graváveis só com os valores da D-10, só com `--yes`, e com o byte de botões zero.
- **W-2:** o eco é validado no worker, no único ponto de leitura, e testado com fake.

A segurança de escrita segue a da iter 1:
- `Dangerous` sem `--yes` não toca o barramento;
- RO/Table são recusados antes de qualquer leitura;
- NC só aceita valores de lista;
- o roteiro de hardware nunca escreve input, power, resets, 0x1E, 0xCA, geometria, 0x7E ou códigos de fabricante.

Próximos passos:
1. `/jdi-confirm-dod full-osd-control` para os 2 itens manuais. De passagem, ajustar a nota do CHANGELOG `:45` sobre 0x1E, que é opcional.
2. Regenerar `.jdi/DECISIONS.md` com a D-9 e a D-10.
3. No PR, listar os adiados das Notas e confirmar os 2 itens manuais pendentes da `cli`.

## DoD Critic (enhanced)

Forçado pelo `/jdi-issue` na iteração 2: 0 linhas hollow (critic retornou `[]`). A linha 7 deixou de ser hollow — as listas de valores do catálogo estão fixadas por igualdade.

**Verdict:** APPROVED
