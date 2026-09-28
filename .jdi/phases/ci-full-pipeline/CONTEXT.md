# Phase 8: CI full pipeline — Context  (slug: ci-full-pipeline)

## Goal
O `ci.yml` vira uma chamada só ao `pipeline.yml` do `slipalison/github-workflows`, com a esteira inteira:
- qualidade, com a UI ganhando piso de cobertura;
- segurança: Gitleaks, TruffleHog, Semgrep, Trivy, SBOM e CodeQL de Rust, JS/TS e Actions;
- Sonar com Quality Gate: Rust pelas regras do Sonar e LCOV, JS por LCOV;
- o `Portao` como check único.

No template, o app desktop passa a entrar pelo `pipeline.yml`, e o Sonar só se desliga com o motivo escrito.

## Locked decisions
- D-2026-09-28-ci-full-pipeline-1: `ci.yml` = um job `esteira` com `uses: …/pipeline.yml@<ref>`. Saem os `versao`, `qualidade` e `lancar` avulsos. As decisões da `release-packaging` viram inputs do pipeline.
- D-2026-09-28-ci-full-pipeline-2: `seguranca.yml` inteiro, com `linguagens_codeql: '["rust","javascript-typescript","actions"]'`. Achado bloqueante se conserta. Exceção só em `.trivyignore`, com a justificativa medida.
- D-2026-09-28-ci-full-pipeline-3: Sonar pelo CI (`slipalison_ddc-control`, Rust 1.98, `sonar_pacotes_sistema`). A cobertura vem de `scripts/ci/sonar-coverage.sh` (LCOV do Rust e da UI). O secret e a Análise Automática desligada são pré-requisitos do usuário.
- D-2026-09-28-ci-full-pipeline-4: `node-ui` com `"cobertura":80`, pelo LCOV do `node --test` (`npm run test:unit`). O `app.js` fica fora desse número.
- D-2026-09-28-ci-full-pipeline-5: template no gw #15, empilhado no #14. O `ci.yml` aponta para a branch descartável `teste/esteira-app-desktop` até o merge, e depois para `@main`.
- D-2026-09-28-ci-full-pipeline-6: o `Portao` como status check obrigatório do ruleset é recomendação ao usuário, e só se aplica com o OK dele.

## Canonical refs
- Pedido do usuário (2026-09-28): "está sem test, validação de segurança, sonarqube e etc. [...] seguir os padrões de validação e de qualidade que o github-workflow exige (se não estiver exigindo deixe de alguma forma que deixe claro a exigencia)".
- gw: `.github/workflows/{pipeline,seguranca,sonar,qualidade,lancar}.yml` e o README ("O mínimo exigido", "Sonar para todo mundo", "Pré-requisitos fora daqui").
- `.jdi/phases/ci-crossbuild/CONTEXT.md:25`: `seguranca.yml` ficou "fora desta phase" e nunca virou todo. O Sonar nunca foi considerado.
- Run 36453793875 (`main`, v0.1.0): o `node-ui` saiu verde com a anotação de erro de cobertura ausente.

## Out of scope
- Consertar o `coluna`, que quebra com o `Portao` novo: vai no PR do gw e no relatório, como uma linha para o usuário.
- Mudar o ruleset (D-6).
- Assinatura de código dos instaladores.
- Cobertura do `app.js` pelo Playwright (V8 → LCOV): exige dependência nova, e fica como candidato a todo.

## Definition of Done

Evidência em `.jdi/phases/ci-full-pipeline/pipeline-evidence.env` (`KEY=VALUE`): `REPO`, `PR`, `RUN_ID`, `HEAD_SHA`, `WORKFLOWS_PR`, `WORKFLOWS_SHA`, `WORKFLOWS_TEST_SHA`. Todo `Verify:` roda em `bash --noprofile --norc -eo pipefail` com `LC_ALL=C.UTF-8`.

