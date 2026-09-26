# Phase 3: CLI — Plan  (slug: cli)

## Goal
Crate `ddc-cli` (clap 4.6): `list`, `caps`, `get <vcp>`, `set <vcp> <val>`, atalhos nomeados (brightness, contrast, input, preset, volume, power), `--json`; validação no monitor real.

## Locked decisions (from CONTEXT.md)
- D-1/D-2: hexagonal. `serde` só em `ddc-cli` (DTOs de `--json`); `ddc-core`/`ddc-adapters` sem serde. Ports e `SoftwareOsd` não mudam.
- D-2026-09-25-core-domain-2/3: o risco vem só de `set_feature`/`authorize_write`. A CLI nunca reclassifica.
- D-2026-09-25-ddc-backends-2/-4/-7: `MonitorId` por EDID; `enumerate()` não sonda; cada invocação paga ~1.1s de enumerate + ~2.6s de caps (motiva cli-1).
- D-2026-09-25-cli-1: `CachingMonitorBackend<B>` em `ddc-adapters/src/caching.rs`, sempre compilado, `new(inner, cache_dir: PathBuf)`. Só `read_capabilities` passa pelo cache e só `Ok` é persistido. `invalidate(&self, id)` = `caps --refresh`. `default_cache_dir()` no mesmo módulo, chamado só pela composition root.
- -cli-2: sem `--monitor`, 1 monitor → auto; 0 ou 2+ → erro listando ids. Com `--monitor`: id exato → índice 1-based → substring case-insensitive única; senão erro.
- -cli-3: `Confirm::Yes` só vem de `--yes`, nunca de prompt. -cli-4: flag oculta `--fake` → `InMemoryMonitorBackend` com o monitor `FAKE-CLI-TEST`.
- -cli-5: `<vcp>` aceita decimal, `0x` e os 6 nomes; `<val>` decimal e `0x`. Exit: 0 ok, 2 uso (clap nativo), 3 monitor, 4 feature/valor, 5 perigosa sem `--yes`, 6 transporte/timeout.

## Assumptions (cadeia autônoma, sem AskUserQuestion)
- A-1 **Binário = `ddc-cli`**: nome do pacote, sem `[[bin]]`. Bate com a CONTEXT (Notes/Deferred), o doer spec e os `-p ddc-cli` do DoD; `ddc-control` fica como nome do projeto/app. Os testes `assert_cmd` usam `env!("CARGO_BIN_EXE_ddc-cli")`, sem API depreciada (o clippy roda com `-D warnings`).
- A-2 **Deps** direto em `crates/ddc-cli/Cargo.toml`, fora de `[workspace.dependencies]`, para serde só aparecer em `ddc-cli`: `clap = { version = "4.6", features = ["derive"] }` (4.6.7 atual, MSRV 1.85), `serde = { version = "1", features = ["derive"] }`, `serde_json = "1"`; dev: `assert_cmd = "2.2"`, `predicates = "3"`.
  - Sem `anyhow`: toda falha, inclusive a construção do backend real, cai na tabela fechada de cli-5 (um `Err` de `main` sairia com 1). Sem `dirs`/`tempfile`: std basta. T-3 fecha o `Cargo.lock`; nenhuma outra task toca `Cargo.*`.
- A-3 `members` ganha `crates/ddc-cli`. O verify #1 da `core-domain` (`! grep -qE 'crates/ddc-cli|apps/ddc-tray' Cargo.toml`) deixa de valer por design do ROADMAP. Não é regressão.
- A-4 **`default_cache_dir() -> Option<PathBuf>`**, só com env vars da std. Fora do Windows: `$XDG_CACHE_HOME` (só se absoluto e não-vazio, spec XDG), senão `$HOME/.cache`, + `ddc-control/caps`. Windows: `%LOCALAPPDATA%\ddc-control\caps`.
  - Lógica pura com `(plataforma, lookup)`, testada para os 2 SOs no Linux; `default_cache_dir` escolhe por `cfg!(windows)` (expressão: os 2 ramos compilam). O subdiretório é `caps` (D-cli-1), não `cache` como na nota de handoff.
  - `None` → `main` liga o backend real sem cache (lento, nunca falha). Nunca `temp_dir()`: é compartilhado e outro usuário plantaria caps.
