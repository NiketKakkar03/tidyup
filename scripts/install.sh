#!/bin/sh
set -eu

install_dir=${TIDYUP_INSTALL_DIR:-"$HOME/.local/bin"}
script_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)

if [ ! -x "$script_dir/tidyup" ]; then
  echo "tidyup binary not found beside install.sh" >&2
  exit 1
fi

mkdir -p "$install_dir"
cp "$script_dir/tidyup" "$install_dir/tidyup"
chmod 755 "$install_dir/tidyup"

echo "Installed TidyUp to $install_dir/tidyup"
case ":$PATH:" in
  *":$install_dir:"*) ;;
  *)
    echo "Add this directory to your PATH by putting this line in ~/.zshrc:"
    echo "  export PATH=\"$install_dir:\$PATH\""
    ;;
esac
echo "Run 'tidyup --help' to get started."
