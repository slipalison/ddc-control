# Phase 4: Full OSD control — Context  (slug: full-osd-control)

## Goal
Controlar tudo que o OSD oferece via DDC/CI: catálogo MCCS 2.2 completo no core (nome, tipo C/NC/T, acesso RO/WO/RW, risco, nomes de valor NC), descoberta por caps + sondagem read-only de códigos fora do caps, CLI `features` (dump com valor atual) e `set` por nome de feature/valor para toda feature gravável, factory resets atrás de confirmação; validação no monitor real.

## Locked decisions
- D-1/D-2: Hexagonal; `ddc-core` só depende de `thiserror`; catálogo é dado estático puro no core, sem framework/serde.
- D-2026-09-25-core-domain-2/3/4/5: shape de `Feature`/`Capabilities`; `get_feature` sempre lê o backend independente do caps. `risk_for_code` seed original absorvida (não contradita) por D-full-osd-control-1; a letra de core-domain-4 ("sem max conhecido → `UnsupportedFeature`") foi refinada por D-2026-09-26-cli-1 e por esta phase (ver abaixo).
- D-2026-09-25-ddc-backends-2/4/6/7 + D-2026-09-26-ddc-backends-1: `MonitorId` por EDID; orçamentos por operação (VCP 1s, caps 8s, enumerate 5s); falha de VCP em display mudo é rápida — base do custo aceitável de sondagem (D-2).
- D-2026-09-25-cli-1..5: `CachingMonitorBackend` (cache só de caps), seleção de `--monitor`, `--yes`/`Confirm`, flag oculta `--fake`, parser de `<VCP>`/`<VALUE>` e tabela de exit codes.
- **D-2026-09-26-cli-1 (committed, phase `cli`):** `set_feature` aprende o max de escrita com UMA leitura quando desconhecido (declarado no caps ou não), em vez de recusar de cara — pediu explicitamente que esta phase restrinja isso a códigos `Continuous`. **D-2026-09-26-cli-2:** backoff de 500ms para `read_capabilities` (não muda nada nesta phase, contexto de timing). **D-2026-09-26-cli-3:** `caps --refresh` só substitui o cache num sucesso PARSEÁVEL, nunca apaga antes (não muda nada nesta phase). **D-2026-09-26-cli-4:** o adapter mapeia a resposta DDC/CI "unsupported VCP code" para `DdcError::UnsupportedFeature` sem retry — a sondagem desta phase depende disso pra distinguir "código não existe" de "barramento falhou".
- **Carry-over `.jdi/phases/cli/REVIEW.md` W-4/W-5** (endereçados pelas decisions desta phase, ver abaixo): W-4 pedia a correção de mapeamento de erro do adapter (resolvida por D-2026-09-26-cli-4, já committed antes desta captura); W-5 pedia que esta phase restrinja o max-por-leitura a `Continuous`, valide NC-sem-lista contra o catálogo, e cheque `Access::ReadOnly` antes de qualquer leitura de max — as três resolvidas na emenda de D-2026-09-26-full-osd-control-1.
- D-2026-09-26-full-osd-control-1: catálogo MCCS 2.2 (`domain::mccs_catalog`, tabela única `CatalogEntry{code,name,kind,access,risk,values}`) fechado nos 33 códigos observados (caps + sondagem) do "RTK QHD HDR" — tabela completa, invariante "RO nunca é Dangerous", consolidação de `risk_for_code`, e a **emenda W-5**: `max_for_write` só lê para aprender max quando `kind == Continuous`; `validate_write` ganha fallback de lista do catálogo quando `allowed_values` (caps) é `None` (inclui `00 Reset` nos 4 códigos de fábrica); `set_feature` checa `Access::ReadOnly || kind == Table` ANTES de `max_for_write` (sem leitura desperdiçada).
- D-2026-09-26-full-osd-control-2: novo método do port `MonitorControl::probe_undeclared_features(id) -> Result<Vec<ProbedFeature>, DdcError>` — só leitura, nunca escreve, sem cache em disco; com a emenda D-cli-4, distingue `UnsupportedFeature` (código não existe) de `Transport`/`Timeout` (barramento falhou — caminho esperado para 0x7E, que falha por retries esgotados, não por resposta "não suportado").
- D-2026-09-26-full-osd-control-3: `ddc-cli features [--probe] [--json]` (novo, com `probe_status: ok|unsupported|unresponsive`) + `get`/`set` ganham nome de valor/interpretação e `set <nome> <valor-por-nome>` (resolvido contra `allowed_values` do caps quando declarado, senão contra a lista do catálogo — mesma fonte que `validate_write` usa).
- D-2026-09-26-full-osd-control-4: `ddc-cli reset <factory|brightness-contrast|geometry|color> [--yes]` — mesmo caminho de `set_feature`/`authorize_write` que `set` já usa, escreve `0` (agora um valor catalogado, não só convenção em prosa) nos códigos 0x04/0x05/0x06/0x08.
- D-2026-09-26-full-osd-control-5: features `Table` ficam fora de escopo de escrita — `Feature::validate_write` rejeita `kind == Table` antes de qualquer outra checagem, no mesmo ponto que o guard de `ReadOnly` da emenda W-5 (antes de `max_for_write`).