### Auto-verifiable
- [ ] O `ci.yml` tem um job só, `esteira`, e um `uses:` só, o do `pipeline.yml`, no `WORKFLOWS_TEST_SHA` ou em `@main`. Os inputs ligam o Sonar (`sonar_projeto`, sem `sonar_dispensa`), o CodeQL das três linguagens e o piso 80 da UI.
      **Verify:** `export LC_ALL=C.UTF-8; . .jdi/phases/ci-full-pipeline/pipeline-evidence.env; python3 -c 'import sys,yaml,json; d=yaml.safe_load(open(".github/workflows/ci.yml")); j=d["jobs"]; assert list(j)==["esteira"], list(j); e=j["esteira"]; u=e["uses"]; assert u in ("slipalison/github-workflows/.github/workflows/pipeline.yml@"+sys.argv[1], "slipalison/github-workflows/.github/workflows/pipeline.yml@main"), u; w=e["with"]; assert w["sonar_projeto"]=="slipalison_ddc-control" and "sonar_dispensa" not in w; assert sorted(json.loads(w["linguagens_codeql"]))==["actions","javascript-typescript","rust"]; c={x["nome"]:x for x in json.loads(w["componentes"])}; assert c["node-ui"]["cobertura"]==80 and c["rust-linux"]["cobertura"]==80; assert e["secrets"]["SONAR_TOKEN"]=="${{ secrets.SONAR_TOKEN }}"; print("OK")' "$WORKFLOWS_TEST_SHA"`
- [ ] Run real de `pull_request` (`RUN_ID`) no `HEAD_SHA`, em `success`. O CONJUNTO de jobs com `success` contém todos os da esteira, e o `lancar` não roda em PR.
      **Verify:** `export LC_ALL=C.UTF-8; . .jdi/phases/ci-full-pipeline/pipeline-evidence.env; r=$(gh api "repos/$REPO/actions/runs/$RUN_ID" --jq '[.event,.head_sha,.conclusion]|join(" ")'); [ "$r" = "pull_request $HEAD_SHA success" ] || { printf 'run: %s\n' "$r"; exit 1; }; ok=$(gh api --paginate "repos/$REPO/actions/runs/$RUN_ID/jobs?per_page=100" --jq '.jobs[] | select(.conclusion=="success") | .name' | sort -u); for n in "esteira / versao / Versao" "esteira / qualidade / rust-linux" "esteira / qualidade / rust-windows" "esteira / qualidade / node-ui" "esteira / seguranca / Varreduras" "esteira / seguranca / SAST - CodeQL (rust)" "esteira / seguranca / SAST - CodeQL (javascript-typescript)" "esteira / seguranca / SAST - CodeQL (actions)" "esteira / sonar / Sonar" "esteira / Portao"; do printf '%s\n' "$ok" | grep -qxF "$n" || { printf 'sem success: %s\n' "$n"; exit 1; }; done; ! printf '%s\n' "$ok" | grep -q "/ lancar /"; printf 'OK\n'`
- [ ] O template que rodou é o do PR #15:
  - no run, o `pipeline.yml` foi resolvido em `WORKFLOWS_TEST_SHA`, e todo workflow aninhado em `WORKFLOWS_SHA`;
  - `WORKFLOWS_TEST_SHA` é filho direto de `WORKFLOWS_SHA`, e só troca `@main` por `@WORKFLOWS_SHA` em `pipeline.yml`;
  - o `WORKFLOWS_SHA` é o head do PR `WORKFLOWS_PR`, ou ancestral dele sem diferença em `.github/workflows/`.
      **Verify:** `export LC_ALL=C.UTF-8; . .jdi/phases/ci-full-pipeline/pipeline-evidence.env; GW=/home/slipalison/repos/github-workflows; git -C $GW fetch -q origin; gh api "repos/$REPO/actions/runs/$RUN_ID" --jq '.referenced_workflows[] | .path + " " + .sha' | sort -u > /tmp/rw.$$; grep -qxF "slipalison/github-workflows/.github/workflows/pipeline.yml@$WORKFLOWS_TEST_SHA $WORKFLOWS_TEST_SHA" /tmp/rw.$$; [ "$(grep -v '/pipeline.yml@' /tmp/rw.$$ | awk '{print $2}' | sort -u)" = "$WORKFLOWS_SHA" ]; [ "$(git -C $GW rev-parse "$WORKFLOWS_TEST_SHA^")" = "$WORKFLOWS_SHA" ]; [ "$(git -C $GW diff --name-only "$WORKFLOWS_SHA" "$WORKFLOWS_TEST_SHA")" = ".github/workflows/pipeline.yml" ]; d=$(git -C $GW diff -U0 "$WORKFLOWS_SHA" "$WORKFLOWS_TEST_SHA" | grep '^[-+] '); [ -n "$d" ]; ! printf '%s\n' "$d" | grep -v '^[-+] *uses: slipalison/github-workflows/\.github/workflows/[a-z-]*\.yml@'; [ "$(printf '%s\n' "$d" | grep '^-' | sed -E 's/^-//; s/@main$//')" = "$(printf '%s\n' "$d" | grep '^+' | sed -E "s/^\+//; s/@$WORKFLOWS_SHA\$//")" ]; H=$(gh pr view "$WORKFLOWS_PR" -R slipalison/github-workflows --json headRefOid --jq .headRefOid); git -C $GW merge-base --is-ancestor "$WORKFLOWS_SHA" "$H"; git -C $GW diff --quiet "$WORKFLOWS_SHA" "$H" -- .github/workflows; rm -f /tmp/rw.$$; printf 'OK\n'`
