# macOS Release Guide

This guide is for TidyUp maintainers. Users should follow `docs/INSTALLATION.md`.

## What a version tag creates

Pushing a tag such as `v0.1.0` runs `.github/workflows/release.yml`. It:

1. tests the locked Rust workspace on both macOS runners
2. builds native Apple Silicon and Intel executables
3. signs each executable when Apple signing secrets are configured
4. packages the executable with install and uninstall scripts
5. notarizes each archive when Apple notarization secrets are configured
6. creates a draft GitHub Release containing both archives and checksums

The release stays in draft form so a maintainer can test both downloads and review the generated notes before publishing.

## Optional repository secrets

Configure these GitHub Actions secrets only if Apple signing and notarization are added later:

- `APPLE_CERTIFICATE_P12`: base64-encoded Developer ID Application certificate
- `APPLE_CERTIFICATE_PASSWORD`: password for the exported certificate
- `APPLE_SIGNING_IDENTITY`: full Developer ID Application identity
- `APPLE_ID`: Apple developer account email
- `APPLE_TEAM_ID`: Apple Developer team identifier
- `APPLE_APP_PASSWORD`: app-specific password used by `notarytool`

These secrets are optional. Without them, the workflow creates unsigned open-source archives and users may need to approve the binary through macOS Privacy & Security settings. Checksums and installation testing are still required before publication.

## Prepare and publish

1. Confirm the MIT license and copyright notice are current.
2. Complete `RELEASE_CHECKLIST.md`.
3. Confirm that `main` is clean and CI is passing.
4. Create and push an annotated version tag.
5. Download both draft-release archives on the corresponding Mac architectures.
6. Verify checksums, install, run the disposable demo, undo it, and uninstall.
7. Publish the draft GitHub Release only after both architecture checks pass.

Apple signing and notarization can be added later without changing the MIT license or users' ability to build and modify the source.

Example tag commands:

```bash
git tag -a v0.1.0 -m "Release TidyUp v0.1.0"
git push origin v0.1.0
```

Do not reuse or move a published release tag.
