# Отчёт о комплексном аудите программы Aura (v1.2.0)

**Дата проведения:** 15 октября 2026 г.  
**Версия приложения:** Aura 2.0 (v1.2.0)  
**Платформа:** Windows 10/11 (x64)  
**Статус аудита:** ✅ **УСПЕШНО ПРОЙДЕН (GREEN / Production Grade)**

---

## 1. Резюме аудита (Executive Summary)

Проведён полный, сквозной, поэтапный аудит программы Aura (повышена до версии 1.2.0). В рамках аудита выполнена проверка кодовой базы (Rust Backend, JavaScript Frontend, Tauri IPC), Win32-хуков и симулятора ввода, архитектуры резидентных нейросетевых движков (Whisper/Parakeet), криптографической безопасности (DPAPI), звукового тракта (WASAPI/Silero VAD), соблюдения дизайн-системы («The Stealth Command Deck» и карантина сайта), качества текстов и анти-слоп копирайтинга, паритета интернационализации (9 словарей) и стандартов доступности (WCAG AA).

### Ключевые показатели надёжности (Quality Gates):
- **Rust Backend Unit Tests**: **203 пройден, 0 упало, 1 пропущен (сетевой)** (`cargo test --lib`)
- **Frontend Static Integrity Gates**: **20 пройдены, 0 упало** (`npm test`)
- **Static Analysis / Clippy**: **0 предупреждений, 0 ошибок** (`cargo clippy -- -D warnings`)
- **Code Formatting**: **100% соответствует rustfmt** (`cargo fmt --check`)
- **Lint & TypeScript Typecheck**: **0 ошибок** (`npm run check`)
- **Security Dependency Audit**: **0 уязвимостей** (`cargo audit` — устранено 4 уязвимости rustls 0.23.45 через апгрейд до 0.23.46)
- **Скрипты статической целостности (7 наборов в `scripts/`)**: **100% пройдены, 0 дефектов**
- **Storybook Build**: **100% пройден, 188 модулей трансформировано, exit code 0** (`npm run build-storybook`)
- **Visual QA Pre-flight**: захвачены и верифицированы скриншоты всех 6 вкладок настроек приложения, состояний HUD Overlay и страниц лендинга.

---

## 2. Результаты аудита по подсистемам

### 2.1. Win32 Low-Level Keyboard Hooks & Input Simulation
- **Файлы**: `src-tauri/src/keyboard_hook.rs`, `src-tauri/src/keyboard_simulator.rs`
- **Проверенные инварианты**:
  1. **Изоляция внедрённых событий (`LLKHF_INJECTED`)**: В `low_level_keyboard_proc` проверяется флаг `0x10`. Все события симулятора ввода гарантированно игнорируются хуком, полностью исключая рекурсивное зацикливание.
  2. **Подавление физических автоповторов**: При удержании горячей клавиши физические события `WM_KEYDOWN` перехватываются (`return 1`), предотвращая утечку посторонних символов в активный редактор.
  3. **Строгая маска модификаторов (`modifiers_satisfied_state`)**: Проверка всех 4 модификаторов (Ctrl, Alt, Shift, Win) гарантирует, что комбинация `Alt+V` не перехватывает `Ctrl+V`, исключая фантомное залипание Alt в Windows.
  4. **Атомарность замены текста (`replace_text`)**: Снятие модификаторов -> посылка Backspace -> посылка символов -> восстановление модификаторов выполняется неделимым блоком.
  5. **Контроль очереди сообщений (Queue Flushing)**:
     - Backspace: пауза 15 мс после каждых 10 нажатий, 3 мс между остальными.
     - Печать: пауза 10 мс после каждых 15 символов, 1 мс между остальными.
  6. **Атомарность суррогатных пар UTF-16**: Эмодзи и 4-байтовые символы никогда не разрываются границами батчей и отправляются пачками `SendInput(4)`.
  7. **Защита при смене фокуса (`FocusTarget`)**: При переключении окна во время диктовки текст безопасно сохраняется в буфер обмена и историю, не впечатываясь в постороннее окно.

