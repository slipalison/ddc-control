D-2026-09-28-release-packaging-4 (2026-09-28): Regra udev empacotada nos pacotes Linux.

`docs/linux-ddc-setup.md` (seção "Option A: udev `uaccess` rule (recommended)") já documenta a regra manual:

```
# /etc/udev/rules.d/60-ddc-control-i2c.rules
SUBSYSTEM=="i2c-dev", KERNEL=="i2c-[0-9]*", ATTRS{class}=="0x030000", TAG+="uaccess"
```

Esta phase promove essa regra de "documentação manual" para "arquivo empacotado": novo arquivo versionado `packaging/linux/60-ddc-control-i2c.rules` com EXATAMENTE esse conteúdo (mesmo nome de arquivo do doc, para quem já a aplicou manualmente não acabar com duas regras divergentes). `apps/ddc-tray/src-tauri/tauri.conf.json` ganha, em `bundle.linux.deb.files` e `bundle.linux.rpm.files`, o mapeamento:

```json
"/usr/lib/udev/rules.d/60-ddc-control-i2c.rules": "../../../packaging/linux/60-ddc-control-i2c.rules"
```

(caminho relativo a `apps/ddc-tray/src-tauri`, mesma convenção de `bundle.icon`). `/usr/lib/udev/rules.d/` é o caminho padrão de pacote de distro para regra udev (o próprio `ddcutil` usa `/usr/lib/udev/rules.d/60-ddcutil-i2c.rules`, citado no mesmo doc) — nunca `/etc/udev/rules.d/`, que é para regra local de administrador, não de pacote.

**Limitação aceita do AppImage:** o bundler `appimage` não roda script pós-instalação nem tem conceito de "pacote instalado no sistema" — um AppImage não pode colocar arquivo em `/usr/lib/udev/rules.d/` sozinho. Quem usa o AppImage continua a aplicar a regra manualmente via `docs/linux-ddc-setup.md`, e o texto desse doc ganha uma frase dizendo isso (revisão de texto é `## Deferred to PR review`, mas o FATO "AppImage não inclui a regra" é uma decisão locked, não uma escolha de redação).

**Prova automática:** o DoD verifica, no `.deb` e no `.rpm` reais baixados do run de evidência, que o arquivo em `/usr/lib/udev/rules.d/60-ddc-control-i2c.rules` existe dentro do pacote e é byte-a-byte idêntico ao `packaging/linux/60-ddc-control-i2c.rules` do commit — nunca um grep de fonte, uma inspeção do artefato binário real com `ar`/`rpm2cpio`.

**Emenda do orquestrador (2026-09-28, antes do plano):**
- Sem o módulo `i2c-dev` carregado não existe `/dev/i2c-*`. O Fedora não o carrega sozinho. Por isso o deb e o rpm também levam `packaging/linux/ddc-control-i2c-dev.conf` (conteúdo: a linha `i2c-dev`) em `/usr/lib/modules-load.d/ddc-control.conf`, para que o módulo carregue a cada boot.
- O pós-instalação do deb e do rpm (`postInstallScript` do Tauri) faz o melhor que puder, sem nunca falhar a instalação: `modprobe i2c-dev`, `udevadm control --reload` e `udevadm trigger --subsystem-match=i2c-dev`, cada um com `|| true` (em container ou chroot não há udev nem módulo). O DoD confere os dois arquivos dentro do deb e do rpm reais, byte a byte com o `packaging/linux/` do commit.
- O binário instalado continua `/usr/bin/ddc-tray`, porque o script KWin de ancoragem do popup casa por esse nome. O `.desktop` tem `Exec=ddc-tray`. Se o Tauri 2.12 renomear o binário pelo `productName`, fixar `mainBinaryName: "ddc-tray"`.
