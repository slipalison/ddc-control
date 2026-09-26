# Phase 4: Full OSD control — Plan  (slug: full-osd-control)

## Goal
Controlar tudo que o OSD oferece via DDC/CI. O core ganha o catálogo MCCS 2.2 (nome, C/NC/T, RO/WO/RW, risco, nomes de valor NC) e a descoberta por caps + sondagem read-only. A CLI ganha `features [--probe]`, `set` por nome de feature/valor para toda feature gravável e `reset` atrás de `--yes`. Fecha com validação no monitor real.

## Locked decisions (from CONTEXT.md)
- D-1/D-2: hexagonal; `ddc-core` só com `thiserror`; catálogo = dado estático no core. Nenhuma dependência nova (`Cargo.*` intocado).
- D-core-domain-2..5 (shapes, `get_feature` sempre lê); D-cli-1..5 e D-2026-09-26-cli-1..4 continuam valendo: cache, seleção, `--yes`, `--fake`, exit codes, max aprendido por leitura, recusa 0x01 → `UnsupportedFeature`.
- full-osd-control-1: tabela única `CatalogEntry`; `risk_for_code` derivado dela (fora do catálogo = `Dangerous`); RO nunca é `Dangerous`. Emenda W-5: max por leitura só para `Continuous`, lista do catálogo como fallback, guard RO/Table antes do max.
- -2: `probe_undeclared_features` no port, só leitura, com outcome por código. -3: `features` e `get`/`set` por nome. -4: `reset <target>` = `set_feature(code, 0)`. -5: `Table` nunca é gravável.
- -6: `catch_unwind` por transação no worker; panic → `Transport`, sem retry, worker vivo. Pré-requisito da sondagem.
- -7: 0xE6/0xF1 no catálogo (C, RW, `Dangerous`, sem nomes de valor) e na sondagem. 0xFD/0xFF são declarados, mas não respondem.

## Assumptions (cadeia autônoma, sem AskUserQuestion)
- **A-1 Catálogo = 39 códigos:** 28 do caps fixture + os 9 sondados de D-1 (0x1E 0x20 0x30 0x62 0x6C 0x6E 0x70 0x7E 0xC9) + 0xE6/0xF1 (D-7). O "33" da CONTEXT é erro de contagem; valem as tabelas de D-1/D-7, e o DoD #1 fixa a lista exata.
- **A-2 `CatalogEntry` ganha `alias: Option<&'static str>`** (palavra-chave da CLI, kebab-case, única). É assim que os 6 atalhos viram aliases do catálogo, sem 2ª tabela (`SHORTCUTS` some). Os 6 nomes atuais não mudam.
  - Novos: 0x02 `new-control-value`, 0x0B `color-temp-increment`, 0x0C `color-temp`, 0x16/0x18/0x1A `red-gain`/`green-gain`/`blue-gain`, 0x1E `auto-setup`, 0x20 `h-position`, 0x30 `v-position`, 0x6C/0x6E/0x70 `red-black-level`/`green-black-level`/`blue-black-level`, 0x7E `trapezoid`, 0x87 `sharpness`, 0xAC `h-frequency`, 0xAE `v-frequency`, 0xB2 `subpixel-layout`, 0xB6 `display-technology`, 0xC8 `controller-type`, 0xC9 `firmware-level`, 0xCA `osd-lock`, 0xCC `osd-language`, 0xDF `vcp-version`.
  - Sem alias: os 4 resets (D-4, só via `reset`), 0x52/0xC6/0xFD/0xFF (sem nome confiável) e 0xE6/0xF1.
