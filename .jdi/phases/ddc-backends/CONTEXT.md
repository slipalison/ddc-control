# Phase 2: DDC backends — Context (slug: ddc-backends)

## Goal
Adapter real via `ddc-hi`: enumeração de monitores (Windows dxva2, Linux /dev/i2c), get/set VCP com retry e timeout, identificação por EDID/modelo, docs de `i2c-dev`/udev para Linux.

## Locked decisions
- D-1: Hexagonal (Ports & Adapters) — port `MonitorBackend` no core; `DdcHiMonitorBackend` é o adapter real, implementa exatamente o port existente sem alterá-lo.
- D-2: `ddc-core` continua dependendo só de `thiserror`; `ddc-hi`/`mccs`/`mccs-caps`/`mccs-db` (transitivos do `ddc-hi`) entram só em `ddc-adapters`.
- D-2026-09-25-ddc-backends-1: `ddc-hi` em `ddc-adapters` = `default-features = false, features = ["ddc-i2c","ddc-winapi"]` (sem `nvapi`/`ddc-macos`), atrás da feature própria `ddc-hi` (default-on). **Correção:** remover `nvapi`/`ddc-macos` sozinho NÃO garante `Send + Sync` (verificado em 2026-09-25 — o `HANDLE` bruto do `ddc-winapi` já é `!Send` no Windows independente de `nvapi`); o feature-gating do Cargo.toml continua válido, mas o mecanismo real de `Send + Sync` é D-2026-09-25-ddc-backends-6.
- D-2026-09-25-ddc-backends-2: `MonitorId` = chave EDID (`manufacturer_id-model-serial`, com sufixo `#N` em colisão de serial-placeholder) quando EDID existe (Linux `ddc-i2c`); `description` normalizada ou `index-N` como último fallback quando EDID está ausente (Windows `ddc-winapi`).
- D-2026-09-25-ddc-backends-3: timeout total default 1s, ≤3 tentativas, backoff 50ms (números continuam válidos). **Transporte atualizado:** não é mais "Mutex por handle" — é a fila única + canal de resposta de D-2026-09-25-ddc-backends-6, que é quem hospeda o retry/backoff de fato.
- D-2026-09-25-ddc-backends-4: `enumerate()` nunca sonda DDC/CI ativamente (sem `get_vcp_feature`/`capabilities_string`) — só traduz `Display::enumerate()`; monitor com EDID legível e DDC mudo (ex.: LG TV) aparece na lista, erro só surge no primeiro `read_vcp`/`write_vcp` real.
- D-2026-09-25-ddc-backends-5: mapeamento `ddc::VcpValue{mh,ml,sh,sl}` → core `VcpValue{current,max}` = `current = be16(sh,sl)`, `max = be16(mh,ml)` (layout padrão VESA MCCS).
- D-2026-09-25-ddc-backends-6: `DdcHiMonitorBackend` = worker thread único (dono de todo `Display`/`Handle`, roda `enumerate()` e toda operação DDC) + `mpsc::Sender<Request>`/`JoinHandle` no struct público (ambos `Send + Sync` triviais, zero `unsafe`); chamador bloqueia em `reply_receiver.recv_timeout(budget)` → `DdcError::Timeout` na expiração, mesmo se o worker seguir preso. Retry/backoff roda dentro do worker. Substitui o "Mutex por handle" de D-3. Trade-off aceito: um monitor genuinamente travado atrasa a fila para os demais (NACKs, o caso comum, falham rápido — não travam). Alternativas rejeitadas: `unsafe impl Send` (afinidade de thread do handle dxva2 não é documentada — risco de UB) e uma thread por monitor (impossível no Windows: `Display` precisa ser criado na thread que o opera).
- D-2026-09-25-ddc-backends-7 (orquestrador, evidência medida em hardware antes da execução): orçamento POR TIPO DE OPERAÇÃO — `read_vcp`/`write_vcp` = 1s, `read_capabilities` = 8s, `enumerate` = 5s (corrige o default único de 1s de D-3; retry ≤3× / 50ms inalterado). Medido com `ddc-hi` no RTK: enumerate 1.13s, VCP get 44–94ms, caps 2.58s; 0xDF = 0x0202 confirma D-5.

