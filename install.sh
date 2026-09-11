#!/usr/bin/env bash
# umbit · instalador para debian, ubuntu e arch (e derivados)
#
#   curl -fsSL https://raw.githubusercontent.com/gmoreno-dev/umbit/main/install.sh | bash
#
# debian/ubuntu: baixa o .deb da última release e instala com apt.
# outras distros (arch, fedora, opensuse, void...): instala o webkit do sistema
#       pelo gerenciador de pacotes e o binário puro em /opt/umbit. sem AppImage.
set -euo pipefail

REPO="gmoreno-dev/umbit"
API="https://api.github.com/repos/$REPO/releases/latest"

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
  *)
    # qualquer outra distro (arch, fedora, opensuse, void...): binário puro com o webkit do sistema
    url="$(asset 'linux-x86_64.tar.gz')"; [ -n "$url" ] || die "a release $tag não tem o tarball linux"
    bold "instalando dependências (vai pedir a sua senha)"
    if command -v pacman >/dev/null; then
      sudo pacman -S --needed --noconfirm webkit2gtk-4.1 gtk3 alsa-lib openssl
    elif command -v dnf >/dev/null; then
      sudo dnf install -y webkit2gtk4.1 gtk3 alsa-lib openssl-libs
    elif command -v zypper >/dev/null; then
      sudo zypper --non-interactive install libwebkit2gtk-4_1-0 libgtk-3-0 libasound2 libopenssl3
    elif command -v xbps-install >/dev/null; then
      sudo xbps-install -Sy webkit2gtk41 gtk+3 alsa-lib openssl
    else
      echo "não reconheci o gerenciador de pacotes. instale: webkit2gtk 4.1, gtk3, alsa e openssl 3."
    fi
    bold "baixando o binário"
    curl -fL --progress-bar "$url" -o "$tmp/umbit.tar.gz"
    tar xzf "$tmp/umbit.tar.gz" -C "$tmp"
    src="$(find "$tmp" -maxdepth 1 -type d -name 'umbit-*' | head -1)"
    [ -n "$src" ] || die "tarball inesperado"
    bold "instalando em /opt/umbit"
    sudo rm -rf /opt/umbit
    sudo mkdir -p /opt/umbit
    sudo cp "$src/umbit" "$src/umbit.png" "$src/LICENSE" /opt/umbit/
    sudo chmod 755 /opt/umbit/umbit
    sudo ln -sf /opt/umbit/umbit /usr/local/bin/umbit
    bold "criando o atalho no menu"
    sed 's|^Exec=umbit|Exec=/opt/umbit/umbit|; s|^Icon=umbit|Icon=/opt/umbit/umbit.png|' "$src/umbit.desktop" | sudo tee /usr/share/applications/umbit.desktop >/dev/null
    command -v update-desktop-database >/dev/null && sudo update-desktop-database 2>/dev/null || true
    ;;
esac

bold "pronto. abra o umbit pelo menu ou rode: umbit"
echo "lembre: o umbit precisa de spotify premium e do seu próprio client id."
echo "passo a passo: https://github.com/$REPO#seu-próprio-client-id-necessário"