### 2.2. Smart Context & Windows UI Automation
- **Файлы**: `src-tauri/src/caret_context.rs`, `src-tauri/src/smart_insertion.rs`
- **Проверенные инварианты**:
  1. **Изоляция потока UIA**: Захват каретки вынесен в системный поток `aura-caret-capture` со строгим сторожевым таймером 20 мс. При неотвечающем приложении захват мгновенно прекращается без задержки аудио.
  2. **Устранение UIA Range Bleed**: Границы диапазонов UIA привязаны к `DocumentRange()`. В веб-редакторах (Gemini, ChatGPT, Claude) курсор на нулевой позиции не затягивает внешние кнопки и сообщения чата.
  3. **Нормализация невидимых символов**: Поддержка `\u{200B}`..=`\u{200D}`, `\u{FEFF}`, `\u{FFFC}` гарантирует сохранение заглавной буквы в пустых полях ввода.
  4. **Защита акронимов и аббревиатур**: Сохранение заглавных букв устоявшихся сокращений (`NASA`, `API`, `НДС`, англ. местоимения `I`) и защита точек в известных сокращениях (`т.д.`, `руб.`, `etc.`).

### 2.3. Резидентные серверные движки и Фоновые процессы
- **Файлы**: `src-tauri/src/whisper_runner.rs`, `src-tauri/src/parakeet_streaming.rs`, `src-tauri/src/llm_runner.rs`
- **Проверенные инварианты**:
  1. **Поддержка CPU и NVIDIA CUDA**:
     - Whisper: resident `whisper-server` (CPU флаг `-ng`, до 8 потоков) и аппаратное ускорение CUDA через `ggml-cuda.dll` с удержанием в VRAM без холодного старта.
     - Parakeet: потоковый WebSocket-сервер на базе Sherpa-ONNX как в CPU, так и в CUDA режиме.
  2. **Предотвращение процессов-зомби (Windows Job Objects)**: Все дочерние процессы связываются с Windows Job Object (`JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE`) до запуска. При падении или закрытии Aura операционная система гарантированно убивает серверы.
  3. **Игровой режим (Gaming Mode) и сон VRAM**:
     - Детектирование полноэкранных 3D-игр через `SHQueryUserNotificationState` с фильтрацией окон браузеров и плееров (`Chrome_WidgetWin_*`, `vlc` и др.).
     - 12-секундный дебаунс (hysteresis) перед отключением игрового режима защищает от дерганья VRAM при Alt+Tab.
     - Таймер сна (`Standby Unloader`: 15/30/60 мин) освобождает 100% видеопамяти при неактивности.

### 2.4. Звуковой тракт, WASAPI и VAD
- **Файлы**: `src-tauri/src/audio_recorder.rs`, `src-tauri/src/vad.rs`, `src-tauri/src/text_normalizer.rs`
- **Проверенные инварианты**:
  1. **WASAPI без блокировок Tokio**: Опрос устройств переведён на `tokio::task::spawn_blocking`.
  2. **Потоковый ресемплер**: 44.1/48 кГц -> 16 кГц моно с 63-точечным FIR-фильтром нижних частот с непрерывным сохранением фазы.
  3. **Silero VAD**: Неразрушающий шлюз отсекает тишину; паддинг в 350 мс защищает тихие окончания слов.
  4. **Фильтрация галлюцинаций Whisper**: Отсечение паразитных фраз тишины («Спасибо за просмотр», «Создавал DimaTorzok») при 100% сохранении искренней речи. Очистка паразитных речевых многоточий (`...`).

### 2.5. Безопасность, шифрование DPAPI и изоляция
- **Файлы**: `src-tauri/src/secure_storage.rs`, `src-tauri/src/settings_secure.rs`, `src-tauri/src/logger.rs`, `src-tauri/capabilities/`
- **Проверенные инварианты**:
  1. **Шифрование DPAPI**: Все API-ключи и история шифруются через Windows `CryptProtectData` с привязкой к учётной записи пользователя.
  2. **Ограничение прав (DACL)**: Файлы настроек сохраняются с правами только текущего пользователя (`TokenUser`) и `SYSTEM`.
  3. **Защита от утечки секретов**: `SettingsView` маскирует ключи (`zeroize()`), фронтенду отдаются только флаги `has_api_key_*`.
  4. **Изоляция окон Tauri**: Окна `main`, `overlay` и `tray-menu` изолированы по принципу наименьших привилегий (Least Privilege). Опасные разрешения (`core:default`, `opener:default`, `updater:default`) отключены.
  5. **Санитайзинг логов**: В `logger.rs` реализовано маскирование персональных данных, API-ключей и текста диктовки.

