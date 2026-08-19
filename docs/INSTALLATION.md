# Installation

## Current Local Install Path

Build the release binary:

```bash
cargo build --release -p tidyup-cli
```

macOS and Linux:

```bash
./target/release/tidyup scan
```

Windows PowerShell:

```powershell
.\target\release\tidyup.exe scan
```

## Release Artifacts

The release workflow builds:

- `tidyup-macos.tar.gz`
- `tidyup-windows.zip`
- matching `.sha256` checksum files

## Data Location

TidyUp stores operation history inside the selected root:

```text
.tidyup/history.sqlite3
```

## Uninstall

If you built locally with Cargo:

- remove the binary under `target/release/`
- optionally remove Cargo build artifacts with `cargo clean`

If you downloaded a release archive:

- delete the extracted `tidyup` or `tidyup.exe` binary

If you want to remove TidyUp-created local data for one organized folder:

- delete that folder’s `.tidyup/` directory
