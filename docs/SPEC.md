# Umbit · especificação do projeto

**Umbit**: cliente Spotify leve, não oficial, para computadores fracos. O nome vem de "um bit", a estética monocromática do app.

## O problema

O cliente oficial do Spotify é um app Electron que embute um Chromium inteiro e
consome centenas de MB de RAM. Em notebooks antigos ele trava o resto do
trabalho. O catálogo não é o problema; o cliente é.

## Decisões tomadas

| Tema | Decisão | Motivo |
|---|---|---|
| Fonte de música | Spotify Premium via librespot | Mantém playlists, curtidas e Connect. Protocolo interno, não afetado pelas restrições da Web API de fev/2026. Biblioteca Rust mantida pela comunidade. |
| Stack | Tauri 2 + Svelte 5 | Núcleo Rust embute o librespot direto. Interface em HTML/CSS com liberdade total de estética. Usa o WebKitGTK do sistema em vez de embutir um navegador. |
| Plataforma v1 | Só Linux | Testar no notebook do autor. Windows e Mac depois que o núcleo estiver estável. |
| Estética | Direção A: "papel 1 bit" | Escolhida entre três direções retrô (papel 1 bit, display âmbar, fita cassete). Ver seção "Estética". |
| Licença | MIT | Mesma do librespot. |
| Backend de áudio | Pluggable por contrato | Frontend não sabe que existe librespot. Um backend alternativo (ex.: YouTube, para quem não tem Premium) pode entrar depois como plugin da comunidade. |

## Metas de desempenho

- Meta original: menos de 80 MB de RAM. **Medido na v0.1.0** (release, tela
  de login, PSS somando o processo principal e os dois processos do
  WebKitGTK 2.52): cerca de 270 MB. O piso é do próprio WebKitGTK + GTK,
  não do app; o binário tem 10 MB e a interface 36 KB comprimida. Ainda é
  uma fração do cliente oficial. Para descer de 100 MB seria preciso trocar
  o webview por uma interface nativa em Rust (egui/iced), que ficou como
  opção futura.
- Abertura em menos de 1 segundo após a primeira execução.
- Nenhuma dependência de GPU. Nada de blur, sombras animadas ou canvas pesado.
- Sem polling: o núcleo emite eventos, o frontend só reage.

## Arquitetura

```
┌───────────────────────────────┐
│  Frontend (Svelte, WebKitGTK) │  desenha e reage a eventos
└──────────────┬────────────────┘
               │ Tauri commands / events (contrato)
┌──────────────┴────────────────┐
│  Núcleo (Rust)                │
│  ├─ session   login OAuth, cache de credencial
│  ├─ player    librespot-playback, fila, seek, volume
│  ├─ connect   Spotify Connect (aparece como dispositivo)
│  ├─ catalog   playlists, curtidas, álbuns, busca
│  └─ media     MPRIS (teclas de mídia no Linux)
└───────────────────────────────┘
               │
          librespot
```

### Contrato entre frontend e núcleo

Comandos (frontend → núcleo), definidos em `src-tauri/src/commands.rs`:
`get_session`, `login`, `logout`, `get_config`, `set_theme`,
`get_now_playing`, `get_queue`, `get_playlists`, `get_playlist_tracks`,
`get_liked`, `get_liked_uri`, `get_saved_albums`, `get_album_tracks`,
`search`, `play_context`, `play_tracks`, `play_egg`, `get_egg_tracks`,
`play_pause`, `next_track`, `prev_track`, `seek`, `seek_relative`,
`set_volume`, `get_cover`.

Eventos (núcleo → frontend): `session`, `now_playing`, `queue`, `error`.
Os tipos estão em `src-tauri/src/models.rs`. A posição de reprodução vem
como (`position_ms`, `at_ms`) e a interface interpola, sem polling.

Decisão de implementação: o Spirc (Spotify Connect) do librespot é a
autoridade da reprodução. O núcleo mantém um espelho da fila a partir da
lista que a interface mandou ao dar play; se outro dispositivo trocar o
contexto, o espelho é descartado e a fila mostra só a faixa atual.

## Escopo

### v1 (uso diário)

