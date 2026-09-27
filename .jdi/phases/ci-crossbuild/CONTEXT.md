# Phase 6: CI cross-build — Context  (slug: ci-crossbuild)

## Goal
GitHub Actions no `slipalison/ddc-control`, a cada PR e push na `main`, rodando em `ubuntu-latest` e `windows-latest`: build, `cargo fmt --check`, `cargo clippy --all-targets --all-features -D warnings`, testes com cobertura (`cargo llvm-cov`, piso 80% no Linux), compilação release do `apps/ddc-tray` nos dois SOs, `cargo audit` com exceções justificadas, testes da UI do popup (`node --test` + Playwright). Tudo via templates reutilizáveis do `slipalison/github-workflows`, estendidos lá — nunca copiados para o ddc-control.

## Locked decisions
- D-2026-09-27-ci-crossbuild-1: arquitetura do `.github/workflows/ci.yml` (triggers, concurrency, permissions, jobs `versao`+`qualidade`), branch temporária do `github-workflows` durante a phase, e arquivo de evidência `.jdi/phases/ci-crossbuild/ci-evidence.env` (`RUN_ID`, `HEAD_SHA`, `WORKFLOWS_BRANCH`, `REPO`) que os `Verify:` abaixo consultam.
- D-2026-09-27-ci-crossbuild-2: extensões retrocompatíveis do `qualidade.yml` no `github-workflows` — campo `so` (runner por componente), `pacotes_sistema` (apt validado por regex, só Linux), `cargo-llvm-cov` resolvido por SO, `auditoria` (cargo audit por binário fixo lendo `.cargo/audit.toml` do chamador).
- D-2026-09-27-ci-crossbuild-3: componentes concretos no `ci.yml` do ddc-control — `rust-linux` (cobertura 80, `pacotes_sistema` do Tauri, `auditoria: true`), `rust-windows` (`so: windows-latest`, cobertura 0 — o piso vale só no Linux —, sem `auditoria` duplicada), `node-ui` (`apps/ddc-tray`, `npm test` passa a rodar `node --test` + Playwright, browsers via `npx playwright install --with-deps chromium` no runner do CI, não na máquina do usuário).
- D-2026-09-27-ci-crossbuild-4: passo explícito `Build ddc-tray (release)` (`cargo build -p ddc-tray --release --locked`) dentro do job rust, nos dois SOs — prova de compilação real do binário de release, não só do modo instrumentado do `cargo llvm-cov`.
- D-2026-09-27-ci-crossbuild-5: correção do pitfall `tauri-apps/tauri#13419` em `apps/ddc-tray/src-tauri/build.rs` — `WindowsAttributes::new_without_app_manifest()` + `/MANIFEST:EMBED`/`/MANIFESTINPUT:<arquivo>` condicionados a `CARGO_CFG_TARGET_OS`/`CARGO_CFG_TARGET_ENV` lidos em runtime do build script (nunca `#[cfg(windows)]`).
- D-2026-09-27-ci-crossbuild-6: `.cargo/audit.toml` no ddc-control com as 4 entradas de `ignore` (RUSTSEC-2018-0005/-2024-0370/-2024-0320/-2024-0429), cada uma comentada com motivo medido, caminho de dependência e condição de reavaliação.
- D-2026-09-27-ci-crossbuild-7: prova de negativo do portão de auditoria via `git worktree` descartável, removendo o ignore de RUSTSEC-2018-0005 e confirmando que `cargo audit` reprova — sem precisar de um segundo run do GitHub Actions.
- D-2026-09-27-ci-crossbuild-8: `versao` só calcula (nunca cria tag/release) nesta phase; documentação (README do `github-workflows` seção Rust, README do ddc-control seção CI) ganha os títulos novos, revisão de fundo é humana (`## Deferred to PR review`).

## Canonical refs
- Card do usuário via `/jdi-issue`, 2026-09-27: "eu preciso que crie um ci/cd com Releases, e use ou adicione nos templates que estão no diretorio /home/slipalison/repos/github-workflows/" (1ª de 2 phases; a 2ª é `release-packaging`).
- Brief completo desta phase (fatos medidos, convenções obrigatórias do `github-workflows`, estratégia de prova): repassado ao asker pelo orquestrador, refletido integralmente nas decisões acima.
- `/home/slipalison/repos/github-workflows/.github/workflows/qualidade.yml`, `.github/actions/preparar/action.yml`, `.github/actions/relatar-cobertura/action.yml`, `bin/cobertura.py`, `actions.lock.json`, `README.md` (seções "Rust", "O que tem aqui", "Segurança") — lidos nesta captura; formato exato do painel de cobertura (`**Aprovado:** X.XX% >= piso de 80%.`) usado nos `Verify:` abaixo vem de `bin/cobertura.py`.
- `apps/ddc-tray/src-tauri/build.rs` (estado atual, sem o fix do manifesto), `apps/ddc-tray/package.json` (`test` só roda Playwright hoje).
- `.jdi/phases/tray-app/CONTEXT.md` (lição do teto de 15 iterações com `Verify:` grep-based — motivo de toda prova aqui ser consulta ao run real).