## Canonical refs
- Card: pedido do usuário no chat via `/jdi-issue`, 2026-09-25 — "o importante é conseguir controlar tudo do monitor" via DDC/CI (origem da chain core-domain → ddc-backends → cli → full-osd-control).
- `crates/ddc-core/src/ports/monitor_backend.rs`, `crates/ddc-core/src/domain/{monitor,error,vcp}.rs` — contrato exato que `DdcHiMonitorBackend` implementa (assinaturas `&self`, sem `Duration` por chamada — por isso o budget vem do construtor/`with_timeout`, não do port).
- `crates/ddc-adapters/src/{lib,in_memory}.rs` — padrão de adapter já estabelecido (`Arc<Mutex<...>>`, `lock()` com recuperação de poison) — referência de estilo; o transporte real usa `mpsc` em vez de `Mutex` por causa de D-6.
- `.jdi/agents/jdi-doer-ddc-control.md` §§ Workspace layout, Monitor-write safety, Concorrência & realidades de hardware, Testes — regras que esta phase implementa.
- `.jdi/phases/core-domain/CONTEXT.md` — decisões D-2026-09-25-core-domain-{1..5} e a capabilities string real do "RTK QHD HDR" já fixada como fixture.
- ddc-hi 0.4.1, fonte lida em `~/.cargo/registry` em 2026-09-25: deps `anyhow, ddc 0.2, edid 0.3, mccs, mccs-caps, mccs-db, log`; defaults `[ddc-i2c, ddc-winapi, nvapi, ddc-macos]`; API `Display::enumerate() -> Vec<Display>`, `Display{handle: Handle, info: DisplayInfo}`, `Handle: ddc::Ddc` (`get_vcp_feature`, `set_vcp_feature`, `capabilities_string`, todos `&mut self`).
- Verificação do orquestrador em 2026-09-25 (crate de scratch, `default-features = false, features = ["ddc-i2c","ddc-winapi"]`): `cargo check --target x86_64-unknown-linux-gnu` de `Mutex<ddc_hi::Display>: Send + Sync` compila; `cargo check --target x86_64-pc-windows-msvc` da mesma asserção falha com E0277 (`*mut c_void` não é `Send`) via `PHYSICAL_MONITOR → ddc_winapi::Monitor → Handle → Display` — base factual de D-2026-09-25-ddc-backends-6. O check Windows sem a asserção (só a dependência) compila normal; o check Linux emite apenas um aviso de future-incompat de `nom v3.2.1` (via `mccs-caps`/`mccs-db`, transitivo do `ddc-hi`, fora do nosso controle).
- Hardware de dev sondado em 2026-09-25: "RTK QHD HDR" em `/dev/i2c-5` (DRM DP-2), EDID mfg `RTK`, model `RTK QHD HDR`, product code `0x8B8A`, serial string vazia / serial binário `0x01010101` (placeholder); LG TV em `/dev/i2c-11` (HDMI) com EDID legível e DDC/CI mudo; `/dev/i2c-0` root-only (`EACCES`); regra udev `/usr/lib/udev/rules.d/60-ddcutil-i2c.rules` já concede `uaccess` aos demais.

## Out of scope
- Crate `ddc-cli` (comandos, atalhos nomeados, `--json`) → phase `cli`.
- Catálogo MCCS 2.2 completo, sondagem read-only fora do caps, `features` dump → phase `full-osd-control` (já registrado em D-2026-09-25-full-osd-control-1).
- Uso do port a partir de `apps/ddc-tray`/`spawn_blocking` → phase `tray-app` (esta phase só garante `Send + Sync`, D-2026-09-25-ddc-backends-6).
- Validação comportamental em hardware Windows real (dxva2) — só o gate de cross-compile (`cargo check --target x86_64-pc-windows-msvc`) roda nesta phase; validação funcional completa fica para a phase `ci-crossbuild`/PR (ver Deferred to PR review).

