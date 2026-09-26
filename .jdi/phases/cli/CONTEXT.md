# Phase 3: CLI — Context  (slug: cli)

## Goal
Crate `ddc-cli` (clap 4.6): `list`, `caps`, `get <vcp>`, `set <vcp> <val>`, atalhos nomeados (brightness, contrast, input, preset, volume, power), `--json`; validação no monitor real.

## Locked decisions
- D-1/D-2: Hexagonal; `ddc-core` só depende de `thiserror`; DTOs com serde vivem só em driving adapters — nesta phase, só em `ddc-cli`.
- D-2026-09-25-core-domain-2/3: shape de `Feature`/`Risk` e a tabela seed de `risk_for_code` — a CLI consome via `authorize_write`, nunca reclassifica.
- D-2026-09-25-ddc-backends-2: `MonitorId` estável por EDID (fallback `index-N`) — base da seleção de monitor desta phase.
- D-2026-09-25-ddc-backends-7: orçamentos por operação medidos em hardware (VCP 1s, caps 8s, enumerate 5s) — o custo de ~3.7s por invocação curta (enumerate + primeira leitura de caps) é o que motiva D-2026-09-25-cli-1.
- D-2026-09-25-cli-1: cache persistente de `read_capabilities` por `MonitorId` como decorator `CachingMonitorBackend<B: MonitorBackend>` em `ddc-adapters` (não um novo port); construtor recebe `cache_dir: PathBuf` explícito (testável); resolução real de diretório (`XDG_CACHE_HOME`/`~/.cache`, `%LOCALAPPDATA%`) só na composition root da CLI; só `Ok` é persistido; `invalidate(&self, id)` inerente é o mecanismo de `caps --refresh`; `enumerate()` nunca é cacheado.
- D-2026-09-25-cli-2: `--monitor` omitido → auto-seleciona se houver exatamente 1, erro listando ids se houver 2+, erro se houver 0 — nunca adivinha. `--monitor <valor>` casa por id exato, depois índice 1-based (ordem de `enumerate()`), depois substring case-insensitive única; ambíguo ou ausente = erro.
- D-2026-09-25-cli-3: `Confirm::Yes` só nasce de `--yes` explícito (nunca prompt interativo); a CLI sempre delega a `set_feature`/`authorize_write` do core para decidir se é perigoso — nunca duplica a tabela de risco.
- D-2026-09-25-cli-4: switch de teste é a flag oculta `--fake` (não env var), ausente de `--help`; troca o backend real por um `InMemoryMonitorBackend` com fixture fixa e determinística cobrindo os 6 atalhos.
- D-2026-09-25-cli-5: `<vcp>`/`<val>` aceitam decimal, hex `0x`, e (só `<vcp>`) os 6 nomes de atalho mapeados para consts de `VcpCode` — sem número mágico; tabela de exit codes fechada (0 ok, 2 uso — nativo do clap, 3 monitor, 4 valor/feature inválidos, 5 escrita perigosa não confirmada, 6 transporte/timeout).

## Canonical refs
- Card: pedido do usuário via `/jdi-issue`, 2026-09-25 — "o importante é conseguir controlar tudo do monitor" via DDC/CI (mesma chain registrada em `.jdi/phases/ddc-backends/CONTEXT.md`).
- `.jdi/decisions/D-2026-09-25-ddc-backends-7.md` — medição de latência (enumerate 1.13s, caps 2.58s, VCP 44–94ms) que motiva D-2026-09-25-cli-1.
- `crates/ddc-core/src/ports/{monitor_backend,monitor_control}.rs` — contrato exato que a CLI consome; assinaturas não mudam nesta phase.
- `crates/ddc-core/src/domain/{feature,vcp,monitor,error}.rs` — `VcpCode` consts, `Risk`/`authorize_write`, `DdcError` (variantes que a CLI mapeia para exit codes).
- `crates/ddc-core/src/app/software_osd.rs` — cache de capabilities já existente, mas só por processo (`Mutex<HashMap<...>>` em memória); a persistência em disco de D-2026-09-25-cli-1 é complementar, não substitui essa lógica.
- `crates/ddc-adapters/src/in_memory.rs` — padrão de fake (`FakeMonitor`, builder, log de chamadas) reaproveitado pela fixture `--fake` (D-2026-09-25-cli-4).
- `.jdi/agents/jdi-doer-ddc-control.md` §§ Errors, Tests, Style & lints — `anyhow` só em `main`, sem `unwrap`/`expect` fora de teste, coverage exclui composition roots (`--ignore-filename-regex`).

