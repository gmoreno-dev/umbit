#!/usr/bin/env bash
# umbit · instalador para debian, ubuntu e arch (e derivados)
#
#   curl -fsSL https://raw.githubusercontent.com/gmoreno-dev/umbit/main/install.sh | bash
#
# debian/ubuntu: baixa o .deb da última release e instala com apt.
# arch: instala as dependências com pacman, extrai o AppImage em /opt/umbit
#       e cria o atalho no menu. nada de fuse, nada de aur.
set -euo pipefail

REPO="gmoreno-dev/umbit"
API="https://api.github.com/repos/$REPO/releases/latest"
RAW="https://raw.githubusercontent.com/$REPO/main"

bold() { printf '\033[1m%s\033[0m\n' "$*"; }
die() { printf 'umbit: %s\n' "$*" >&2; exit 1; }
need() { command -v "$1" >/dev/null 2>&1 || die "preciso de '$1' instalado"; }

need curl
[ "$(uname -m)" = "x86_64" ] || die "por enquanto só há pacotes para x86_64"
[ -r /etc/os-release ] || die "não reconheci a distribuição (sem /etc/os-release)"
. /etc/os-release
family="${ID:-} ${ID_LIKE:-}"

bold "umbit · procurando a última release"
json="$(curl -fsSL "$API")" || die "não consegui falar com o github"
tag="$(printf '%s' "$json" | grep -m1 '"tag_name"' | sed -E 's/.*"([^"]+)".*/\1/')"
asset() { printf '%s' "$json" | grep '"browser_download_url"' | grep -i "$1" | head -1 | sed -E 's/.*"(https[^"]+)".*/\1/'; }
[ -n "$tag" ] || die "nenhuma release encontrada"
bold "versão $tag"

# pasta temporária legível pelo usuário _apt (o apt recusa arquivos que não consegue ler)
tmp="$(mktemp -d /tmp/umbit.XXXXXX)"
chmod 755 "$tmp"
trap 'rm -rf "$tmp"' EXIT

case "$family" in
  *debian*|*ubuntu*|*linuxmint*|*pop*)
    url="$(asset '.deb')"; [ -n "$url" ] || die "a release $tag não tem .deb"
    bold "baixando o .deb"
    curl -fL --progress-bar "$url" -o "$tmp/umbit.deb"
    chmod 644 "$tmp/umbit.deb"
    bold "instalando com apt (vai pedir a sua senha)"
    sudo apt install -y "$tmp/umbit.deb"
    ;;
  *arch*|*manjaro*|*endeavouros*|*cachyos*|*garuda*)
    url="$(asset '.AppImage')"; [ -n "$url" ] || die "a release $tag não tem AppImage"
    bold "instalando dependências com pacman (vai pedir a sua senha)"
    sudo pacman -S --needed --noconfirm webkit2gtk-4.1 gtk3 alsa-lib openssl
    bold "baixando o AppImage"
    curl -fL --progress-bar "$url" -o "$tmp/umbit.AppImage"
    chmod +x "$tmp/umbit.AppImage"
    bold "extraindo em /opt/umbit"
    (cd "$tmp" && ./umbit.AppImage --appimage-extract >/dev/null)
    sudo rm -rf /opt/umbit
    sudo mkdir -p /opt/umbit
    sudo cp -r "$tmp/squashfs-root/." /opt/umbit/
    sudo chmod -R a+rX /opt/umbit
    sudo ln -sf /opt/umbit/AppRun /usr/local/bin/umbit
    bold "criando o atalho no menu"
    curl -fsSL "$RAW/packaging/umbit.desktop" | sudo tee /usr/share/applications/umbit.desktop >/dev/null
    curl -fsSL "$RAW/src-tauri/icons/128x128.png" | sudo tee /usr/share/icons/hicolor/128x128/apps/umbit.png >/dev/null
    command -v update-desktop-database >/dev/null && sudo update-desktop-database || true
    ;;
  *)
    die "distribuição não suportada ainda: ${PRETTY_NAME:-$family}. baixe o AppImage em https://github.com/$REPO/releases"
    ;;
esac

bold "pronto. abra o umbit pelo menu ou rode: umbit"
echo "lembre: o umbit precisa de spotify premium e do seu próprio client id."
echo "passo a passo: https://github.com/$REPO#seu-próprio-client-id-necessário"