### 2.6. Дизайн-система, UI/UX и Карантин
- **Файлы**: `src/index.html`, `src/style.css`, `src/overlay.html`, `src/tray-menu.css`, `website/`
- **Проверенные инварианты**:
  1. **The Stealth Command Deck**: 100% ахроматическая палитра (`#000000`, `#090909`, `#0d0d0d`, `#121212`). Запрещены синие/индиго оттенки (`#030305`, `rgba(20, 20, 26)`).
  2. **The Rarity Rule**: Тактический оранжевый (`#ff4200`) строго $\le 10\%$ экрана. Запрещена сплошная оранжевая заливка фона кнопок на ховере.
  3. **HUD Overlay 3 состояния**: Запись (live VU-волна, луч 0%), обработка (двойной луч conic-gradient, кроссфейд 550 мс), успех (морфинг в квадрат 39×39px, точное центрирование монограммы, нулевое искажение масштаба).
  4. **Строгий карантин**: Стили лендинга, градиенты и шрифты Google Font `Onest` изолированы и никогда не проникают в десктопное приложение.

### 2.7. Тексты, Копирайтинг и Anti-Slop
- **Файлы**: `src/index.html`, `src/main.js`, `website/`, `README.md`, `README.ru.md`
- **Проверенные инварианты**:
  1. **Де-слопинг (Anti-Slop)**: Полное отсутствие маркетинговых клише («умный ИИ», «голосовой помощник», «революционный»).
  2. **Фактологическая точность**: Локальные модели транскрибируют; удаление слов-паразитов и вторичная редактура выполняются облачными LLM.
  3. **Синхронизация голосовых команд**: Все примеры командных чипов во всех 9 локалях согласованы с парсером `edit_intent.rs`.

### 2.8. Интернационализация (i18n) и Доступность (a11y)
- **Файлы**: `src/main.js`, `src/overlay.js`, `src/tray-menu.js`, `src/ui-accessibility.js`
- **Проверенные инварианты**:
  1. **100% паритет словарей**: 9 локалей (`ru`, `en`, `de`, `es`, `fr`, `it`, `zh`, `pt`, `tr`), 270 уникальных ключей без пропусков.
  2. **Нулевой хардкод строк**: Устранены все статические строки в DOM и JS.
  3. **Доступность (WCAG AA)**: Навигация стрелками по табам с зацикливанием, фокус-ловушки в модальных окнах, корректные ARIA-атрибуты, поддержка `prefers-reduced-motion`.

---

## 3. Выявленные и устранённые дефекты в ходе аудита

