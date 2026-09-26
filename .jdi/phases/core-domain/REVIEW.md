# Phase 1: Review  (slug: core-domain)

**Verdict:** APPROVED_PENDING_MANUAL

Re-verify (iter 2) depois da rodada de correção de warnings do `/jdi-issue` (Step 6): `c15f6e8` (W-2), `6022aa1` (W-3), `6fb5d9e` (SUMMARY). Review anterior: `git show ad4632d:.jdi/phases/core-domain/REVIEW.md`.

## Gates
| Gate | Status | Details |
|---|---|---|
| Build | PASS | `cargo build --workspace --locked` exit 0; cross-check `cargo check -p ddc-core -p ddc-adapters --locked --target x86_64-unknown-linux-gnu` exit 0 (host Linux nativo) |
| Tests | PASS | 61 passed, 0 failed, 0 ignored (hardware): ddc-adapters unit 10, ddc-core unit 26, integração `monitor_control` 25, doctests 0. Contra a review anterior (52): +9, sem queda (1 teste substituído, 7 de integração e 2 unitários novos) |
| Coverage | PASS | 100.00% lines (TOTAL: 387 linhas, 0 perdidas; regions 98.38%), threshold 80% (TOTAL row, main.rs/build.rs excluded). `--fail-under-lines 80` exit 0 |
| Lint | PASS | `cargo fmt --all --check` exit 0; `cargo clippy --workspace --all-targets --locked -- -D warnings` exit 0, confirmado também num `CARGO_TARGET_DIR` limpo (sem cache); nenhum `#[allow(...)]`/`#![allow(...)]` no workspace |
| Hexagonal/Safety/Hygiene | PASS | 5.1–5.11 sem achados; 5.10 `cargo audit` 0.22.2 exit 0 (1271 advisories, 8 crates, nenhuma vulnerabilidade): W-1 resolvido |
| Consistency | WARN | todos os `files_modified` do PLAN aparecem em `fe17ef9..HEAD`; commits `fix(core-domain)` com `Refs: D-XX` e sem misturar `.jdi/`; D-1, D-2, D-2026-09-25-core-domain-1..5 conformes (fix round julgado abaixo); W-4 |
| UI Validation | SKIPPED | has_frontend=false |
| DoD | PASS_PENDING_MANUAL | 12/12 auto, 2 manual pending |

### Gate 5: detalhe
- 5.1 deps do core: só `thiserror` em `[dependencies]`; `[dev-dependencies]` só `ddc-adapters`. Sem saída.
- 5.2 I/O / cfg de plataforma no core: sem saída.
- 5.3 `impl MonitorBackend for` no core: sem saída. `pub trait` fora do core: sem saída.
- 5.4 adapter construído fora de composition root / adapter importando adapter: sem saída.
- 5.5 `unsafe`: nenhum no workspace; `#![forbid(unsafe_code)]` em `crates/ddc-core/src/lib.rs`; `[workspace.lints.rust] unsafe_code = "deny"` cobre `ddc-adapters`.
- 5.6 `unwrap`/`expect`/`panic!`/`todo!` fora de teste: sem saída. O fix round usa `lock()` com `unwrap_or_else(PoisonError::into_inner)` e `unwrap_or_else` sobre `Option` (sem panic).
- 5.7 `Risk::Dangerous` em `crates/ddc-core/src/domain/feature.rs:32,122,130`; `Confirm::Yes` só em testes (`feature/tests.rs:82`, `write_features.rs:160,176`); nenhum `DdcHiMonitorBackend` em testes. Ordem de guarda preservada no fix: `authorize_write` continua antes de qualquer chamada ao backend (`software_osd.rs:131`), agora também provado com caps ilegível (`write_features.rs:247` `dangerous_write_is_rejected_before_capabilities_are_fetched`, log vazio).
- 5.8 device paths / nomes de plataforma no core (inclusive `tests/`): sem saída.
- 5.9 comandos Tauri: n/a (`apps/` ausente).
- 5.10 supply chain: `cargo audit` exit 0; `Cargo.lock` não mudou no fix round.
- 5.11 segredos / TODO sem issue: sem saída.
- Skills (dry/kiss/yagni/clean-code/hexagonal) no diff do fix round: sem achado. `CapabilitiesOutcome` é alias usado em 3 pontos; a política "caps ilegível = nada declarado" é orquestração e mora em `app/` (`SoftwareOsd`), não em adapter; nenhuma dep nova; nenhum número mágico. Nit, sem warning: em `hex_bytes` (`capabilities.rs:158-159`) os dois `filter_map(...ok())` não falham mais, porque `is_hex_byte_run` já garante ASCII hex. São redundantes, mas inofensivos.

