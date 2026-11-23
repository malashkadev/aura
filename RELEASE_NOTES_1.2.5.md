### ⚡ Noticeably Faster & Cleaner Workflow
- **0.5–0.8 s faster per dictation**: Aura captures selected text faster and uses an adaptive end-of-speech pause so recordings finish sooner without clipping final words.
- **No more phantom `Ctrl+C` or cleared clipboards**: When Windows exposes the selection directly, Aura skips synthetic `Ctrl+C` — eliminating accidental line copies in IDEs and `^C` interrupts in terminals while keeping Explorer files, Excel cells, and images in your clipboard intact.
- **Clean `Win+V` clipboard history**: Dictated text is marked transient so it no longer clutters Windows clipboard history or cloud clipboard sync.
- **Lower idle overhead**: Event-driven microphone hotplugging and cached settings eliminate background polling when Aura is idle.

### ✍️ Overhauled Voice Selection Editing
- **Translate into 32 languages** (up from 10): Voice translation now covers 32 target languages, and a short *"translate"* command detects the selection language properly instead of translating into the same language.
- **More accurate grammar & punctuation fixes**: Single-pass proofreading preserves original meaning and no longer rewrites non-Russian Cyrillic (Ukrainian, Belarusian, Kazakh, Serbian, etc.), Latin, or CJK text into another language.
- **Strict local mode & separate custom edit model**: Local editing (Qwen3) never silently falls back to cloud providers, and custom OpenAI-compatible servers now have a dedicated text-editing model field.
- **Cleaner setup & passthrough dictation**: If no local edit model or cloud API key is configured yet, Settings shows a direct setup notice. Speaking normally over selected text (without an edit command) now keeps your text replacements and silence filtering intact.

### 🎨 Overlay Visualizers, Sound Themes & UI Polish
- **6 HUD overlay styles**: Pick between **Wave**, **Matrix**, **Scale**, **Tape**, **Beam**, and **Ribbon** visualizers right in Hotkeys settings.
- **New sound themes**: **Xylophone** and **Pebble** replace *Rhodes* and *Sci-Fi* (existing configs and `v1.2.0` backups automatically migrate to *Zen*).
- **Refreshed icons & notifications**: Unified monochrome SVG icons across Settings and the tray menu, manually dismissible toasts, and wider overlay notices that no longer clip long messages.

### 🔒 Reliability & Security
- **Safer API-key management**: Saved keys are protected from accidental empty-input overwrites and now have an explicit clear button.
- **Model & silence guards**: Incomplete models cannot be selected, the active model cannot be deleted by mistake, errors are shown in plain language, and Whisper/OpenAI prompt echoes on silence are discarded.
- **Hardening**: Automatic diagnostic log recovery, HTTPS enforcement for custom endpoints, and RustSec dependency updates.

### Package Manager (WinGet)
```cmd
winget install Malashka.Aura
```

---

**Full Changelog**: https://github.com/malashkadev/aura/compare/v1.2.0...v1.2.5

[![Download Aura on SourceForge](https://a.fsdn.com/con/app/sf-download-button)](https://sourceforge.net/projects/aura-voice-typing/files/latest/download)