- **A-3 Nome de valor:** `value_for_name(code, name)` ignora caixa e todo caractere não alfanumérico (`6500k` = `6500 K`, `displayport-1` = `DisplayPort-1`); um teste garante unicidade normalizada por código. Na leitura, o nome sai do byte SL de `current`, onde o MCCS põe o valor NC (o ddcutil faz igual).
- **A-4 `interpret(code, raw) -> Option<Interpretation { Named(&str), Hertz(f64), Version(u8, u8) }>`** fica no catálogo, então a CLI não carrega número mágico. `hertz` só vale para 0xAE. Pelo MCCS/ddcutil, 0xAC vem em Hz inteiros (24 bits, com o ML em `max`), e 0xAC/100 seria string errada, o que o rationale de D-1 proíbe. 0xAC fica raw. **Diverge da letra de D-1**; o orquestrador registra emenda se quiser travar.
- **A-5 `<VALUE>` = `ValueArg { Number(u16), Name(String) }`.** O clap aceita número ou nome que exista para ALGUM código do catálogo; o resto sai com exit 2 nativo (D-3). Em `run`, nome que não é valor do código pedido (`set brightness srgb`) → `CliError::Usage`, exit 2, listando os nomes do código. A CLI só traduz nome → byte; a interseção com a lista efetiva (caps, senão catálogo) é feita uma vez em `validate_write` (`ValueNotAllowed`, exit 4). Mesmo efeito de D-3, sem copiar a regra para a CLI.
- **A-6 `features`:** por padrão, lista todo código declarado no caps, do catálogo ou não (no RTK e na fixture = catálogo ∩ caps), cada um via `get_feature`; `--probe` acrescenta `probe_undeclared_features`. Metadados de cada linha = `caps.feature(code)`.
  - Toda linha leva `probe_status` (`ok|unsupported|unresponsive`), porque código declarado também pode não responder (0xFD/0xFF). `UnsupportedFeature` → `unsupported`; qualquer outro erro → `unresponsive`; `MonitorNotFound` em qualquer linha aborta com exit 3.
  - Caps ilegível (fora `MonitorNotFound`): aviso no stderr + caps vazio + exit 0, a mesma filosofia de `get`. Com caps vazio, `--probe` sonda o catálogo inteiro.
- **A-7 JSON:** `features` é um array com estes campos sempre presentes (`null` quando não há valor): `code, name (alias), description (nome MCCS), kind, access, risk, declared_in_capabilities, probe_status, current, max, value_name, interpreted`. `get`/`set` ganham `value_name`/`interpreted` com `skip_serializing_if` (JSON da `cli` intacto); `name` continua sendo o alias. Renderizar `feature.risk` é apresentação: `risk_for_code`/`authorize_write` seguem fora de `crates/ddc-cli/src` (D-cli-3).
- **A-8 Guard `Feature::ensure_writable()`** (`Table` → `UnsupportedFeature`, depois `ReadOnly` → `UnsupportedFeature`): 1ª linha de `validate_write`, chamado também por `set_feature` antes de `max_for_write`. Nenhum código do catálogo é `Table` (D-5), então o DoD #9 prova `Table` no mesmo guard com uma `Feature` montada à mão, e RO via `SoftwareOsd`.
- **A-9 `requires_known_max()` = `kind == Continuous && allowed_values.is_none()`** (regra 1 da emenda W-5). Código fora do catálogo mantém D-cli-1: `Continuous` pela heurística, max lido, já `Dangerous`.
- **A-10 Testes:** casos de uso com o fake ficam em `crates/ddc-core/tests/monitor_control/` (A-6 da `core-domain`); nomes do DoD únicos no workspace; `foo/tests.rs` conta como parte de `foo.rs`.
- **A-11 `reset`** imprime só o envio (D-4) e ignora o read-back de `set_feature`. A fixture `--fake` ganha valores para 0xCC e 0x04/0x05/0x06/0x08, porque o fake só aceita escrita em código com valor.

## Riscos
- **R-1 Read-back pós-reset:** `set_feature` sempre relê, e um código WO pode não responder logo após o reset → exit 4 ou 6 com o reset já aplicado. Não validável (`reset` nunca roda em hardware). Se acontecer, D-XX para pular o read-back em `WriteOnly`. Vai para Known limitations.
- **R-2 Windows (W-6):** o `dxva2` esconde a recusa 0x01, então `--probe` mostra `unresponsive` onde o Linux mostra `unsupported` e paga 3 tentativas por código. Só documentado; não há hardware Windows.
- **R-3 0x7E (W-7):** com D-6, o panic do `ddc-i2c` 0.2.2 vira `Transport` → `unresponsive`, como a CONTEXT prevê. A mensagem padrão do panic continua no stderr (D-6 não instala hook). Correção upstream ou `[patch]`: fora do escopo.

