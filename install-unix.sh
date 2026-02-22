#!/bin/sh

set -e

repo_url="https://github.com/heabijay/crtcli"
metadata_url_default="$repo_url/releases/latest/download/release.json"

fetch() {
  if command -v curl >/dev/null 2>&1; then
    if [ "$#" -eq 2 ]; then curl -fL -o "$1" "$2"; else curl -fsSL "$1"; fi
  elif command -v wget >/dev/null 2>&1; then
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

build_urls_from_tag() {
  release_tag="$1"
  release_archive_name="crtcli-$release_tag-$target.tar.gz"
  release_url="$repo_url/releases/download/$release_tag/$release_archive_name"
}

parse_metadata_for_target() {
  metadata="$1"
  metadata_compact=$(printf '%s' "$metadata" | tr -d '\r\n')
  asset_entry=$(printf '%s' "$metadata_compact" \
    | sed -n "s/.*\"$target\"[[:space:]]*:[[:space:]]*{\\([^}]*\\)}.*/\\1/p")

  if [ -z "$asset_entry" ]; then
    return 1
  fi

  release_url=$(printf '%s' "$asset_entry" \
    | sed -n 's/.*"download_url"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p')
  release_archive_name=$(printf '%s' "$asset_entry" \
    | sed -n 's/.*"archive_name"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p')

  test -n "$release_url" && test -n "$release_archive_name"
}

resolve_latest_tag() {
  if command -v curl >/dev/null 2>&1; then
    effective_url=$(curl -fsSL -o /dev/null -w '%{url_effective}' "$repo_url/releases/latest" 2>/dev/null || true)
    latest_tag=$(printf '%s' "$effective_url" | sed -n 's#.*/tag/\([^/?#]*\).*#\1#p')

    if [ -n "$latest_tag" ]; then
      printf '%s' "$latest_tag"
      return 0
    fi
  fi

  release_page=$(fetch "$repo_url/releases/latest")
  latest_tag=$(printf '%s' "$release_page" | awk '
    match($0, /releases\/tag\/[^"?# ]+/) {
      print substr($0, RSTART + 13, RLENGTH - 13)
      exit
    }
  ')

  if [ -z "$latest_tag" ]; then
    return 1
  fi

  printf '%s' "$latest_tag"
}

fetch_release_info() {
  metadata_url="${CRTCLI_INSTALL_METADATA_URL:-$metadata_url_default}"

  if [ -n "$CRTCLI_INSTALL_VERSION_TAG" ]; then
    build_urls_from_tag "$CRTCLI_INSTALL_VERSION_TAG"
    return
  fi

  if release_metadata=$(fetch "$metadata_url" 2>/dev/null); then
    if parse_metadata_for_target "$release_metadata"; then
      return
    fi
  fi

  echo "Warning: Unable to use release metadata at $metadata_url. Falling back to release tag discovery."

  latest_tag=$(resolve_latest_tag || true)
  if [ -z "$latest_tag" ]; then
    echo "Error: Cannot resolve latest release tag."
    exit 1
  fi

  build_urls_from_tag "$latest_tag"
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

fetch_release_info

enter_temp_dir

echo "Downloading: $release_archive_name"
if ! fetch crtcli.tar.gz "$release_url"; then
  echo
  echo "Error: Failed to download $release_url"
  exit 1
fi

determinate_install_dirs

tar xzf crtcli.tar.gz && rm crtcli.tar.gz

install_from_current_dir

echo
echo "$("$install_dir_bin"/crtcli --version) has been installed to:"
echo " * $install_dir_bin/crtcli ($install_dir_share/crtcli)"
