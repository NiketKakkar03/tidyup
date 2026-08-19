# Installation

## Current MVP Install Path

The current MVP should be presented as macOS-first.

Build the release binary:

```bash
cargo build --release -p tidyup-cli
```

macOS:

```bash
./target/release/tidyup scan
```

## Windows Status

Windows workflow scaffolding exists in the repository, but Windows distribution is deferred until the packaging and validation issues are intentionally completed.

## Release Artifacts

The repository contains release workflow scaffolding for:

- `tidyup-macos.tar.gz`
- `tidyup-windows.zip`
- matching `.sha256` checksum files

The workflow definition lives at:

- `.github/workflows/release.yml`

For the current MVP, the supported showcase path is macOS.

## Downloaded Artifact Usage

macOS:

```bash
tar -xzf tidyup-macos.tar.gz
./tidyup scan
```

## Checksum Verification

macOS:

```bash
shasum -a 256 -c tidyup-macos.tar.gz.sha256
```

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

- delete the extracted `tidyup` binary

If you want to remove TidyUp-created local data for one organized folder:

- delete that folder’s `.tidyup/` directory