## Orchestrator amendment (pós-plano)
- D-2026-09-26-full-osd-control-8 trava: resets escrevem **0x01** (catálogo `values: [(0x01, "Reset")]` para 0x04/05/06/08 — T-2 e T-5); features `WriteOnly` não leem max e não fazem read-back (T-3: junto do bloqueio RO/Table; resolve R-1); 0xAE/100 Hz e 0xAC bruto (A-4 aceita); A-1 (39 códigos) e A-2 (`alias`) aceitas. Testes nomeados extras: `reset_writes_one_and_skips_read_back` (T-3, core) e o caso `reset factory --yes` via `--fake` (T-5).

## Tasks
Specialist único: `jdi-doer-ddc-control` (glob `**/*`). Scope `full-osd-control`, com D-XX citada no corpo. Nunca código + `.jdi/` no mesmo commit; nenhum commit fora do scope (W-8). Todo commit mantém verdes:
- `cargo fmt --all --check`, `cargo clippy --workspace --all-targets --locked -- -D warnings`, `cargo test --workspace --locked`;
- `cargo check -p ddc-core -p ddc-adapters -p ddc-cli --locked --target x86_64-unknown-linux-gnu`;
- `cargo check -p ddc-cli -p ddc-adapters --locked --target x86_64-pc-windows-msvc`.

Lições das reviews: teste cuja premissa mudou é reescrito, nunca apagado. As provas de mutação pedidas vão para o SUMMARY, sem commit da mutação.

### Wave 1 (parallel-eligible)

#### T-1: Isolar panic do `ddc-hi` por transação no worker (D-6)
- **Specialist:** jdi-doer-ddc-control
- **Files modified:** `crates/ddc-adapters/src/ddc_hi_backend.rs`, `crates/ddc-adapters/src/ddc_hi_backend/worker.rs`, `crates/ddc-adapters/src/ddc_hi_backend/worker/tests.rs`, `crates/ddc-adapters/src/ddc_hi_backend/retry.rs`, `crates/ddc-adapters/tests/real_monitor.rs`
- **Acceptance:**
  - Cada tentativa de `Worker::transact` (caps, read, write) roda em `catch_unwind(AssertUnwindSafe(..))`. O panic vira um `HandleError` de tipo próprio e final (sem retry, sem checagem de presença) → `Transport("ddc-hi panicked: <payload &str|String>")`. Tabela de monitores intacta, worker segue servindo; sem panic hook, sem `unwrap`/`expect`.
  - Novo `a_panic_inside_one_transaction_fails_only_that_call_and_the_worker_keeps_serving`, com um `Behaviour` que entra em panic só num código: a leitura desse código dá `Transport` com o texto do panic, após 1 tentativa; a leitura seguinte (outro código) e o `enumerate` dão `Ok` no mesmo client. `maps_backend_failures_…` é ajustado, e `worker_gone` continua coberto por um `enumerate` que entra em panic (fora de D-6).
  - Hardware `#[ignore]`, só leitura: `hardware_reading_0x7e_never_stops_the_worker` (0x7E → `Err(Transport)`, depois 0x10 → `Ok`). Mutação no SUMMARY: sem `catch_unwind`, o teste novo falha.
- **Dependencies:** none
- **Test:** `cargo test -p ddc-adapters --locked worker`
- **Status:** pending

#### T-2: Catálogo MCCS único + risco e `Capabilities::feature` derivados dele (D-1, D-7)
- **Specialist:** jdi-doer-ddc-control
- **Files modified:** `crates/ddc-core/src/domain/mod.rs`, `crates/ddc-core/src/domain/vcp.rs`, `crates/ddc-core/src/domain/mccs_catalog.rs`, `crates/ddc-core/src/domain/mccs_catalog/tests.rs`, `crates/ddc-core/src/domain/feature.rs`, `crates/ddc-core/src/domain/feature/tests.rs`, `crates/ddc-core/src/domain/capabilities.rs`, `crates/ddc-core/src/domain/capabilities/tests.rs`, `crates/ddc-core/tests/monitor_control/read_features.rs`, `crates/ddc-core/tests/monitor_control/write_features.rs`
- **Acceptance:**
  - `pub mod mccs_catalog`: tabela estática ordenada de `CatalogEntry { code, name, alias, kind, access, risk, values }` com os 39 códigos (A-1/A-2; `00 Reset` nos 4 resets). Funções: `catalog_entry` (busca binária); `catalog_codes()`, derivado da tabela por `const` (sem 2ª lista); `value_name`; `value_for_name` (A-3); `code_for_alias` (case-insensitive); `hertz`; `version_from_u16`; `interpret` (A-4). Consts novas em `VcpCode` só para códigos citados fora da tabela.
  - `risk_for_code` = `catalog_entry(code).map(|e| e.risk).unwrap_or(Risk::Dangerous)`. `Capabilities::feature` usa `kind`/`access` do catálogo, com o fallback atual fora dele; `allowed_values` continua vindo só do caps.
  - DoD #1, #2 e #5 em `mccs_catalog/tests.rs`; #4 em `capabilities/tests.rs`. Extras: tabela ordenada e sem repetição; aliases únicos e minúsculos, com os 6 antigos nas mesmas consts; valores únicos por código depois de normalizados; varredura de 0..=255 (`Safe` ⇒ catalogado; fora do catálogo ⇒ `Dangerous`; 0xE0..=0xFF `Dangerous`, exceto 0xFD/0xFF, que são RO e nunca graváveis).
  - Premissas que mudaram, com os testes reescritos: 0x52 deixa de ser o exemplo de "não classificado" (troca por 0x8D), e preset sem caps passa a `NonContinuous`.