## Canonical refs
- Card: pedido do usuário via `/jdi-issue`, 2026-09-25/26 — "o importante é conseguir controlar tudo do monitor" via DDC/CI; esta é a phase que fecha o card.
- Fatos de hardware (RTK QHD HDR) e a tabela completa do catálogo: `.jdi/decisions/D-2026-09-26-full-osd-control-{1..5}.md` (inclui as emendas W-4/W-5).
- `.jdi/decisions/D-2026-09-26-cli-{1,2,3,4}.md` e `.jdi/phases/cli/REVIEW.md` §§ Gate 6 (d), W-4, W-5 — decisões/achados da `cli` que esta phase precisa respeitar.
- `crates/ddc-core/src/domain/{vcp,feature,capabilities}.rs`, `src/app/software_osd.rs`, `src/ports/{monitor_backend,monitor_control}.rs`, `crates/ddc-adapters/src/ddc_hi_backend/hardware.rs` — código exato (pós-iter-2 da `cli`) que esta phase estende.
- `crates/ddc-core/tests/fixtures/rtk_qhd_hdr_caps.txt` — caps real e completa do monitor de dev, já fixada em `core-domain`.
- `.jdi/phases/cli/{CONTEXT.md,PLAN.md,SUMMARY.md,REVIEW.md}` — superfície `ddc-cli` já implementada e `APPROVED` (pendente só de confirmação manual de DoD) que esta phase constrói em cima.
- `.jdi/agents/jdi-doer-ddc-control.md` §§ Monitor-write safety, Errors, Style & lints.

## Out of scope
- Tabela VESA MCCS 2.2 teórica completa (~200 códigos) além dos 33 observados/sondados no "RTK QHD HDR" — extensão para outros monitores fica para quando houver evidência de hardware (D-2026-09-26-full-osd-control-1).
- Escrita de features `Table` (LUTs) — port `write_vcp(u16)` não representa payload multi-byte (D-2026-09-26-full-osd-control-5).
- Conversão Kelvin de 0x0C/0x0B (fórmula VESA não confirmada com leitura correlacionada real) — raw value exposto, presets nomeados de 0x14 já cobrem o caso comum.
- `set`/`reset` em 0x02 (New Control Value) e 0x1E (Auto Setup): NC sem lista do caps e sem valores catalogados — ficam `UnsupportedFeature` por fail-safe (mesmo efeito de antes de D-2026-09-26-cli-1); registrado, não bloqueia.
- Uso da nova superfície a partir de `apps/ddc-tray` → phase `tray-app` (esta phase só toca `ddc-core`/`ddc-adapters`/`ddc-cli`).
- Validação funcional em hardware Windows real → phase `ci-crossbuild`/PR; aqui só `cargo check --target x86_64-pc-windows-msvc` (check-only, D-2026-09-25-ddc-backends-1).

## Definition of Done

### Auto-verifiable
- [ ] Catálogo cobre exatamente os 33 códigos observados (caps + sondados) do "RTK QHD HDR" com o kind/access/risk travados em D-2026-09-26-full-osd-control-1, incluindo a invariante "nenhum código `ReadOnly` é `Dangerous`" e os novos `Dangerous` (0x1E/0x20/0x30/0x7E).
      **Verify:** `cargo test -p ddc-core --locked -- mccs_catalog_covers_every_observed_code_with_the_locked_kind_access_and_risk_including_the_read_only_never_dangerous_invariant 2>&1 | grep -qE 'test result: ok\. 1 passed; 0 failed' && echo OK`
      **Source:** CONTEXT