- A-5 **Arquivo de cache**. Nome: o id mantém `[A-Za-z0-9-]` e todo outro byte vira `_XX` hex. É injetivo, sem `/`, `\` ou `.`, logo sem path traversal (`…-01010101#2` → `…-01010101_232.caps`).
  - Conteúdo: a string crua de `B`. Escrita atômica (tmp único no mesmo dir + `rename`), `create_dir_all` sob demanda. I/O do cache nunca derruba a operação: erro de leitura ou arquivo vazio = miss; erro de escrita é ignorado e o `Ok` de `B` volta.
- A-6 **`invalidate(&self, id)`**, retorno `()` (D-cli-1): remove o arquivo em best-effort e marca o id em memória, para que a próxima `read_capabilities` desta instância ignore o disco mesmo se a remoção falhar.
  - Para a composition root chamar `invalidate` depois de entregar o backend ao `SoftwareOsd`, `ddc-adapters` também implementa `MonitorBackend for &CachingMonitorBackend<B>` (delegação pura, padrão `impl Read for &File`): `SoftwareOsd::new(&caching)`. O core não muda (gate 5.3).
- A-7 **`run` testável**: `ddc_cli::run(cli: &Cli, control: &impl MonitorControl, refresh: &dyn Fn(&MonitorId), out: &mut dyn Write, err: &mut dyn Write) -> Exit`. `Exit` é um enum `#[repr(u8)]` cujos discriminantes são a tabela de cli-5, com `From<Exit> for ExitCode`; os testes comparam `Exit`.
  - `main.rs` é só composition root: `Cli::parse()` (uso = 2 nativo) → `--fake`: `InMemoryMonitorBackend` com `fixture_monitor()` e refresh no-op. Senão: `DdcHiMonitorBackend::new()` (se falhar, mesma renderização de erro, sai 6) → `CachingMonitorBackend` → `SoftwareOsd::new(&caching)` → `refresh = |id| caching.invalidate(id)`. Nenhuma regra além da escolha de backend.
  - `caps --refresh` chama `refresh(&id)` depois de resolver o monitor e antes de `capabilities`.
- A-8 **Fixture `--fake`** (cli-4): `ddc_cli::fixture::fixture_monitor() -> FakeMonitor`, `#[doc(hidden)]`. Só dado; o adapter é construído em `main` e nos testes. Id `FAKE-CLI-TEST`; caps `(prot(monitor)type(LCD)model(FAKE)vcp(10 12 14(01 02 03) 60(01 0F 11) 62 D6(01 04 05))mccs_ver(2.2))`.
  - Valores: 0x10 50/100, 0x12 70/100, 0x62 30/100, 0x14=0x01, 0x60=0x0F, 0xD6=0x01. Mais a const nomeada `SIMULATED_TIMEOUT = VcpCode(0xDF)` → `Timeout` (knob da T-2), adição que prova o exit 6 ponta a ponta sem contradizer cli-4.
  - A fixture tem 1 monitor fixo. Os cenários de 0/2+ monitores, substring ambígua e log sem `WriteVcp` (o log de um subprocesso não é observável) rodam in-process via `run` com `InMemoryMonitorBackend` próprio, dentro do MESMO teste nomeado que faz a parte `assert_cmd`.
- A-9 **Seleção** (cli-2): valor só com dígitos = índice. Fora do intervalo dá erro 3, sem cair para substring: `--monitor 2` nunca pode escolher um id que contém "2" (segurança).
  - Todo erro de seleção = 3. A mensagem lista `índice  id` no formato de `list` (índice da ordem completa do enumerate) + a dica literal `--monitor <id|index>` com exemplo (no box de dev: RTK + LG). `list` ignora `--monitor`.
- A-10 **Saída**: stdout traz o resultado; stderr traz erros e avisos em texto, também com `--json` (o contrato de script é o exit code). Sem risco na saída: a CLI não toca `Risk::`.
  - JSON com números decimais, campos mínimos: list `[{index,id,manufacturer,model,serial}]`; caps `{monitor,protocol,type,model,mccs_version,commands,vcp:[{code,values}]}`; get `{monitor,code,name,current,max,declared_in_capabilities}`; set `{monitor,code,name,requested,current,max,applied}`.
  - Texto com `0xNN` + nome do atalho; `get` avisa "not declared in capabilities". Read-back ≠ pedido → aviso no stderr, `applied: false`, exit 0 (tabela fechada). Falha ao escrever em stdout/stderr é ignorada, sem panic.
