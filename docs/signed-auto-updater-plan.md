# Integration plan: full signed auto-updater

Status: **proposal / hand-off spec**. What exists today (v1.0.2): an in-app "check GitHub's
latest release, show a badge, open the release page on click" — see `check_for_update` /
`open_url` in [`lib.rs`](../src-tauri/src/lib.rs). This document describes the remaining step:
**one-click download-and-install** via Tauri's official updater plugin.

## Why this needs you, not just code

Tauri's updater **verifies a cryptographic signature** on every update before installing it —
this is a security feature, not red tape: without it, anyone who intercepts the download (or
compromises the file host) could push malware as an "update". That means:

1. **You must generate a signing keypair** (one-time, local, free — no CA/certificate purchase
   needed, unlike code-signing certs). Command: `npm run tauri signer generate -- -w ~/.tauri/aura.key`
   (prompts for a password). This produces a private key (keep secret — store as a **GitHub
   Actions secret**, never commit it) and a public key (goes into `tauri.conf.json`, safe to
   publish).
2. Every release build must then be signed with that private key as part of `tauri build`.
3. Optional but recommended: a real Windows code-signing certificate (~$100–400/yr) additionally
   removes the SmartScreen "unknown publisher" warning — separate from and independent of the
   Tauri updater signature above. Skippable; the updater works without it.

## Implementation steps

1. **Generate the keypair** (you, once): the command above. Save the public key string.
2. **`Cargo.toml`**: add `tauri-plugin-updater = "2"`.
3. **`tauri.conf.json`**: add
   ```jsonc
   "plugins": {
     "updater": {
       "pubkey": "<your public key>",
       "endpoints": [
         "https://github.com/malashkadev/aura/releases/latest/download/latest.json"
       ]
     }
   }
   ```
4. **`lib.rs`**: register `.plugin(tauri_plugin_updater::Builder::new().build())` in `run()`
   next to the existing `autostart`/`single-instance` plugins.
5. **Frontend**: replace (or extend) today's `check_for_update`/badge-click-opens-browser flow
   with the plugin's JS API (`check()` → `download()` → `install()`), keeping the same nav-dot +
   About-tab badge UI already built — just swap what happens on click: instead of `open_url`,
   call the updater's download/install and restart the app.
6. **CI (`.github/workflows/release.yml`)**: after `tauri build`, the Tauri CLI automatically
   emits a signed `latest.json` + `.sig` files alongside the installer *if* `TAURI_SIGNING_PRIVATE_KEY`
   (and `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` if you set one) are present as environment variables
   pulled from GitHub Actions secrets. Upload `latest.json` to the release alongside the `.exe`/`.msi`
   (the existing `softprops/action-gh-release` step just needs `latest.json` and `*.sig` added to
   its `files:` list).

## What can be done without your involvement

Steps 2–6 are pure code/CI changes I (or Antigravity) can make blind. **Step 1 cannot be
automated for you** — it's your keypair, and the private half must never leave your control
(committing it to the repo would defeat the entire point: anyone could then sign "updates").

## Recommended order

Given `docs/parakeet-integration-plan.md` and `docs/silero-vad-plan.md` are already queued, this
is independent of both — safe to tackle in parallel or whenever you've generated the keypair.
