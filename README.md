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
- Teclas de mídia do teclado (por enquanto só no Linux, via MPRIS).
- Quatro temas de duas cores: papel, âmbar, fósforo, gameboy.

## Instalar

Linux (x86_64) e Windows 10 e 11 (x64).

### O jeito rápido (qualquer distro)

```bash
curl -fsSL https://raw.githubusercontent.com/gmoreno-dev/umbit/main/install.sh | bash
```

O script descobre a sua distribuição, baixa a última
[release](../../releases/latest) e instala. Em Debian e Ubuntu usa o
`.deb`; nas outras (Arch, CachyOS, Manjaro, Fedora, openSUSE, Void)
instala o WebKit pelo gerenciador de pacotes e coloca o binário em
`/opt/umbit`. Pede a senha do sudo uma vez.

### Debian e Ubuntu, à mão

1. Baixe o `.deb` da [última release](../../releases/latest) para `/tmp`
   (o apt recusa arquivos dentro de uma home com permissão fechada).
2. `sudo apt install /tmp/Umbit_*_amd64.deb`

### Arch, Fedora, openSUSE e outras, à mão

1. Instale as dependências: `webkit2gtk-4.1`, `gtk3`, `alsa-lib` e
   `openssl` (os nomes variam um pouco entre distros; no Arch é
   `sudo pacman -S webkit2gtk-4.1 gtk3 alsa-lib openssl`).
2. Baixe o `umbit-*-linux-x86_64.tar.gz` da [última release](../../releases/latest),
   extraia e rode `./umbit`. O `.desktop` e o ícone estão dentro para quem
   quiser o atalho no menu.

No Arch também dá para compilar do fonte com o PKGBUILD do repositório
(`cd packaging/arch && makepkg -si`), o que demora uns 15 minutos.

O AppImage continua na release como último recurso, mas ele embute um
WebKit antigo e abre em branco em algumas placas de vídeo. Prefira o
tarball.

### Windows 10 e 11

Baixe o `Umbit_*_x64-setup.exe` da [última release](../../releases/latest)
e dê dois cliques. Instala só para o seu usuário, sem pedir administrador,
e cria o atalho no menu iniciar. Se o Windows Defender SmartScreen
reclamar de "editor desconhecido", clique em **Mais informações** e
**Executar assim mesmo**: o instalador não é assinado porque certificado
custa caro; o código é aberto e a release é gerada pelo GitHub.

Ou, no PowerShell, um comando que baixa e instala em silêncio:

```powershell
irm https://raw.githubusercontent.com/gmoreno-dev/umbit/main/install.ps1 | iex
```

No Windows as teclas de mídia do teclado ainda não funcionam (só no
Linux, via MPRIS). Entra numa próxima versão.

### Depois de instalar

Crie o seu client id (seção abaixo) e entre com a sua conta Premium.

## Atualizações

O Umbit verifica se há versão nova ao abrir, no máximo uma vez por dia, com
uma única requisição ao GitHub. Cada pacote é assinado; o app só instala o
que confere com a chave pública embutida.

- **Windows e AppImage**: baixa e instala sozinho, e reabre.
- **`.deb` e binário em `/opt`** (instalação pelo script no Linux): esses
  precisam de root, então o app avisa e mostra o comando para rodar no
  terminal (o mesmo do instalador). `:sobre` mostra o estado, e
  `:atualizar` força uma verificação.

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

## Seu próprio client id (necessário)

A Web API do Spotify limita as chamadas por aplicação. O client id embutido
no librespot é compartilhado por todo cliente não oficial do mundo e vive
no limite, o que aparece como erro `429` na busca e na biblioteca. Por
isso o Umbit usa um client id seu, como todo cliente open source faz.
Leva dois minutos:

1. Entre em [developer.spotify.com/dashboard](https://developer.spotify.com/dashboard)
   com a sua conta e clique em **Create app**.
2. Nome e descrição livres. Em **Redirect URIs** coloque exatamente
   `http://127.0.0.1:8898/login`. Em **APIs used** marque **Web API**.
3. Salve, abra o app criado e copie o **Client ID**.
4. Cole no campo **client id** da tela de login do Umbit e entre. O
   navegador pode pedir autorização duas vezes na primeira vez: uma para o
   seu app (biblioteca e busca) e, se o Spotify não aceitar esse token para
   tocar música, outra para o motor de áudio.

Para convidar até 5 pessoas sem que elas criem um app: no painel, em
**User Management**, adicione o e-mail da conta Spotify delas, e elas usam
o seu client id na tela de login.

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