- A-11 Nomes do DoD são únicos no workspace; nenhum teste extra os contém. O `foo/tests.rs` irmão conta como parte de `foo.rs`.

## Riscos
- R-1 O cache por `MonitorId` herda a instabilidade dos ids de fallback (`index-N`, descrição genérica no Windows, `#N`): no hotplug, a caps de um monitor pode ser servida a outro. Efeito limitado: o risco é por código e o max de feature contínua vem de leitura. Escape: `caps --refresh`, documentado no README (T-8).
- R-2 Os defeitos do adapter (primeira `read_vcp` sem enumerate; re-enumerate do display mudo acima do budget) são do fix round da `ddc-backends`. A CLI não os contorna; ela já chama `list_monitors()` para resolver o monitor (cli-2).

## Tasks
Specialist único: `jdi-doer-ddc-control` (glob `**/*`). Scope `cli`, D-XX citada no corpo, nunca junto com `.jdi/`. Todo commit mantém verdes:
- `cargo fmt --all --check`, `cargo clippy --workspace --all-targets --locked -- -D warnings`, `cargo test --workspace --locked`, `cargo check -p ddc-core -p ddc-adapters [-p ddc-cli] --locked --target x86_64-unknown-linux-gnu`.
- `cargo check -p ddc-adapters --locked --features ddc-hi --target x86_64-pc-windows-msvc`; de T-3 em diante também `cargo check -p ddc-cli --locked --target x86_64-pc-windows-msvc` (dev-deps não entram).

### Wave 1 (parallel-eligible)

#### T-1: `CachingMonitorBackend<B>` + `default_cache_dir()` (D-cli-1)
- **Specialist:** jdi-doer-ddc-control
- **Files modified:** `crates/ddc-adapters/src/lib.rs`, `crates/ddc-adapters/src/caching.rs`, `crates/ddc-adapters/src/caching/tests.rs`, `crates/ddc-adapters/tests/caching_backend.rs`
- **Acceptance:**
  - Forma: sem `#[cfg(feature)]`, `pub use caching::{CachingMonitorBackend, default_cache_dir}`. `enumerate`/`read_vcp`/`write_vcp` só delegam. Segue A-4/A-5/A-6, com a impl para `&CachingMonitorBackend<B>`. `caching.rs` não importa `in_memory`/`ddc_hi` (gate 5.4). Lock com `unwrap_or_else(PoisonError::into_inner)`.
  - DoD `caching_backend_reads_capabilities_from_disk_without_calling_wrapped_backend_after_first_fetch` (`tests/caching_backend.rs`): uma instância lê (1 `ReadCapabilities`). Uma instância NOVA, com fake novo e o mesmo dir, devolve a mesma string, e o log dela não tem `ReadCapabilities`.
  - DoD `caching_backend_never_persists_failures_and_refresh_forces_a_fresh_overwrite`: com `with_transient_capabilities_failures(1)`, o `Err` não cria arquivo e o `Ok` seguinte grava. Um arquivo velho pré-escrito é servido sem `B`. Depois de `invalidate`, a leitura chama `B`, devolve a string nova e sobrescreve o arquivo.
  - Extras: via `SoftwareOsd::new(&caching)`; `enumerate` nunca cacheado (2 chamadas → 2 `Enumerate`). Unit em `caching/tests.rs`, sem fake: encoding de nome (`#`, `/`, `..`, injetivo) e resolução de dir (XDG absoluto/relativo/vazio, fallback `HOME`, nada definido, `LOCALAPPDATA`), comparando com `join`, não com literal. Dir temporário só com std, único por teste, removido no `Drop`.
- **Dependencies:** none
- **Test:** `cargo test -p ddc-adapters --locked caching` + verifies #2/#3 do DoD
- **Status:** completed

#### T-2: `FakeMonitor::with_vcp_failure(code, error)` no fake
- **Specialist:** jdi-doer-ddc-control
- **Files modified:** `crates/ddc-adapters/src/in_memory.rs`, `crates/ddc-adapters/src/in_memory/tests.rs`
- **Acceptance:**
  - `read_vcp`/`write_vcp` desse código respondem o `DdcError` roteirizado. A chamada continua logada, o valor fica intacto e os outros códigos não mudam. Só dado de teste, sem regra de negócio.
  - Unit test cobre leitura, escrita e isolamento. Commit `test`.
