D-2026-10-02-usb-switch-follow-1: Phase 'USB switch follow' (slug: usb-switch-follow) added before `profiles-hotkeys`. Reason: pedido do usuário via `/jdi-issue`, 2026-10-02, sem tracker ("trocar de canal sem desconectar o usb e sem usar o controle físico do monitor").

A investigação no PC mostrou que o monitor RTK só responde DDC/CI na entrada ATIVA. Com o notebook na tela, o PC não tem como trazer o monitor de volta, e religar a saída de vídeo também não faz o monitor voltar.

A solução é a máquina DEIXADA pelo switch USB trocar a entrada. O DDC/CI funciona no notebook Linux, e o usuário confirmou isso.
