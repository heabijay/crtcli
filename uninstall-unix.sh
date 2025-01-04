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

ensure_prompt() {
  if [ "$CRTCLI_INSTALL_CONFIRM_YES" = "true" ]; then
    return
  fi
  
  while true; do
    echo
    printf "  Are you sure? [Y/n]: "
    read -r yn < /dev/tty
    echo 
    case $yn in
      [Yy]* ) break;;
      [Nn]* ) echo "Aborted."; exit;;
      * ) echo "Please answer yes or no.";;
    esac
  done
}


echo
echo "  Welcome to crtcli uninstall script!"
echo

determinate_install_dirs

echo "  The follow directories and files will be removed:"
echo "- $install_dir_bin/crtcli"
echo "- $install_dir_share/crtcli"

ensure_prompt

if test -w "$install_dir_bin" && test -w "$install_dir_share"; then
  rm -rf "$install_dir_share/crtcli"
  rm -f "$install_dir_bin/crtcli"
else
  sudo rm -rf "$install_dir_share/crtcli"
  sudo rm -f "$install_dir_bin/crtcli"
fi

echo "Completed!"