## Status dos warnings da review anterior (ad4632d)
- **W-1 RESOLVIDO**: `cargo-audit` 0.22.2 instalado; `cargo audit` exit 0, sem vulnerabilidades nas 8 crates do lock.
- **W-2 RESOLVIDO** (`c15f6e8`): caps ilegível/desbalanceado não bloqueia mais `get_feature`/`set_feature` (`software_osd.rs:56-63`, `capabilities_or_empty`) e a falha é memorizada, sem refetch a cada leitura (`software_osd.rs:42-51`). Provado por `read_features.rs:94` (`unbalanced_capabilities_do_not_block_reads`, `ReadVcp` no log, `declared_in_capabilities == false`) e `read_features.rs:121` (`ReadCapabilities` uma vez para duas leituras).
- **W-3 RESOLVIDO** (`6022aa1`): `hex_bytes` separa por whitespace e descarta token ímpar ≥3 ou não-hex (`capabilities.rs:154-167`). Os exemplos da review anterior agora dão `"10 bad 12"` → `[0x10, 0x12]` e `ace` não declara nada (`capabilities/tests.rs:110,117`). `"10 1 12"` → `[0x10, 0x01, 0x12]` fica mantido de propósito (dígito isolado = byte, documentado). A fixture RTK passa sem alteração.

## Fix round julgado contra D-2026-09-25-core-domain-4 / -5 (Gate 6)
- **Falha de caps memorizada por monitor** (`software_osd.rs:42-51`): conforme D-4. A leitura nunca é filtrada nem bloqueada pelo caps, e `declared_in_capabilities` continua derivado de `vcp.contains_key` (vazio → `false`). Isso remove a contradição com o rationale de D-4 que motivou o W-2.
- **`MonitorNotFound` não memorizado** (`software_osd.rs:47`, `:59`): correto. Monitor ausente propaga como erro antes de `read_vcp` e não vira falha de caps em cache (`read_features.rs:162`, dois `ReadCapabilities` no log).
- **`capabilities()` explícito refaz a leitura depois de falha memorizada** (`software_osd.rs:106-111`): conforme D-5, que manda desbalanceado continuar sendo `Transport`, agora exposto só pelo caminho explícito (`read_features.rs:140`). Sem deadlock: o guard do `if let` é liberado antes de `fetch_capabilities` (edition 2024), e o teste chama o caminho duas vezes. O ramo "retry bem-sucedido substitui a falha no cache" não tem teste; ver W-4.
- **`set_feature` sem caps** (`software_osd.rs:131-139`): conforme D-4. Sem leitura prévia → `UnsupportedFeature`, sem escrever às cegas (`write_features.rs:189`, log só `ReadCapabilities`). Depois de `get_feature`, vale o max cacheado, a "leitura prévia no mesmo processo" que D-4 permite (`write_features.rs:202`).
- **Tokens como `10,12` agora descartados** (efeito colateral declarado no SUMMARY): conforme D-5. A tolerância travada cobre ausência de espaço, tag desconhecida e garbage depois do fechamento, sem nunca gerar erro, e isso continua valendo (`tolerates_missing_spaces_and_unknown_tags` verde; `0c10` → `[0x0C, 0x10]`). Separador não-whitespace não faz parte do contrato de D-5, e descartar é a direção fail-safe: menos códigos declarados exigem leitura prévia para escrever, e lista NC menor aceita menos valores.
- Desvio de premissa: a PLAN A-5 ("falha de leitura/parse propaga") foi invertida para `get_feature`/`set_feature`. **Aceito**: A-5 é premissa autônoma, não decisão locked; a mudança atende ao W-2 e está registrada no SUMMARY, no README e no CHANGELOG.

