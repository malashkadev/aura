### ⚡ Deterministic Voice Editing on Selected Text
* **Decoupled Instruction & Data (`edit_intent.rs`)**: Voice editing commands spoken over selected text ("переведи на английский", "сократи", "исправь грамматику", "сделай формальнее", "по пунктам") are now parsed with deterministic rules rather than mixed-prompt classification. The model receives only the selected document text inside data delimiters and never sees the command phrase itself, preventing the model from self-translating or echoing commands.
* **Output Validation Gate (`edit_validate.rs`)**: Validates editing output by target language alphabet match, length invariants (shortening must be shorter, grammar fixes must preserve semantic similarity), and rejects echo leaks before replacing selection.

### 🛠️ Unified Text Editing Settings Card
* **Single Master Control**: Consolidated duplicate cards on the Text tab into a single unified card with master toggle, route selector (`Auto` / `Local` / `Cloud`), and local model downloader cards (Qwen3 0.6B / 1.7B / 4B).
* **Default Target Translation Language**: Added configurable target language for bare «переведи» / «translate» commands with intelligent fallback to application UI language when the selected text is already in the target language.

### 🐛 Bug Fixes & Stability
* **Fixed Command Self-Translation Bug**: Fixed issue where saying "переведи на английский" translated the command phrase itself into "Translate to English." and pasted it over user selection instead of translating the selected text.
* **Fixed Echo Detection Aborts**: Fixed false `edit-failed` aborts on "сократи" and "исправь грамматику" caused by over-aggressive echo heuristics.
* **Fixed Layout Language Leak**: Fixed issue where holding the hotkey with an English keyboard layout forced Whisper into English transcription mode for Russian edit commands.

### Package Manager (WinGet)
```cmd
winget install Malashka.Aura
```

---

**Full Changelog**: https://github.com/malashkadev/aura/compare/v1.1.1...v1.1.2

[![Download Aura on SourceForge](https://a.fsdn.com/con/app/sf-download-button)](https://sourceforge.net/projects/aura-voice-typing/files/latest/download)
