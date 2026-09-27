# Phase 5: Review  (slug: tray-app)

**Verdict:** APPROVED_PENDING_MANUAL

Iter 13 (rodada 3). Revisão completa, escrita do zero em `03a768a`, no Linux (Fedora 44, KDE Plasma Wayland), em 2026-09-27. Os commits da iteração são:
- `c181544` (fix): `tauri/custom-protocol` ligado no `[dependencies]`;
- `b1b6389` (test): `build_tests::the_tray_is_not_a_dev_build`;
- `03a768a` (docs): SUMMARY.

Cada `Verify:` do CONTEXT.md (19) e do PROJECT.md (3) foi extraído do `.md` por script e rodado literalmente com `bash` a partir da raiz. Nesse `bash`, `grep` é `/usr/bin/grep`, e `DDC_HW_TESTS`, `DDC_TRAY_FAKE` e `DDC_TRAY_DEBUG` ficaram fora do ambiente. Nenhum teste `#[ignore]` rodou, e nada foi escrito no monitor real. O C11 usou o backend real só com leituras, e o C17 usou o monitor simulado.

Desde a revisão da iter 12 (`19752b8`), fora de `.jdi/` mudaram só `apps/ddc-tray/src-tauri/Cargo.toml` (+3/−1) e `apps/ddc-tray/src-tauri/src/lib.rs` (+11). `Cargo.lock`, `package*.json`, `src/` do popup e o harness não mudaram.

## Gates
| Gate | Status | Details |
|---|---|---|
| Build | PASS | `cargo build --workspace --locked` exit 0. O cross-check `cargo check -p ddc-core -p ddc-adapters -p ddc-cli --locked --target x86_64-pc-windows-msvc` deu exit 0. O tray fica fora dele, pela D-2026-09-26-tray-app-9 |
| Tests | PASS | 384 passed, 0 failed, 9 ignored (hardware). São +1 em relação à iter 12, o teste novo `the_tray_is_not_a_dev_build`. `node --test`: 153 pass e 0 fail/cancelled/skipped/todo |
| Coverage | PASS | 83.23% lines, threshold 80%. Vem da linha TOTAL, com `main.rs`/`build.rs` excluídos, e o `--fail-under-lines 80` saiu com exit 0. O comando literal do PROJECT, sem exclusões, dá 82.79% |
| Lint | PASS | `cargo fmt --all --check` exit 0 e `cargo clippy --workspace --all-targets --locked -- -D warnings` exit 0. Nenhum `#[allow(` fora de testes |
| Hexagonal/Safety/Hygiene | WARN | 5.1 a 5.9 e 5.11 sem achados. 5.10 = W-1 (conhecido) |
| Consistency | PASS | 3 commits com scope `tray-app` e tipos coerentes (`fix`, `test`, `docs`). Código e `.jdi/` nunca no mesmo commit. As 8 tasks do PLAN estão `completed`. Conforme a D-2026-09-27-tray-app-10, a D-1, a D-2 e as demais D-tray-app (ver abaixo) |
| UI Validation | PASS | Playwright: 134 passed, 6 skipped (= screenshots), 0 failed/flaky; `n=10`, `dr=2`, `ps=54`, `dd=dl=16`, `sk=sl=6`. `pseudo-locale` com 58 ✓ e `confirm.spec` com 14 ✓. A porta 1420 estava livre antes e depois |
| DoD | PASS_PENDING_MANUAL | 22/22 auto e 2 manual pendentes |

### Detalhes por check (Gate 5)
- **5.1** (deps do core): nenhuma além de `thiserror`.
- **Sem hits:**
  - **5.2**: I/O ou `cfg` de plataforma no core;
  - **5.3a**: `impl MonitorBackend` no core;
  - **5.3b**: `pub trait` fora do core;
  - **5.4**: adapter construído fora do composition root, ou adapter que importa adapter.
- **5.5**:
  - `unsafe` fora de `ddc-adapters`: o único hit é `apps/ddc-tray/src-tauri/src/lib.rs:146`, dentro do doc comment que explica por que o código não usa `set_var`. Não é código.
  - Nenhum `unsafe {` em `ddc-adapters`.
  - O `forbid(unsafe_code)` está nos três crate roots.
- **5.6**:
  - os `unwrap` de `kwin_placement.rs:246-267`, `stop_signals.rs:105-108` e `lib.rs:269` ficam dentro de `#[cfg(test)]`, nas linhas `kwin_placement.rs:195`, `stop_signals.rs:82` e `lib.rs:239`;
  - o `assert!` novo fica em `#[cfg(test)] mod build_tests` (`lib.rs:304-313`);
  - o `panic!` de `worker.rs:141` é doc comment.
