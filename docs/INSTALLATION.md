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

Windows packaging is deferred for `v0.1.0`. If you need Windows support before an official artifact exists, build from source in your own environment.

## Release Artifacts

The release workflow builds:

- `tidyup-macos.tar.gz`
- matching `.sha256` checksum files

The workflow definition lives at:

- `.github/workflows/release.yml`

The public `v0.1.0` draft release is distributed under the MIT license in `LICENSE`.

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
