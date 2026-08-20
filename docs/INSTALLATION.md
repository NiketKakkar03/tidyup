# Install TidyUp on macOS

TidyUp supports Apple Silicon and Intel Macs. You do not need Rust or administrator access when installing a release download.

## 1. Choose the correct download

Run:

```bash
uname -m
```

Download both the archive and matching checksum from the GitHub Releases page:

| Result | Archive |
| --- | --- |
| `arm64` | `tidyup-macos-apple-silicon.tar.gz` |
| `x86_64` | `tidyup-macos-intel.tar.gz` |

## 2. Verify the download

In the folder containing both downloaded files, run:

```bash
shasum -a 256 -c tidyup-macos-<type>.tar.gz.sha256
```

Replace `<type>` with `apple-silicon` or `intel`. A valid download prints `OK`.

## 3. Install

```bash
tar -xzf tidyup-macos-<type>.tar.gz
cd tidyup-macos-<type>
./install.sh
```

The installer copies TidyUp to `~/.local/bin/tidyup`. If `~/.local/bin` is not already in your command path, it prints the exact line to add to `~/.zshrc`.

Open a new Terminal window, then verify the installation:

```bash
tidyup --help
```

## 4. Try it safely

Move into a folder you want to inspect. `scan` and `plan` do not change files.

```bash
cd ~/Downloads
tidyup scan
tidyup plan
tidyup apply
```

`apply` shows every proposed destination and asks before moving anything. Use `tidyup apply --verbose` when you want full paths.

## Install somewhere else

Set `TIDYUP_INSTALL_DIR` for both installation and removal:

```bash
TIDYUP_INSTALL_DIR=/usr/local/bin ./install.sh
```

The selected directory must be writable. Installing to `/usr/local/bin` may require administrator permission on some Macs; the default `~/.local/bin` does not.

## Uninstall

From the extracted release folder:

```bash
./uninstall.sh
```

This removes the executable from `~/.local/bin`. It deliberately leaves each organized folder's `.tidyup/history.sqlite3` in place, preserving operation and undo records.

To remove an installation from a custom directory:

```bash
TIDYUP_INSTALL_DIR=/usr/local/bin ./uninstall.sh
```

## Build from source

Developers with Rust installed can build the same executable:

```bash
git clone https://github.com/NiketKakkar03/tidyup.git
cd tidyup
cargo build --release --locked -p tidyup-cli
./target/release/tidyup --help
```

## Release security

Each archive has a SHA-256 checksum. Official public releases should also be signed and notarized with an Apple Developer ID; the release workflow supports this when the maintainer configures the documented GitHub secrets.

## Windows status

Windows distribution is deferred. The current supported release path is macOS.