- **Dependencies:** none
- **Test:** `cargo test -p ddc-adapters --locked in_memory`
- **Status:** completed

#### T-3: Crate `ddc-cli` + argumentos clap (D-cli-5a)
- **Specialist:** jdi-doer-ddc-control
- **Files modified:** `Cargo.toml`, `Cargo.lock`, `crates/ddc-cli/Cargo.toml`, `crates/ddc-cli/src/lib.rs`, `crates/ddc-cli/src/args.rs`, `crates/ddc-cli/src/args/tests.rs`
- **Acceptance:**
  - Cargo: `members` + `crates/ddc-cli`; deps de A-2 + `ddc-core`/`ddc-adapters` (workspace, features default); `[lints] workspace = true`. Lock gerado uma vez sem `--locked`, no mesmo commit (`feat`). Verify #1 do DoD = OK; check Windows do `ddc-cli` verde. `cargo audit` sem RUSTSEC novo além de RUSTSEC-2018-0005/2024-0320 (anotar no SUMMARY).
  - `lib.rs` com `//!` e `#![forbid(unsafe_code)]`. `Cli` (derive, `name = "ddc-cli"`): globais `--monitor/-m`, `--json` e `--fake` (`hide = true`); subcomandos `list`, `caps [--refresh]`, `get <VCP>`, `set <VCP> <VALUE> [--yes/-y]`.
  - Parser único de número (decimal ou `0x`/`0X`). `<VCP>` → `VcpCode` (u8), ou um dos 6 nomes case-insensitive pela tabela `SHORTCUTS` sobre as consts do core (`! grep -nE 'VcpCode\(0x' crates/ddc-cli/src/args.rs`), com `shortcut_name(code)` para a saída. `<VALUE>` → u16, só número. O erro lista os nomes válidos.
  - DoD `vcp_argument_accepts_decimal_hex_and_named_shortcuts_and_rejects_unknown_names` (`args/tests.rs`, `Cli::try_parse_from`): `16`, `0x10`, `0X10`, `brightness` e `BRIGHTNESS` → 0x10; os 6 nomes → as consts; `foo`, `0x`, `0x1G`, `256`, `-1` e `""` → `Err` com `exit_code() == 2`.
  - Extras: limites de `<VALUE>` (65535 ok; 65536 e nome → erro), `--yes` só em `set`, globais depois do subcomando.
- **Dependencies:** none
- **Test:** verifies #1 e #8 do DoD + `cargo test -p ddc-cli --locked`
- **Status:** completed

### Wave 2

#### T-4: Renderização texto/JSON (`output.rs`)
- **Specialist:** jdi-doer-ddc-control
- **Files modified:** `crates/ddc-cli/src/lib.rs`, `crates/ddc-cli/src/output.rs`, `crates/ddc-cli/src/output/tests.rs`
- **Acceptance:**
  - A-10: DTOs `#[derive(Serialize)]` só aqui, mapeados de `MonitorInfo`/`Capabilities`/`FeatureReading`/`VcpValue` (D-2). `enum Format { Text, Json }`, não `bool`. Uma fn por comando, escrevendo em `&mut dyn Write`. A linha de monitor de `list` é uma fn pública, reutilizada pela T-5.
  - Testes: os 4 comandos nos 2 formatos; JSON relido com `serde_json::Value`, campo a campo; `declared_in_capabilities == false` avisado no texto; código sem nome (`name: null`); `applied: false`. `! grep -rnE 'Risk::|risk_for_code|authorize_write' crates/ddc-cli/src` (D-cli-3).
- **Dependencies:** T-3
- **Test:** `cargo test -p ddc-cli --locked output`
- **Status:** completed

### Wave 3

