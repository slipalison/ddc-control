D-2026-09-28-release-packaging-8 (2026-09-28): Estratégia de prova sem publicar a release real. Substitui a versão anterior, cujo item 3 era um rascunho criado à mão pela API e não exercitava o `lancar.yml`.

1. **Run real de `pull_request`** (`RUN_ID`, `HEAD_SHA` em `release-evidence.env`):
   - `versao` calcula `salto=inicial versao=0.1.0`, porque não há tag;
   - `qualidade` inteira verde, com os pacotes dos dois SOs como artefatos `pacotes-rust-linux` e `pacotes-rust-windows`;
   - o job `lancar` aparece na API como `skipped`. Esse é o negativo: pull_request nunca publica.
2. **Artefatos baixados e inspecionados por ferramenta** (`ar`/`tar`, `rpm -qp`, `rpm2cpio`, `unzip`, `file`, magic bytes, e o `ddc-cli --version` do tar.gz executado). Isso inclui a VERSÃO carimbada: o `Cargo.toml` do repositório diz `0.0.0` (D-10), então `0.1.0` dentro dos pacotes só pode vir do carimbo.
3. **Ensaio real do `lancar.yml`** (`ENSAIO_RUN_ID`, `ENSAIO_BRANCH`, `DRAFT_RELEASE_ID`):
   - o doer empurra o MESMO `HEAD_SHA` para uma branch `ensaio-release/<algo>`;
   - o `ci.yml` desse commit roda `versao`, `qualidade` e `lancar` com `rascunho: true`;
   - o próprio workflow cria o RASCUNHO `v0.1.0` com os 7 pacotes + `SHA256SUMS` e as notas do CHANGELOG;
   - o DoD confere o rascunho pela API: `draft`, `tag_name`, alvo, o conjunto de assets igual ao conjunto de artefatos do run + `SHA256SUMS`, o `sha256sum -c` dos assets baixados e o texto do CHANGELOG no corpo.
4. **O rascunho FICA** até o merge, para o humano baixar e instalar os pacotes (Deferred). O primeiro `lancar` real na `main` apaga o rascunho da mesma tag antes de publicar (D-2). A branch `ensaio-release/*` pode ser apagada depois do run.
5. **Nenhuma tag `v*` existe** no repositório durante a phase (`git/matching-refs/tags/v` vazio). Rascunho não cria tag, e a release real é Deferred.