- **Dependencies:** none
- **Test:** DoD #1, #2, #4 e #5 (Verify da CONTEXT) + `cargo test -p ddc-core --locked`
- **Status:** pending

### Wave 2

#### T-3: Emenda W-5 + D-5: guard RO/Table antes de ler, max só de contínuo, lista do catálogo como fallback
- **Specialist:** jdi-doer-ddc-control
- **Files modified:** `crates/ddc-core/src/domain/feature.rs`, `crates/ddc-core/src/domain/feature/tests.rs`, `crates/ddc-core/src/app/software_osd.rs`, `crates/ddc-core/src/ports/monitor_control.rs`, `crates/ddc-core/tests/monitor_control/support.rs`, `crates/ddc-core/tests/monitor_control/write_features.rs`
- **Acceptance:**
  - Segue A-8/A-9. `validate_write`: `ensure_writable` → lista (a do caps; senão os bytes de `catalog_entry(code).values`, quando houver) → `known_max.ok_or(UnsupportedFeature)` → `value <= max`. `set_feature`: `authorize_write` → caps → `ensure_writable` → `max_for_write` → `validate_write` → write → read-back. Docs do port e do struct atualizados.
  - DoD #3 (`feature/tests.rs`): uma feature `Table` com o valor na lista e max conhecido → `UnsupportedFeature`.
  - DoD #7: renomeia `safe_write_to_an_answered_but_undeclared_code_reads_its_max_first` para o nome do DoD e cobre também 0x6C; log `ReadCapabilities, ReadVcp, WriteVcp, ReadVcp`.
  - DoD #8, substituindo `without_capabilities_a_listed_feature_is_bounded_by_its_max`: com caps ilegível, preset 0x03 → `ValueNotAllowed`, preset 0x05 e `osd-language` 0x02 passam; 0x04 com `Confirm::Yes` aceita 0 e recusa 1 (`ValueNotAllowed`); 0x02 (declarado) e 0x1E (com `Yes`) → `UnsupportedFeature`. Em todos os casos, nenhum `ReadVcp` antes do `WriteVcp` ou do erro.
  - DoD #9: 0xAC e 0xDF (declarados) e 0xC9 (não declarado), com `No` e com `Yes` e com valor no fake → `UnsupportedFeature`, log só com `ReadCapabilities`; `Table` provado via `ensure_writable`. Mutações no SUMMARY: sem a restrição a `Continuous`, o #8 falha; sem o guard, o #9 falha.
- **Dependencies:** T-2
- **Test:** DoD #3, #7, #8 e #9 (Verify da CONTEXT)
- **Status:** pending

### Wave 3 (parallel-eligible)