- **5.7**:
  - `Risk::Dangerous` aparece em `crates/ddc-core/src/domain/feature.rs:33`, `:156` e `:163`.
  - Nenhum `Confirm::Yes` fora de `ddc-cli/src` e de `src-tauri/src/commands*`.
  - `crates/ddc-adapters/tests/real_monitor.rs` tem 7 de 7 testes `#[ignore]`, gated por `DDC_HW_TESTS`. Em `apps/`, o `DdcHiMonitorBackend` só aparece no composition root (`lib.rs:28`, `:77`).
- **5.8** e **5.9**: nenhum hit.
- **5.10** = W-1.
- **5.11**: nenhum hit.

### Mudança desta iteração (avaliada)
- **Código:**
  - `tauri = { version = "2.12.0", features = ["image-png", "custom-protocol"] }` no `[dependencies]` (`apps/ddc-tray/src-tauri/Cargo.toml:27`), com comentário que cita a D-10;
  - o teste `assert!(!tauri::is_dev(), "tauri/custom-protocol is off")` fica num `#[cfg(test)]` sem `target_os`, então roda em todo SO.
- **Mecanismo, conferido na fonte do registry:**
  - `tauri-2.12.0/src/lib.rs:317`: `is_dev()` é `!cfg!(feature = "custom-protocol")`;
  - `tauri-2.12.0/src/manager/mod.rs:361-372`: só no modo dev serve `dev_csp.or(csp)`;
  - `tauri-codegen-2.7.0/src/context.rs:161-169`: faz a mesma escolha para as asset CSPs;
  - `tauri-macros-2.7.0/src/context.rs:155`: `dev: cfg!(not(feature = "custom-protocol"))`.
- **A feature vale nos dois alvos.** `cargo tree -p ddc-tray --locked -e normal,features -i tauri` lista `tauri feature "custom-protocol"` 1 vez para `x86_64-unknown-linux-gnu` e 1 vez para `x86_64-pc-windows-msvc`. A entrada `cfg(not(target_os = "linux"))` é unificada com a do `[dependencies]`.
- **`Cargo.lock` intocado** (`git diff 9dbe794 HEAD -- Cargo.lock` vazio).
- **Configuração:** `tauri.conf.json` é o único arquivo de config em `src-tauri/`, com `devCsp=null` e `devUrl=null`. Não há `*.json5` nem `Tauri*.toml`.
- **Conformidade:**
  - com a D-2026-09-27-tray-app-10: feature no `[dependencies]`, nenhum `devCsp`/`devUrl` e o teste pedido existe;
  - com a D-2026-09-26-tray-app-7, a CSP de produção continua estrita.

## Blockers
Nenhum.

## Warnings
- **W-1 (5.10, conhecido e adiado para `ci-crossbuild`):** o `cargo audit` 0.22.2 acusa 1 vulnerabilidade e 3 warnings permitidos.
  - Vulnerabilidade: RUSTSEC-2018-0005 (`serde_yaml` 0.7.5, transitivo do `ddc-hi`).
  - Warnings: RUSTSEC-2024-0370 (`proc-macro-error`, unmaintained), RUSTSEC-2024-0320 (`yaml-rust`, unmaintained) e RUSTSEC-2024-0429 (`glib` 0.18.5, unsound, transitivo do Tauri/GTK).
  - `Cargo.lock` intocado nesta iteração.
  - Informativo: o `nom` v3.2.1 emite aviso de future-incompat no build (transitivo, anterior a esta phase).