| ID | Приоритет | Компонент | Описание проблемы | Статус исправления |
| :--- | :--- | :--- | :--- | :--- |
| **AUDIT-12-01** | **P1 (Security)** | `Cargo.lock` | В зависимости `rustls` (v0.23.45) 14.10.2026 были зарегистрированы 4 новые уязвимости (RUSTSEC-2026-0346, 0347, 0348, 0349). `cargo audit` завершался с кодом 1. | **ИСПРАВЛЕНО**: Выполнен апгрейд `cargo update -p rustls` до версии 0.23.46. `cargo audit` сообщает: 0 уязвимостей. |
| **AUDIT-12-02** | **P2 (i18n)** | `src/index.html` | Элемент `<span class="commands-guide-summary" id="commands-guide-summary">` содержал жестко закодированный текст `"8 из 8 команд активно"`, нарушающий интернационализацию. | **ИСПРАВЛЕНО**: Статический русский текст удалён из разметки; вывод переведён на динамический вызов `getTranslation("commands_guide_active_summary", { active, total })` во всех 9 языках. |
| **AUDIT-12-03** | **P2 (i18n)** | `src/main.js` | Кнопка тестирования микрофона и обработчик ошибок содержали жестко закодированные русские фоллбэки `"Остановить тест"` и `"Ошибка микрофона: "`. | **ИСПРАВЛЕНО**: Фоллбэки заменены на словарные ключи `mic_meter_test_btn_stop` и `mic_start_error`, присутствующие во всех 9 локалях. |
| **AUDIT-12-04** | **P3 (CI / Formatting)** | `src-tauri/src/text_normalizer.rs` | Форматирование нескольких тестов и условий не соответствовало правилам `rustfmt`. Проверка `cargo fmt --check` падала. | **ИСПРАВЛЕНО**: Выполнен `cargo fmt`. Все файлы backend форматированы по стандарту, `cargo fmt --check` завершается с кодом 0. |
| **AUDIT-12-05** | **P3 (Audit Tooling)** | `scripts/check-overlay-i18n.mjs` | Скрипт пытался извлечь удалённый маркер `contextIndicatorTranslations` и падал с `TypeError: Cannot convert undefined or null to object`. | **ИСПРАВЛЕНО**: Скрипт адаптирован для опционального извлечения и корректной проверки `processingTranslations`, `noticeTranslations` и `errorTranslations`. |
| **AUDIT-12-06** | **P2 (Release Sync)** | Весь проект | Требовалось повышение версии до v1.2.0 с обеспечением 100% согласованности между всеми файлами проекта. | **ИСПРАВЛЕНО**: Версия 1.2.0 синхронизирована в `package.json`, `package-lock.json`, `Cargo.toml`, `Cargo.lock`, `tauri.conf.json`, `release-manifest-1.2.0.json`, 9 словарях `main.js` и `index.html`. Тест authoritative version пройден. |

---

## 4. Сводная таблица проверочных шлюзов (Quality Gates)

| Проверка | Команда | Результат | Статус |
| :--- | :--- | :--- | :--- |
| **Rust Tests** | `cargo test --lib` | 203 passed, 0 failed, 1 ignored | ✅ PASS |
| **Frontend Tests** | `npm test` | 20 passed, 0 failed | ✅ PASS |
| **Typecheck & Lint** | `npm run check` | 0 errors, 0 type issues | ✅ PASS |
| **Rustfmt Check** | `cargo fmt --check` | 100% clean formatting | ✅ PASS |
| **Clippy Linter** | `cargo clippy -- -D warnings` | 0 warnings | ✅ PASS |
| **Security Audit** | `cargo audit` | 0 vulnerabilities found | ✅ PASS |
| **Comprehensive Audit** | `node scripts/comprehensive-audit.mjs` | 164 IDs, 270 keys, 36 IPC cmds — 0 issues | ✅ PASS |
| **Deep Integrity Audit** | `node scripts/deep-audit.mjs` | Themes, langs, events: 0 issues | ✅ PASS |
| **All Elements Audit** | `node scripts/audit-all-elements.mjs` | 31 buttons (0 issues), 27 inputs (0 issues) | ✅ PASS |
| **IPC Signatures** | `node scripts/check-ipc.mjs` | 36 commands registered & permitted | ✅ PASS |
| **CSS Invariants** | `node scripts/check-css-invariants.mjs` | Zero illegal glow/shadow violations | ✅ PASS |
| **A11y & ARIA** | `node scripts/check-aria.mjs` | All required aria attributes valid | ✅ PASS |
| **Overlay i18n Audit** | `node scripts/check-overlay-i18n.mjs` | 13 notices, 12 errors across all locales | ✅ PASS |
| **Hardcoded Text Audit** | `node scripts/check-hardcoded-text.mjs` | 0 unlocalized UI sentences | ✅ PASS |
| **Storybook Build** | `npm run build-storybook` | 188 modules transformed, exit code 0 | ✅ PASS |
| **Harness Init** | `./init.sh` | Clean verification baseline | ✅ PASS |

---

## 5. Заключение

Программа **Aura 2.0 (v1.2.0)** успешно прошла все 7 этапов комплексного аудита. Все выявленные архитектурные, криптографические, языковые и форматирующие замечания полностью устранены и верифицированы автоматическими тестами. Программа находится в стабильном, защищённом и готовом к релизу состоянии.
