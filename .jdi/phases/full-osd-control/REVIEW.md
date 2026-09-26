# Phase 4: Review  (slug: full-osd-control)

**Verdict:** APPROVED_PENDING_MANUAL

## Gates
| Gate | Status | Details |
|---|---|---|
| Build | PASS | `cargo build --workspace --locked` 0; check Linux (`-p ddc-core -p ddc-adapters -p ddc-cli --target x86_64-unknown-linux-gnu`) 0; check Windows (`-p ddc-cli -p ddc-adapters --target x86_64-pc-windows-msvc`) 0. Só o aviso future-incompat de `nom` 3.2.1 (transitivo, anterior à phase). |
| Tests | PASS | 243 passed, 0 failed, 7 ignored (hardware, `crates/ddc-adapters/tests/real_monitor.rs`). Na `cli` eram 194 passed e 6 ignored: entraram 49 testes e 1 `#[ignore]` (`hardware_reading_0x7e_never_stops_the_worker`). Nenhum removido; os que mudaram de premissa foram reescritos (0x52→0x8D, preset sem caps = NC, backoff 50→200 ms). |
| Coverage | PASS | 95.92% lines (TOTAL row, main.rs/build.rs excluded), threshold 80%. `crates/ddc-cli/src/main.rs` intocado na phase. Por arquivo: `mccs_catalog.rs` 76.92% (linhas `row`/`with_values` avaliadas em `const`), `hardware.rs` 65.62% e `ddc_hi_backend.rs` 60.61% (cola de hardware, coberta só pelos testes `#[ignore]`). |
| Lint | PASS | `cargo fmt --all --check` 0; `cargo clippy --workspace --all-targets --locked -- -D warnings` 0. Dois `#[allow(clippy::panic)]` novos, ambos em teste (`worker/tests.rs:212`, `:219`) e com `// reason:`. |
| Hexagonal/Safety/Hygiene | PASS | 5.1–5.11 limpos (detalhe abaixo). Segurança de escrita de ponta a ponta conferida no código e nos testes. |
| Consistency | WARN | D-1, D-2 e D-2026-09-26-full-osd-control-1..9 conformes (-8 sobre -1/-4, -9 sobre ddc-backends-3). WARN de plano: o `129febd` ficou fora de `files_modified`, e a ligação dele não tem teste sem hardware (W-2). |
| UI Validation | SKIPPED | has_frontend=false |
| DoD | PASS_PENDING_MANUAL | 15/15 auto, 2 manual pending |

### Gate 5 em detalhe
- **5.1:** `ddc-core` `[dependencies]` = só `thiserror`. Nenhum `Cargo.toml`/`Cargo.lock` mudou na phase.
- **5.2 / 5.8:** nenhuma saída. O catálogo é dado estático puro: sem I/O, sem `cfg` de plataforma, sem nome de dispositivo.
- **5.3:** nenhum `impl MonitorBackend` no core e nenhum `pub trait` fora dele. `probe_undeclared_features` entrou no port driving `MonitorControl` (D-2).
- **5.4:** nenhuma saída. `crates/ddc-cli/src/main.rs` continua sendo o único composition root, e não mudou.
- **5.5:** nenhum `unsafe`. Todas as raízes têm `#![forbid(unsafe_code)]`, e o workspace tem `unsafe_code = "deny"`.
- **5.6:** o único hit é um doc comment (`worker.rs:113`). O `catch_unwind` não traz nenhum `unwrap`/`expect`.
- **5.7:**
  - `Dangerous` é classificado pelo catálogo.
  - `Confirm::Yes` aparece só em `crates/ddc-cli/src/run.rs:166`, a partir de `--yes`.
  - Os 7 usos de `DdcHiMonitorBackend` estão em `real_monitor.rs`, todos `#[ignore]`, gated por `DDC_HW_TESTS=1` e só de leitura (o novo lê 0x7E e depois 0x10).