## Out of scope
- Catálogo MCCS 2.2 completo, nomes de valor (ex.: `input dp1`), CLI `features` (dump), sondagem read-only de códigos fora do caps → phase `full-osd-control` (D-2026-09-25-full-osd-control-1).
- Validação funcional completa em hardware Windows real e em CI → phases já cobertas por `ddc-backends`/`ci-crossbuild`; esta phase só valida no Linux de dev (ver Deferred to PR review).
- Qualquer UI gráfica (tray/ícone) → phase `tray-app`.

## Definition of Done

### Auto-verifiable
- [ ] `crates/ddc-cli` é membro do workspace, depende de `clap` (derive) e das duas crates do hexágono; `serde`/`serde_json` (DTOs de `--json`) aparecem no workspace só em `ddc-cli` — `ddc-core`/`ddc-adapters` continuam sem serde (D-2).
      **Verify:** `grep -q 'crates/ddc-cli' Cargo.toml && grep -qE 'clap' crates/ddc-cli/Cargo.toml && grep -qiE 'serde' crates/ddc-cli/Cargo.toml && ! grep -qi serde crates/ddc-core/Cargo.toml && ! grep -qi serde crates/ddc-adapters/Cargo.toml && cargo build --workspace --locked && echo OK`
      **Source:** CONTEXT
- [ ] Teste nomeado prova que, após a primeira leitura, `CachingMonitorBackend` serve `read_capabilities` do arquivo em disco sem chamar o backend interno de novo — mesmo numa instância nova apontando pro mesmo `cache_dir` (simula uma segunda invocação curta da CLI).
      **Verify:** `cargo test -p ddc-adapters --locked -- caching_backend_reads_capabilities_from_disk_without_calling_wrapped_backend_after_first_fetch 2>&1 | grep -qE 'test result: ok\. 1 passed; 0 failed' && echo OK`
      **Source:** CONTEXT
- [ ] Teste nomeado prova que `CachingMonitorBackend` nunca persiste uma falha de `read_capabilities`, e que `invalidate()` força a próxima leitura a ir para o backend interno e sobrescrever o arquivo (mecanismo de `caps --refresh`, D-2026-09-25-cli-1).
      **Verify:** `cargo test -p ddc-adapters --locked -- caching_backend_never_persists_failures_and_refresh_forces_a_fresh_overwrite 2>&1 | grep -qE 'test result: ok\. 1 passed; 0 failed' && echo OK`
      **Source:** CONTEXT
- [ ] Teste nomeado (`assert_cmd`, backend `--fake`) prova a seleção de monitor sem `--monitor`: exatamente 1 monitor é usado automaticamente; 0 ou 2+ monitores fazem o comando falhar sem adivinhar, listando os ids quando houver mais de um (D-2026-09-25-cli-2).
      **Verify:** `cargo test -p ddc-cli --locked -- resolving_monitor_without_the_flag_auto_selects_the_only_one_or_refuses_when_zero_or_many 2>&1 | grep -qE 'test result: ok\. 1 passed; 0 failed' && echo OK`
      **Source:** CONTEXT
- [ ] Teste nomeado (`assert_cmd`, `--fake`) prova o casamento de `--monitor <valor>` por id exato, por índice 1-based (ordem de `list`) e por substring case-insensitive única; uma substring que casa 2+ ids falha listando os candidatos.
      **Verify:** `cargo test -p ddc-cli --locked -- monitor_flag_matches_by_exact_id_index_or_unique_substring_and_refuses_ambiguous_matches 2>&1 | grep -qE 'test result: ok\. 1 passed; 0 failed' && echo OK`
      **Source:** CONTEXT
- [ ] Teste nomeado prova que `--fake` seleciona um `InMemoryMonitorBackend` determinístico (sem tocar `ddc-hi`) e que a flag está ausente da saída de `--help`.
      **Verify:** `cargo test -p ddc-cli --locked -- fake_flag_selects_a_deterministic_in_memory_backend_and_stays_hidden_from_help 2>&1 | grep -qE 'test result: ok\. 1 passed; 0 failed' && echo OK`
      **Source:** CONTEXT
