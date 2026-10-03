# Отчёт о комплексном аудите программы Aura (v1.1.2)

**Дата проведения:** 4 октября 2026 г.  
**Версия приложения:** Aura 2.0 (v1.1.2)  
**Платформа:** Windows 10/11 (x64)  
**Статус аудита:** ✅ **УСПЕШНО ПРОЙДЕН (GREEN / Production Grade)**

---

## 1. Резюме аудита (Executive Summary)

Проведён полный, сквозной аудит кодовой базы, архитектуры, подсистем ввода-вывода, звукового тракта, безопасности и пользовательского интерфейса настольного приложения Aura (версия 1.1.2).

### Ключевые показатели надёжности:
- **Rust Backend Unit Tests**: **161 пройден, 0 упало, 1 пропущен** (`cargo test`)
- **Frontend Static Integrity Gates**: **18 пройдены, 0 упало** (`npm test`)
- **Static Analysis / Clippy**: **0 предупреждений, 0 ошибок** (`cargo clippy -- -D warnings`)
- **Security Dependency Audit**: **0 уязвимостей** (`cargo audit`)
- **Lint & TypeScript Typecheck**: **0 ошибок** (`npm run check`)
- **Скрипты целостности системы (6 наборов)**: **100% пройдены, 0 дефектов**

---

## 2. Результаты аудита по подсистемам

### 2.1. Win32 Low-Level Keyboard Hooks & Input Simulation
- **Файлы**: `src-tauri/src/keyboard_hook.rs`, `src-tauri/src/keyboard_simulator.rs`, `src-tauri/src/lib.rs`
- **Проверенные инварианты**:
  1. **Изоляция внедрённых событий (`LLKHF_INJECTED`)**: В `low_level_keyboard_proc` проверяется флаг `0x10`. Все события, сгенерированные симулятором ввода Aura, гарантированно игнорируются хуком, полностью исключая рекурсивное зацикливание.
  2. **Подавление физических автоповторов**: При зажатой горячей клавише (Alt+V) во время диктовки физические события `WM_KEYDOWN` клавиши `V` перехватываются (`return 1`), что предотвращает утечку посторонних символов текущей раскладки в текстовое поле.
  3. **Атомарность замены текста (`replace_text`)**: Снятие модификаторов (Alt/Ctrl/Shift) -> посылка Backspace -> посылка символов -> восстановление модификаторов выполняется единым неделимым блоком.
  4. **Контроль очереди сообщений (Queue Flushing)**:
     - Backspace: пауза 15 мс после каждых 10 нажатий, 3 мс между остальными.
     - Печать: пауза 10 мс после каждых 15 символов, 1 мс между остальными.
  5. **Атомарность суррогатных пар UTF-16**: Эмодзи и 4-байтовые символы никогда не разрываются границами батчей `SendInput` и отправляются блоками `SendInput(4)`.
  6. **Защита фокуса (`FocusTarget`)**: При смене окна пользователем во время диктовки текст никогда не впечатывается в чужое окно — он сохраняется в буфер обмена и историю.
  7. **Безопасный handoff буфера обмена (`ClipboardBackup::Preserve`)**: Нетекстовые форматы (файлы, таблицы Excel, форматированный RTF) определяются через `CountClipboardFormats()` и никогда не очищаются вслепую при передаче текста.

