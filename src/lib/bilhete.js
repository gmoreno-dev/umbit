// o bilhete. tudo que está escrito nele mora aqui, para ser reescrito à vontade.
// as frases são um rascunho; a voz tem que ser a sua.

export const BILHETE = {
  para: 'para: ana lívia',

  // linhas do bilhete que aparece na busca (digitadas uma a uma)
  abertura: ['lembrei de você.', 'separei duas músicas.'],
  aberturaAniversario: ['feliz aniversário, amor.', 'separei duas músicas.'],

  // as duas faixas, como linhas do bilhete
  faixas: [
    { n: '01', nome: 'matilda', nota: 'a que era sua' },
    { n: '02', nome: 'sign of the times', nota: 'a que é sua agora. e nossa.' },
  ],

  // bilhete colado na capa enquanto cada uma toca
  durante: {
    'egg-hers': ['essa era você.', 'tendo que ser forte, fingir que estava tudo bem, deixar ir.', 'eu vi tudo isso. e vi você mudar.'],
    'egg-ours': ['essa é você agora. e a gente.', 'a promessa de que vai ficar tudo bem virou nossa.', 'eu amo você.'],
  },

  // nota na tela da fila
  fila: {
    'egg-hers': 'primeiro a sua',
    'egg-ours': 'agora a nossa',
  },

  // o bilhete fechado, no fim da nossa
  fim: ['fim do bilhete.', 'você não é mais aquela música.', 'essa é a nossa agora.', 'te amo.'],
};