- [ ] Teste nomeado (`--fake`) prova que um `set` numa feature `Dangerous` (nome `input`/`power` ou um código numérico fora da tabela seed) sem `--yes` é recusado ANTES de tocar o backend (log de chamadas do fake sem `WriteVcp`) e com o exit code de D-2026-09-25-cli-5; com `--yes`, a escrita é aplicada e lida de volta.
      **Verify:** `cargo test -p ddc-cli --locked -- dangerous_write_without_yes_is_refused_before_touching_the_backend_and_applied_with_yes 2>&1 | grep -qE 'test result: ok\. 1 passed; 0 failed' && echo OK`
      **Source:** CONTEXT
- [ ] Teste nomeado prova que `<vcp>` aceita decimal, hex `0x`, e os 6 nomes de atalho (case-insensitive) resolvendo para o `VcpCode` correto; um nome desconhecido ou número malformado é erro de uso do `clap`.
      **Verify:** `cargo test -p ddc-cli --locked -- vcp_argument_accepts_decimal_hex_and_named_shortcuts_and_rejects_unknown_names 2>&1 | grep -qE 'test result: ok\. 1 passed; 0 failed' && echo OK`
      **Source:** CONTEXT
- [ ] Teste nomeado (`assert_cmd`, `--fake`) prova a tabela de exit codes de D-2026-09-25-cli-5: monitor não encontrado (3), valor/feature inválidos (4), escrita perigosa não confirmada (5), transporte/timeout simulado (6) — cada cenário com seu código exato.
      **Verify:** `cargo test -p ddc-cli --locked -- exit_codes_are_stable_for_not_found_invalid_value_unconfirmed_and_transport_failures 2>&1 | grep -qE 'test result: ok\. 1 passed; 0 failed' && echo OK`
      **Source:** CONTEXT

### Manual
- _(none)_

## Deferred to PR review
- `ddc-cli list` no hardware real (Linux, RTK QHD HDR em `/dev/i2c-5` + LG TV mudo em `/dev/i2c-11`): ambos aparecem na lista; o LG só falha ao ser efetivamente lido/escrito, nunca no `list` (D-2026-09-25-ddc-backends-4).
- `ddc-cli caps` no hardware real: a string bate com a fixture já travada em `core-domain`; `caps --refresh` de fato dispara uma nova leitura (tempo visivelmente maior que a chamada em cache).
- `ddc-cli get brightness` (0x10, declarado no caps) e `ddc-cli get volume` (0x62, fora do caps — D-2026-09-25-core-domain-4) no hardware real, confirmando `declared_in_capabilities` correto em ambos.
- `ddc-cli set brightness <novo valor> --yes` reversível no hardware real: ler o valor atual, escrever um novo (dentro do máximo, feature `Safe`), confirmar a leitura de volta, e restaurar o valor original antes de terminar — nunca uma escrita `Dangerous` (input/power/código desconhecido) no hardware real, seja com ou sem `--yes`.
- Medir, no hardware real, que a segunda invocação curta de `get`/`set` (com o cache de D-2026-09-25-cli-1 já populado por um `caps` ou `get` anterior) cai de ~3.7s para próximo de ~1.1–1.3s (só o `enumerate`, sem a leitura de caps de 2.58s).

## Notes
- `main.rs` (composition root: parse de `--fake`/`--yes`, escolha do backend real vs. fixture, resolução de `default_cache_dir()`) é excluído do gate de cobertura (`--ignore-filename-regex '(^|[/\\])(main|build)\.rs$'`, herdado de `ddc-backends`) — por isso parsing de valores, resolução de `--monitor`, formatação `--json` e mapeamento de erro→exit code devem morar em módulos testáveis de `ddc-cli` (ex.: `args.rs`, `resolve.rs`, `output.rs`, `exit.rs`), nunca dentro de `main()`. Mover essa lógica para `main.rs` só para escapar do gate é bloqueante no reviewer.
- Baseline de `.jdi/PROJECT.md` (`cargo test --workspace` verde, cobertura ≥80%, sem TODO/FIXME sem issue) é herdada automaticamente pelo Gate 8 do reviewer — não duplicada aqui.
- Nome de binário/pacote sugerido: `ddc-cli` (default do cargo, sem `[[bin]] name` extra) — não é uma decisão travada, liberdade do planner/doer.
- `CachingMonitorBackend` deve ficar em `crates/ddc-adapters/src/caching.rs`, sempre compilado (sem `#[cfg(feature = "ddc-hi")]` — é genérico sobre `B`, útil e testável mesmo sem o backend real).