- [ ] `mccs_catalog::value_name` bate com os nomes de valor declarados para preset (0x14), input (0x60), idioma (0xCC), power (0xD6) e os 4 comandos de reset de fábrica (`00 Reset`).
      **Verify:** `cargo test -p ddc-core --locked -- mccs_catalog_value_names_match_the_declared_lists_for_preset_input_language_power_mode_and_the_factory_reset_commands 2>&1 | grep -qE 'test result: ok\. 1 passed; 0 failed' && echo OK`
      **Source:** CONTEXT
- [ ] `Feature::validate_write` rejeita `FeatureKind::Table` com `UnsupportedFeature` antes de checar access/allowed_values/max.
      **Verify:** `cargo test -p ddc-core --locked -- validate_write_rejects_table_kind_features_before_checking_access_or_allowed_values 2>&1 | grep -qE 'test result: ok\. 1 passed; 0 failed' && echo OK`
      **Source:** CONTEXT
- [ ] `Capabilities::feature(code)` usa `kind`/`access` do catálogo mesmo para um código ausente da capabilities string do monitor (independente do caps, D-core-domain-4).
      **Verify:** `cargo test -p ddc-core --locked -- capabilities_feature_uses_the_catalog_kind_and_access_even_for_a_code_absent_from_the_capabilities_string 2>&1 | grep -qE 'test result: ok\. 1 passed; 0 failed' && echo OK`
      **Source:** CONTEXT
- [ ] `hertz`/`version_from_u16` convertem os valores raw medidos em hardware (14400 -> 144.00 Hz; 0x0202 -> (2,2); 0x0001 -> (0,1)).
      **Verify:** `cargo test -p ddc-core --locked -- hertz_and_version_from_u16_convert_the_raw_frequency_and_version_values_reported_by_the_rtk_monitor 2>&1 | grep -qE 'test result: ok\. 1 passed; 0 failed' && echo OK`
      **Source:** CONTEXT
- [ ] `MonitorControl::probe_undeclared_features` só lê códigos do catálogo ausentes do caps, nunca escreve, distingue `UnsupportedFeature` (não suportado) de `Transport`/`Timeout` (sem resposta) por código, e uma falha num código não aborta a sondagem dos demais.
      **Verify:** `cargo test -p ddc-core --locked -- probe_undeclared_features_reads_only_catalog_codes_absent_from_capabilities_and_distinguishes_unsupported_from_unresponsive_without_writing 2>&1 | grep -qE 'test result: ok\. 1 passed; 0 failed' && echo OK`
      **Source:** CONTEXT
- [ ] `set_feature` aprende o max com UMA leitura para um código `Continuous` respondido mas não declarado no caps, e uma escrita `Safe` com esse max é aceita (prova o teste nomeado previsto por D-2026-09-26-cli-1).
      **Verify:** `cargo test -p ddc-core --locked -- set_feature_learns_the_max_with_one_read_for_a_continuous_code_that_answers_but_is_undeclared_and_a_safe_write_succeeds 2>&1 | grep -qE 'test result: ok\. 1 passed; 0 failed' && echo OK`
      **Source:** CONTEXT
- [ ] `set_feature` NUNCA lê o backend para aprender max de um código `NonContinuous` sem lista conhecida (não declarado, ou caps ilegível) — valida contra a lista do catálogo quando ela existe (aceita/recusa), inclui o caso de um código de reset (`WriteOnly`) aceitando seu único valor catalogado `0`, e recusa `UnsupportedFeature` quando nem caps nem catálogo têm lista.
      **Verify:** `cargo test -p ddc-core --locked -- set_feature_never_reads_a_max_for_a_non_continuous_code_without_a_known_list_and_validates_against_the_catalog_or_refuses_it 2>&1 | grep -qE 'test result: ok\. 1 passed; 0 failed' && echo OK`
      **Source:** CONTEXT
- [ ] `set_feature` recusa um código `Access::ReadOnly` ou `FeatureKind::Table` ANTES de qualquer leitura do backend (sem `ReadVcp` no log do fake).
      **Verify:** `cargo test -p ddc-core --locked -- set_feature_rejects_a_read_only_or_table_code_before_reading_its_value_from_the_backend 2>&1 | grep -qE 'test result: ok\. 1 passed; 0 failed' && echo OK`
      **Source:** CONTEXT
