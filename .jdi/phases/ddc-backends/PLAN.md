# Phase 2: DDC backends — Plan  (slug: ddc-backends)

## Goal
Adapter real via `ddc-hi`: enumeração de monitores (Windows dxva2, Linux /dev/i2c), get/set VCP com retry e timeout, identificação por EDID/modelo, docs de `i2c-dev`/udev para Linux.

## Locked decisions (from CONTEXT.md)
- D-1/D-2: `DdcHiMonitorBackend` implementa o port `MonitorBackend` existente sem alterá-lo; `ddc-hi`/`mccs*` só em `ddc-adapters`.
- D-2026-09-25-ddc-backends-1: `ddc-hi` opcional, `default-features = false`, `["ddc-i2c","ddc-winapi"]`, atrás da feature própria `ddc-hi` (default-on).
- -2: `MonitorId` = chave EDID (`#N` em colisão); sem EDID, descrição normalizada ou `index-N`. -4: `enumerate()` não sonda DDC.
- -3: 1s por operação, ≤3 tentativas, 50ms de backoff. -5: `current = be16(sh,sl)`, `max = be16(mh,ml)`.
- -7 (orquestrador, pós-plano): orçamento por operação — VCP get/set 1s, caps 8s, enumerate 5s (medido: 1.13s enumerate, 2.58s caps, 44–94ms VCP). Resolve R-1.
- -6: worker thread único, dono de todo `Display`/`Handle`; o struct guarda `Sender` + `JoinHandle`; `recv_timeout(budget)` → `Timeout`; retry dentro do worker; zero `unsafe`.

## Assumptions (cadeia autônoma, sem AskUserQuestion)
- A-1 O módulo se chama `ddc_hi_backend`, porque `ddc_hi` colidiria com o crate externo. Fica atrás de `#[cfg(feature = "ddc-hi")]`, com estes arquivos: `ddc_hi_backend.rs` (tipo público), `identity.rs` (pura), `worker.rs` (seam + worker + cliente), `hardware.rs` (casca fina + traduções de tipos `ddc_hi`) e `retry.rs` (T-4). Testes em `<mod>/tests.rs` (padrão A-8 da `core-domain`). Irmãos são importados via `super::`, nunca `crate::ddc_hi_backend::`, porque o grep 5.4 casa `ddc_hi` por prefixo. Traits internos são `pub(crate)` (grep 5.3). Nenhum `#[cfg(target_os)]`: o `ddc-hi` já escolhe o backend da plataforma.
- A-2 Seam de transporte: `DisplaySource { type Handle: DdcHandle; fn enumerate(&mut self) -> Vec<(DisplayIdentity, Self::Handle)> }` e `DdcHandle { read_capabilities(&mut self) -> Result<Vec<u8>, HandleError>; read_vcp(VcpCode) -> Result<VcpValue, HandleError>; write_vcp(VcpCode, u16) -> Result<(), HandleError> }`. `HandleError` guarda só o texto (`{:#}`), então nenhum tipo `ddc_hi`/`anyhow` sai da casca. Em produção: `DdcHiDisplays` + `impl DdcHandle for ddc_hi::Handle`.
- A-3 Protocolo de D-6: `Request` é um job `Box<dyn FnOnce(&mut Worker<..>) + Send>` com os args, o deadline e um `Sender` de resposta tipado. O worker roda um job por vez e responde com `let _ = reply.send(..)`. `WorkerClient<S>` é genérico, testável com fakes, e implementa `MonitorBackend`; `DdcHiMonitorBackend` é um newtype sobre `WorkerClient<DdcHiDisplays>` que só delega. Sem enum de resposta, não há ramo morto nem `unreachable!`. Nenhum `Drop` faz `join`, porque um worker preso não pode travar quem descarta o backend. O `JoinHandle` fica no struct (D-6; o campo pode se chamar `_worker`).
- A-4 Construir não faz I/O. `new() -> Result<Self, DdcError>`: falha de `thread::Builder::spawn` vira `Transport`, sem panic. Orçamentos default por operação (D-7): VCP get/set 1s, caps 8s, enumerate 5s, como consts nomeadas; builder(s) de override (ex.: um struct `Budgets` + `with_budgets`, ou `with_timeout` para VCP + `with_capabilities_timeout` + `with_enumerate_timeout`). `Display::enumerate()` só roda quando chega a primeira request.
- A-5 A tabela do worker (`Vec<(MonitorId, Handle)>`) é trocada inteira a cada enumerate. Id desconhecido → um re-enumerate; se continuar ausente, `MonitorNotFound` (a CLI pode chamar `read_vcp` sem `enumerate` antes). T-4: falha depois dos retries → re-enumerate. Se o id sumiu, `MonitorNotFound`, para que um monitor desplugado não vire "caps ilegível" memorizado no core (nota do reviewer); se continua presente, `Transport`.
- A-6 Identidade (D-2): tem EDID ⇔ `manufacturer_id` presente.
  - Chave `{fabricante}-{modelo}-{serial}`. Modelo = `model_name` não-vazio, senão `model_id` em `{:04X}`. Serial = `serial_number` não-vazio, senão `serial` em `{:08X}`. Segmento ausente é omitido.
  - Sanitização: trim, não-alfanumérico ASCII vira `-`, `-` repetidos colapsam, pontas aparadas.
  - Sem EDID: `DisplayInfo.id` sanitizado (no winapi é a descrição); se vazio, `index-{N}`.
  - Chave repetida no mesmo enumerate recebe `#2`, `#3`… na ordem. O RTK de dev fica `RTK-RTK-QHD-HDR-01010101`.
  - `MonitorInfo.manufacturer/model/serial` = fabricante / `model_name` / segmento serial, só com EDID; sem EDID, `None`.
