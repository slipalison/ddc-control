# Phase 1: Review  (slug: core-domain)

**Verdict:** APPROVED_PENDING_MANUAL

## Gates
| Gate | Status | Details |
|---|---|---|
| Build | PASS | `cargo build --workspace --locked` exit 0; cross-check `cargo check -p ddc-core -p ddc-adapters --locked --target x86_64-unknown-linux-gnu` exit 0 (host Linux nativo) |
| Tests | PASS | 52 passed, 0 failed, 0 ignored (hardware) — ddc-adapters unit 10, ddc-core unit 24, integração `monitor_control` 18, doctests 0. Phase 1: sem baseline anterior (crate raiz era hello world sem testes) |
| Coverage | PASS | 100.00% lines (TOTAL: 367 linhas, 0 perdidas; regions 98.29%), threshold 80% (TOTAL row, main.rs/build.rs excluded) — `--fail-under-lines 80` exit 0. Único arquivo excluído: `crates/ddc-core/tests/monitor_control/main.rs` (harness de 5 linhas, só `mod`) |
| Lint | PASS | `cargo fmt --all --check` exit 0; `cargo clippy --workspace --all-targets --locked -- -D warnings` exit 0; nenhum `#[allow(...)]`/`#![allow(...)]` no workspace (nem em testes) |
| Hexagonal/Safety/Hygiene | WARN | 5.1–5.9 e 5.11 sem achados; 5.10 WARN (cargo-audit não instalado) — ver W-1 |
| Consistency | WARN | todos os `files_modified` do PLAN aparecem em `fe17ef9..HEAD`; commits com scope `core-domain` e tipos coerentes; desvios do SUMMARY aceitos no mérito; D-1, D-2, D-2026-09-25-core-domain-1..5 conformes; W-2 (suspeita sobre D-2026-09-25-core-domain-4) e W-3 (acceptance da T-3) |
| UI Validation | SKIPPED | has_frontend=false |
| DoD | PASS_PENDING_MANUAL | 12/12 auto, 2 manual pending |

### Gate 5 — detalhe
- 5.1 deps do core: só `thiserror` — sem saída.
- 5.2 I/O / cfg de plataforma no core: sem saída.
- 5.3 `impl MonitorBackend for` no core: sem saída; `pub trait` fora do core: sem saída. Os dois ports (`MonitorBackend`, `MonitorControl`) vivem em `ddc_core::ports`, só assinaturas.
- 5.4 adapter construído fora de composition root / adapter importando adapter: sem saída (fake só montado em `crates/ddc-core/tests/monitor_control/support.rs`).
- 5.5 `unsafe`: nenhum no workspace; `#![forbid(unsafe_code)]` em `crates/ddc-core/src/lib.rs:8`; `[workspace.lints.rust] unsafe_code = "deny"` cobre `ddc-adapters`.
- 5.6 `unwrap`/`expect`/`panic!`/`todo!` fora de teste: sem saída (os `lock()` usam `unwrap_or_else(PoisonError::into_inner)`).
- 5.7 `Risk::Dangerous` presente em `crates/ddc-core/src/domain/feature.rs:32,122,130`; `Confirm::Yes` só em testes (`feature/tests.rs:82`, `write_features.rs:158,174`); nenhum `DdcHiMonitorBackend` em testes.
- 5.8 device paths / nomes de plataforma no core: sem saída (inclusive em testes, após `5a419c9`).
- 5.9 comandos Tauri: n/a (`apps/` ausente).
- 5.10 supply chain: WARN (W-1).
- 5.11 segredos / TODO sem issue: sem saída.
- Skills (dry/kiss/yagni/clean-code/hexagonal): sem achado bloqueante. Fixture RTK em 2 arquivos (unit + integração, fronteira de crate explicada em A-6) e helper `lock()` em 2 crates ficam abaixo da regra de 3 do DRY. `MonitorControl` com 1 impl e `SoftwareOsd<B>` genérico com 1 tipo hoje são exigidos pelo Hexagonal locked (D-1) e ganham o adapter real em `ddc-backends`. Variantes ainda sem produtor (`FeatureKind::Table`, `Access::ReadOnly/WriteOnly`, `DdcError::Timeout`) são exigidas por D-2026-09-25-core-domain-2 / PLAN T-2 — não é YAGNI. Sem número mágico fora de `vcp.rs` e testes.

## Blockers
- (nenhum)