- [ ] `ddc-cli features` (`--fake`) lista por padrão só os códigos declarados com nome/kind/access/valor; com `--probe`, acrescenta os ausentes do caps distinguindo `unsupported` de `unresponsive` sem falhar o comando.
      **Verify:** `cargo test -p ddc-cli --locked -- features_command_shows_declared_codes_by_default_and_adds_probed_codes_distinguishing_unsupported_from_unresponsive_under_probe 2>&1 | grep -qE 'test result: ok\. 1 passed; 0 failed' && echo OK`
      **Source:** CONTEXT
- [ ] `ddc-cli set <nome> <valor-por-nome>` resolve um nome de valor do catálogo para o byte quando declarado (caps) ou catalogado (fallback), e recusa com exit code 4 quando o nome existe no catálogo mas não é aceito por este monitor.
      **Verify:** `cargo test -p ddc-cli --locked -- set_command_resolves_a_catalog_value_name_to_its_byte_when_declared_and_rejects_it_with_exit_four_when_not 2>&1 | grep -qE 'test result: ok\. 1 passed; 0 failed' && echo OK`
      **Source:** CONTEXT
- [ ] `ddc-cli reset <target>` mapeia os 4 alvos nomeados para os códigos de restore-factory corretos, é recusado sem `--yes` (exit 5) e aplicado com `--yes`.
      **Verify:** `cargo test -p ddc-cli --locked -- reset_subcommand_maps_named_targets_to_factory_reset_codes_and_is_refused_without_yes_applied_with_yes 2>&1 | grep -qE 'test result: ok\. 1 passed; 0 failed' && echo OK`
      **Source:** CONTEXT

### Manual
- _(none)_

## Deferred to PR review
- `ddc-cli features`/`features --probe` no hardware real (RTK QHD HDR): todos os códigos declarados + sondados aparecem com nome/valor corretos; 0x62/0x6C/0x6E/0x70/0xC9 aparecem via `--probe` (fora do caps) como `ok`; 0x7E aparece como `unresponsive` (não `unsupported`) sem derrubar o comando.
- `ddc-cli get preset` -> "sRGB" e `ddc-cli get input` -> "DisplayPort-1" no hardware real, confirmando a resolução de `value_name`.
- Escritas SAFE e reversíveis no hardware real (brilho por nome; volume por nome — prova D-2026-09-26-cli-1 no hardware, já pendente da phase `cli`; troca de preset de cor sRGB -> 6500 K -> sRGB) — nunca `input`/`power`/`reset`/OSD lock/qualquer código `Dangerous` no hardware real, com ou sem `--yes`.
- Fórmula de temperatura de cor (0x0C/0x0B) revisitada com uma leitura correlacionada real, caso vire prioridade numa phase futura (registrado como todo em `## Out of scope`).
- Confirmação manual dos 2 itens `Manual` (CHANGELOG/README) da phase `cli`, ainda pendente via `/jdi-confirm-dod cli` — não bloqueia `full-osd-control`, mas precede o `/jdi-ship` da `cli`.

## Notes
- Sugestão de módulo (liberdade do planner): `crates/ddc-core/src/domain/mccs_catalog.rs`, reexportado por `domain/mod.rs`; `probe_undeclared_features` em `app/software_osd.rs` reaproveitando o corpo de `get_feature` via uma função privada compartilhada; o guard `ReadOnly || Table` e a restrição `Continuous` do max-por-leitura entram em `set_feature`/`max_for_write` no mesmo arquivo.
- Nomes de teste acima são contrato mínimo (substring via `cargo test -- <nome>`); módulo/arquivo é liberdade do planner, mesma convenção das phases anteriores.
- README/CHANGELOG cobrindo `features`, `--probe`, `set <nome> <valor-por-nome>` e `reset` são exigidos pela baseline MANUAL de `.jdi/PROJECT.md` (herdada automaticamente pelo Gate 8) — não duplicados aqui.
- Baseline de `.jdi/PROJECT.md` (`cargo test --workspace` verde, cobertura >= 80%, sem TODO/FIXME sem issue) é herdada automaticamente pelo Gate 8 do reviewer.