- **5.9:** n/a (`apps/` não existe).
- **5.10:** `cargo audit` só mostra RUSTSEC-2018-0005 (`serde_yaml` 0.7.5) e RUSTSEC-2024-0320 (`yaml-rust`), já conhecidos e inalcançáveis. Nenhuma dependência nova.
- **5.11:** nenhuma saída.
- **Guards da T-5:**
  - `! grep -rnE 'risk_for_code|authorize_write' crates/ddc-cli/src` → vazio;
  - nenhum `VcpCode(0x` em `args.rs`;
  - `SHORTCUTS`/`shortcut_name` removidos;
  - `Risk::` na CLI só aparece em `risk_label` (apresentação, A-7).

## Blockers (if any)
- (nenhum)

## Warnings (if any)

### W-1: README e CHANGELOG dizem que `osd-lock` (0xCA) é gravável com `--yes`, mas o core sempre recusa
- **Onde:** 0xCA é `NC, RW, Dangerous` e não tem valores no catálogo (`crates/ddc-core/src/domain/mccs_catalog.rs:141`, sem `.with_values`). O caps do RTK declara `CA` sem lista (`... C8 CA CC(...)`).
  - Resultado: `allowed_values = None`, `requires_known_max()` é falso para NC (`feature.rs:84`), `lists()` devolve `None` e `validate_write` sai com `known_max.ok_or(UnsupportedFeature)` (`feature.rs:103`).
  - O mesmo vale para qualquer monitor que não liste valores de 0xCA.
- **Reproduzido só com o fake:**
  - `ddc-cli --fake set osd-lock 1` → exit 5 ("repeat the command with --yes");
  - `ddc-cli --fake set osd-lock 1 --yes` → exit 4 ("feature 0xCA is not supported");
  - `ddc-cli --fake set auto-setup 1` → exit 5, e com `--yes` → exit 4.
- **O que está errado:**
  - `README.md:234` (tabela do catálogo) diz "dangerous, `--yes`".
  - `README.md:252` (Known limitations) cita só 0x02 e 0x1E como recusados.
  - O `CHANGELOG.md:44` (Changed) também cita só (`0x02`, `0x1E`). Ele omite que 0xCA passou de "gravável com `--yes` depois de ler o max" (D-2026-09-26-cli-1) para recusado.
- **Por que WARN e não BLOCK:**
  - o código segue a regra genérica da emenda W-5 de D-2026-09-26-full-osd-control-1, cuja lista de exemplos (0x02, 0x1E) é que esqueceu 0xCA;
  - o resultado é fail-safe (nada é escrito);
  - README e CHANGELOG são itens `Manual` do DoD, não gates.
- **Ressalva de UX (mesma raiz):** para um código `Dangerous` sem lista nenhuma (0x1E, e 0xCA no RTK), o `set` sem `--yes` manda "repetir com --yes" (exit 5), e com `--yes` recusa (exit 4).
  - É a armadilha que a invariante "RO nunca é Dangerous" da D-1 evita para os códigos RO.
  - Aqui a ordem `authorize_write` primeiro é a correta, porque segurança é a prioridade 1 e ela não toca o barramento. Só a documentação precisa refletir isso.
- **Ação:**
  - corrigir a linha 0xCA da tabela ("refused: no value list"), o item de Known limitations e o Changed do CHANGELOG;
  - opcional: uma D-XX com os valores de 0xCA, mas só com evidência de hardware. O RTK lê `1/2`, e o significado MCCS desses bytes não foi confirmado nele.