- [ ] O Sonar analisou Rust e JS com cobertura, e o Quality Gate passou (emenda: o analisador de Rust do Sonar não roda o Clippy, D-3).
  - No log do job `esteira / sonar / Sonar` do `RUN_ID`:
    - o script escreveu os dois relatórios (`Rust: <n> files, UI: <m> files`, com n e m > 0);
    - o sensor de LCOV do Rust e o sensor `Rust Enterprise` terminaram;
    - o sensor de JS leu o `resultados/lcov-ui.info`;
    - saiu `QUALITY GATE STATUS: PASSED`.
  - Na API do SonarCloud, o PR `PR` tem `alert_status` OK e cobertura ≥ 80%. Rust é a maior parte das linhas, então sem o LCOV dele a cobertura cairia abaixo disso.
      **Verify:** `export LC_ALL=C.UTF-8; . .jdi/phases/ci-full-pipeline/pipeline-evidence.env; J=$(gh api --paginate "repos/$REPO/actions/runs/$RUN_ID/jobs?per_page=100" --jq '.jobs[] | select(.name=="esteira / sonar / Sonar") | .id'); gh api --allow-escape-sequences "repos/$REPO/actions/jobs/$J/logs" | sed -E 's/\x1b\[[0-9;]*m//g; s/^[0-9T:.Z-]+ //' > /tmp/sl.$$; grep -qE '^Rust: [1-9][0-9]* files, UI: [1-9][0-9]* files$' /tmp/sl.$$; grep -qF 'Sensor Rust LCOV Coverage [rustenterprise] (done)' /tmp/sl.$$; grep -qF 'Sensor Rust Enterprise [rustenterprise] (done)' /tmp/sl.$$; grep -qE 'Analysing \[.*/resultados/lcov-ui\.info\]' /tmp/sl.$$; grep -qF 'QUALITY GATE STATUS: PASSED' /tmp/sl.$$; m=$(curl -fsS "https://sonarcloud.io/api/measures/component?component=slipalison_ddc-control&pullRequest=$PR&metricKeys=coverage,alert_status" | python3 -c 'import json,sys; d={m["metric"]:m["value"] for m in json.load(sys.stdin)["component"]["measures"]}; print(d["alert_status"], d["coverage"])'); set -- $m; [ "$1" = OK ]; python3 -c 'import sys; sys.exit(0 if float(sys.argv[1]) >= 80 else 1)' "$2"; rm -f /tmp/sl.$$; printf 'OK coverage=%s\n' "$2"`
- [ ] O `node-ui` aplica o piso: o log do job diz `Aprovado: <n>% >= piso de 80%`, e nenhuma anotação do job fala em relatório ausente.
      **Verify:** `export LC_ALL=C.UTF-8; . .jdi/phases/ci-full-pipeline/pipeline-evidence.env; J=$(gh api --paginate "repos/$REPO/actions/runs/$RUN_ID/jobs?per_page=100" --jq '.jobs[] | select(.name=="esteira / qualidade / node-ui") | .id'); gh api --allow-escape-sequences "repos/$REPO/actions/jobs/$J/logs" > /tmp/nl.$$; grep -qE 'Aprovado:\*\* [0-9.]+% >= piso de 80%' /tmp/nl.$$; rm -f /tmp/nl.$$; [ -z "$(gh api "repos/$REPO/check-runs/$J/annotations" --jq '.[].message' | grep -i 'nenhum relatorio')" ]; printf 'OK\n'`
