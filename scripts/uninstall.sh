#!/bin/sh
set -eu

install_dir=${TIDYUP_INSTALL_DIR:-"$HOME/.local/bin"}
binary="$install_dir/tidyup"

if [ ! -e "$binary" ]; then
  echo "TidyUp is not installed at $binary"
  exit 0
fi

rm "$binary"
echo "Removed $binary"
echo "Folder-specific .tidyup history was left in place so undo records are preserved."
