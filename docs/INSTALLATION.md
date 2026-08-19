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

The workflow definition lives at:

- `.github/workflows/release.yml`

## Downloaded Artifact Usage

macOS:

```bash
tar -xzf tidyup-macos.tar.gz
./tidyup scan
```

Windows PowerShell:

```powershell
Expand-Archive .\tidyup-windows.zip -DestinationPath .
.\tidyup.exe scan
```

## Checksum Verification

macOS:

```bash
shasum -a 256 -c tidyup-macos.tar.gz.sha256
```

Windows PowerShell:

```powershell
$expected = (Get-Content .\tidyup-windows.zip.sha256).Split(' ')[0]
$actual = (Get-FileHash .\tidyup-windows.zip -Algorithm SHA256).Hash
$actual.ToLower() -eq $expected.ToLower()
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

- delete the extracted `tidyup` or `tidyup.exe` binary

If you want to remove TidyUp-created local data for one organized folder:

- delete that folder’s `.tidyup/` directory