- A-7 Caps: remove todo byte NUL (o terminador final e os terminadores por fragmento de alguns scalers — um NUL interno partiria um token no parser do core) e depois aplica `String::from_utf8_lossy`. Nenhum outro filtro.
- A-8 Retry (T-4): toda falha de `DdcHandle` conta como transitória, porque o `ddc-hi` não tipa NAK. Retentar write é seguro: Set VCP é um valor absoluto. O worker não dorme se `agora + backoff ≥ deadline`; desiste com `Transport` da última falha. Um job que o worker pega já vencido não toca o hardware: um write cujo caller recebeu `Timeout` nunca é aplicado depois. O tempo passa por um trait interno `Clock { now; sleep }` (`SystemClock` em produção; fake com tempo virtual nos testes).
- A-9 Panic dentro do worker (bug do `ddc-hi`) fecha o canal, e as chamadas seguintes recebem `Transport`. Sem respawn nesta phase.
- A-10 Testes de hardware em `crates/ddc-adapters/tests/real_monitor.rs` (`#![cfg(feature = "ddc-hi")]`), só leitura. `DdcHiMonitorBackend` só aparece dentro de corpos `#[ignore]`, com caminho qualificado e sem `use` (gate 5.7). O fixture de caps é copiado verbatim do core, que não o exporta.
- A-11 Nomes do DoD são únicos no workspace, e nenhum teste extra contém um deles como substring. T-1 fecha o `Cargo.lock`; nenhuma task posterior toca `Cargo.*`.

## Riscos
- R-1 **Resolvido por D-2026-09-25-ddc-backends-7** (medição do orquestrador: caps 2.58s, enumerate 1.13s com `ddc-hi`). Texto original: Orçamento de caps. O `ddc 0.2.2` espera ~40ms de resposta + ~50ms entre comandos por fragmento de 32 bytes. A caps RTK (259 bytes) dá ~10 fragmentos, ≈ 0.9–1.0s, colado no 1s de D-3. Um fragmento com NAK reinicia a leitura, e o re-enumerate de A-5 também consome budget. T-5 mede com o budget default e imprime os tempos. Se der `Timeout`, o doer NÃO muda o default: o orquestrador registra uma D-XX (ex.: budget próprio para caps) antes do PR.
- R-2 O build Linux passa a exigir os headers do libudev (`systemd-devel`/`libudev-dev`). Fica documentado em T-6, e a `ci-crossbuild` vai precisar instalá-los.

## Tasks
Specialist único: `jdi-doer-ddc-control` (glob `**/*`). Todo commit mantém verdes:
- `cargo fmt --all --check`
- `cargo clippy --workspace --all-targets --locked -- -D warnings`
- `cargo test --workspace --locked`
- `cargo check -p ddc-core -p ddc-adapters --locked --target x86_64-unknown-linux-gnu`
- de T-1 em diante, `cargo check -p ddc-adapters --locked --features ddc-hi --target x86_64-pc-windows-msvc`

Scope `ddc-backends`, D-XX citada no corpo, nunca junto com `.jdi/`.

### Wave 1 (parallel-eligible)