#### T-5: Seleção de monitor + erro→exit code (`select.rs`, `exit.rs`)
- **Specialist:** jdi-doer-ddc-control
- **Files modified:** `crates/ddc-cli/src/lib.rs`, `crates/ddc-cli/src/select.rs`, `crates/ddc-cli/src/select/tests.rs`, `crates/ddc-cli/src/exit.rs`, `crates/ddc-cli/src/exit/tests.rs`
- **Acceptance:**
  - `select_monitor(&[MonitorInfo], Option<&str>) -> Result<MonitorId, SelectionError>`, puro, segue A-9. Erro de seleção renderizado com a linha de monitor da T-4 + a dica `--monitor`.
  - `Exit` (A-7) + `CliError { Selection, Ddc }`, com `match` exaustivo sobre `DdcError`, sem `_`: `MonitorNotFound` → 3; `UnsupportedFeature`/`InvalidValue`/`ValueNotAllowed` → 4; `DangerousWriteNotConfirmed` → 5; `Timeout`/`Transport` → 6. O 5 traz a mensagem (em inglês) "classificado como perigoso, repita com `--yes`". Sem retry e sem prompt: nenhum `stdin` em `crates/ddc-cli/src` (D-cli-3).
  - Testes de seleção: lista vazia; 1 → auto; 2+ → erro com todos os índices; id exato vence substring (`ABC` vs `ABC-2`); índice `0`/fora do intervalo sem fallback; substring case-insensitive única; substring ambígua lista só os candidatos, com índice global.
  - Testes de erro: cada variante de `DdcError` → seu código; cada mensagem traz id, índice ou código.
- **Dependencies:** T-4
- **Test:** `cargo test -p ddc-cli --locked -- select exit`
- **Status:** completed

### Wave 4

#### T-6: `run` + fixture `--fake` + composition root + testes ponta a ponta
- **Specialist:** jdi-doer-ddc-control
- **Files modified:** `crates/ddc-cli/src/lib.rs`, `crates/ddc-cli/src/run.rs`, `crates/ddc-cli/src/fixture.rs`, `crates/ddc-cli/src/main.rs`, `crates/ddc-cli/tests/cli/main.rs`, `crates/ddc-cli/tests/cli/support.rs`, `crates/ddc-cli/tests/cli/selection.rs`, `crates/ddc-cli/tests/cli/reads.rs`, `crates/ddc-cli/tests/cli/writes.rs`
- **Acceptance:**
  - `run` (A-7): `list` → `list_monitors`; `caps`/`get`/`set` → `list_monitors` + `select_monitor` → [`refresh`] → port. `--yes` → `Confirm::Yes`, senão `Confirm::No`, sempre via `set_feature` (D-cli-3). Read-back conforme A-10.
  - Fixture conforme A-8. `main.rs` conforme A-7, com `#![forbid(unsafe_code)]` (gate 5.5) e sem lógica testável (gate 3 exclui `main.rs`). Adapter só é construído em `main.rs`/`tests/` (gate 5.4).
  - DoD `resolving_monitor_without_the_flag_auto_selects_the_only_one_or_refuses_when_zero_or_many` (`selection.rs`): `assert_cmd --fake get brightness` → 0, com `50`. In-process com 0 monitores → 3 ("no monitor"). In-process com 3 monitores → 3, com os 3 ids + índices + dica `--monitor` no stderr, sem `ReadCapabilities`/`ReadVcp` no log.
  - DoD `monitor_flag_matches_by_exact_id_index_or_unique_substring_and_refuses_ambiguous_matches`: `--fake` com `--monitor FAKE-CLI-TEST`, `1` e `cli-test` → 0. In-process com 3 ids: índice e substring única selecionam; substring de 2 ids → 3 listando os candidatos; `nope` → 3.
  - DoD `fake_flag_selects_a_deterministic_in_memory_backend_and_stays_hidden_from_help` (`reads.rs`): `--fake list` mostra exatamente `FAKE-CLI-TEST`, com saída idêntica em 2 runs; `--help` e `get --help` não contêm `fake`.
  - DoD `dangerous_write_without_yes_is_refused_before_touching_the_backend_and_applied_with_yes` (`writes.rs`, in-process com `fixture_monitor()`): `set input 0x11` e `set 0xE5 1` → 5, com log só `Enumerate` (sem `ReadCapabilities`/`WriteVcp`). `set input 0x11 --yes` → 0; o log termina em `WriteVcp(0x60,0x11), ReadVcp(0x60)` e a saída mostra 0x11. `assert_cmd --fake set power 0x04` → 5, com `--yes` no stderr.
  - Extras: `list`/`caps`/`get` em texto e `--json` via `--fake`; `caps --refresh` chama `refresh` com o id resolvido antes de `ReadCapabilities` (closure que grava); `set brightness 60` → read-back 60; `ignoring_writes_to` → aviso + `applied: false` + 0. Esperado ~550 linhas (o hook avisa acima de 800).
- **Dependencies:** T-1, T-2, T-5
- **Test:** verifies #4–#7 do DoD + `cargo test -p ddc-cli --locked`
- **Status:** completed