- **W-2 (Gate 8, C8: o Verify prova menos do que a D-10 e a própria linha afirmam; o código do HEAD está conforme).** Reproduzi as duas vias abaixo, (a) e (b), em cópias descartáveis de `git archive HEAD` no scratchpad, com target dir próprio. As cópias já foram apagadas e o repo não foi tocado. Nas duas, o C8 literal imprime `OK`.
  - **(a) Uma config `Tauri.linux.toml` sozinha escapa.**
    - **O guard falha:** `ls $c/tauri*.json5 $c/Tauri*.toml >/dev/null 2>&1 && ok=0` só reprova quando existem **os dois** tipos de arquivo. Com um só, o outro glob fica literal e o `ls` sai com 2. Conferido: só json5 → rc 2; só toml → rc 2; os dois → rc 0.
    - **Os `jq` não veem o arquivo:** eles só leem `tauri.conf.json` e `tauri.*.conf.json`.
    - **A mutação:** `features = [..., "config-toml"]` no `tauri` e no `tauri-build`, mais um `Tauri.linux.toml` com `csp = "default-src 'self'; script-src 'self' 'unsafe-inline' 'unsafe-eval' https://reviewer-probe.invalid; style-src 'self' 'unsafe-inline'"`.
    - **O lock não muda:** no `tauri-utils` 2.10.0, `config-toml = []`, e o `toml` já é dependência obrigatória. Por isso o `--locked` passa.
    - **Resultado:**
      - o C8 imprime `OK`;
      - o binário compilado da cópia contém a política frouxa inteira. O marcador foi encontrado 1× no binário mutado e 0× no `target/debug/ddc-tray` do repo;
      - o `read_from` do `tauri-utils` (`config/parse.rs:180-205`) mescla a config de plataforma por RFC 7396;
      - e, fora do modo dev, o Tauri serve `app.security.csp` (`manager/mod.rs:361-372`).
    - **A falha é anterior à iter 13:** esse guard já estava no C8 da iter 12.
  - **(b) A feature só em `[dev-dependencies]` também passa.** O `grep -qE '^tauri[[:space:]]*=.*"custom-protocol"'` casa com qualquer linha `tauri =`, inclusive a de `[dev-dependencies]`.
    - **A mutação:** tirar a feature do `[dependencies]` e acrescentar `[dev-dependencies] tauri = { version = "2.12.0", features = ["custom-protocol"] }`.
    - **Por que passa:** o `cargo test` unifica a feature, então `is_not_a_dev_build` dá `1 passed; … 136 filtered out`, e o C8 imprime `OK`.
    - **O build normal/release fica sem a feature:** `cargo tree -e normal,features -i tauri` não a lista (0 contra 1 no HEAD). Esse é exatamente o build "dev" que a D-10 proíbe.
    - **Impacto:** sozinha, (b) não afrouxa a CSP, porque nenhum `devCsp` passa no `jq` dos `.json`. Junto com (a), sim: um `devCsp` frouxo no `Tauri.linux.toml` bastaria.
  - **Além disso, o corpo do teste não está travado.** `lib.rs` fica fora do manifesto do harness. Um `fn the_tray_is_not_a_dev_build() {}` com a feature só num comentário da linha `tauri =` (a M2 do SUMMARY) passaria no grep e no teste.
  - **Endurecimento sugerido.** Cada item foi testado no HEAD, onde passa, e reprova a mutação correspondente:
    1. trocar o `ls` por `for g in $c/tauri*.json5 $c/Tauri*.toml; do [ -e "$g" ] && ok=0; done`, somado a `! grep -qE 'config-(json5|toml)' $c/Cargo.toml`;
    2. trocar o grep da linha por `for t in x86_64-unknown-linux-gnu x86_64-pc-windows-msvc; do cargo tree -p ddc-tray --locked -e normal,features -i tauri --target $t | grep -qF 'tauri feature "custom-protocol"' || ok=0; done`. Ele resolve sem compilar e prova também o alvo Windows. No HEAD dá 1/1, e na mutação (b), 0;
    3. travar `grep -qF 'assert!(!tauri::is_dev()' apps/ddc-tray/src-tauri/src/lib.rs`.

## Observações (sem efeito no veredito)
- **O binário instalado ainda é um build "dev" do Tauri.** O `/home/slipalison/.local/bin/ddc-tray` é de 03:10, anterior ao `c181544` (08:28). Nenhum `tauri.conf.json` versionado teve `devCsp`, então ele serve a `csp` estrita. Reinstalá-lo a partir do HEAD fica com o orquestrador/usuário.
- **Protocolo da instância do usuário:**
  - encerrada com `pkill -x ddc-tray` antes do C11 e antes do C17, esperando o processo sair;
  - reaberta com `setsid -f` logo depois de cada um;
  - PIDs: 2119597 → 2155780 → 2156309, que é a única instância viva no fim.
- **Smokes:**
  - o C11 passou na 1ª execução, sem `tray activated` externo: PID 2155519, `popup shown` e ainda mostrado 1,5 s depois;
  - o C17 passou: PID 2156058, `75 -> 80` na vertical, nada na horizontal em 1,5 s, e `80 -> 75`.

## DoD Checklist (gate 8)