#### T-1: Declarar `ddc-hi` atrás da feature default-on
- **Specialist:** jdi-doer-ddc-control
- **Files modified:** `crates/ddc-adapters/Cargo.toml`, `Cargo.lock`
- **Acceptance:**
  - `ddc-hi = { version = "0.4.1", default-features = false, features = ["ddc-i2c", "ddc-winapi"], optional = true }` numa linha só, porque os greps do DoD são por linha. `[features]` com `default = ["ddc-hi"]` e `ddc-hi = ["dep:ddc-hi"]`. Nenhuma menção a `"nvapi"`/`"ddc-macos"`, nem em comentário. `Cargo.toml` raiz intacto.
  - Lock gerado uma vez sem `--locked`, no mesmo commit (`chore`). Verify #1 do DoD = OK; check Windows verde. `cargo audit` rodado e anotado no SUMMARY (WARN, não bloqueia — CONTEXT).
- **Dependencies:** none
- **Test:** verify #1 do DoD + gates acima
- **Status:** completed

#### T-2: Fake com falha transitória de caps + recuperação no core (W-4)
- **Specialist:** jdi-doer-ddc-control
- **Files modified:** `crates/ddc-adapters/src/in_memory.rs`, `crates/ddc-adapters/src/in_memory/tests.rs`, `crates/ddc-core/tests/monitor_control/read_features.rs`
- **Acceptance:**
  - `FakeMonitor` ganha um builder (ex.: `with_transient_capabilities_failures(n)`): as `n` primeiras `read_capabilities` respondem `Transport`, as seguintes a string roteirizada. É só dado de teste, sem regra de negócio, com unit test no fake.
  - `explicit_capabilities_request_recovers_from_transient_failure`: caps RTK + 1 falha. `get_feature(0x10)` dá `declared_in_capabilities == false`; `capabilities()` dá `Ok` (model `RTK`); o `get_feature(0x10)` seguinte dá `declared_in_capabilities == true`. Log exato `[ReadCapabilities, ReadVcp, ReadCapabilities, ReadVcp]`: o cache foi substituído, sem refetch.
- **Dependencies:** none
- **Test:** `cargo test -p ddc-adapters --locked in_memory && cargo test -p ddc-core --locked --test monitor_control`
- **Status:** completed

### Wave 2

