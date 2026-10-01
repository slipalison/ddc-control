D-2026-10-01-input-switch-autostart-5 (2026-10-01): Sétima rodada do critic (rodada 2, iteração 2). A linha 1 segue oca em toda a família "só a política manda", e isto a fecha de uma vez. A premissa da D-2026-10-01-input-switch-autostart-4(a), "qualquer piso acima da janela injetada excede o limite de leituras do (f)", era falsa: com um piso também no PASSO, o worker lê poucas vezes e passa.

Oito mutações sobreviviam, e nenhuma muda produção (o `Default` é 250 ms / 3 s): piso no passo, piso na janela + piso no passo, teto na janela (no worker e no cliente), teto no passo (absoluto e relativo à janela), dormir um passo inteiro além da janela e piso no orçamento do cliente.

(a) A prova da exatidão sai do relógio real e vai para o VIRTUAL, numa TABELA de políticas extremas:
- 5 ms/100 ms;
- 500 ms/4 s;
- 500 ms/4,25 s (janela que não é múltiplo do passo);
- 300 ms/1 s;
- 2 s/10 s;
- 10 min/20 min;
- `passo == janela` (7 s/7 s);
- 12 h/1 dia.

Para cada política, com o monitor mantendo o valor antigo, (b) exige quatro coisas:
- `Ok(())`;
- `leituras == ceil(janela/passo)`;
- `sleeps == [passo; n−1] ++ [janela − passo·(n−1)]`, exatos;
- `Σ sleeps == janela`.

Para cada política, (e) exige `write_budget_of(0x60) == budgets.vcp + janela` e `budgets.vcp` para os outros códigos. Isso mata todo piso abaixo de 100 ms…1 dia e todo teto acima de 5 ms…12 h, no passo, na janela e no orçamento do cliente, sem custo de tempo real. Os nomes dos 7 testes não mudam.

(b) O (f) volta a ser a prova pelo cliente REAL, e não a da exatidão. O limite de leituras ganha a tolerância de UMA leitura (`ceil(janela/passo) + 1`): no Linux o máximo medido é exatamente o limite (margem zero), e no Windows o timer de alta resolução trunca o pedido em 100 ns e pode somar uma leitura sem defeito no worker. Pisos e tetos são pegos, exatos, pela tabela.

(c) Linha 8, não objetiva: um `#[path = ...]` antes de `#[cfg(test)] mod tests;` trocaria o módulo de teste compilado por um arquivo não congelado. A regex de atributos dos 3 arquivos passa a pegar também `path`.