- [ ] As varreduras rodaram de verdade:
  - o log de `Varreduras` diz `Gitleaks: <n> commits varridos` com n > 0, e `Semgrep: <n> regras` com n > 0;
  - as análises de code scanning do PR (`refs/pull/$PR/merge`) têm o CONJUNTO de ferramentas {CodeQL, Semgrep, Trivy}.
      **Verify:** `export LC_ALL=C.UTF-8; . .jdi/phases/ci-full-pipeline/pipeline-evidence.env; J=$(gh api --paginate "repos/$REPO/actions/runs/$RUN_ID/jobs?per_page=100" --jq '.jobs[] | select(.name=="esteira / seguranca / Varreduras") | .id'); gh api --allow-escape-sequences "repos/$REPO/actions/jobs/$J/logs" | sed 's/\x1b\[[0-9;]*m//g' > /tmp/vl.$$; g=$(grep -oE 'Gitleaks: [0-9]+ commits varridos' /tmp/vl.$$ | grep -oE '[0-9]+' | tail -1); s=$(grep -oE 'Semgrep: [0-9]+ regras' /tmp/vl.$$ | grep -oE '[0-9]+' | tail -1); [ "${g:-0}" -gt 0 ] && [ "${s:-0}" -gt 0 ]; t=$(gh api --paginate "repos/$REPO/code-scanning/analyses?ref=refs/pull/$PR/merge&per_page=100" --jq '.[].tool.name' | sed -E 's/^(Semgrep).*/\1/; s/^(Trivy).*/\1/' | sort -u | paste -sd,); printf '%s\n' "$t" | grep -q 'CodeQL' && printf '%s\n' "$t" | grep -q 'Semgrep' && printf '%s\n' "$t" | grep -q 'Trivy'; rm -f /tmp/vl.$$; printf 'OK gitleaks=%s semgrep=%s tools=%s\n' "$g" "$s" "$t"`
- [ ] O check que o ruleset exigiria existe com o nome do README: um check run `esteira / Portao` em `success` no `HEAD_SHA`, com a tabela do portão no painel.
      **Verify:** `export LC_ALL=C.UTF-8; . .jdi/phases/ci-full-pipeline/pipeline-evidence.env; [ "$(gh api "repos/$REPO/commits/$HEAD_SHA/check-runs?check_name=esteira%20/%20Portao" --jq '[.check_runs[] | select(.conclusion=="success")] | length')" -ge 1 ]; printf 'OK\n'`
- [ ] O script de cobertura do Sonar roda fora do CI e escreve os dois LCOV. Todo `SF:` é um caminho relativo à raiz que existe no repositório, e o da UI tem o CONJUNTO dos módulos que os testes carregam.
      **Verify:** `export LC_ALL=C.UTF-8; rm -rf resultados apps/ddc-tray/coverage; env -u CI bash scripts/ci/sonar-coverage.sh >/dev/null 2>&1; for f in resultados/lcov.info resultados/lcov-ui.info; do [ -s "$f" ]; done; grep '^SF:' resultados/lcov-ui.info | sed 's/^SF://' | while IFS= read -r p; do case "$p" in /*) exit 1;; esac; [ -f "$p" ] || exit 1; done; [ "$(grep -h '^SF:' resultados/lcov-ui.info | sed 's#^SF:apps/ddc-tray/##' | sort | paste -sd,)" = "src/bridge.js,src/debounce.js,src/demo-data.js,src/dropdown.js,src/i18n/en.js,src/i18n/guard.js,src/i18n/index.js,src/i18n/pt-BR.js,src/icons.js,src/view-model.js" ]; grep -q '^SF:.*crates/ddc-core/src/' resultados/lcov.info; rm -rf resultados apps/ddc-tray/coverage; printf 'OK\n'`

### Manual
- (baseline do PROJECT: CHANGELOG e README)

## Deferred to PR review
- `Portao` como status check obrigatório no ruleset "main so por PR" (D-6).
- Pré-requisitos do Sonar, do lado do usuário: o secret `SONAR_TOKEN` e a Análise Automática desligada em `slipalison_ddc-control`.
- O `coluna` quebra com o `Portao` novo do gw #15 (uma linha: `sonar_dispensa` ou Sonar ligado).

## Notes
- Os nomes de job são `<job que chama> / <job do reutilizável> / <nome>`: aqui `esteira / …`. Os runs das phases anteriores continuam com `qualidade / …`, e as evidências delas apontam para aqueles runs.
