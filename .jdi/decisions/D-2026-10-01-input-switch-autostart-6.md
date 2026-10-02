D-2026-10-01-input-switch-autostart-6 (2026-10-01): Oitava rodada do critic (rodada 2, iteração 3). A linha 1 continua oca porque o texto prometia "qualquer piso ou teto", o que uma tabela finita não prova. Sobraram quatro classes:
- razão janela/passo fora da tabela (limite de leituras ≥ 20, pisos ou tetos relativos);
- piso só no caminho de produção (`spawn`/`SystemClock`);
- outro código orçado como o input;
- pós-processamento do orçamento dentro de `WorkerClient::write_vcp`.

Decisão: fechar o que é testável e passar o texto a afirmar SÓ as faixas cobertas, declarando o que fica fora.

(a) A tabela de (b) ganha `1 µs/100 ms` (razão 10⁵) e `1 dia/1 µs` (passo maior que a janela; sleeps `[janela]`), num total de 10 políticas.

(b) Em (e), para cada política, há dois clientes:
- um com VCP curto, `min(70 ms, janela/2)`;
- outro folgado: `vcp = 1000·janela + 1 s`, com `capabilities` e `enumerate` maiores.

Os dois exigem `write_budget_of(0x60) == vcp + janela` e `vcp` para TODOS os outros 255 códigos.

(c) O (f) ganha o limite de baixo `leituras * 2 >= ceil(janela/passo)`. Com isso, no caminho de produção, ele pega um piso no passo acima de cerca de 11 ms e um teto abaixo de cerca de 4,8 ms com a política de 5 ms/100 ms.

(d) O Verify da linha 1 fixa o corpo de `WorkerClient::write_vcp`: só `let budget = self.write_budget_of(code);` seguido do `transact`. Assim, o orçamento real é exatamente o que (e) prova.

(e) As linhas 1 e 8 conferem, pelo dep-info do binário de teste de `ddc-adapters` (`cargo test --no-run --message-format=json`), o CONJUNTO exato de `.rs` compilados sob `ddc_hi_backend/`. Um `#[path]` em qualquer forma (W-24) muda esse conjunto. Isso garante que os arquivos congelados por SHA-256 são os que compilam.

Não afirmado (indetectável por tabela finita ou fora do escopo):
- piso ≤ 1 µs no passo ou na janela;
- teto acima dos maiores valores da tabela;
- fatores relativos fora das faixas escritas na linha;
- piso no passo só no caminho de produção de cerca de 11 ms ou menos;
- a origem da janela.