## Definition of Done

### Auto-verifiable
- [ ] `crates/ddc-adapters/Cargo.toml` declara `ddc-hi` opcional, `default-features = false`, com `features = ["ddc-i2c", "ddc-winapi"]` (sem `nvapi`/`ddc-macos`), atrás de uma feature própria `ddc-hi` incluída em `default` (D-2026-09-25-ddc-backends-1).
      **Verify:** `grep -qE 'ddc-hi[[:space:]]*=.*optional[[:space:]]*=[[:space:]]*true' crates/ddc-adapters/Cargo.toml && grep -qE 'ddc-hi[[:space:]]*=.*default-features[[:space:]]*=[[:space:]]*false' crates/ddc-adapters/Cargo.toml && grep -qE '"ddc-i2c"' crates/ddc-adapters/Cargo.toml && grep -qE '"ddc-winapi"' crates/ddc-adapters/Cargo.toml && grep -qE '"dep:ddc-hi"' crates/ddc-adapters/Cargo.toml && grep -qE '^default[[:space:]]*=.*"ddc-hi"' crates/ddc-adapters/Cargo.toml && ! grep -qE '"nvapi"' crates/ddc-adapters/Cargo.toml && ! grep -qE '"ddc-macos"' crates/ddc-adapters/Cargo.toml && echo OK`
      **Source:** CONTEXT
- [ ] `ddc-adapters` compila (check-only) para o alvo Windows a partir desta máquina Linux, e esse MESMO check prova `DdcHiMonitorBackend: Send + Sync` no Windows via uma asserção estática em código NÃO-`#[cfg(test)]` (ex.: `const _: () = { const fn assert_send_sync<T: Send + Sync>() {} assert_send_sync::<DdcHiMonitorBackend>(); };`, dentro do módulo `#[cfg(feature = "ddc-hi")]`) — mecanismo real é o worker thread + canal de D-2026-09-25-ddc-backends-6, não a simples remoção de `nvapi`.
      **Verify:** `cargo check -p ddc-adapters --locked --features ddc-hi --target x86_64-pc-windows-msvc && echo OK`
      **Source:** CONTEXT
- [ ] Teste nomeado prova (em complemento à asserção estática acima, também executável nesta máquina Linux) que `DdcHiMonitorBackend` é `Send + Sync`.
      **Verify:** `cargo test -p ddc-adapters --locked --features ddc-hi -- ddc_hi_backend_is_send_and_sync 2>&1 | grep -qE 'test result: ok\. 1 passed; 0 failed' && echo OK`
      **Source:** CONTEXT
- [ ] Teste nomeado prova o mapeamento de bytes `ddc::VcpValue{mh,ml,sh,sl}` → core `VcpValue{current,max}` (D-2026-09-25-ddc-backends-5) com fixture literal, sem hardware.
      **Verify:** `cargo test -p ddc-adapters --locked --features ddc-hi -- maps_ddc_hi_vcp_reply_to_core_current_and_max 2>&1 | grep -qE 'test result: ok\. 1 passed; 0 failed' && echo OK`
      **Source:** CONTEXT
- [ ] Teste nomeado prova a derivação de `MonitorId` a partir de identidade EDID-like (manufacturer/model/serial) e a desambiguação por sufixo quando duas entradas colidem no mesmo `enumerate()` (D-2026-09-25-ddc-backends-2a).
      **Verify:** `cargo test -p ddc-adapters --locked --features ddc-hi -- derives_monitor_id_from_edid_and_disambiguates_duplicate_serials 2>&1 | grep -qE 'test result: ok\. 1 passed; 0 failed' && echo OK`
      **Source:** CONTEXT