#### T-4: `probe_undeclared_features` no port e no `SoftwareOsd` (D-2)
- **Specialist:** jdi-doer-ddc-control
- **Files modified:** `crates/ddc-core/src/domain/mod.rs`, `crates/ddc-core/src/domain/feature.rs`, `crates/ddc-core/src/ports/monitor_control.rs`, `crates/ddc-core/src/app/software_osd.rs`, `crates/ddc-core/tests/monitor_control/main.rs`, `crates/ddc-core/tests/monitor_control/support.rs`, `crates/ddc-core/tests/monitor_control/probe_features.rs`
- **Acceptance:**
  - Tipo `ProbedFeature { code, outcome: Result<FeatureReading, DdcError> }`. Método do port documentado: só leitura, e só `MonitorNotFound` é `Err` externo. O corpo de `get_feature` vira `read_and_track` (lê + `remember_max`), compartilhado com a sondagem. Nunca chama `write_vcp`; sem cache em disco.
  - DoD #6 (`probe_features.rs`), com o caps do RTK e o fake assim: 0x62/0x6C respondem, 0x7E → `Transport("ddc-hi panicked: …")`, 0x20 → `Timeout`, o resto sem valor. Os sondados são exatamente catálogo − caps (0x1E 0x20 0x30 0x62 0x6C 0x6E 0x70 0x7E 0xC9 0xE6 0xF1), em ordem, cada um lido 1×. Outcome certo por código, nenhum `WriteVcp`, e uma falha não interrompe a sondagem.
  - Extras: `set` Safe num código só sondado (0x6C) não relê o max; caps ilegível → sonda o catálogo inteiro; monitor inexistente → `Err(MonitorNotFound)`. Mutação no SUMMARY: abortar no 1º erro faz o #6 falhar.
- **Dependencies:** T-1 (D-6 antes da sondagem), T-3
- **Test:** DoD #6 (Verify da CONTEXT) + `cargo test -p ddc-core --locked`
- **Status:** pending

#### T-5: CLI por nome — aliases do catálogo, `<VALUE>` por nome, valor nomeado em `get`/`set`, `reset` (D-3, D-4)
- **Specialist:** jdi-doer-ddc-control
- **Files modified:** `crates/ddc-cli/src/args.rs`, `crates/ddc-cli/src/args/tests.rs`, `crates/ddc-cli/src/run.rs`, `crates/ddc-cli/src/exit.rs`, `crates/ddc-cli/src/exit/tests.rs`, `crates/ddc-cli/src/output.rs`, `crates/ddc-cli/src/output/tests.rs`, `crates/ddc-cli/src/fixture.rs`, `crates/ddc-cli/tests/cli/main.rs`, `crates/ddc-cli/tests/cli/reads.rs`, `crates/ddc-cli/tests/cli/writes.rs`, `crates/ddc-cli/tests/cli/reset.rs`
- **Acceptance:**
  - Saem `SHORTCUTS`/`shortcut_name`: `<VCP>` = número ou `code_for_alias`; rótulo e `name` do JSON = alias. Valores por nome conforme A-5 (`ValueArg`, `CliError::Usage` → exit 2). `reset <target> [--yes]`: `ResetTarget` é `ValueEnum` (`factory`, `brightness-contrast`, `geometry`, `color` → consts `RESTORE_*`) e chama `set_feature(id, code, 0, confirm)`, sem `get` depois, imprimindo só o envio (A-11). Guards: nenhum `VcpCode(0x` em `args.rs`; `! grep -rnE 'risk_for_code|authorize_write' crates/ddc-cli/src`.
  - Saída de `get`/`set`: texto `1 (0x01) sRGB` ou `14400 (0x3840) 144.00 Hz`; JSON com `value_name`/`interpreted` (A-7). Fixture com os valores de A-11.
  - DoD #11 (`writes.rs`), com ao menos um caso pelo binário `--fake`: `set preset srgb`/`SRGB`/`display-native` → `WriteVcp(0x14, 1|2)`, exit 0; `set osd-language english` (fora do caps da fixture) → `WriteVcp(0xCC, 2)`, exit 0; `set preset 6500k` → exit 4, sem `WriteVcp`; `set brightness srgb` e `set preset nonsense` → exit 2, sem tocar o monitor.
  - DoD #12 (`reset.rs`): os 4 alvos → 0x04/0x05/0x06/0x08. Sem `--yes`: exit 5, log só com `Enumerate`. Com `--yes`: exit 0 e `WriteVcp(code, 0)`. Pelo binário, `--fake reset factory` → exit 5, com `--yes` no stderr. Alvo desconhecido → exit 2.
  - Testes de saída exata passam a esperar o alias (ex.: `0x87 sharpness:`); o JSON de `get volume` da phase `cli` continua idêntico.
- **Dependencies:** T-3
- **Test:** DoD #11 e #12 (Verify da CONTEXT) + `cargo test -p ddc-cli --locked`
- **Status:** pending

### Wave 4