### W-2: `129febd`, fix fora do plano: correto no mérito, mas a ligação não tem teste sem hardware
- **Mérito (conforme):**
  - `ddc` 0.2.2 guarda o `data[2]` da resposta (eco do opcode VCP) em `ty` e nunca compara (`commands.rs:107-110`, `// data[2] == vcp code from request`);
  - `ddc-winapi` 0.2.2 põe em `ty` o tipo `MC_SET_PARAMETER/MC_MOMENTARY` (`lib.rs:177-182`), então o `cfg(target_os = "linux")` de `echoed_code` (`hardware.rs:82-90`) é o recorte certo e o Windows não muda (check msvc 0);
  - recusar como transiente (retry) é o mesmo tratamento do `ddcutil` ("Unexpected VCP opcode … should be …", issue #398);
  - fica no adapter, o lugar certo (D-1). A recusa 0x01 continua decodificada antes pelo `ddc`, então D-2026-09-26-cli-4 fica intacta;
  - protege o max aprendido e o read-back, que é prioridade 2.
- **Lacuna:**
  - o único teste novo (`hardware/tests.rs`, `a_reply_that_answers_another_vcp_code_is_a_transient_failure`) cobre só o helper puro `ensure_reply_answers`;
  - a chamada em `Handle::read_vcp` (`hardware.rs:38`) e o `echoed_code` Linux (`:83`) só são exercitados pelo hardware. Apagar a linha 38 deixa `cargo test` verde.
  - A evidência de hardware do SUMMARY (0x7E deixou de ler `80/100`) vale, mas não protege contra regressão.
- **Plano:** `hardware.rs` e `hardware/tests.rs` (e também `docs/linux-ddc-setup.md`, da D-9) ficaram fora de `files_modified`. O SUMMARY sinaliza isso, e o commit tem o scope certo e cita D-XX.
- **Ação sugerida:** juntar eco + `vcp_value` numa função pura (ex.: `checked_value(code, reply: ddc_hi::VcpValue)`) e testá-la sob `#[cfg(target_os = "linux")]` com o helper `reply(..)` que os testes já têm, montando um `ty` diferente do pedido.

### Pontos julgados a pedido do orquestrador
- **Catálogo** (`mccs_catalog.rs`):
  - **Fonte única:** uma tabela `CATALOG: [CatalogEntry; 39]`, e dela derivam:
    - `catalog_codes()` (const-eval `CODES`, sem 2ª lista);
    - `risk_for_code` (`map_or(Dangerous)`);
    - `Capabilities::feature` (kind/access do catálogo, fallback só fora dele);
    - `value_name`/`value_for_name`, `code_for_alias` e `interpret`.
    - Os 6 atalhos antigos viraram `alias` da mesma tabela, e `SHORTCUTS` sumiu.
  - **Contagem:** 39 = 28 do caps fixture + 9 sondados + 0xE6/0xF1, conforme D-8(d)/A-1.
  - **Fail-safe:** fora do catálogo → `Dangerous`; 0xE6/0xF1 → `C, RW, Dangerous`, sem valores (D-7).
    - 0xFD/0xFF são `RO, Safe` por decisão explícita da D-1 (invariante RO) e nunca graváveis: o `ensure_writable` recusa antes de qualquer leitura.
    - Varredura 0..=255 em `every_code_is_safe_only_when_catalogued_and_the_manufacturer_range_is_dangerous_or_read_only`, e invariante "RO nunca é Dangerous" no DoD #1.
  - **Nomes de valor vs MCCS 2.2:** subconjuntos corretos do MCCS.
    - 0x14: 01 sRGB, 02 Display Native, 04/05/06/08 = 5000/6500/7500/9300 K, 0B User 1;
    - 0x60: 01 VGA-1, 03/04 DVI-1/2, 0F/10 DisplayPort-1/2, 11/12 HDMI-1/2;
    - 0xCC: 01 Chinese (traditional), 02 English, 03 French, 04 German, 06 Japanese, 0A Spanish, 0D Chinese (simplified);
    - 0xD6: 01 On, 04 Off, 05 Off (write-only);
    - resets: `(0x01, "Reset")` (D-8a).
  - **Normalização:** únicos por código (teste dedicado).
  - **Interpretação:** 0xAE /100 Hz, 0xC9/0xDF versão, 0xAC bruto (D-8c).
- **Segurança de escrita, ponta a ponta:**
  - `set_feature` (`software_osd.rs`) segue a ordem `authorize_write` → caps → `ensure_writable` → `max_for_write` → `validate_write` → write → read-back.
  - **`Dangerous` sem `--yes` nunca chega ao backend, inclusive no `reset`:**
    - no core, `dangerous_write_is_rejected_before_capabilities_are_fetched` exige log vazio;
    - na CLI, `reset` sem `--yes` deixa só `Enumerate` no log (DoD #12).
  - **RO/Table recusados antes de qualquer leitura VCP:** o log fica só com `ReadCapabilities` (DoD #9), exatamente como pede o item 3 da emenda W-5 ("após resolver a Feature").
  - **Max por leitura só para `Continuous` sem lista e legível** (`requires_known_max && is_readable`, `software_osd.rs:99`).
  - **NC validado contra a lista do caps, senão a do catálogo, senão recusado** (DoD #8, que também cobre o reset aceitando só 0x01).
  - **`WriteOnly` sem leitura de max e sem read-back:** `reset_writes_one_and_skips_read_back`, e a CLI imprime "sent … (write-only, not read back)".
  - **Provas de mutação:** as 4 do SUMMARY batem com o que os testes afirmam; o reviewer não as reproduziu.
- **Probe:**
  - **Só leitura:** nenhum `WriteVcp` no DoD #6 nem no DoD #10.
  - **Uma falha não aborta a sondagem:** só `MonitorNotFound` encerra, por curto-circuito do `collect::<Result<Vec<_>,_>>`.
  - **`unsupported` e `unresponsive` são distinguidos** por `ProbeStatus::of`.
  - **Isolamento de panic da D-6 provado em duas camadas:**
    - adapter: `a_panic_inside_one_transaction_fails_only_that_call_and_the_worker_keeps_serving` (depois do panic, read, write e enumerate respondem no mesmo client) e `a_panic_is_never_retried_nor_followed_by_a_presence_check`;
    - core/CLI: o `Transport("ddc-hi panicked: …")` roteirizado não interrompe o probe.
  - Não há `panic = "abort"` em nenhum `[profile]`.

### Notas (sem ação obrigatória)
- **Carry-overs:**
  - W-5 da `cli`: resolvido (emenda W-5 + DoD #7/#8/#9).
  - W-7: resolvido pela D-6. Continuam em aberto a mensagem do panic no stderr (R-3) e o off-by-one upstream do `ddc-i2c` 0.2.2 (reportar ou `[patch]`).
  - W-6 (Windows mostra `unresponsive` onde o Linux mostra `unsupported`): continua documentado (R-2). Com a D-9, cada código recusado custa agora ~2×200 ms no Windows.
  - W-8 (commits `chore(repo)` da branch `cli`): fora deste range, mas ainda vale para o PR.
- **Texto do DoD da CONTEXT defasado da D-8:** o DoD #1 diz "33 códigos" e o DoD #2 diz "`00 Reset`". Os testes seguem a D-8 (39 códigos, `0x01`), então não são ocos. O reviewer não edita o DoD.
- **DRY menor:** `RESET_VALUE_NAME = "Reset"` (`crates/ddc-cli/src/args.rs:11`) repete o nome do catálogo (`mccs_catalog.rs:68`). Os testes de `reset` protegem contra divergência, mas daria para derivar de `catalog_entry(code).values`.
  - `is_write_only` (`output.rs:252`) lê o mesmo catálogo que o core usa. É consistente, só registro.
- **Otimização opcional:** para código catalogado, `ensure_writable` poderia rodar antes do caps (kind/access não dependem dele), poupando a leitura de caps num `set` recusado com o cache frio.
- **`features` lê os resets WO declarados** (0x04..0x08 → `0/1` no RTK). É só Get VCP, e a D-8(b) restringe só o `set_feature`.
- **Nit no `README.md:47`:** "exits 5 before anything is read from … the monitor". A enumeração (EDID) ainda roda; o que não roda é nenhuma transação VCP.
- **Adiado para o PR (fora do gate):**
  - o roteiro `docs/hardware-validation.md` §3 (4 escritas SAFE reversíveis, pendentes desde a `cli`) ainda não foi executado;
  - `get v-frequency` lê 448.18 Hz, e não os 144.00 Hz esperados pela CONTEXT. É firmware, registrado na D-9, e o `ddcutil` lê os mesmos bytes.

## DoD Checklist (gate 8)

| # | Criterion | Source | Type | Status | Evidence |
|---|---|---|---|---|---|
| 1 | `cargo test --workspace` exits 0 | PROJECT | Auto | PASS | exit 0: 243 passed, 0 failed, 7 ignored (Gate 2) |
| 2 | Coverage >= 80% of lines | PROJECT | Auto | PASS | TOTAL 95.92% lines (Gate 3, main.rs/build.rs excluded) |
| 3 | No `TODO`/`FIXME` without linked issue reference | PROJECT | Auto | PASS | grep vazio em `src/ crates/ apps/` |
| 4 | CHANGELOG.md updated with entry per release | PROJECT | Manual | MANUAL_REQUIRED | suggested: `## [Unreleased]` com a phase em Added (catálogo, probe, `features`, nomes, significado em get/set, `reset`), Changed (risco do catálogo, NC sem lista, ordem do `set_feature`, rótulos, backoff 200 ms) e Fixed (panic do worker, resposta de outro código). Não há heading versionado `## [x.y.z]`, e o Changed não cita 0xCA (W-1) |
| 5 | README accurately describes current behavior | PROJECT | Manual | MANUAL_REQUIRED | suggested: status da fase 4 (`:5`), Usage/`features`/`reset` (`:17-47`), exemplos reais só de leitura e sem `--fake` (`:49-164`), exit 2 para nome de outra feature (`:176`), tabela de 39 linhas (`:197-241`), Known limitations (`:245-258`), Monitor-write safety (`:283-294`). Conferir a linha 0xCA (`:234`) e a lista de recusados (`:252`), ver W-1 |
| 6 | Catálogo cobre os códigos observados com kind/access/risk travados + invariante RO≠Dangerous + novos Dangerous | CONTEXT | Auto | PASS | OK — `test result: ok. 1 passed; 0 failed` (o teste fixa 39 = observados, D-8d) |
| 7 | `value_name` para 0x14/0x60/0xCC/0xD6 e os 4 resets | CONTEXT | Auto | PASS | OK — o teste fixa `(0x01, "Reset")` (D-8a) |
| 8 | `validate_write` rejeita `Table` antes de access/lista/max | CONTEXT | Auto | PASS | OK |
| 9 | `Capabilities::feature` usa kind/access do catálogo para código fora do caps | CONTEXT | Auto | PASS | OK |
| 10 | `hertz`/`version_from_u16` convertem 14400, 0x0202 e 0x0001 | CONTEXT | Auto | PASS | OK |
| 11 | `probe_undeclared_features` só lê catálogo − caps, distingue unsupported/unresponsive, não aborta | CONTEXT | Auto | PASS | OK — 11 códigos em ordem, 1 leitura cada, 0 `WriteVcp` |
| 12 | `set_feature` aprende o max com 1 leitura para `Continuous` não declarado | CONTEXT | Auto | PASS | OK |
| 13 | `set_feature` nunca lê max de NC sem lista; valida contra o catálogo ou recusa | CONTEXT | Auto | PASS | OK — 8 casos, nenhum `ReadVcp` antes do write ou do erro |
| 14 | `set_feature` recusa RO/`Table` antes de ler o backend | CONTEXT | Auto | PASS | OK — log só com `ReadCapabilities`, com `No` e com `Yes` |
| 15 | `ddc-cli features` (padrão / `--probe`) | CONTEXT | Auto | PASS | OK — pelo binário e in-process, JSON campo a campo |
| 16 | `ddc-cli set <nome> <valor-por-nome>` resolve ou sai com 4 | CONTEXT | Auto | PASS | OK — também exit 2 para nome de outra feature, sem I/O |
| 17 | `ddc-cli reset <target>`: exit 5 sem `--yes`, aplicado com `--yes` | CONTEXT | Auto | PASS | OK — escreve `0x01`; extras da D-8 `reset_writes_one_and_skips_read_back` e `reset_factory_with_yes_through_the_binary_…` também verdes |

**Totals:** 17 items | Auto: 15 (15 PASS, 0 FAIL) | Manual: 2 pending

**Manual confirmation required** (only if any MANUAL_REQUIRED item exists):
Run `/jdi-confirm-dod full-osd-control` to confirm each manual item with evidence. Without that, `/jdi-ship` will refuse the phase.

## Recommendation
A phase fecha o card "controlar tudo do monitor" dentro do que o DDC/CI do RTK expõe. Os pontos de segurança da escrita estão conferidos no código e nos testes, sem bloqueio:
- risco vindo de uma fonte única, fail-safe;
- `--yes` antes de qualquer I/O;
- RO/Table recusados antes de qualquer leitura VCP;
- max por leitura só para `Continuous`;
- NC contra lista do caps ou do catálogo;
- WO sem read-back;
- probe só de leitura e à prova de panic.

Antes do `/jdi-confirm-dod full-osd-control`:
1. **W-1:** corrigir a linha 0xCA do README (`:234`), o item de recusados (`:252`) e o Changed do CHANGELOG (`:44`), dizendo que `osd-lock` é recusado por não ter lista de valores e que, sem `--yes`, o comando pede `--yes` antes de recusar. É só documentação; o código segue a regra travada.
2. **W-2 (opcional, recomendado):** tornar a checagem de eco do `129febd` testável sem hardware (função pura com o `ddc_hi::VcpValue` montado nos testes, sob `cfg(target_os = "linux")`).

No PR:
- listar os itens adiados: o roteiro de escritas SAFE (`docs/hardware-validation.md` §3), 0xAE = 448.18 Hz (firmware, D-9), a semântica de 0xE6/0xF1/0xCA, R-2/W-6 (Windows), R-3 (panic no stderr e off-by-one do `ddc-i2c`) e o W-8 da `cli`;
- confirmar os 2 itens manuais pendentes da `cli`.

## DoD Critic (enhanced)

Forçado pelo `/jdi-issue`.
- DoD row «7 — nomes de valor do catálogo para 0x14/0x60/0xCC/0xD6 e resets» (hollow, OBJETIVO): o teste `mccs_catalog_value_names_match_the_declared_lists_for_preset_input_language_power_mode_and_the_factory_reset_commands` (`crates/ddc-core/src/domain/mccs_catalog/tests.rs:109-166`) só verifica que cada byte declarado tem o nome certo; não fixa que as listas `COLOR_PRESETS`/`INPUT_SOURCES`/`OSD_LANGUAGES`/`POWER_MODES` (`mccs_catalog.rs:70-104`) contenham SÓ esses valores. Mutação (cópia via `git archive`): acrescentar `(0x0C,"User 2")`, `(0x1B,"USB-C")`, `(0x0E,"Italian")` mantém o Verify OK e o workspace verde; com caps ilegível, `set_feature(0xCC, 0x0E, Confirm::No)` gravou sem `--yes` (as listas do catálogo são a lista aceita quando o caps não traz lista — D-1 emenda W-5). Correção: fixar `catalog_entry(code).values` por igualdade para 0x14/0x60/0xCC/0xD6, como já é feito para os resets.

**Verdict:** BLOCKED
