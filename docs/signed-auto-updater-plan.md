# Signed auto-updater

Status: **implemented in v1.2.0; release signing needs the operator's secrets.**

## What is already in the codebase

- `tauri-plugin-updater` is registered in `src-tauri/src/lib.rs`.
- `src-tauri/tauri.conf.json` contains the updater public key and the
  `https://github.com/malashkadev/aura/releases/latest/download/latest.json`
  endpoint, with `bundle.createUpdaterArtifacts: true`.
- The in-app update flow checks GitHub, downloads and verifies the signed
  update, installs it, and restarts Aura.
- `.github/workflows/release.yml` passes `TAURI_SIGNING_PRIVATE_KEY` and
  `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` from repository secrets to
  `tauri-apps/tauri-action`, which publishes `latest.json` and the `.sig`
  files alongside the installers.

## What the operator must provide

1. **Updater signing secrets** — generate the keypair once, keep the private
   key offline, and add the two repository secrets used by `release.yml`:
   `TAURI_SIGNING_PRIVATE_KEY` and (if the key has a password)
   `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`.
2. **Optional Windows Authenticode certificate** — removes the SmartScreen
   "unknown publisher" warning. This is independent of the updater signature;
   the updater works without it. No certificate is configured in this
   repository, so local and CI installers are currently unsigned.

## Local builds without the private key

`tauri build` can produce MSI/NSIS installers without the updater private key,
but it cannot produce a valid updater signature. Such a build is suitable for
local testing only; do not publish it as an auto-update source. Set the two
environment variables (or the CI secrets) before producing release artifacts.
