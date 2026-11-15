# Product

## Register

product

## Users

- Windows users who want fast voice-to-text in any application.
- Developers, writers, and power users who dictate and edit selected text by voice.

## Product Purpose

Aura is a fast, private voice-to-text utility for Windows. Hold the global hotkey, speak, and the transcript is typed at the cursor in the active application. Recognition runs locally on CPU or NVIDIA CUDA through resident Whisper.cpp and Parakeet servers, or through an optional cloud provider. Selected text can be edited by voice with a small set of deterministic commands.

## What Aura is not

- **Not an AI text editor or a general-purpose assistant.** The primary purpose is accurate voice typing.
- **Local models transcribe; they do not rewrite text or remove filler words.** Secondary rewriting is available only when the user explicitly chooses an AI editing command and a cloud or local LLM route.
- **Silence-hallucination filtering is a safety guard, not an AI cleanup feature.** It discards known phantom phrases produced on silence; it is not marketed as filler-word removal.
- **Voice punctuation commands are not part of the product.** Whisper and Parakeet format punctuation natively; saying "comma" or "period" is not required.

## Brand Personality

- **Fast, private, precise.**
- **Emotional goals:** reliability, confidence, quiet utility.

## Anti-references

- Generic SaaS dashboards with white cards and thick gray borders.
- Neon gradients, glows, radial halos, and screaming accent colors.
- Cluttered legacy system tools with tiny unaligned controls.

## Design Principles

1. **Frictionless velocity.** Every interaction should feel immediate: hotkey response, overlay appearance, text insertion, settings persistence.
2. **Polished invisibility.** Aura lives in the tray; the overlay is a compact status indicator that never steals focus.
3. **High-contrast dark elegance.** Achromatic surfaces, one tactical orange accent, local Onest typography, and no outer shadows or glows.
4. **Strict context respect.** Never type into an unintended window, never permanently overwrite the clipboard, and never promise behavior the active engine cannot deliver.

## Capabilities

- Direct cursor typing in any Windows application.
- Resident local engines: Whisper.cpp and NVIDIA Parakeet, each on CPU or NVIDIA CUDA.
- Optional cloud recognition: Gemini, Groq, OpenAI, Hugging Face, or a custom OpenAI-compatible endpoint.
- Selection editing: fix grammar and translate, through cloud AI or a local Qwen3 model.
- Smart context insertion, custom vocabulary, offline text replacements, and cursor snippets.
- Encrypted transcription history and API keys through Windows DPAPI with restricted ACLs.
- Nine interface languages, keyboard navigation, and reduced-motion support.

## Privacy

- No telemetry, no analytics, no accounts, no subscriptions.
- Audio is processed locally unless the user explicitly selects a cloud provider.
- API keys and history are encrypted locally and never included in settings exports.
