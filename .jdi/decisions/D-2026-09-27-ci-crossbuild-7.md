D-2026-09-27-ci-crossbuild-7 (2026-09-27): O portão de auditoria é provado vermelho de duas formas.
- (a) No CI real:
  - no PR, um commit que SÓ tira de `.cargo/audit.toml` a exceção de RUSTSEC-2018-0005;
  - o run dele reprova no passo `cargo audit` do `rust-linux`, com o advisory no log;
  - o commit seguinte devolve o arquivo ao estado anterior;
  - o run negativo usa o mesmo `ci.yml` (mesmo `WORKFLOWS_SHA`) do run de evidência;
  - `NEG_RUN_ID`/`NEG_SHA` ficam em `ci-evidence.env`.
- (b) Localmente, num `git worktree` descartável, a mesma remoção faz `cargo audit` sair diferente de zero, citando o advisory.

O (a) prova o passo do template. O (b) prova a regra do arquivo sem depender da rede do CI.