## Blockers
- (nenhum)

## Warnings
- **W-4 (Gate 6, consistência plano/teste: comportamento documentado sem teste):** `crates/ddc-core/src/app/software_osd.rs:102-111`. O doc do impl promete que `capabilities()` "replaces it in the cache once it succeeds — this is how a caller recovers from a transient capabilities failure", e o README repete a promessa, mas o caminho falha memorizada → retry com sucesso → cache substituído → `get_feature` passa a ver o caps parseado não é exercitado por nenhum teste. O próprio SUMMARY declara a lacuna: o fake não tem script de "falha N vezes, depois responde". Cobertura de linhas é 100% porque a inserção é compartilhada com o caminho feliz, então o gate numérico não pega isso. Risco baixo, mas é o único caminho de recuperação documentado na API pública. Sugestão: dar ao `FakeMonitor` um modo de falha transitória de caps em `ddc-adapters` (dado de teste, sem regra de negócio) e cobrir a recuperação. Pode ir para `ddc-backends`, onde falha transitória de hardware passa a ser real.

## DoD Checklist (gate 8)

| # | Criterion | Source | Type | Status | Evidence |
|---|---|---|---|---|---|
| 1 | `cargo test --workspace` exits 0 | PROJECT | Auto | PASS | reaproveitado do Gate 2: exit 0, 61 passed, 0 failed |
| 2 | Coverage >= 80% of lines | PROJECT | Auto | PASS | reaproveitado do Gate 3: TOTAL Lines 100.00% (387 linhas, 0 perdidas) |
| 3 | No `TODO`/`FIXME` without linked issue reference | PROJECT | Auto | PASS | grep sem hits (exit 0) |
| 4 | CHANGELOG.md updated with entry per release | PROJECT | Manual | MANUAL_REQUIRED | suggested: único heading de versão é `## [Unreleased]` (CHANGELOG.md:8), atualizado em `c15f6e8` com o comportamento de caps ilegível; ainda sem `## [x.y.z]` (nenhuma release cortada) |
| 5 | README accurately describes current behavior | PROJECT | Manual | MANUAL_REQUIRED | suggested: status "Phase 1 (`core-domain`) is implemented" (README.md:5); § Monitor-write safety atualizado em `c15f6e8` (caps ilegível = nada declarado, falha memorizada, só `capabilities` explícito refaz), coerente com `software_osd.rs:56-63,106-111` |
| 6 | Workspace root é `[workspace]` puro com members exatamente `crates/ddc-core` e `crates/ddc-adapters`; `src/main.rs` removido | CONTEXT | Auto | PASS | OK (exit 0); `members = ["crates/ddc-core", "crates/ddc-adapters"]`; diretório `src/` ausente |
| 7 | `crates/ddc-core/Cargo.toml` `[dependencies]` = somente `thiserror` (D-2) | CONTEXT | Auto | PASS | OK (exit 0) |
| 8 | Workspace compila (incluindo dev-dependency cíclica do fake) | CONTEXT | Auto | PASS | `cargo build --workspace --locked` → OK |
| 9 | Teste nomeado prova o parser contra a caps real e completa do RTK QHD HDR | CONTEXT | Auto | PASS | `domain::capabilities::tests::parses_real_rtk_caps_string ... ok` (1 passed); fixture de 259 chars byte a byte igual à da CONTEXT em `capabilities/tests.rs` e `monitor_control/support.rs`; teste não foi alterado no fix round |
| 10 | Teste nomeado prova tolerância do parser | CONTEXT | Auto | PASS | `domain::capabilities::tests::tolerates_missing_spaces_and_unknown_tags ... ok` (1 passed); verde também com o novo `hex_bytes` |
| 11 | Escrita `Dangerous` sem `Confirm::Yes` → `DangerousWriteNotConfirmed`, sem chamar o backend | CONTEXT | Auto | PASS | `write_features::dangerous_write_without_confirm_is_rejected ... ok` (1 passed); inalterado no fix round |
| 12 | `value > max` → `InvalidValue` antes de qualquer chamada ao backend | CONTEXT | Auto | PASS | `write_features::write_value_above_max_is_rejected ... ok` (1 passed) |
| 13 | Read-back após write | CONTEXT | Auto | PASS | `write_features::write_reads_back_value_after_success ... ok` (1 passed) |
| 14 | `get_feature` não rejeita código ausente do caps (D-2026-09-25-core-domain-4) | CONTEXT | Auto | PASS | `read_features::get_feature_succeeds_for_code_absent_from_capabilities ... ok` (1 passed) |

