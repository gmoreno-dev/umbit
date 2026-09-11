# umbit

Cliente Spotify leve, não oficial, para computadores fracos. Uma janela
pequena, duas cores, capas em 1 bit, tudo pelo teclado.

O Spotify oficial embute um navegador inteiro e come centenas de MB de RAM.
O Umbit usa o [librespot](https://github.com/librespot-org/librespot) como
motor e o WebKit do sistema para a interface. O binário tem 10 MB, a
interface inteira pesa 36 KB, e o app todo (com os processos do WebKit)
fica em torno de 270 MB de memória, contra o dobro ou o triplo do cliente
oficial. Abre em menos de um segundo.

> **Precisa de Spotify Premium.** É uma limitação do protocolo do Spotify,
> não do Umbit. Contas gratuitas não conseguem entrar.

## O que faz

- Login com a sua conta (OAuth no navegador, uma vez só).
- Suas playlists, músicas curtidas e álbuns salvos.
- Busca de faixas, álbuns e artistas.
- Tocar, pausar, pular, avançar, volume, fila.
- Aparece como dispositivo no **Spotify Connect**: controle pelo celular.
- Teclas de mídia do teclado (MPRIS no Linux).
- Quatro temas de duas cores: papel, âmbar, fósforo, gameboy.

## Instalar

Por enquanto só Linux. Baixe o `.deb` ou o `.AppImage` da página de
[releases](../../releases).

```bash
sudo apt install ./umbit_*.deb
```

Windows e Mac vêm depois que a v1 estabilizar no Linux.

## Teclado

| tecla | ação |
|---|---|
| `1` `2` `3` `4` | tocando · biblioteca · busca · fila |
| `/` | buscar |
| `espaço` | tocar / pausar |
| `n` `p` | próxima / anterior |
| `←` `→` | −5 s / +5 s (com Shift, 30 s) |
| `-` `=` | volume |
| `j` `k` | mover o cursor |
| `enter` | abrir / tocar |
| `b` `esc` | voltar |
| `t` | trocar o tema |
| `:` | comandos (`:sobre`, `:tema`, `:topo`, `:sair`) |

## Compilar do zero

Precisa de Rust (1.85+), Node 20+ e as bibliotecas do Tauri 2. No Debian
e Ubuntu:

```bash
sudo apt install build-essential pkg-config curl file libssl-dev \
  libwebkit2gtk-4.1-dev libjavascriptcoregtk-4.1-dev libgtk-3-dev \
  libsoup-3.0-dev librsvg2-dev libayatana-appindicator3-dev libxdo-dev \
  libasound2-dev
```

```bash
npm install
npm run tauri dev      # roda em modo desenvolvimento
npm run tauri build    # gera .deb e .AppImage em src-tauri/target/release/bundle
```

## Configuração

Fica em `~/.config/umbit/config.toml`. Credenciais e volume ficam em
`~/.cache/umbit/`. Para sair da conta de vez, use `:sair` no app ou apague
`~/.cache/umbit/credentials/credentials.json`.

## Aviso

O Umbit não tem relação com o Spotify. Ele fala o protocolo do cliente
oficial por meio do librespot, que é engenharia reversa e tecnicamente
contraria os termos de uso do Spotify. Na prática, clientes assim existem
há mais de dez anos e o Spotify os tolera, mas use por sua conta e risco.

## Licença

MIT. Feito com ♥ para a.l.