### 2.2. Резидентные серверные движки (Whisper, Parakeet, Llama-Server)
- **Файлы**: `src-tauri/src/whisper_runner.rs`, `src-tauri/src/parakeet_streaming.rs`, `src-tauri/src/llm_runner.rs`, `src-tauri/src/ai_client.rs`
- **Проверенные инварианты**:
  1. **Поддержка CPU и NVIDIA CUDA**:
     - Whisper: resident `whisper-server` (CPU флаг `-ng`, до 8 потоков) и аппаратное ускорение CUDA через `ggml-cuda.dll` с удержанием в VRAM без холодного старта.
     - Parakeet: потоковый WebSocket-сервер на базе Sherpa-ONNX как в CPU, так и в CUDA режиме.
  2. **Предотвращение процессов-зомби (Windows Job Objects)**: Все дочерние процессы связываются с Windows Job Object (`JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE`) *до* запуска. При завершении или падении Aura операционная система мгновенно и гарантированно завершает все резидентные серверы.
  3. **Игровой режим (Gaming Mode) и сон VRAM**:
     - Детектирование полноэкранных 3D-игр через `SHQueryUserNotificationState` с фильтрацией окон браузеров и плееров (`Chrome_WidgetWin_*`, `vlc` и др.).
     - 12-секундный дебаунс (hysteresis) перед отключением игрового режима защищает от дерганья VRAM при Alt+Tab.
     - Таймер сна (`Standby Unloader`: 15/30/60 мин) освобождает 100% видеопамяти при неактивности.
  4. **Голосовое редактирование выделенного текста**:
     - Командные фразы («переведи», «сократи», «исправь») изолированы в системном промпте через `edit_intent.rs`.
     - Пользовательский контент содержит исключительно выделенный текст в блоке `<<<SELECTED_TEXT_START>>>`, полностью исключая самоперевод команды.
     - Валидатор `edit_validate.rs` контролирует отсутствие эха, утечки промпта и соответствие алфавита.

### 2.3. Облачные провайдеры (Cloud AI)
- **Файлы**: `src-tauri/src/ai_client.rs`
- **Проверенные инварианты**:
  1. **Каскад моделей Gemini**: Список кандидатов включает `gemini-3.8-flash` -> `gemini-3.7-flash` -> `gemini-3.6-flash` -> `gemini-2.5-flash` -> `gemini-1.5-flash` -> `gemini-1.5-pro`.
  2. **Устойчивость к ошибкам API**: При ошибках `400 Bad Request`, `404 Not Found` или `429 Rate Limit` клиент прозрачно переключается на следующую модель-кандидат. Прерывание цикла происходит строго при ошибках авторизации `401 | 403`.
  3. **Пул соединений**: Пул соединений `reqwest` с TCP Keep-Alive (60 с) и поддержкой системного прокси Windows.

### 2.4. Звуковой тракт, захват WASAPI и VAD
- **Файлы**: `src-tauri/src/audio_recorder.rs`, `src-tauri/src/vad.rs`
- **Проверенные инварианты**:
  1. **WASAPI без блокировок Tokio**: Опрос устройств переведён на `tokio::task::spawn_blocking`.
  2. **Потоковый ресемплер**: 44.1/48 кГц -> 16 кГц моно с 63-точечным FIR-фильтром нижних частот без фазовых искажений.
  3. **Silero VAD**: Неразрушающий шлюз `gate_wav_file` отсекает тишину без укорачивания аудиофайла.
  4. **Tail Hold**: 450-мс буфер удержания предотвращает обрезку тихих окончаний слов.

### 2.5. Безопасность, шифрование DPAPI и изоляция
- **Файлы**: `src-tauri/src/settings_secure.rs`, `src-tauri/src/secure_storage.rs`, `src-tauri/src/logger.rs`, `src-tauri/capabilities/`
- **Проверенные инварианты**:
  1. **Шифрование DPAPI**: Все API-ключи и история шифруются через Windows `CryptProtectData` с привязкой к учётной записи пользователя.
  2. **Ограничение прав (DACL)**: Файлы настроек сохраняются с правами только текущего пользователя (`TokenUser`) и `SYSTEM`.
  3. **Защита от утечки секретов**: Структура `SettingsView` маскирует ключи (`zeroize()`), фронтенду отдаются только флаги `has_api_key_*`.
  4. **Изоляция окон Tauri**: Окна `main`, `overlay` и `tray-menu` изолированы по принципу наименьших привилегий (Least Privilege). Опасные разрешения (`core:default`, `opener:default`, `updater:default`) отключены.
  5. **Санитайзинг логов**: В `logger.rs` реализовано маскирование персональных данных, API-ключей и текста диктовки (если отключена опция разработчика).

### 2.6. UI/UX, доступность и интернационализация
- **Файлы**: `src/index.html`, `src/main.js`, `src/style.css`, `src/tray-menu.js`, `src/ui-accessibility.js`
- **Проверенные инварианты**:
  1. **The Stealth Command Deck**: Тактический оранжевый (`#ff4200`) строго <= 10% площади экрана. Спокойные `.btn.btn-secondary`, отсутствие сплошной оранжевой заливки при наведении.
  2. **Единая типографика**: Сброс `font-family: inherit; font-size: inherit; color: inherit;` на всех элементах форм.
  3. **Интернационализация**: 9 полных словарей (`ru`, `en`, `de`, `es`, `fr`, `it`, `zh`, `pt`, `tr`), 242 уникальных ключа без пропусков.
  4. **Доступность (WCAG AA)**: Циклическая навигация стрелками по вкладкам, правильные роли `tablist`/`tabpanel`/`radiogroup`, наличие `aria-label` у всех интерактивных элементов.