**Totals:** 14 items | Auto: 12 (12 PASS, 0 FAIL) | Manual: 2 pending

**Manual confirmation required:**
Run `/jdi-confirm-dod core-domain` to confirm each manual item with evidence. Without that, `/jdi-ship` will refuse the phase.

## Recommendation
A phase continua aprovada nos gates automáticos: build, testes (61/61), 100% de linhas, fmt/clippy limpos, pureza hexagonal e segurança de escrita sem achados, e `cargo audit` limpo. Os três warnings anteriores foram resolvidos, e as mudanças de comportamento do fix round estão conformes a D-2026-09-25-core-domain-4 e -5. Falta a confirmação humana dos 2 itens manuais do PROJECT (CHANGELOG, README) via `/jdi-confirm-dod core-domain`; no modo autônomo, isso fica para a revisão do PR. O W-4 não bloqueia. Recomendo não abrir outra rodada de correção só por ele e levá-lo para `ddc-backends`, junto com o fake de falha transitória.

Observações para phases futuras (não são findings desta phase):
- **ddc-backends:** o adapter real precisa mapear "monitor sumiu" para `MonitorNotFound`, e não para `Transport`/`Timeout`. Caso contrário, a falha fica memorizada como "caps ilegível" até alguém chamar `capabilities()`. Pelo mesmo motivo, um `Timeout` transitório no primeiro `get_feature` fica memorizado pela vida do `SoftwareOsd`.
- **ddc-backends:** NUL ou caractere de controle *no meio* da caps agora descarta o token vizinho (cópia de `hex_bytes` rodada fora do repo: `"10 12\0 14"` → `[0x10, 0x14]`; antes o NUL separava). `outer_body` só apara nas pontas. Se o adapter concatenar fragmentos com terminador, normalize lá.
- **full-osd-control:** com caps ilegível, os códigos NC `Safe` (0x14, 0xCC) caem para `Continuous` e aceitam qualquer valor ≤ max lido. `write_features.rs:232` grava 0x03 em 0x14, fora da lista RTK. É conforme D-4 (max de leitura prévia) e D-2026-09-25-core-domain-2 (`allowed_values` só vem do caps), mas a PLAN A-4 já avisava que o max de NC não é confiável: o catálogo deve dar `kind`/valores NC independentes do caps.
- **cli / tray-app:** `declared_in_capabilities == false` passou a cobrir tanto "não declarado" quanto "caps ilegível", porque o shape `bool` está travado em D-4. Para distinguir os casos, chame `capabilities()`. Processos longos (tray) só se recuperam de falha transitória por essa chamada explícita, então prevejam um refresh.
- Da review anterior, ainda válidas: `WriteOnly` (0x04) vs read-back obrigatório; manter fonte única de risco entre `authorize_write` e `Feature.risk`; falha no read-back depois de uma escrita bem-sucedida é indistinguível de falha na escrita.

## DoD Critic (enhanced)

Forçado pelo `/jdi-issue` na re-verificação pós fix round: 0 linhas hollow entre as Auto PASS (critic retornou `[]`).

**Verdict:** APPROVED
