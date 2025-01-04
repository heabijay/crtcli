#!/bin/sh

## Thanks to another project author, used as reference (/boilerplate):
## https://github.com/ducaale/xh/blob/master/install.sh

set -e

fetch() {
  if which curl >/dev/null; then
    if [ "$#" -eq 2 ]; then curl -fL -o "$1" "$2"; else curl -fsSL "$1"; fi
  elif which wget >/dev/null; then
    if [ "$#" -eq 2 ]; then wget -O "$1" "$2"; else wget -nv -O - "$1"; fi
  else
    echo "Cannot find curl or wget, can't download package"
    exit 1
  fi
}

detect_os() {
  if [ "$(uname -s)" = "Darwin" ] && [ "$(uname -m)" = "x86_64" ]; then
    target="x86_64-apple-darwin"
  elif [ "$(uname -s)" = "Darwin" ] && [ "$(uname -m)" = "arm64" ]; then
    target="aarch64-apple-darwin"
  elif [ "$(uname -s)" = "Linux" ] && [ "$(uname -m)" = "x86_64" ]; then
    target="x86_64-unknown-linux-musl"
  elif [ "$(uname -s)" = "Linux" ] && [ "$(uname -m)" = "aarch64" ]; then
    target="aarch64-unknown-linux-musl"
  # elif [ "$(uname -s)" = "Linux" ] && ( uname -m | grep -q -e '^arm' ); then
  #     target="arm-unknown-linux-gnueabihf"
  else
    echo "Error: Sorry, unsupported OS or architecture. Consider to use manual installation."
    exit 1
  fi
}

fetch_target_url() {
  releases=$(fetch https://api.github.com/repos/heabijay/crtcli/releases/latest)
  url=$(echo "$releases" | grep -wo -m1 "https://.*$target.tar.gz" || true)
  if ! test "$url"; then
    echo "Error: Cannot find release info for $target."
    exit 1
  fi
}

enter_temp_dir() {
  temp_dir=$(mktemp -dt crtcli.XXXXXX)
  trap 'rm -rf "$temp_dir"' EXIT INT TERM
  cd "$temp_dir"
}

determinate_install_dirs() {
  if [ -z "$CRTCLI_INSTALL_DIR_BIN" ] && [ -z "$CRTCLI_INSTALL_DIR_SHARE" ]; then
    user_bin="$HOME/.local/bin"
    case $PATH in 
    *:"$user_bin":* | "$user_bin":* | *:"$user_bin")
      install_dir_bin="$HOME/.local/bin"
      install_dir_share="$HOME/.local/share"
      ;;
    *)
      install_dir_bin='/usr/local/bin'
      install_dir_share='/usr/local/share'
      ;;
    esac
  elif [ -n "$CRTCLI_INSTALL_DIR_BIN" ] && [ -n "$CRTCLI_INSTALL_DIR_SHARE" ]; then
    install_dir_bin="$CRTCLI_INSTALL_DIR_BIN"
    install_dir_share="$CRTCLI_INSTALL_DIR_SHARE"
  else
    echo "Error: CRTCLI_INSTALL_DIR_BIN and CRTCLI_INSTALL_DIR_SHARE must be set together or not set at all."
    exit 1
  fi
}

install_from_current_dir() {
  if test -w "$install_dir_bin" && test -w "$install_dir_share"; then
    mkdir -p "$install_dir_share/crtcli"
    mkdir -p "$install_dir_bin"
    mv * "$install_dir_share/crtcli"
    ln -sf "$install_dir_share/crtcli/crtcli" "$install_dir_bin/crtcli"
  else
    sudo mkdir -p "$install_dir_share/crtcli"
    sudo mkdir -p "$install_dir_bin"
    sudo mv * "$install_dir_share/crtcli"
    sudo ln -sf "$install_dir_share/crtcli/crtcli" "$install_dir_bin/crtcli"
  fi
}


echo
echo "  Welcome to crtcli install script!"
echo

detect_os

echo "Detected target: $target"
echo

fetch_target_url

enter_temp_dir

if ! fetch crtcli.tar.gz "$url"; then
  echo
  echo "Error: Failed to download $url"
  exit 1
fi

determinate_install_dirs

tar xzf crtcli.tar.gz && rm crtcli.tar.gz

install_from_current_dir

echo
echo "$("$install_dir_bin"/crtcli --version) has been installed to:"
echo " * $install_dir_bin/crtcli ($install_dir_share/crtcli)"
