# Aura 1.1.0

Статус: релизная сборка готова к публикации.

## Главные нововведения

- **Игровой режим (Gaming Mode)**:
  - Автоматическое распознавание полноэкранных 3D-игр на базе DirectX/Vulkan (`SHQueryUserNotificationState` + `MonitorFromWindow`).
  - Мгновенная выгрузка фоновых резидентных серверов распознавания (Whisper.cpp / Sherpa-ONNX) с освобождением 100% VRAM.
  - Полный байпас глобального низкоуровневого хука клавиатуры (`HOOK_PAUSED`) для исключения любого инпут-лага в играх.
  - Динамическая оранжевая иконка в трее (`32x32_gaming.png`) и ручной тумблер включения в меню трея и настройках.

- **Сон и сбережение VRAM (Standby Unloader)**:
  - Автоматическая выгрузка резидентных моделей из оперативной и видеопамяти при бездействии (таймер: 15, 30, 60 минут или отключено).
  - Бесшовный повторный запуск при нажатии горячей клавиши диктовки.

- **Кастомное меню трея с адаптацией к High-DPI и мультимониторным системам**:
  - Быстрое контекстное меню прямо из системного трея: переключение движков (Whisper / Parakeet / Облако), тумблер игрового режима, выбор из всех 11 языков распознавания.
  - Точный расчет геометрии через Win32 API (`MonitorFromPoint` + `GetMonitorInfoW`), исключающий срезание и сдвиги на 4K-дисплеях и экранах с разным коэффициентом масштабирования DPI.

- **Автозамена текста и динамические макросы (Вкладка «Текст»)**:
  - Новая выделенная 6-я вкладка интерфейса «Текст».
  - Пользовательские правила автозамены сокращений и терминов.
  - Поддержка динамических макросов даты и времени: `{date}`, `{date_ru}`, `{time}`, `{datetime}`.
  - Атомарная замена текста с контролем очереди ввода Win32 (Queue Flushing): порционная отправка бэкспейсов и символов без потери знаков.

- **Журнал истории на 300 записей и группировка по датам**:
  - Увеличение глубины журнала до 300 последних транскрипций.
  - Группировка записей по датам («Сегодня», «Вчера», даты).
  - Быстрый поиск и копирование фрагментов в буфер обмена.
  - Фильтрация по источникам распознавания (Облако / Локально).

- **WASAPI Hotplug и стабильность звукового тракта**:
  - Автоматическое обнаружение подключения и отключения гарнитур/микрофонов без необходимости перезапуска приложения.
  - 450-мс буфер удержания записи (Tail Hold), гарантирующий сохранение окончаний фраз и тихих слогов.
  - 6 процедурных звуковых тем на Web Audio API (`zen`, `rhodes`, `scifi`, `classic`, `bubble`, `haptic`).

## Безопасность и архитектура

- Все API-ключи, пользовательские настройки и история защищены шифрованием Windows DPAPI (`CryptProtectData`) со строгой изоляцией дескрипторов безопасности DACL (`TokenUser` + `SYSTEM`).
- Разделение прав Tauri Capabilities по принципу наименьших привилегий (`main`, `overlay`, `tray-menu`).
- Архитектура «The Stealth Command Deck»: строгое ограничение акцентного оранжевого цвета, отсутствие размытых теней и избыточного декора.

## Проверка и верификация

- **Rust Backend**: 139 unit-тестов пройдены успешно (`cargo test --lib`).
- **Frontend / Static Gate**: Все тесты пройдены (`npm test`, `npm run check`).
- **WinGet**: Созданы пакетные манифесты для версии `1.1.0` (`Malashka.Aura`).
- **Контрольная сумма установщика**:
  - Файл: `Aura_1.1.0_x64-setup.exe`
  - Размер: `23 449 392` байт
  - SHA-256: `40400126D30931E552C8E30CE050C6DEF52BFE7EF3123A82AFDDFDA66F8F3534`

---

## English Summary

### Highlights
- **DirectX & Vulkan Gaming Mode**: Automatic detection of 3D games with keyboard hook suspension (`HOOK_PAUSED`) and resident model unloading to free 100% VRAM and eliminate input lag. Includes a dedicated orange tray icon (`32x32_gaming.png`) and manual toggle.
- **Idle VRAM Standby (Sleep)**: Automatic background model unloading after configurable inactivity (15, 30, 60 minutes) with zero-latency re-activation on next dictation.
- **Custom High-DPI Tray Menu**: Drill-down tray popup with per-monitor DPI awareness (`MonitorFromPoint`), engine switching, gaming mode toggle, and all 11 recognition languages.
- **Text Replacements & Dynamic Macros (Dedicated 6th Tab)**: User-defined text expansion rules with dynamic macros (`{date}`, `{date_ru}`, `{time}`, `{datetime}`) and atomic Win32 queue flushing.
- **300-Item Encrypted History**: Day grouping (Today / Yesterday / calendar dates), live search, DPAPI encryption, and source filtering.
- **WASAPI Microphone Hotplug**: Real-time headset/mic reconnection detection and 450ms tail hold buffer to prevent clipped word endings.
- **6 Sound Themes**: Procedural Web Audio themes (Zen, Rhodes, Sci-Fi, Bell, Water Drop, Haptic Switch).
- **Windows Package Manager**: `winget install Malashka.Aura`.