### Wave 5 (parallel-eligible)

#### T-7: Contrato de exit codes ponta a ponta (D-cli-5b)
- **Specialist:** jdi-doer-ddc-control
- **Files modified:** `crates/ddc-cli/tests/cli/main.rs`, `crates/ddc-cli/tests/cli/exit_codes.rs`
- **Acceptance:**
  - DoD `exit_codes_are_stable_for_not_found_invalid_value_unconfirmed_and_transport_failures` (`assert_cmd --fake`): 3 com `--monitor nope get brightness` e `--monitor 2 get brightness`; 4 com `set brightness 101`, `set preset 0x07` e `get 0x87`; 5 com `set input 0x11`; 6 com `get 0xDF` (`SIMULATED_TIMEOUT`). Cada caso com código exato, stdout vazio e mensagem no stderr.
  - Extras: `get foo` → 2 (clap nativo, não reinterceptado); `get brightness` → 0. Commit `test`.
- **Dependencies:** T-6
- **Test:** verify #9 do DoD
- **Status:** completed

#### T-8: README + CHANGELOG (DoD manual do PROJECT)
- **Specialist:** jdi-doer-ddc-control
- **Files modified:** `README.md`, `CHANGELOG.md`
- **Acceptance:**
  - README (inglês): status das phases 1–3; Usage com `list`, `caps [--refresh]`, `get <vcp>`, `set <vcp> <val> [--yes]`, `--monitor` (id/índice/substring, ambiguidade = erro) e `--json`; os 6 atalhos; tabela de exit codes; local do cache por SO + R-1; `cargo install --path crates/ddc-cli`.
  - Saídas de exemplo reais, capturadas no box só com leitura (`list`/`get`), nunca `set` em hardware. Sem mencionar `--fake` (D-cli-4). CHANGELOG `[Unreleased]`: `ddc-cli`, `CachingMonitorBackend`, `FakeMonitor::with_vcp_failure`. Commit `docs`.
- **Dependencies:** T-6
- **Test:** `grep -q 'ddc-cli' README.md && ! grep -q -- '--fake' README.md && grep -q '^## \[Unreleased\]' CHANGELOG.md`
- **Status:** completed

## Execution
- Total tasks: 8 | Waves: 5 (W1 e W5 paralelizáveis) | Speedup estimado: 1.6x
- Tipos: T-1/T-3/T-4/T-5/T-6 `feat`, T-2/T-7 `test`, T-8 `docs`. DoD da CONTEXT: #1 e #8 → T-3; #2 e #3 → T-1; #4–#7 → T-6; #9 → T-7.
- Hardware (orquestrador, Deferred to PR, `B=target/release/ddc-cli`): `list` (RTK + LG, sem sondar); `get brightness` sem `-m` → 3 com a dica; `-m RTK caps --refresh` + `diff` de `~/.cache/ddc-control/caps/RTK-RTK-QHD-HDR-01010101.caps` com a fixture da `core-domain`.
  - Depois: `-m RTK get brightness`/`get volume` (`declared` sim/não); `set brightness` reversível, restaurando o original, nunca `input`/`power`; `-m LG get brightness` → 6; `time` da 2ª invocação ~1.1–1.3s.

## Files modified (all tasks)
- `Cargo.toml`, `Cargo.lock`, `crates/ddc-cli/Cargo.toml`
- `crates/ddc-adapters/src/{lib,caching,in_memory}.rs`, `crates/ddc-adapters/src/{caching,in_memory}/tests.rs`, `crates/ddc-adapters/tests/caching_backend.rs`
- `crates/ddc-cli/src/{lib,main,args,output,select,exit,run,fixture}.rs`, `crates/ddc-cli/src/{args,output,select,exit}/tests.rs`
- `crates/ddc-cli/tests/cli/{main,support,selection,reads,writes,exit_codes}.rs`
- `README.md`, `CHANGELOG.md`

## Test requirements
- Unit + integração: `cargo test --workspace --locked`. Hardware: só o orquestrador, só leitura + `set brightness` reversível.
- Cobertura ≥ 80% de linhas: `cargo llvm-cov --workspace --locked --summary-only --fail-under-lines 80 --ignore-filename-regex '(^|[/\\])(main|build)\.rs$'`. `main.rs` = composition root; toda lógica em módulos cobertos in-process.