## Out of scope
- Bundles (MSI/NSIS/deb/rpm/AppImage), regra udev empacotada, binários do `ddc-cli` para download, tag e GitHub Release → phase `release-packaging`.
- `seguranca.yml` (Semgrep/Gitleaks/Trivy/CodeQL) → candidato a todo, registrado, fora desta phase.
- Testes de hardware (`DDC_HW_TESTS=1`) no CI — nunca rodam lá (sem monitor real no runner); comportamento já estabelecido nas phases anteriores, não muda aqui.
- Cross-check Windows fora do CI — sem `llvm-rc` local (D-2026-09-26-tray-app-9), toda prova Windows desta phase é o run real do GitHub Actions.

## Definition of Done

### Auto-verifiable
- [ ] O run real registrado como evidência (`RUN_ID`) terminou com sucesso no `HEAD_SHA`. Esse commit é ancestral do HEAD e tem a árvore de código idêntica à do HEAD (`crates/`, `apps/`, `Cargo.*`, `.cargo/`). Todo `uses:` do `github-workflows` no `ci.yml` daquele commit aponta para `@$WORKFLOWS_SHA`. O `ci.yml` do HEAD é o mesmo, a menos dessa referência.
      **Verify:** `. .jdi/phases/ci-crossbuild/ci-evidence.env; r=$(gh run view "$RUN_ID" --repo "$REPO" --json conclusion,headSha --jq '"\(.conclusion) \(.headSha)"'); [ "$r" = "success $HEAD_SHA" ] && git merge-base --is-ancestor "$HEAD_SHA" HEAD && git diff --quiet "$HEAD_SHA" HEAD -- crates apps Cargo.toml Cargo.lock .cargo && R=$(git show "$HEAD_SHA":.github/workflows/ci.yml | grep -oE 'slipalison/github-workflows/[^@[:space:]]+@[^[:space:]]+') && [ "$(echo "$R" | wc -l)" -ge 2 ] && ! echo "$R" | grep -qv "@$WORKFLOWS_SHA\$" && diff <(git show "$HEAD_SHA":.github/workflows/ci.yml | sed -E 's#(slipalison/github-workflows/[^@[:space:]]+)@[^[:space:]]+#\1@REF#g') <(git show HEAD:.github/workflows/ci.yml | sed -E 's#(slipalison/github-workflows/[^@[:space:]]+)@[^[:space:]]+#\1@REF#g') >/dev/null && echo OK` (D-2026-09-27-ci-crossbuild-1)
      **Source:** CONTEXT
- [ ] O código do template que rodou é exatamente o do PR aberto no `github-workflows`: o head do PR `WORKFLOWS_PR` é `WORKFLOWS_SHA`. O CI do próprio `github-workflows` nesse SHA passou nos jobs `Scripts`, `Pins das actions` e `Sintaxe dos workflows` (actionlint + exemplos + pins), sem nenhum check falho ou cancelado.
      **Verify:** `. .jdi/phases/ci-crossbuild/ci-evidence.env; W=slipalison/github-workflows; h=$(gh pr view "$WORKFLOWS_PR" --repo $W --json headRefOid --jq .headRefOid); C=$(gh api "repos/$W/commits/$WORKFLOWS_SHA/check-runs?per_page=100"); [ "$h" = "$WORKFLOWS_SHA" ] && [ "$(echo "$C" | jq '[.check_runs[] | select(.name=="Scripts" or .name=="Pins das actions" or .name=="Sintaxe dos workflows") | select(.conclusion=="success")] | length')" = 3 ] && ! echo "$C" | jq -r '.check_runs[].conclusion' | grep -qE 'failure|cancelled|timed_out|action_required' && echo OK` (D-2026-09-27-ci-crossbuild-1/-2)
      **Source:** CONTEXT