- [ ] Teste nomeado prova o fallback de `MonitorId` quando EDID está ausente: `description` normalizada, ou `index-N` se a descrição também vier vazia (D-2026-09-25-ddc-backends-2b).
      **Verify:** `cargo test -p ddc-adapters --locked --features ddc-hi -- derives_monitor_id_from_description_or_index_when_edid_absent 2>&1 | grep -qE 'test result: ok\. 1 passed; 0 failed' && echo OK`
      **Source:** CONTEXT
- [ ] Teste nomeado prova a política de retry/backoff (até 3 tentativas, 50ms entre elas, respeitando o orçamento total do `Duration` do caller) rodando dentro do worker, sem depender de hardware nem de sleep real de segundos (D-2026-09-25-ddc-backends-3 + -6).
      **Verify:** `cargo test -p ddc-adapters --locked --features ddc-hi -- retries_transient_errors_up_to_three_times_within_timeout_budget 2>&1 | grep -qE 'test result: ok\. 1 passed; 0 failed' && echo OK`
      **Source:** CONTEXT
- [ ] Teste nomeado prova que o chamador recebe `DdcError::Timeout` dentro do orçamento configurado quando o worker não responde a tempo (via a seam de transporte interna do adapter — sem hardware, sem sleep real além de poucos milissegundos), confirmando o mecanismo `recv_timeout` de D-2026-09-25-ddc-backends-6.
      **Verify:** `cargo test -p ddc-adapters --locked --features ddc-hi -- caller_receives_timeout_when_worker_does_not_answer_within_budget 2>&1 | grep -qE 'test result: ok\. 1 passed; 0 failed' && echo OK`
      **Source:** CONTEXT
- [ ] Teste nomeado prova que falhas do backend real nunca vazam tipos de `ddc_hi`/`anyhow` através do port — sempre mapeadas para `DdcError::Transport`/`DdcError::Timeout`.
      **Verify:** `cargo test -p ddc-adapters --locked --features ddc-hi -- maps_backend_failures_to_transport_or_timeout_without_leaking_ddc_hi_errors 2>&1 | grep -qE 'test result: ok\. 1 passed; 0 failed' && echo OK`
      **Source:** CONTEXT
- [ ] Documento Linux (`docs/linux-ddc-setup.md`) cobre módulo `i2c-dev`, regra udev de `uaccess`/grupo, ressalva sobre driver NVIDIA proprietário e instrução explícita de nunca rodar com `sudo`.
      **Verify:** `test -f docs/linux-ddc-setup.md && grep -qi 'i2c-dev' docs/linux-ddc-setup.md && grep -qi 'udev' docs/linux-ddc-setup.md && grep -qiE 'uaccess|group' docs/linux-ddc-setup.md && grep -qi 'nvidia' docs/linux-ddc-setup.md && grep -qi 'sudo' docs/linux-ddc-setup.md && echo OK`
      **Source:** CONTEXT

### Manual
- _(none)_