#### T-6: `ddc-cli features [--probe] [--json]` (D-3)
- **Specialist:** jdi-doer-ddc-control
- **Files modified:** `crates/ddc-cli/src/args.rs`, `crates/ddc-cli/src/args/tests.rs`, `crates/ddc-cli/src/run.rs`, `crates/ddc-cli/src/output.rs`, `crates/ddc-cli/src/output/tests.rs`, `crates/ddc-cli/src/fixture.rs`, `crates/ddc-cli/tests/cli/main.rs`, `crates/ddc-cli/tests/cli/features.rs`
- **Acceptance:**
  - Segue A-6/A-7. Tabela em texto, uma linha por código, em ordem: código, alias, `C|NC|T`, `RO|WO|RW`, risco, origem (`caps`/`probe`), valor (`atual/max` + nome ou interpretação, ou `not supported by this monitor`/`not responding`) e nome MCCS. `features` nunca escreve e não aceita `--yes`. A lógica fica em `run.rs`/`output.rs`; `main.rs` não muda.
  - Fixture ganha 0x7E → `Transport("ddc-hi panicked: …")`, espelhando o RTK.
  - DoD #10 (`features.rs`), pelo binário e in-process:
    - sem flag: só os 5 códigos do caps (preset `sRGB`, input `DisplayPort-1`), com `ReadVcp` só deles;
    - com `--probe`: acrescenta todo código do catálogo fora do caps — 0x62 `ok`, 0x87 `unsupported`, 0xDF (timeout) e 0x7E (transport) `unresponsive` —, exit 0, sem `WriteVcp`;
    - `--json` relido com `serde_json` campo a campo, e o texto com as duas mensagens.
  - Extras: caps ilegível → aviso, e só as linhas sondadas com `--probe`; `MonitorNotFound` numa linha → exit 3.
- **Dependencies:** T-4, T-5
- **Test:** DoD #10 (Verify da CONTEXT) + `cargo test -p ddc-cli --locked`
- **Status:** pending

### Wave 5

#### T-7: README + CHANGELOG como manual do usuário
- **Specialist:** jdi-doer-ddc-control
- **Files modified:** `README.md`, `CHANGELOG.md`
- **Acceptance:**
  - README (en), conteúdo:
    - Status na fase 4. Usage com `features [--probe]`, `get`/`set` por alias e por nome de valor, e `reset`.
    - Tabela completa do catálogo (código, alias, nome, C/NC/T, RO/WO/RW, safe/dangerous→`--yes`, nomes de valor) + a regra de normalização de A-3.
    - Segurança: risco vem do catálogo e código desconhecido é `dangerous`; RO e `Table` nunca graváveis; NC validado contra caps ou catálogo; contínuo limitado pelo max lido; read-back; `reset` = `set <code> 0 --yes`.
    - Exit codes: acrescenta que nome de valor de outra feature também sai com 2.
    - Known limitations: só os 39 códigos do RTK; 0x0C/0x0B e 0xAC raw; 0xFD/0xFF sem resposta; 0xE6/0xF1 sem semântica (nunca gravar); 0x02/0x1E sem lista; R-1, R-2 e R-3; itens do OSD sem VCP são inalcançáveis.
  - README, exemplos: reais e só de leitura, capturados no box (`features`, `features --probe`, `get preset`, `get input`). Sem `--fake`; nenhum `set`/`reset` em hardware.
  - CHANGELOG `[Unreleased]`: Added; Changed (0x0C e 0x6C/0x6E/0x70 graváveis sem `--yes`; NC sem lista no caps deixa de aceitar qualquer valor ≤ max); Fixed (panic que derrubava o worker).
- **Dependencies:** T-6
- **Test:** `grep -q 'features --probe' README.md && grep -q 'reset factory' README.md && ! grep -q -- '--fake' README.md && grep -q '^## \[Unreleased\]' CHANGELOG.md`
- **Status:** pending

### Wave 6