## Warnings
- **W-1 (5.10 supply chain):** `cargo-audit` não instalado — sem auditoria automática. Deps novas no `Cargo.lock`: `thiserror` 2.0.21, `thiserror-impl`, `proc-macro2`, `quote`, `syn`, `unicode-ident`. Checagem manual: `https://rustsec.org/packages/thiserror.html` responde 404 (rustsec só publica página de pacote com advisory) e a busca em rustsec.org não trouxe advisory para essas crates. Instalar: `cargo install cargo-audit --locked`.
- **W-2 (Gate 6, D-2026-09-25-core-domain-4 — suspeita, não violação confirmada):** `crates/ddc-core/src/app/software_osd.rs:83` — `get_feature` executa `load_capabilities(id)?` antes de `read_vcp`; se a capabilities string não vier ou vier desbalanceada, a leitura nunca chega ao monitor (comportamento fixado por `crates/ddc-core/tests/monitor_control/read_features.rs:92`). A letra de D-4 (código ausente do caps não filtra a leitura) está cumprida (`read_features.rs:8`), e o PLAN A-5 declarou a propagação. Mas o rationale de D-4 ("o core já precisa aceitar leitura sem caps hoje") e o fato de scalers comumente falharem no capabilities request enquanto respondem a get VCP indicam risco: um monitor com caps ilegível fica totalmente ilegível e ingravável pelo core. Falhas também não são cacheadas, então cada `get_feature` repete a leitura de caps (lenta em hardware real). Sugestão para `ddc-backends`/`full-osd-control`: tratar caps indisponível como `Capabilities::default()` (`declared_in_capabilities = false`), sem breaking change na API pública — e registrar a escolha como D-XX.
- **W-3 (Gate 6 plan consistency + clean-code):** `crates/ddc-core/src/domain/capabilities.rs:148-156` — o doc de `hex_bytes` diz "two digits each" e a acceptance da T-3 diz "hex inválido ... ignorados", mas um dígito isolado e palavras feitas só de letras hex viram códigos: `"10 1 12"` → `[0x10, 0x01, 0x12]`; `"10 bad 12"` → `[0x10, 0xBA, 0x0D, 0x12]` (confirmado rodando uma cópia da função fora do repo). O teste `invalid_hex_tokens_and_junk_between_tags_are_skipped` (`crates/ddc-core/src/domain/capabilities/tests.rs:101`) só cobre lixo não-hex (`zz`, `--`, `??`). Impacto de segurança baixo (código espúrio cai em `Dangerous`, salvo colisão com a seed Safe), mas um código espúrio passa a `declares() == true` e libera a leitura de max em `max_for_write`. Sugestão: descartar runs de comprimento ímpar e cobrir com teste.

## Desvios do PLAN julgados (SUMMARY § Desvios do PLAN)
- T-5 tocou `crates/ddc-adapters/src/in_memory.rs` (+ `in_memory/tests.rs`): clones do fake compartilham estado via `Arc`. **Aceito** — permite inspecionar o log sem getter de backend no `SoftwareOsd` (que abriria bypass das regras de escrita) e sem `impl MonitorBackend for &T` no core (gate 5.3). O fake continua sem regra de negócio (hexagonal regra 14) e o comportamento tem teste (`clones_share_monitors_and_call_log`).
- T-6 tocou `crates/ddc-core/src/domain/mod.rs` só para re-exportar `authorize_write`: trivial e necessário. **Aceito.**
- Commit extra `5a419c9` (`test(core-domain)`): tipo e scope corretos, isolado da T-5, corrige comentário que o gate 5.8 bloquearia. **Aceito.**
- `max_for_write` recebe `&Capabilities` em vez de `bool`: evita parâmetro booleano (clean-code). **Aceito.**
- `Feature::validate_write` recusa `ReadOnly`: fora do PLAN, mas é regra de segurança pura no domínio, sem custo e coberta (`feature.rs` 100% de linhas). **Aceito.**

## DoD Checklist (gate 8)