## Deferred to PR review
- Validação em hardware real Linux (RTK QHD HDR, `/dev/i2c-5`): `enumerate()` encontra o monitor, `MonitorId` derivado segue o esquema de D-2026-09-25-ddc-backends-2, `read_capabilities` bate com a string fixture já travada em `core-domain`, `read_vcp(0x10)`/`read_vcp(0xDF)` respondem — testes `#[ignore]` gated por `DDC_HW_TESTS=1`; o reviewer NUNCA os roda, o orquestrador roda nesta máquina e anexa evidência no corpo do PR.
- Confirmação em hardware real de D-2026-09-25-ddc-backends-4: o LG TV (`/dev/i2c-11`, EDID legível, DDC mudo) aparece em `enumerate()` sem custo extra de sondagem, e uma chamada real de `read_vcp`/`read_capabilities` contra ele retorna `DdcError::Transport`/`Timeout` dentro do orçamento de 1s (nunca hang, nunca panic) — só observável com o hardware físico presente; também confirma que o worker único de D-6 se recupera após esse NAK rápido (não fica preso).
- Validação funcional completa em Windows real (dxva2: `enumerate`, `get_vcp_feature`, `set_vcp_feature` num monitor físico, incluindo o comportamento do worker thread único de D-2026-09-25-ddc-backends-6 sob hardware real) — este box é Linux; só o gate de cross-compile + a asserção estática de `Send + Sync` (`cargo check --target x86_64-pc-windows-msvc`) estão disponíveis aqui. Fica para a phase `ci-crossbuild` (job `windows-latest`) ou para quando houver máquina Windows disponível; sinalizar no corpo do PR, não bloqueia esta phase.
- `cargo audit`: advisories de severidade WARN (5.10) vindas de deps transitivos antigos (`serde_yaml`/`nom`) puxados por `mccs-db` (dependência do próprio `ddc-hi`, fora do nosso controle direto) — não bloqueiam esta phase; revisitar se `full-osd-control` reintroduzir `mccs`/`mccs-caps` no core ou se a severidade escalar. O aviso de future-incompat de `nom v3.2.1` observado no check Linux é da mesma origem — informativo, não bloqueante.

## Notes
- **Amendment 2026-09-25 (pós-verificação do orquestrador):** a hipótese inicial de D-2026-09-25-ddc-backends-1 ("remover `nvapi` basta para `Send + Sync`") estava incompleta — o `HANDLE` bruto do `ddc-winapi` é `!Send` por si só no Windows. D-2026-09-25-ddc-backends-6 é a decisão que corrige isso e é a fonte de verdade para a forma de concorrência real (worker thread único + `mpsc`, zero `unsafe`). D-2026-09-25-ddc-backends-3 mantém os números de timeout/retry, mas o "Mutex por handle" nela descrito foi substituído pelo transporte de D-6.
- Módulo/caminho dos testes acima é liberdade do planner (mesma convenção fixada em `core-domain`): os nomes são contrato mínimo, casados por substring via `cargo test -- <nome>`. O `grep` exige a linha completa `test result: ok. 1 passed; 0 failed` (não só a substring `1 passed`) — isso já cobre o caso que o critic de DoD da phase anterior sinalizou (uma corrida com falhas mistas, ex. "1 passed; 2 failed", nunca passa nesse grep) sem precisar travar o caminho de módulo completo exigido por `--exact`.
- `ddc-hi` como feature default-on significa que `cargo test -p ddc-adapters --locked` sozinho já rodaria os testes acima; o `--features ddc-hi` explícito nos comandos é defensivo caso o default mude no futuro.
- A prova de `Send + Sync` tem DOIS pontos de verificação redundantes de propósito: a asserção estática em código não-test (pega no `cargo check --target x86_64-pc-windows-msvc`, plataforma onde o bug real existe) e o teste nomeado (roda nesta máquina Linux via `cargo test`, útil no dia a dia de dev sem precisar do target Windows). Ambos usam a mesma técnica (`fn assert_send_sync<T: Send + Sync>() {}`).
- O teste de timeout (`caller_receives_timeout_when_worker_does_not_answer_within_budget`) precisa de uma seam de transporte interna ao adapter (ex.: um trait que a produção implementa sobre `ddc_hi::Display`/`Handle` e que um teste pode implementar com um corpo que nunca responde) — o teste usa um orçamento (`Duration`) pequeno (poucos ms) para manter a suíte rápida; não precisa mockar sleep, só um chamador com paciência curta contra um worker deliberadamente preso.
- Baseline de `.jdi/PROJECT.md` (`cargo test --workspace` verde, cobertura ≥80%, sem TODO/FIXME sem issue) é herdada automaticamente pelo Gate 8 do reviewer — não duplicada aqui.
