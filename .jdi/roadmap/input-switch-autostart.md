---
order: 5.9
name: Input switch fix and autostart
---
- **Slug:** input-switch-autostart
- **Goal:** corrigir a troca de entrada (`0x60`): ao escolher DisplayPort 2 a entrada muda e volta para DisplayPort 1 — achar a causa (leitura imediata após o write, que lê o valor antigo ou falha enquanto o monitor comuta; fallback de auto source do monitor sem sinal na entrada pedida) e resolver de forma que o app só informe o que o monitor manteve, depois de assentar, e explique uma reversão; e adicionar "iniciar com o sistema" (autostart) no ddc-tray, ligável pelo menu da bandeja, em Linux e Windows
- **Reason:** pedido do usuário, 2026-09-30: "ao selecionar a entrada do monitor por exemplo DisplayPort 1 ele muda, porém se eu coloco DisplayPort 2 ele muda porém volta para o DisplayPort 1 [...] E adicione uma função iniciar com o sistema para que ele carregue automaticamente quando ligar o computador"
