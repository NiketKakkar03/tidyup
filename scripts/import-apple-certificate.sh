#!/bin/sh
set -eu

keychain="$RUNNER_TEMP/tidyup-signing.keychain-db"
certificate="$RUNNER_TEMP/tidyup-signing.p12"
keychain_password=$(openssl rand -hex 24)

printf '%s' "$APPLE_CERTIFICATE_P12" | openssl base64 -d -A -out "$certificate"
security create-keychain -p "$keychain_password" "$keychain"
security set-keychain-settings -lut 21600 "$keychain"
security unlock-keychain -p "$keychain_password" "$keychain"
security import "$certificate" -P "$APPLE_CERTIFICATE_PASSWORD" -A -t cert -f pkcs12 -k "$keychain"
security list-keychain -d user -s "$keychain"
security set-key-partition-list -S apple-tool:,apple: -s -k "$keychain_password" "$keychain"
