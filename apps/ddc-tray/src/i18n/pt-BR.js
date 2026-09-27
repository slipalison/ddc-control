// Brazilian Portuguese texts of the popup, by key; same keys as `en.js`
// (D-2026-09-26-tray-app-6).

export default Object.freeze({
  'state.loading': 'Procurando monitores…',
  'state.empty': 'Nenhum monitor com DDC/CI foi encontrado.',
  'action.retry': 'Tentar de novo',
  'hint.i2c':
    'No Linux, o DDC/CI precisa do módulo i2c-dev e de acesso de leitura e escrita a /dev/i2c-*. Veja {doc}.',

  'format.percent': '{value}%',
  'format.fraction': '{current} de {max}',
  'format.unnamed': 'Valor {hex}',
  'format.code': 'Ajuste {hex}',

  'origin.caps': 'Declarado pelo monitor',
  'origin.probe': 'Encontrado na sondagem',
  'reading.unsupported': 'Não suportado',
  'reading.unresponsive': 'Sem resposta',

  'confirm.title': 'Confirmar alteração',
  'confirm.body.input':
    'Mudar a entrada para {to}? A tela passará a mostrar essa fonte e pode ficar escura se ela não tiver sinal.',
  'confirm.body.power':
    'Mudar a energia para {to}? A tela pode desligar, e talvez seja preciso usar o botão de energia do monitor para religá-la.',
  'confirm.body.generic':
    'Definir {feature} como {to}? Este ajuste pode mudar o comportamento do monitor.',
  'confirm.accept': 'Aplicar',
  'confirm.cancel': 'Cancelar',

  'error.not_found': 'Monitor não encontrado. Talvez ele tenha sido desconectado.',
  'error.unsupported': 'Este monitor não suporta este ajuste.',
  'error.invalid_value': 'O monitor não aceita este valor.',
  'error.needs_confirmation': 'Esta alteração precisa da sua confirmação.',
  'error.timeout': 'O monitor não respondeu a tempo.',
  'error.transport': 'Não foi possível se comunicar com o monitor via DDC/CI.',
  'error.backend_unavailable': 'O controle de monitores não está disponível neste computador.',
  'error.unknown': 'Algo deu errado.',

  'feature.brightness': 'Brilho',
  'feature.contrast': 'Contraste',
  'feature.volume': 'Volume',
  'feature.input': 'Entrada',
  'feature.preset': 'Predefinição de cor',
  'feature.power': 'Energia',
  'feature.color-temp': 'Temperatura de cor',
  'feature.red-gain': 'Ganho de vermelho',
  'feature.green-gain': 'Ganho de verde',
  'feature.blue-gain': 'Ganho de azul',
  'feature.auto-setup': 'Ajuste automático',
  'feature.h-position': 'Posição horizontal',
  'feature.v-position': 'Posição vertical',
  'feature.red-black-level': 'Nível de preto do vermelho',
  'feature.green-black-level': 'Nível de preto do verde',
  'feature.blue-black-level': 'Nível de preto do azul',
  'feature.trapezoid': 'Trapézio',
  'feature.sharpness': 'Nitidez',
  'feature.osd-lock': 'Menu na tela',
  'feature.osd-language': 'Idioma do menu',
  'feature.new-control-value': 'Novo valor de controle',

  'value.display-native': 'Nativo',
  'value.user-1': 'Usuário 1',
  'value.on': 'Ligado',
  'value.off-dpm': 'Em espera (DPM)',
  'value.off-write-only': 'Desligado (botão de energia)',
  'value.off': 'Desligado',
  'value.run': 'Executar',
  'value.continuous': 'Contínuo',
  'value.osd-disabled': 'Desativado',
  'value.osd-enabled': 'Ativado',
  'value.chinese-traditional': 'Chinês (tradicional)',
  'value.english': 'Inglês',
  'value.french': 'Francês',
  'value.german': 'Alemão',
  'value.japanese': 'Japonês',
  'value.spanish': 'Espanhol',
  'value.chinese-simplified': 'Chinês (simplificado)',
});
