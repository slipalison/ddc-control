---
order: 3.5
name: Full OSD control
---
- **Slug:** full-osd-control
- **Goal:** controlar tudo que o OSD oferece via DDC/CI: catálogo MCCS 2.2 completo no core (nome, tipo C/NC/T, acesso RO/WO/RW, risco, nomes de valores NC), descoberta por caps + sondagem read-only de códigos fora do caps (0x62 volume, 0x6C/0x6E/0x70 black level, 0x20/0x30 posição, 0x1E, 0xC9 respondem no monitor de dev), CLI `features` (dump com valor atual) e `set` por nome de feature/valor para toda feature gravável, factory resets atrás de confirmação; validação no monitor real