---

## 3. Выявленные и устранённые дефекты в ходе аудита

| ID | Приоритет | Компонент | Описание проблемы | Статус исправления |
| :--- | :--- | :--- | :--- | :--- |
| **AUDIT-DEF-01** | **P2** | `src/main.js` | Кнопка `#btn-toggle-key-edit` (показ/скрытие API-ключа в карточке «Редактирование выделенного текста») отсутствовала в функции `applyLanguage`, из-за чего её `title` и `aria-label` оставались на русском языке при переключении интерфейса на другие языки. | **ИСПРАВЛЕНО**: Добавлено динамическое обновление `title` и `aria-label` в `applyLanguage` по аналогии с `#btn-toggle-key`. |
| **AUDIT-DEF-02** | **P2** | `src/tray-menu.js` | Функция `updateGamingState` запрашивала элемент `gaming-status-badge`, который был удалён из `tray-menu.html` в v1.1.2 для выравнивания тумблера к правому краю. Скрипт целостности `comprehensive-audit.mjs` фиксировал несоответствие DOM. | **ИСПРАВЛЕНО**: Мёртвый запрос к `gaming-status-badge` удалён из `tray-menu.js`. `comprehensive-audit.mjs` возвращает 0 ошибок. |
| **AUDIT-DEF-03** | **P3** | `scripts/audit-all-elements.mjs` | Скрипт аудита элементов использовал примитивный поиск `${id}.placeholder` и `${id}.setAttribute`, предполагая, что имя JS-переменной совпадает с HTML-id. Из-за дефисов в именах (`btn-window-minimize`) возникали ложные предупреждения. | **ИСПРАВЛЕНО**: Скрипт переведён на поиск через `matchAll` всех присвоенных переменных для каждого ID. Скрипт сообщает: 25 кнопок — 0 проблем, 25 полей ввода — 0 проблем. |

---

## 4. Сводная таблица проверочных шлюзов (Quality Gates)

| Проверка | Команда | Результат | Статус |
| :--- | :--- | :--- | :--- |
| **Rust Tests** | `cargo test` | 161 passed, 0 failed, 1 ignored | ✅ PASS |
| **Frontend Tests** | `npm test` | 18 passed, 0 failed | ✅ PASS |
| **Typecheck & Lint** | `npm run check` | 0 errors, 0 type issues | ✅ PASS |
| **Clippy Linter** | `cargo clippy -- -D warnings` | 0 warnings | ✅ PASS |
| **Security Audit** | `cargo audit` | 0 vulnerabilities found | ✅ PASS |
| **Comprehensive Audit** | `node scripts/comprehensive-audit.mjs` | 139 IDs checked, 242 keys checked, 0 missing | ✅ PASS |
| **Deep Integrity Audit** | `node scripts/deep-audit.mjs` | Sound themes, langs, events: 0 issues | ✅ PASS |
| **All Elements Audit** | `node scripts/audit-all-elements.mjs` | 25 buttons (0 issues), 25 inputs (0 issues) | ✅ PASS |
| **IPC Signatures** | `node scripts/check-ipc.mjs` | 35 commands registered & permitted | ✅ PASS |
| **CSS Invariants** | `node scripts/check-css-invariants.mjs` | Zero illegal glow/shadow violations | ✅ PASS |
| **A11y & ARIA** | `node scripts/check-aria.mjs` | All required aria attributes valid | ✅ PASS |
| **Storybook Build** | `npm run build-storybook` | 188 modules transformed, exit code 0 | ✅ PASS |

---

## 5. Заключение

Программа **Aura 2.0 (v1.1.2)** полностью соответствует каноническим архитектурным инвариантам, стандартам безопасности и правилам работы с Win32 хуками. Кодовая база находится в полностью стабилизированном, верифицированном и чистом состоянии.
