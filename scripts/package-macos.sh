#!/bin/sh
set -eu

if [ "$#" -ne 2 ]; then
  echo "usage: $0 <rust-target> <artifact-label>" >&2
  exit 2
fi

rust_target=$1
artifact_label=$2
package_name="tidyup-${artifact_label}"
package_dir="dist/${package_name}"
archive="dist/${package_name}.tar.gz"

rm -rf "$package_dir"
mkdir -p "$package_dir"
cp "target/${rust_target}/release/tidyup" "$package_dir/tidyup"
cp scripts/install.sh scripts/uninstall.sh README.md LICENSE docs/INSTALLATION.md "$package_dir/"
chmod 755 "$package_dir/tidyup" "$package_dir/install.sh" "$package_dir/uninstall.sh"

tar -czf "$archive" -C dist "$package_name"
(cd dist && shasum -a 256 "${package_name}.tar.gz" > "${package_name}.tar.gz.sha256")

"$package_dir/tidyup" --help >/dev/null
echo "Created $archive"