| # | Criterion | Source | Type | Status | Evidence |
|---|---|---|---|---|---|
| 1 | `cargo test --workspace` exits 0 | PROJECT | Auto | PASS | `cargo test --workspace --locked && echo OK` → OK (52 passed, 0 failed) |
| 2 | Coverage >= 80% of lines | PROJECT | Auto | PASS | TOTAL Lines 100.00% (367 linhas, 0 perdidas) — reaproveitado do Gate 3 |
| 3 | No `TODO`/`FIXME` without linked issue reference | PROJECT | Auto | PASS | grep sem hits → OK (exit 0) |
| 4 | CHANGELOG.md updated with entry per release | PROJECT | Manual | MANUAL_REQUIRED | suggested: `CHANGELOG.md` criado em `e8de5ba` (Keep a Changelog) com `## [Unreleased]` listando as adições da phase; ainda sem heading `## [x.y.z]` (nenhuma release cortada) |
| 5 | README accurately describes current behavior | PROJECT | Manual | MANUAL_REQUIRED | suggested: README reescrito em `e8de5ba` — status "Phase 1 (`core-domain`) is implemented", layout real dos 2 crates, seção "Monitor-write safety" coerente com `software_osd.rs` |
| 6 | Workspace root é `[workspace]` puro com members exatamente `crates/ddc-core` e `crates/ddc-adapters`; `src/main.rs` removido | CONTEXT | Auto | PASS | OK (exit 0); diretório `src/` ausente |
| 7 | `crates/ddc-core/Cargo.toml` `[dependencies]` = somente `thiserror` (D-2) | CONTEXT | Auto | PASS | OK (exit 0) |
| 8 | Workspace compila (incluindo dev-dependency cíclica do fake) | CONTEXT | Auto | PASS | `cargo build --workspace --locked` → OK |
| 9 | Teste nomeado prova o parser contra a caps real e completa do RTK QHD HDR | CONTEXT | Auto | PASS | `domain::capabilities::tests::parses_real_rtk_caps_string ... ok`; fixture byte-a-byte igual à string da CONTEXT (259 chars), nos dois arquivos que a usam |
| 10 | Teste nomeado prova tolerância do parser | CONTEXT | Auto | PASS | `domain::capabilities::tests::tolerates_missing_spaces_and_unknown_tags ... ok` |
| 11 | Escrita `Dangerous` sem `Confirm::Yes` → `DangerousWriteNotConfirmed`, sem chamar o backend | CONTEXT | Auto | PASS | `write_features::dangerous_write_without_confirm_is_rejected ... ok`; assert `backend.calls().is_empty()` (0x60 e 0x52) |
| 12 | `value > max` → `InvalidValue` antes de qualquer chamada ao backend | CONTEXT | Auto | PASS | `write_features::write_value_above_max_is_rejected ... ok`; log idêntico ao anterior ao `set_feature` |
| 13 | Read-back após write | CONTEXT | Auto | PASS | `write_features::write_reads_back_value_after_success ... ok`; log termina em `WriteVcp` → `ReadVcp`; com quirk retorna o valor antigo |
| 14 | `get_feature` não rejeita código ausente do caps (D-2026-09-25-core-domain-4) | CONTEXT | Auto | PASS | `read_features::get_feature_succeeds_for_code_absent_from_capabilities ... ok`; `declared_in_capabilities == false`, `ReadVcp(0x62)` no log |

**Totals:** 14 items | Auto: 12 (12 PASS, 0 FAIL) | Manual: 2 pending

**Manual confirmation required:**
Run `/jdi-confirm-dod core-domain` to confirm each manual item with evidence. Without that, `/jdi-ship` will refuse the phase.

## Recommendation
Phase aprovada nos gates automáticos: build, testes (52/52), cobertura 100% de linhas, fmt/clippy limpos, pureza hexagonal e segurança de escrita sem achados, e todas as decisões locked conformes. Falta só a confirmação humana dos 2 itens manuais do PROJECT (CHANGELOG, README) via `/jdi-confirm-dod core-domain` — no modo autônomo, levar para a revisão do PR. Os 3 warnings não bloqueiam: W-1 some ao instalar `cargo-audit`; W-2 e W-3 devem virar decisão/tarefa em `ddc-backends` ou `full-osd-control`.

Observações para phases futuras (não são findings desta phase):
- Features `WriteOnly` (em MCCS, 0x04 restore factory defaults é write-only): quando o catálogo de `full-osd-control` marcar um código assim, o read-back obrigatório e a leitura de max de `set_feature` vão falhar — decidir o contrato antes.
- `authorize_write` usa `risk_for_code(code)` enquanto `Feature.risk` também carrega o risco; hoje é a mesma fonte, mas com o catálogo mantenha uma fonte única (o gate precisa continuar dependendo só do código, pois roda antes de ler o caps).
- Uma falha no read-back depois de um write bem-sucedido volta como `Err`, indistinguível de falha na escrita — a CLI e o tray devem formular a mensagem sabendo disso.

## DoD Critic (enhanced)

Forçado pelo `/jdi-issue`. 0 linhas hollow entre as 12 Auto PASS: cada critério foi conferido no artefato real (fixture RTK byte a byte, asserções substantivas, `-- --list` com 1 match por filtro).
Observações não-hollow (Verify mais fraco que o critério; endurecer em phase futura): `grep 'crates/ddc-core'` casa também `[workspace.dependencies]` e não prova "exatamente 2 members"; o awk de deps do core não vê `[target.*.dependencies]`; `cargo build` não compila a dev-dep cíclica (coberto por `cargo test`); `grep '1 passed'` casaria `11 passed` — usar `-- --exact` + `test result: ok\. 1 passed; 0 failed`.

**Verdict:** APPROVED