| # | Criterion | Source | Type | Status | Evidence |
|---|---|---|---|---|---|
| 1 | fmt + clippy `-D warnings` incluindo `ddc-tray` | CONTEXT | Auto | PASS | `OK` |
| 2 | `ddc-tray` compila em release no Linux | CONTEXT | Auto | PASS | `OK` |
| 3 | Só o composition root constrói `DdcHiMonitorBackend` (1×) | CONTEXT | Auto | PASS | `OK` |
| 4 | `panel.rs` puro sobre `MonitorControl` | CONTEXT | Auto | PASS | `OK` |
| 5 | `#![forbid(unsafe_code)]` em `src-tauri`, nenhum `unsafe` | CONTEXT | Auto | PASS | `OK` |
| 6 | `node --test`: debounce, view-model, bridge demo (fora de `src/`) | CONTEXT | Auto | PASS | `OK` (153 pass, 0 fail/cancelled/skipped/todo) |
| 7 | Paridade i18n + nenhum texto hardcoded | CONTEXT | Auto | PASS | `OK` |
| 8 | CSP sem `unsafe-inline`, `script-src 'self'`, sem `devCsp`/`devUrl`, `custom-protocol` + `is_not_a_dev_build` | CONTEXT | Auto | PASS | `OK` (`1 passed`). Ver W-2: o Verify passa também com as mutações (a) e (b) |
| 9 | Capabilities sem `shell`/`fs`/`http`/`opener` | CONTEXT | Auto | PASS | `OK` |
| 10 | `single-instance` 1º no builder | CONTEXT | Auto | PASS | `OK` |
| 11 | Smoke SNI `--activate` (backend real, só leituras) | CONTEXT | Auto | PASS | `OK` na 1ª execução (PID 2155519) |
| 12 | Teste de hardware `#[ignore]` gated por `DDC_HW_TESTS=1` | CONTEXT | Auto | PASS | `OK` (só `--list`, não executado) |
| 13 | Gate 7: console limpo + axe nos `critical_paths` | CONTEXT | Auto | PASS | `OK`: 134 passed, 6 skipped; `n=10 dr=2 ps=54 dd=dl=16 sk=sl=6` |
| 14 | Bridge nunca cai no demo fora de servidor local | CONTEXT | Auto | PASS | `OK` |
| 15 | Hash do harness congelado | CONTEXT | Auto | PASS | `OK`: `fd6985f9…8560` sobre 21 arquivos; última mudança em `c14bb5e` |
| 16 | Nenhum `<select>` nativo | CONTEXT | Auto | PASS | `OK` |
| 17 | `ksni` + testes `scroll` + smoke `--fake --scroll` | CONTEXT | Auto | PASS | `OK` (PID 2156058) |
| 18 | Nenhum TODO/FIXME sem issue em arquivo versionado do produto | CONTEXT | Auto | PASS | `OK` |
| 19 | Screenshots regenerados e byte a byte iguais | CONTEXT | Auto | PASS | `OK`. O `sha256sum -c` dos PNGs antes e depois confere |
| 20 | `cargo test --workspace` exits 0 | PROJECT | Auto | PASS | `OK` (384 passed, 0 failed, 9 ignored) |
| 21 | Coverage >= 80% of lines | PROJECT | Auto | PASS | TOTAL Lines 82.79% (comando literal), exit 0 |
| 22 | No TODO/FIXME without linked issue | PROJECT | Auto | PASS | `OK` |
| 23 | CHANGELOG.md updated with entry per release | PROJECT | Manual | MANUAL_REQUIRED | suggested: `## [Unreleased]` em `CHANGELOG.md:8`. Nenhuma entrada para a D-10, e a iteração não muda o comportamento visível |
| 24 | README accurately describes current behavior | PROJECT | Manual | MANUAL_REQUIRED | suggested: `README.md` § "Tray app" (l. 265) / "Build and run" (l. 290), que continua correto: `cargo build -p ddc-tray --release --locked` agora gera o build de produção |

**Totals:** 24 items | Auto: 22 (22 PASS, 0 FAIL) | Manual: 2 pending

**Manual confirmation required:**
Rode `/jdi-confirm-dod tray-app` para confirmar cada item manual com evidência. Sem isso, o `/jdi-ship` recusa a phase.

## Recommendation
- **Código:** nenhum blocker. A iter 13 fecha o achado do critic da iter 12, e o repositório está conforme a D-2026-09-27-tray-app-10 e a D-2026-09-26-tray-app-7.
- **W-2:** pede endurecer o Verify do C8 com os 3 ajustes testados acima (glob por arquivo com recusa de `config-json5`/`config-toml`; `cargo tree` do build normal nos dois alvos; trava do `assert!`). Nenhum deles muda o hash do harness, porque só o CONTEXT.md e o `lib.rs` estão envolvidos. Como é uma lacuna objetiva de prova, é provável que o DoD critic a aponte se ela ficar aberta.
- **W-1:** continua com a `ci-crossbuild`.
- **Itens manuais:** CHANGELOG e README seguem para o `/jdi-confirm-dod`.

## Nota do orquestrador (pós-review, antes do critic)

W-2 era do `Verify:` do C8 (escrito pelo orquestrador), não do código: o C8 foi endurecido com os três ajustes testados pelo reviewer — cada arquivo `tauri*.json5`/`Tauri*.toml`/`tauri*.toml` reprova sozinho e `config-(json5|toml)` no Cargo.toml reprova; `cargo tree … -e normal,features -i tauri` exige `custom-protocol` na resolução NORMAL (Linux e Windows), não em dev-deps; e `assert!(!tauri::is_dev()` precisa estar no `lib.rs`. HEAD imprime OK; os mutantes (a) `Tauri.linux.toml` com CSP frouxa e (b) feature só em `[dev-dependencies]` reprovam (numa cópia descartável). Nenhum arquivo do harness congelado mudou.
