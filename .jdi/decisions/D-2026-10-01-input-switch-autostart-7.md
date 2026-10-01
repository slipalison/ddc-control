D-2026-10-01-input-switch-autostart-7 (2026-10-01): Nona rodada do critic (rodada 2, iteração 4; 9 das 15 iterações absolutas). Cada fechamento por teste da linha 1 expôs uma variante no código de produção vizinho:
- `Worker::run`;
- `spawn`;
- `Clock`;
- `WorkerClient::call`/`transact`;
- `Worker::serve`;
- uma troca de módulo só no build de produção.

O critic mostrou que até as correções pontuais propostas (testar `worker.policies` pela fila, fixar `call`/`transact`) deixam passar N5dl e N5serve. A família não converge por testes antes do teto.

Decisão: fechamento ESTRUTURAL. A linha 1 passa a verificar o código REVISADO. O Verify exige quatro coisas:
- a árvore git de `crates/ddc-adapters` (todo o crate: `src/**`, `tests/`, `Cargo.toml`) igual ao id `e87361dd…`;
- o `Cargo.toml` e o `Cargo.lock` da raiz iguais aos blobs `a32052f8…` e `56607a7f…`;
- nenhuma diferença do working tree contra o HEAD nesses caminhos;
- nenhum arquivo não rastreado sob `crates/ddc-adapters`.

Ids de árvore e de blob dependem só do conteúdo, então a linha continua valendo depois do squash-merge. Sobre esse código congelado, os testes nomeados provam o comportamento afirmado. Qualquer mudança no crate, inclusive testes, `lib.rs`, `hardware.rs`, `identity.rs`, `caching.rs`, um `#[path]`/`cfg_attr` ou uma macro, faz o Verify falhar até nova revisão. As fixações anteriores ficam, redundantes e inofensivas:
- SHA-256 dos testes;
- corpo de `write_vcp`;
- conjunto `INPUT_SETTLE_*`;
- dep-info.

A linha 8 ganha a mesma conferência, o que fecha a W-24 (macro/`include_str!`) e a H8 (helper sem `cfg`).

O "NÃO afirmado" da linha 1 ganha dois itens:
- o comportamento com um monitor real (`DdcHiDisplays`, deferido ao PR, W-3);
- as camadas acima do adapter, salvo o que a linha 2 afirma.

A faixa do piso relativo do passo passa a "k ≤ 99 900": acima disso, `Duration` trunca 100 ms/k para 1 µs (P1).

Linha 2 (objective). O texto "o assentamento só existe no adapter" ia além do Verify. Ele passa a afirmar o que é conferido: o assentamento não toca o `ddc-core` nem as camadas que chamam o `set_feature` do core (`crates/ddc-cli`, `apps/ddc-tray/src-tauri/src/commands.rs`), todos sem nenhum byte de diff contra a base.