#### T-8: Roteiro de validação em hardware (só documenta; o orquestrador executa)
- **Specialist:** jdi-doer-ddc-control
- **Files modified:** `docs/hardware-validation.md`
- **Acceptance:** roteiro exato em inglês, com resultados esperados e a lista do que nunca rodar. O doer não executa escrita.
  - 0) `cargo build --release -p ddc-cli --locked`; `B=target/release/ddc-cli`; ≥ 6 s entre leituras de caps; antes de tudo, registrar os originais de brightness/contrast/preset/volume com `$B -m RTK get <x> --json`.
  - 1) `$B list` (RTK + LG); `$B -m RTK caps`; `DDC_HW_TESTS=1 cargo test -p ddc-adapters --locked --test real_monitor -- --ignored --test-threads=1` todo verde, incluindo o teste de 0x7E.
  - 2) `$B -m RTK features` → 28 linhas, exit 0 (`--json | jq length` = 28). `features --probe` → +11 linhas: 0x1E 0x20 0x30 0x62 0x6C 0x6E 0x70 0xC9 0xE6 0xF1 `ok` e 0x7E `unresponsive`, exit 0; a mensagem de panic no stderr é aceita; medir com `time`. `get preset` → `sRGB`; `get input` → `DisplayPort-1`; `get v-frequency` → `144.00 Hz`; `get vcp-version` → `2.2`.
  - 3) Escritas SAFE, só estas, cada uma voltando ao original e saindo com exit 0 e `applied: true`: `set brightness 80`; `set contrast 55`; `set preset 6500k` (lê `6500 K`) → `set preset srgb`; `set volume 35` (D-cli-1, pendente desde a `cli`).
  - 4) Recusas sem escrita: `set preset 0x03` → exit 4; `set brightness srgb` → exit 2.
  - 5) Estado final igual aos originais. Opcional: `-m LG features --probe` (DDC mudo) → exit 0, tudo `unresponsive`.
  - NUNCA, com ou sem `--yes`: input, power, qualquer `reset`, osd-lock, 0x1E, 0x20, 0x30, 0x7E, 0xE6, 0xF1, código desconhecido.
- **Dependencies:** T-6, T-7
- **Test:** `grep -q 'features --probe' docs/hardware-validation.md && grep -q 'set preset srgb' docs/hardware-validation.md && ! grep -qE '\$B .*( reset |set (input|power|osd-lock|auto-setup|h-position|v-position|trapezoid|0x))' docs/hardware-validation.md`
- **Status:** pending

## Execution
- 8 tasks, 6 waves (W1 e W3 paralelizáveis); speedup estimado 1.3x.
- Tipos: T-1 e T-3 `fix`; T-2/T-4/T-5/T-6 `feat`; T-7 e T-8 `docs`.
- DoD da CONTEXT por task: #1/#2/#4/#5 → T-2; #3/#7/#8/#9 → T-3; #6 → T-4; #11/#12 → T-5; #10 → T-6.
- Hardware (orquestrador, Deferred to PR): o roteiro de T-8. Itens em aberto para o PR: A-4 (diverge da letra de D-1 em 0xAC), a semântica de 0xE6/0xF1 (D-7), R-1..R-3 e o W-8 da `cli`.

## Files modified (all tasks)
- `crates/ddc-adapters/src/ddc_hi_backend.rs`, `crates/ddc-adapters/src/ddc_hi_backend/{worker,retry}.rs`, `crates/ddc-adapters/src/ddc_hi_backend/worker/tests.rs`, `crates/ddc-adapters/tests/real_monitor.rs`
- `crates/ddc-core/src/domain/{mod,vcp,mccs_catalog,feature,capabilities}.rs`, `crates/ddc-core/src/domain/{mccs_catalog,feature,capabilities}/tests.rs`
- `crates/ddc-core/src/app/software_osd.rs`, `crates/ddc-core/src/ports/monitor_control.rs`
- `crates/ddc-core/tests/monitor_control/{main,support,read_features,write_features,probe_features}.rs`
- `crates/ddc-cli/src/{args,run,exit,output,fixture}.rs`, `crates/ddc-cli/src/{args,exit,output}/tests.rs`
- `crates/ddc-cli/tests/cli/{main,reads,writes,reset,features}.rs`
- `README.md`, `CHANGELOG.md`, `docs/hardware-validation.md`

## Test requirements
- Unit + integração: `cargo test --workspace --locked`, com os 12 Verify da CONTEXT imprimindo OK. Hardware só pelo orquestrador, com o roteiro de T-8.
- Cobertura ≥ 80% de linhas: `cargo llvm-cov --workspace --locked --summary-only --fail-under-lines 80 --ignore-filename-regex '(^|[/\\])(main|build)\.rs$'`. `main.rs` continua só composition root.