- [ ] O run tem exatamente um job `rust-linux` com label `ubuntu-latest` e um `rust-windows` com label `windows-latest`. Terminam com `conclusion == success` (nenhum `skipped`): no Linux, `cargo fmt --check`, `cargo clippy`, `cargo test com cobertura`, `Cobertura`, `cargo audit` e `Build ddc-tray (release)`; no Windows, os mesmos, menos `cargo audit`.
      **Verify:** `. .jdi/phases/ci-crossbuild/ci-evidence.env; J=$(gh api "repos/$REPO/actions/runs/$RUN_ID/jobs?per_page=100"); L=$(echo "$J" | jq -c '[.jobs[] | select(.name|test("(^|/ )rust-linux$"))]'); X=$(echo "$J" | jq -c '[.jobs[] | select(.name|test("(^|/ )rust-windows$"))]'); [ "$(echo "$L" | jq length)" = 1 ] && [ "$(echo "$X" | jq length)" = 1 ] && echo "$L" | jq -e '.[0].labels == ["ubuntu-latest"]' >/dev/null && echo "$X" | jq -e '.[0].labels == ["windows-latest"]' >/dev/null && [ "$(echo "$L" | jq '[.[0].steps[] | select(.name as $n | ["cargo fmt --check","cargo clippy","cargo test com cobertura","Cobertura","cargo audit","Build ddc-tray (release)"] | index($n)) | select(.conclusion=="success")] | length')" = 6 ] && [ "$(echo "$X" | jq '[.[0].steps[] | select(.name as $n | ["cargo fmt --check","cargo clippy","cargo test com cobertura","Cobertura","Build ddc-tray (release)"] | index($n)) | select(.conclusion=="success")] | length')" = 5 ] && echo OK` (D-2026-09-27-ci-crossbuild-2/-3/-4; nomes são contrato, ver `## Notes`)
      **Source:** CONTEXT
- [ ] No log do `rust-linux`: o teste `build_tests::the_effective_csp_is_the_strict_policy_on_every_target` do `ddc-tray` passa; há ao menos um `test result: ok. N passed; 0 failed`; nenhum binário de teste falha; e o painel da cobertura aprova as linhas no piso de 80%.
      **Verify:** `. .jdi/phases/ci-crossbuild/ci-evidence.env; J=$(gh api "repos/$REPO/actions/runs/$RUN_ID/jobs?per_page=100"); id=$(echo "$J" | jq -r '.jobs[] | select(.name|test("(^|/ )rust-linux$")) | .id'); LOG=$(gh run view --repo "$REPO" --job "$id" --log); echo "$LOG" | grep -qE 'test build_tests::the_effective_csp_is_the_strict_policy_on_every_target \.\.\. ok' && echo "$LOG" | grep -qE 'test result: ok\. [1-9][0-9]* passed; 0 failed' && ! echo "$LOG" | grep -qE 'test result: FAILED|[1-9][0-9]* failed;' && echo "$LOG" | grep -qE '\*\*Aprovado:\*\* (8[0-9]|9[0-9]|100)\.[0-9]{2}% >= piso de 80%' && echo OK` (texto do painel vem de `bin/cobertura.py::painel`, que também o imprime no stdout)
      **Source:** CONTEXT