- Login por OAuth com credencial em cache.
- Playlists do usuário, músicas curtidas, álbuns salvos.
- Busca de faixas, álbuns e artistas.
- Tocar, pausar, anterior, próxima, seek, volume, fila (ver, e tocar a
  partir de qualquer ponto; reordenar e remover ficam para depois, porque o
  Spotify Connect é a fonte de verdade da fila e o librespot 0.8 não expõe
  edição).
- Aparecer como dispositivo Spotify Connect (controlar pelo celular).
- Teclas de mídia via MPRIS.
- Navegação completa por teclado.

### Depois da v1

- Página de artista e de álbum.
- Bandeja do sistema, janela "sempre no topo".
- Temas (pares de cor tinta/papel).
- Letras.
- Atalhos customizáveis.
- Podcasts.
- Windows e Mac.

## Estética

Direção escolhida: **A · papel 1 bit** (Mac de 1984, Playdate). Um único
painel, sem chrome, monocromático, com capas ditherizadas e fonte pixel.
Mockups das cinco telas da v1: canvas "Umbit"
(https://claude.ai/code/artifact/7955552b-39bc-4bbb-8b5e-2c8677a7b971).
As direções descartadas (display âmbar, fita cassete) ficam na segunda
página do canvas como referência.

Princípios:

1. **Duas cores por tema**: tinta e papel. Temas são só pares de cor:

   | Tema | Tinta | Papel |
   |---|---|---|
   | papel (padrão) | `#141210` | `#f2efe6` |
   | âmbar | `#ffb000` | `#0b0a08` |
   | fósforo | `#5cf08a` | `#06100a` |
   | gameboy | `#0f380f` | `#9bbc0f` |

   Tons intermediários só por opacidade da tinta, nunca uma terceira cor.
2. **Capas em 1 bit**: capas de álbum passam por dithering ordenado (Bayer)
   em tempo real e viram tinta-sobre-papel. É a assinatura visual e custa
   quase nada de CPU.
3. **Tipografia pixel/mono**: Pixelify Sans (OFL) para títulos, rótulos e
   abas; Space Mono (OFL) para listas e texto corrido. As duas vão
   embutidas no app, nada carrega da rede. Títulos longos rolam como
   letreiro de LCD.
4. **Janela pequena por padrão** (cerca de 400×640), redimensionável. Fica
   no canto da tela enquanto se trabalha, como um widget. Isso é ao mesmo
   tempo a estética e a razão de ser leve.
5. **Teclado primeiro**: `/` busca, `j`/`k` navegam, `espaço` toca/pausa,
   `n`/`p` próxima/anterior. Mouse funciona, mas nada exige mouse.
6. **Barra de progresso segmentada**, volume em blocos, sem gradientes.

Telas da v1 (todas desenhadas no canvas):

- **Login**: capa ditherizada, nome do app, um botão "entrar com spotify".
- **Tocando**: capa ditherizada 352 px, título em letreiro, artista,
  progresso segmentado, controles, volume em blocos.
- **Biblioteca**: filtro playlists / curtidas / álbuns, uma lista, um
  cursor invertido (tinta sobre papel vira papel sobre tinta).
- **Busca**: campo com cursor de bloco, resultados agrupados (faixas,
  álbuns com miniatura ditherizada, artistas).
- **Fila**: faixa atual invertida no topo, próximas numeradas.

Elementos fixos: abas no topo (tocando / biblioteca / busca / fila, ativa
sublinhada), dicas de teclado em 11 px no rodapé, e uma minibarra de
reprodução com borda superior nas telas que não são a "Tocando".
Linhas de lista têm 44 px de altura, altura mínima de alvo de clique.

## Ambiente de desenvolvimento (Linux, Debian 13)

Falta instalar: toolchain Rust, `pkg-config`, dev libs do WebKitGTK 4.1,
GTK 3, libsoup 3, e dev libs de áudio (ALSA + PulseAudio ou PipeWire).

## Repositório

- README com aviso: exige Spotify Premium; cliente não oficial baseado em
  engenharia reversa do protocolo (librespot); zona cinzenta dos termos de
  uso, tolerada há mais de dez anos.
- GitHub Actions gerando binário Linux (AppImage e .deb) a cada tag.
- Windows e Mac entram na matriz depois da v1.

## Easter egg

Existe um, pessoal, definido em conversa e implementado no código. Não está
documentado aqui de propósito. Dispara só para contas específicas, para não
atrapalhar quem busca de verdade por um artista com o mesmo nome.
