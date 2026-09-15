### 🇳🇱 Dutch Language & Regional Layout Support
* **Native Dutch Transcription**: Added Dutch (`nl` / Nederlands) as an official fixed recognition language across local Whisper, cloud AI providers, and the system tray menu.
* **Windows Dutch Keyboard Layout Mapping**: Added automatic detection for Dutch (Netherlands, `0x0413`) and Dutch (Belgium, `0x0813`) keyboard layouts when using "Follow Keyboard Layout" mode.
* **Native Punctuation Prompting**: Integrated authentic Dutch dictation sample prompting for Whisper to ensure correct capitalization, commas, and sentence boundaries.

### 🛡️ Whisper Auto-Detect Prompt Neutralization
* **Eliminated Cyrillic Context Leakage ([#5](https://github.com/malashkadev/aura/issues/5))**: Fixed a critical issue where the wildcard fallback in `build_whisper_prompt` defaulted to Russian text. This prior context previously biased Whisper's decoder tokens and caused short non-Russian audio dictations (such as Dutch) to be incorrectly detected as Russian.
* **Clean Decoder Priming**: The auto-detection mode now initializes with a clean, unbiased prompt, passing only explicit user vocabulary terms when a custom dictionary is configured.

### Package Manager (WinGet)
```cmd
winget install Malashka.Aura
```

---

**Full Changelog**: https://github.com/malashkadev/aura/compare/v1.1.0...v1.1.1

[![Download Aura on SourceForge](https://a.fsdn.com/con/app/sf-download-button)](https://sourceforge.net/projects/aura-voice-typing/files/latest/download)