- [ ] No log do `rust-windows`, os testes do `ddc-tray` rodam e passam no Windows (o mesmo teste nomeado da linha anterior). Nenhum binário de teste falha, e não aparece `STATUS_ENTRYPOINT_NOT_FOUND` (tauri#13419). O número de binários de teste com `test result: ok.` é igual ao do Linux no mesmo run, então nenhum crate saiu dos testes no Windows.
      **Verify:** `. .jdi/phases/ci-crossbuild/ci-evidence.env; J=$(gh api "repos/$REPO/actions/runs/$RUN_ID/jobs?per_page=100"); iw=$(echo "$J" | jq -r '.jobs[] | select(.name|test("(^|/ )rust-windows$")) | .id'); il=$(echo "$J" | jq -r '.jobs[] | select(.name|test("(^|/ )rust-linux$")) | .id'); LW=$(gh run view --repo "$REPO" --job "$iw" --log); LL=$(gh run view --repo "$REPO" --job "$il" --log); echo "$LW" | grep -qE 'test build_tests::the_effective_csp_is_the_strict_policy_on_every_target \.\.\. ok' && ! echo "$LW" | grep -qE 'test result: FAILED|[1-9][0-9]* failed;|STATUS_ENTRYPOINT_NOT_FOUND' && nw=$(echo "$LW" | grep -cE 'test result: ok\.') && nl=$(echo "$LL" | grep -cE 'test result: ok\.') && [ "$nw" -ge 4 ] && [ "$nw" = "$nl" ] && echo OK` (D-2026-09-27-ci-crossbuild-5)
      **Source:** CONTEXT
- [ ] O job `node-ui` do run termina com sucesso, e o passo `npm test` tem `conclusion == success`. No log, o `node --test` (TAP) mostra `# pass` ≥ 150 e `# fail 0`, e o Playwright mostra ≥ 130 `passed` sem nenhum `failed`. A base medida em 2026-09-27 é de 157 testes node e 138 do Playwright.
      **Verify:** `. .jdi/phases/ci-crossbuild/ci-evidence.env; J=$(gh api "repos/$REPO/actions/runs/$RUN_ID/jobs?per_page=100"); id=$(echo "$J" | jq -r '.jobs[] | select(.name|test("(^|/ )node-ui$")) | .id'); c=$(echo "$J" | jq -r '.jobs[] | select(.name|test("(^|/ )node-ui$")) | .conclusion'); s=$(echo "$J" | jq -r '.jobs[] | select(.name|test("(^|/ )node-ui$")) | .steps[] | select(.name=="npm test") | .conclusion'); LOG=$(gh run view --repo "$REPO" --job "$id" --log); np=$(echo "$LOG" | grep -oE '# pass [0-9]+' | tail -1 | grep -oE '[0-9]+$'); pp=$(echo "$LOG" | grep -oE '[0-9]+ passed \(' | tail -1 | grep -oE '^[0-9]+'); [ "$c" = success ] && [ "$s" = success ] && [ "${np:-0}" -ge 150 ] && echo "$LOG" | grep -qE '# fail 0' && [ "${pp:-0}" -ge 130 ] && ! echo "$LOG" | grep -qE '[1-9][0-9]* failed' && echo OK` (D-2026-09-27-ci-crossbuild-3)
      **Source:** CONTEXT
- [ ] O portão de auditoria morde no CI real. O commit `NEG_SHA` (ancestral do HEAD) muda SÓ `.cargo/audit.toml`, e só removendo linhas, entre elas a exceção de RUSTSEC-2018-0005. O arquivo do HEAD voltou ao estado anterior ao `NEG_SHA`. O `ci.yml` do `NEG_SHA` é byte a byte o do `HEAD_SHA` (mesmo template). O run `NEG_RUN_ID` desse commit reprovou, com o passo `cargo audit` do `rust-linux` em `failure` e o advisory no log.
      **Verify:** `. .jdi/phases/ci-crossbuild/ci-evidence.env; [ -n "$NEG_RUN_ID" ] && [ -n "$NEG_SHA" ] && git merge-base --is-ancestor "$NEG_SHA" HEAD && [ "$(git diff --name-only "$NEG_SHA^" "$NEG_SHA")" = ".cargo/audit.toml" ] && git diff "$NEG_SHA^" "$NEG_SHA" -- .cargo/audit.toml | grep -qE '^-.*RUSTSEC-2018-0005' && ! git diff "$NEG_SHA^" "$NEG_SHA" -- .cargo/audit.toml | grep -qE '^\+[^+]' && git diff --quiet "$NEG_SHA^" HEAD -- .cargo/audit.toml && [ "$(git show "$NEG_SHA":.github/workflows/ci.yml | sha256sum)" = "$(git show "$HEAD_SHA":.github/workflows/ci.yml | sha256sum)" ] && r=$(gh run view "$NEG_RUN_ID" --repo "$REPO" --json headSha,conclusion --jq '"\(.headSha) \(.conclusion)"') && [ "$r" = "$NEG_SHA failure" ] && J=$(gh api "repos/$REPO/actions/runs/$NEG_RUN_ID/jobs?per_page=100") && [ "$(echo "$J" | jq -r '.jobs[] | select(.name|test("(^|/ )rust-linux$")) | .steps[] | select(.name=="cargo audit") | .conclusion')" = failure ] && id=$(echo "$J" | jq -r '.jobs[] | select(.name|test("(^|/ )rust-linux$")) | .id') && gh run view --repo "$REPO" --job "$id" --log | grep -q RUSTSEC-2018-0005 && echo OK` (D-2026-09-27-ci-crossbuild-7(a); o doer espera o run negativo TERMINAR antes de empurrar a reversão, porque o `cancel-in-progress` o cancelaria)
      **Source:** CONTEXT
- [ ] `.cargo/audit.toml` ignora exatamente as 4 advisories, e `[advisories]` não tem nenhuma outra chave que esconda achado. `cargo audit` sai 0 no HEAD. Numa cópia descartável (`git worktree`) sem a exceção de RUSTSEC-2018-0005, `cargo audit` sai diferente de 0, citando o advisory. A cópia é removida.
      **Verify:** `python3 -c 'import tomllib,sys; a=tomllib.load(open(".cargo/audit.toml","rb")).get("advisories",{}); sys.exit(0 if sorted(a.get("ignore",[]))==["RUSTSEC-2018-0005","RUSTSEC-2024-0320","RUSTSEC-2024-0370","RUSTSEC-2024-0429"] and set(a)<={"ignore"} else 1)' && cargo audit >/dev/null 2>&1 && W=$(mktemp -d) && git worktree add -q --detach "$W" HEAD && sed -i '/RUSTSEC-2018-0005/d' "$W/.cargo/audit.toml" && { (cd "$W" && cargo audit) >"$W.log" 2>&1; rc=$?; git worktree remove --force "$W"; [ "$rc" != 0 ] && grep -q RUSTSEC-2018-0005 "$W.log"; } && rm -f "$W.log" && echo OK` (D-2026-09-27-ci-crossbuild-6/-7(b); `--file` do cargo-audit é o Cargo.lock, por isso não aparece aqui)
      **Source:** CONTEXT

### Manual
- _(none)_

## Deferred to PR review`; assume os dois repositórios clonados lado a lado, como estão hoje)
      **Source:** CONTEXT

### Manual
- _(none)_

## Deferred to PR review
- Merge do PR do `github-workflows` PRIMEIRO. Depois, o commit final do ddc-control troca `@<WORKFLOWS_SHA>` por `@main` em `.github/workflows/ci.yml`, e o CI roda de novo, verde, contra a `main` do template.
- Revisão do diff dos templates do `github-workflows` (`qualidade.yml` e, se mudar, `preparar`): é código de terceiro que roda com o token dos dois repositórios.
- Revisão do texto das seções novas de README (ddc-control: CI; github-workflows: Rust/Windows/`pacotes_sistema`/`auditoria`/build de release) e do exemplo em `exemplos/`. Não há checagem automática de README (D-2026-09-27-ci-crossbuild-8).
- Tirar do rascunho o PR da `phase/ci-crossbuild`, que foi aberto cedo para haver runs de `pull_request`.
- Confirmação manual dos 2 itens `Manual` da baseline de `.jdi/PROJECT.md` (CHANGELOG/README), herdados pelo Gate 8.

## Notes
- Contrato de nomes exigido pelos `Verify:`:
  - componentes `rust-linux`, `rust-windows` e `node-ui` (na API aparecem como `qualidade / <nome>`, e os `Verify:` casam o sufixo);
  - passos `cargo fmt --check`, `cargo clippy`, `cargo test com cobertura` e `Cobertura` (já existem no template);
  - passos `cargo audit` e `Build ddc-tray (release)` (novos);
  - no `node-ui`, o passo `npm test` (já existe).
- `.jdi/phases/ci-crossbuild/ci-evidence.env` é do doer, versionado, `KEY=VALUE` por linha: `REPO=slipalison/ddc-control`, `RUN_ID`, `HEAD_SHA`, `WORKFLOWS_SHA`, `WORKFLOWS_PR`, `NEG_RUN_ID` e `NEG_SHA`.
  - É reescrito a cada run relevante.
  - O Gate 8 consulta sempre o que está ali, nunca um id fixo no CONTEXT.
  - A ordem natural no fim da phase é:
    1. commit negativo;
    2. esperar o run negativo terminar vermelho;
    3. commit de reversão;
    4. run verde de evidência, com o mesmo `ci.yml`.
- O PR do `github-workflows` sai de uma branch a partir de `origin/main` de lá, com commits no padrão daquele repositório: `tipo(escopo): assunto` em pt-BR sem acento, conferido pelo `hooks/commit-msg` dele via `bin/versao.py conferir`.
- Os dois repositórios ficam clonados lado a lado em `/home/slipalison/repos/`.
- A baseline de `.jdi/PROJECT.md` (`cargo test --workspace`, cobertura ≥ 80%, TODO/FIXME) e a do Gate 7 (suíte Playwright do `apps/ddc-tray`) continuam herdadas pelo reviewer, sem duplicação aqui.
- `versao_inicial: "0.1.0"` casa com `[workspace.package] version = "0.1.0"`.