#### T-3: `DdcHiMonitorBackend` sobre worker único (D-6), com identidade e traduções
- **Specialist:** jdi-doer-ddc-control
- **Files modified:** `crates/ddc-adapters/src/lib.rs`, `crates/ddc-adapters/src/ddc_hi_backend.rs`, `crates/ddc-adapters/src/ddc_hi_backend/tests.rs`, `crates/ddc-adapters/src/ddc_hi_backend/identity.rs`, `crates/ddc-adapters/src/ddc_hi_backend/identity/tests.rs`, `crates/ddc-adapters/src/ddc_hi_backend/worker.rs`, `crates/ddc-adapters/src/ddc_hi_backend/worker/tests.rs`, `crates/ddc-adapters/src/ddc_hi_backend/hardware.rs`, `crates/ddc-adapters/src/ddc_hi_backend/hardware/tests.rs`
- **Acceptance:**
  - Forma A-1..A-5:
    - `lib.rs` exporta `DdcHiMonitorBackend` só com a feature, e toda chamada `ddc_hi` roda na thread do worker.
    - `enumerate()` só traduz `Display::enumerate()` (D-4). Cada op faz uma tentativa (o retry vem em T-4); falha → `Transport` com o texto do erro.
    - Cliente: `recv_timeout(budget)` expirado → `Timeout`; envio falho ou resposta descartada → `Transport`.
    - Asserção estática `Send + Sync` em código não-teste (DoD #2). `cargo check -p ddc-adapters --locked --no-default-features` verde.
  - Traduções puras em `hardware.rs`: `ddc_hi::VcpValue` → core (D-5), bytes de caps → `String` (A-7) e `ddc_hi::DisplayInfo` → `DisplayIdentity`, testada com `DisplayInfo::new(Backend::I2cDevice|WinApi, ..)` + campos. A derivação de A-6 fica em `identity.rs`.
  - Testes do DoD:
    - `ddc_hi_backend_is_send_and_sync` (`tests.rs`).
    - `maps_ddc_hi_vcp_reply_to_core_current_and_max` (`hardware/tests.rs`): `{mh:0x01, ml:0x02, sh:0x03, sl:0x04}` → `{current:0x0304, max:0x0102}`; `{0,100,0,50}` → `{50,100}`.
    - `derives_monitor_id_from_edid_and_disambiguates_duplicate_serials` (`identity/tests.rs`): 2× RTK dev → `RTK-RTK-QHD-HDR-01010101` e `…#2`; serial string ganha do binário.
    - `derives_monitor_id_from_description_or_index_when_edid_absent` (`identity/tests.rs`): 2× `Generic PnP Monitor` → `Generic-PnP-Monitor` e `…#2`; descrição vazia na posição 2 → `index-2`.
    - `caller_receives_timeout_when_worker_does_not_answer_within_budget` (`worker/tests.rs`, via `WorkerClient` com fake): handle preso num gate até o teste liberar, budget ~20ms → `Timeout` em < 1s.
    - `maps_backend_failures_to_transport_or_timeout_without_leaking_ddc_hi_errors` (`worker/tests.rs`, via `WorkerClient` com fake): falha de caps/read/write → `Transport` com o texto do fake; worker morto → `Transport`.
  - Testes extras: id desconhecido = 1 re-enumerate + `MonitorNotFound`; enumerate troca a tabela; NUL e UTF-8 inválido na caps.
  - Casca que não dá para testar sem monitor: `DdcHiDisplays::enumerate`, os 3 métodos de `impl DdcHandle for ddc_hi::Handle` (1 linha cada: delegam e traduzem) e `new`/`with_timeout`/delegações do newtype. Soma ≤ 40 linhas. Nenhum teste fora de `#[ignore]` envia request ao backend real; só construir é permitido (A-4). Nenhum `unsafe`; greps 5.3/5.4/5.6 sem saída; funções ≤ 30 linhas.
- **Dependencies:** T-1
- **Test:** `cargo test -p ddc-adapters --locked` + verifies #2–#6, #8, #9 do DoD
- **Status:** completed

### Wave 3

#### T-4: Retry/backoff, deadline e presença dentro do worker (D-3, D-6)
- **Specialist:** jdi-doer-ddc-control
- **Files modified:** `crates/ddc-adapters/src/ddc_hi_backend.rs`, `crates/ddc-adapters/src/ddc_hi_backend/retry.rs`, `crates/ddc-adapters/src/ddc_hi_backend/worker.rs`, `crates/ddc-adapters/src/ddc_hi_backend/worker/tests.rs`
- **Acceptance:**
  - `retry.rs` traz `RetryPolicy` (consts de 3 tentativas / 50ms, sem número mágico) e `Clock`/`SystemClock`. O worker aplica A-8 em `read_capabilities`/`read_vcp`/`write_vcp`; enumerate não falha e não retenta. Job vencido não toca o hardware. A falha final passa pela checagem de presença (A-5), e a mensagem de `Transport` cita o número de tentativas.
  - DoD `retries_transient_errors_up_to_three_times_within_timeout_budget` (`worker/tests.rs`: op do `Worker` com handle fake + clock virtual):
    - 2 falhas + sucesso → `Ok`, 3 chamadas, sleeps `[50ms, 50ms]`.
    - Falha persistente → `Transport`, 3 chamadas, nenhuma 4ª.
    - Budget de 80ms → 2 chamadas, 1 sleep, `Transport`.
  - Testes extras:
    - `expired_write_is_never_sent_to_the_monitor`: op preso + write com budget curto → caller recebe `Timeout` → o teste libera → o fake não viu o write.
    - `monitor_gone_after_a_failure_is_not_found`.
  - Testes do DoD de T-3 intactos. Testes com worker spawnado injetam clock fake ou política sem backoff: nada dorme de verdade mais que poucos ms, fora o teste de timeout. Cobertura ≥ 80% (esperado ≥ 90%).
- **Dependencies:** T-3
- **Test:** verify #7 do DoD + `cargo test -p ddc-adapters --locked` + cobertura (abaixo)
- **Status:** completed

### Wave 4

#### T-5: Testes read-only no monitor real (`#[ignore]` + `DDC_HW_TESTS=1`)
- **Specialist:** jdi-doer-ddc-control
- **Files modified:** `crates/ddc-adapters/tests/real_monitor.rs`
- **Acceptance:**
  - Cada teste é `#[ignore = "..."]` e retorna cedo sem `DDC_HW_TESTS=1`. Backend novo por teste, `enumerate()` primeiro, RTK achado por fabricante `RTK` + modelo `RTK QHD HDR`. Testes:
    - `hardware_enumerates_the_rtk_monitor_by_edid_id`: id `RTK-RTK-QHD-HDR-01010101`.
    - `hardware_capabilities_match_the_core_fixture`: igualdade exata.
    - `hardware_reads_brightness_and_vcp_version`: 0x10 com `0 < max` e `current ≤ max`; 0xDF com `current == 0x0202` (MCCS 2.2 — valida D-5 no hardware).
    - `hardware_mute_displays_fail_fast_and_worker_recovers`: em todo outro display, `read_vcp(0x10)` volta em ≤ budget + 250ms, com erro só `Transport`/`Timeout`; depois o RTK ainda lê.
    - Todos imprimem os tempos (evidência de D-7: enumerate < 5s, caps < 8s, VCP < 1s com os defaults).
  - Só leitura: `grep -n write_vcp crates/ddc-adapters/tests/real_monitor.rs` sem saída; hits do grep 5.7 só dentro de corpos `#[ignore]`. `cargo test --workspace --locked` → 4 ignored, 0 failed.
  - A rodada real é do orquestrador: `DDC_HW_TESTS=1 cargo test -p ddc-adapters --locked --test real_monitor -- --ignored --test-threads=1 --nocapture`. O doer pode rodar (só lê), mas não é gate do commit.
- **Dependencies:** T-4
- **Test:** `cargo test -p ddc-adapters --locked --test real_monitor -- --ignored` sem a env (passa sem executar nada)
- **Status:** completed

### Wave 5

#### T-6: Doc Linux + README + CHANGELOG
- **Specialist:** jdi-doer-ddc-control
- **Files modified:** `docs/linux-ddc-setup.md`, `README.md`, `CHANGELOG.md`
- **Acceptance:**
  - `docs/linux-ddc-setup.md` (em inglês) cobre:
    - headers do libudev para o build (R-2);
    - carregar e persistir `i2c-dev`;
    - permissão via regra udev `uaccess` ou grupo `i2c`, e como verificar;
    - ressalva do driver NVIDIA proprietário (DDC bloqueado em algumas saídas → `Transport` ou monitor ausente);
    - nunca rodar com `sudo`: o setup do sistema usa root uma vez, a ferramenta nunca;
    - display com DDC mudo aparece na lista e falha na leitura (D-4); lista vazia = permissão.
    - Verify #10 do DoD = OK.
  - README:
    - status: phases 1–2, ainda sem binário;
    - layout com `DdcHiMonitorBackend`: feature `ddc-hi` default-on, worker único, esquema de `MonitorId`, 1s/3×/50ms;
    - dev setup: `rustup target add x86_64-pc-windows-msvc` + check Windows, libudev, comando dos testes de hardware;
    - link para o doc Linux.
  - `CHANGELOG.md` `[Unreleased]` com o que a phase adicionou. Commit `docs`.
- **Dependencies:** T-2, T-5
- **Test:** verify #10 do DoD
- **Status:** completed

## Execution
- Total tasks: 6 | Waves: 5 (só a W1 é paralelizável) | Speedup estimado: 1.2x
- Tipos de commit: T-1 `chore`, T-2 `test`, T-3/T-4 `feat`, T-5 `test`, T-6 `docs`. T-3 é a maior task (~600 linhas com testes, abaixo do WARN de 800 do hook). Ela não se divide sem código morto: todo item interno só fica vivo através do `DdcHiMonitorBackend` público.
- DoD da CONTEXT: #1 → T-1; #2–#6, #8, #9 → T-3; #7 → T-4; #10 → T-6.
- Carry-overs: W-4 → T-2; `MonitorNotFound` de monitor sumido → T-3/T-4 (A-5); NUL na caps → T-3 (A-7).

## Files modified (all tasks)
- `crates/ddc-adapters/Cargo.toml`, `Cargo.lock`
- `crates/ddc-adapters/src/lib.rs`, `crates/ddc-adapters/src/in_memory.rs`, `crates/ddc-adapters/src/in_memory/tests.rs`
- `crates/ddc-adapters/src/ddc_hi_backend.rs`, `crates/ddc-adapters/src/ddc_hi_backend/{tests,identity,worker,hardware,retry}.rs`
- `crates/ddc-adapters/src/ddc_hi_backend/{identity,worker,hardware}/tests.rs`
- `crates/ddc-adapters/tests/real_monitor.rs`
- `crates/ddc-core/tests/monitor_control/read_features.rs`
- `docs/linux-ddc-setup.md`, `README.md`, `CHANGELOG.md`

## Test requirements
- Unit + integração: `cargo test --workspace --locked`. Hardware: comando de T-5, rodado pelo orquestrador ou pelo dev, nunca pelo reviewer.
- Cobertura ≥ 80% de linhas: `cargo llvm-cov --workspace --locked --summary-only --fail-under-lines 80 --ignore-filename-regex '(^|[/\\])(main|build)\.rs$'`.
- A casca não coberta fica em ≤ 40 linhas (T-3); o resto é coberto por fakes, então o TOTAL esperado é ≥ 90%.
