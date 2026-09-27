shipped_at: 2026-09-27T15:34:55Z
verdict: APPROVED_WITH_WARNINGS
by: alison amorim

## Learnings
- O ddc-i2c 0.2.2 entra em panic num off-by-one em algumas leituras (0x7E): toda transação do worker precisa de catch_unwind.
- No Windows os erros do dxva2 não têm tipo: código recusado parece transitório (3 tentativas, exit 6).
- Mudanças de postura de segurança do repo (.claude/settings.json) vão em PR próprio, nunca dentro de uma branch de phase (W-8).
