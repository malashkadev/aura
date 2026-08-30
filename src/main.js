import { bindTabKeyboardNavigation } from "./ui-accessibility.js";

// Retrieve Tauri APIs from window.__TAURI__
const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;

window.logEvent = function(level, tag, message) {
  invoke("log_frontend_event", { level, tag, session: null, message }).catch(err => {
    console.error("Failed to log frontend event", err);
  });
};

const i18nDict = {
  "ru": {
    "title_settings": "Настройки",
    "tab_general": "Основные",
    "tab_speech": "Голос",
    "tab_hotkeys": "Управление",
    "section_engine": "Движок",
    "section_recognition": "Распознавание",
    "section_input": "Ввод",
    "section_dictionary": "Словарь",
    "tab_history": "История",
    "tab_about": "О программе",
    "general_autostart_title": "Автозапуск Aura",
    "general_autostart_desc": "Запускать приложение автоматически при входе в операционную систему Windows.",
    "general_autostart_checkbox": "Запускать Aura при старте системы",
    "engine_title": "Способ распознавания",
    "engine_desc": "Выберите между облачной обработкой высокого качества или полностью автономным локальным распознаванием речи.",
    "engine_cloud": "Облачный ИИ",
    "engine_cloud_meta": "Gemini / OpenAI / Groq (требуется API-ключ)",
    "engine_local": "Локальный ИИ",
    "engine_local_meta": "Whisper / Parakeet (100% офлайн)",
    "lang_bias_title": "Язык распознавания",
    "lang_bias_desc": "Выберите принудительный язык ввода или включите автоопределение.",
    "lang_bias_label": "Выберите язык",
    "lang_opt_auto": "Автоопределение",
    "lang_opt_layout": "По раскладке клавиатуры",
    "streaming_title": "Режим ввода текста",
    "streaming_desc": "Выберите способ отображения наговариваемого текста.",
    "streaming_checkbox": "Потоковый ввод в реальном времени (экспериментальный)",
    "streaming_subdesc": "Если выключено: текст вставится целиком только после того, как вы отпустите клавиши.",
    "vocab_title": "Пользовательский словарь",
    "vocab_desc": "Внесите термины, имена или брендовые названия через запятую, чтобы улучшить их распознавание.",
    "vocab_placeholder": "Например: Аура, коммит, репозиторий...",
    "engine_health_parakeet_running": "Parakeet: сервер запущен ({provider}, порт {port})",
    "engine_health_parakeet_stopped": "Parakeet: сервер не запущен",
    "engine_health_whisper_running": "Whisper: сервер запущен ({provider}, порт {port})",
    "engine_health_whisper_stopped": "Whisper: сервер не запущен",
    "engine_starting": "Запуск движка…",
    "local_model_title": "Локальное распознавание",
    "local_model_desc": "Настройте локальный движок распознавания речи для полной приватности.",
    "local_model_label": "Размер модели",
    "model_meta_tiny": "~75 МБ — сверхбыстрая",
    "model_meta_base": "~145 МБ — рекомендуемая",
    "model_meta_small": "~465 МБ — точная для русского",
    "model_meta_medium": "~1.5 ГБ — продвинутая",
    "model_meta_turbo": "~1.6 ГБ — лучшая точность для RU/EN",
    "model_meta_turbo_q5": "~550 МБ — почти как Turbo, вдвое легче",
    "model_cancel_download": "Отменить загрузку",
    "model_download_cancelled": "Загрузка отменена",
    "update_available": "Доступно обновление",
    "hotkey_title": "Глобальная горячая клавиша",
    "hotkey_desc": "Зажмите выбранную комбинацию для начала записи, отпустите для распознавания.",
    "hotkey_label": "Комбинация",
    "hotkey_toggle_mode": "Режим переключателя (короткое нажатие)",
    "hotkey_toggle_mode_desc": "Короткое нажатие начинает запись без удержания клавиши. Повторный клик останавливает запись.",
    "sound_title": "Звуковое сопровождение",
    "sound_desc": "Звуковые эффекты оверлея при записи.",
    "sound_enable": "Включить звуки оверлея",
    "sound_volume_label": "Громкость звука",
    "sound_theme_label": "Звуковая тема",
    "sound_theme_zen": "Дзен",
    "sound_theme_rhodes": "Rhodes",
    "sound_theme_scifi": "Sci-Fi",
    "sound_theme_classic": "Колокольчик",
    "api_title": "Авторизация API-ключей",
    "api_desc": "Укажите ваши API-ключи для авторизации в облачных сервисах Gemini, OpenAI или Groq.",
    "api_provider": "Провайдер API",
    "section_cloud_ai": "Облачные ИИ",
    "api_key": "API-ключ",
    "api_key_placeholder": "Введите ваш API-ключ...",
    "hotkey_prompt": "Нажмите клавиши...",
    "key_saved_placeholder": "•••••••• (сохранён безопасно)",
    "key_placeholder": "Введите API-ключ",
    "api_get_key": "Получить ключ API",
    "history_title": "История транскрипций",
    "history_clear": "Очистить историю",
    "history_desc": "Последние надиктованные фразы хранятся локально.",
    "history_empty": "История пуста. Ваши надиктованные тексты будут отображаться здесь.",
    "history_badge_cloud": "Облако",
    "history_badge_local": "Локально",
    "history_engine_whisper": "Whisper",
    "history_engine_parakeet": "NVIDIA Parakeet",
    "history_unit_ms": "мс",
    "history_unit_sec": "с",
    "about_app_title": "Голосовой ввод Aura",
    "about_version": "v1.0.10",
    "about_description": "Инструмент глобального голосового ввода для Windows. Программа переводит речь в текст и вставляет его в любое активное окно с автоматическим форматированием и расстановкой пунктуации.",
    "status_ready": "Готово",
    "btn_save": "Сохранить настройки",
    "confirm_title": "Подтверждение",
    "confirm_message": "Вы действительно хотите выполнить это действие?",
    "confirm_cancel": "Отмена",
    "confirm_ok": "Подтвердить",
    "status_loading": "Загрузка настроек...",
    "status_modified": "Настройки изменены (не сохранены)",
    "status_saving": "Сохранение настроек...",
    "status_saved": "Настройки успешно сохранены!",
    "status_error": "Ошибка: ",
    "model_status_ready": "Установлено",
    "model_action_download": "Скачать",
    "model_action_delete": "Удалить",
    "api_get_key_pattern": "Получить ключ на {name}",
    "status_loaded": "Настройки загружены",
    "status_load_error": "Ошибка загрузки настроек: ",
    "status_save_error": "Ошибка сохранения настроек: ",
    "status_streaming_degraded": "Живое превью Parakeet отключено — финальный текст готовится пакетно",
    "model_downloading_pattern": "Запуск скачивания для модели '{model}'...",
    "model_download_error_pattern": "Ошибка скачивания: {err}",
    "delete_model_title": "Удаление модели",
    "delete_model_confirm_pattern": "Вы действительно хотите удалить локальную модель '{model}'?",
    "delete_model_btn": "Удалить",
    "model_deleting_pattern": "Удаление модели '{model}'...",
    "model_deleted_success": "Модель успешно удалена",
    "model_delete_error_pattern": "Ошибка удаления: {err}",
    "model_downloaded_success_pattern": "Модель '{model}' скачана!",
    "confirm_clear_history_title": "Очистить историю",
    "confirm_clear_history_msg": "Вы действительно хотите очистить всю историю транскрипций?",
    "general_ui_lang_title": "Язык интерфейса",
    "general_ui_lang_desc": "Выберите язык для отображения настроек и уведомлений приложения.",
    "update_checks_title": "Проверка обновлений",
    "update_checks_desc": "Aura обращается к GitHub только при ручной проверке или если вы включили автоматическую проверку.",
    "update_checks_checkbox": "Автоматически проверять обновления при запуске",
    "update_check_now": "Проверить обновления",
    "update_current": "Установлена актуальная версия Aura.",
    "update_available_pattern": "Доступна Aura v{version}.",
    "update_check_error_pattern": "Не удалось проверить обновления: {error}",
    "update_installing": "Скачивание, проверка подписи и установка обновления...",
    "update_installed_restarting": "Обновление установлено. Перезапуск...",
    "update_install_error_open_release": "Не удалось установить обновление. Открываю страницу релиза...",
    "hotkey_reset_title": "Сбросить на Alt+V",
    "local_engine_label": "Движок распознавания",
    "local_engine_whisper": "Whisper.cpp",
    "local_engine_parakeet": "NVIDIA Parakeet",
    "parakeet_model_label": "Модель Parakeet",
    "model_meta_parakeet": "~670 МБ — оптимизировано NVIDIA",
    "fallback_title": "Автопереключение при недоступности облака",
    "fallback_desc": "Если облачный ИИ недоступен (VPN, блокировка региона, нет сети), автоматически использовать уже скачанную локальную модель для этой записи.",
    "fallback_checkbox": "Включить автопереключение на локальную модель",
    "copy_context_title": "Редактирование выделенного текста",
    "copy_context_desc": "Если включено, Aura отправляет Ctrl+C и передаёт выделенный текст выбранному облачному провайдеру как контекст для команды редактирования. Отключите эту функцию при работе в терминале.",
    "copy_context_checkbox": "Разрешить захват и облачное редактирование выделения",
    "gpu_accel_label": "Локальное аппаратное ускорение",
    "gpu_accel_cpu_title": "CPU (без ускорения)",
    "gpu_accel_cpu_desc": "Стандартный режим. Надёжен, но нагружает процессор.",
    "gpu_accel_cuda_title": "NVIDIA CUDA (максимальная скорость)",
    "gpu_accel_cuda_desc": "Для видеокарт GeForce RTX/GTX. Использует тензорные ядра.",
    "cuda_license_title": "Установка компонентов NVIDIA CUDA",
    "cuda_license_message": "Для работы аппаратного ускорения Aura загрузит библиотеки NVIDIA (~1.5 ГБ). Если совместимые компоненты уже установлены в системе, Aura использует их автоматически.",
    "cuda_license_footnote": "Лицензии:",
    "cuda_license_cuda_link": "NVIDIA CUDA Toolkit",
    "cuda_license_cudnn_link": "cuDNN",
    "cuda_license_accept": "Скачать",
    "gpu_accel_dml_title": "DirectML (универсальный)",
    "gpu_accel_dml_desc": "Для видеокарт AMD, Intel и NVIDIA. Базовое ускорение.",
    "btn_copy_diagnostics": "Скопировать отчет диагностики",
    "toast_diagnostics_copied": "Отчет диагностики скопирован в буфер обмена!",
    "diag_speech_text_title": "Логирование текста речи (режим разработчика)",
    "diag_title": "Диагностика",
    "diag_speech_text_desc": "Сохранять точный текст распознанной речи в диагностические логи. По умолчанию выключено для приватности.",
    "diag_speech_text_checkbox": "Записывать текст речи в логи",
    "api_custom_url": "Адрес API сервера",
    "api_custom_model": "Имя модели",
    "provider_opt_custom": "Пользовательский сервер",
    "overlay_timer_title": "Таймер и статус оверлея",
    "overlay_timer_desc": "Отображать счетчик времени и статус обработки под индикатором голоса. Ошибки отображаются всегда.",
    "overlay_timer_checkbox": "Показывать таймер и статус в оверлее",
    "mic_meter_title": "Проверка микрофона",
    "mic_meter_desc": "Проверьте уровень входящего сигнала и распознавание голоса в реальном времени.",
    "audio_device_label": "Микрофон (устройство ввода)",
    "audio_device_default": "По умолчанию (системное)",
    "audio_device_desc": "Выберите физический микрофон для записи голоса.",
    "mic_meter_test_btn_start": "Начать тест",
    "mic_meter_test_btn_stop": "Остановить тест",
    "mic_meter_speech": "Голос",
    "mic_meter_silence": "Тишина",
    "mic_start_error": "Ошибка запуска микрофона: ",
    "history_search_placeholder": "Поиск по истории...",
    "history_filter_all": "Все",
    "history_filter_cloud": "Облако",
    "history_filter_local": "Локально",
    "gpu_status_installing": "Установка..."
  },
  "en": {
    "title_settings": "Settings",
    "tab_general": "General",
    "tab_speech": "Speech",
    "tab_hotkeys": "Hotkeys",
    "section_engine": "Recognition engine",
    "section_recognition": "Recognition",
    "section_input": "Input",
    "section_dictionary": "Dictionary",
    "tab_history": "History",
    "tab_about": "About",
    "general_autostart_title": "Aura Autostart",
    "general_autostart_desc": "Launch the app automatically when starting Windows.",
    "general_autostart_checkbox": "Start Aura at system boot",
    "engine_title": "Processing Type",
    "engine_desc": "Choose between high-quality cloud transcription or fully local speech recognition.",
    "engine_cloud": "Cloud AI",
    "engine_cloud_meta": "Gemini / OpenAI / Groq (API key required)",
    "engine_local": "Local AI",
    "engine_local_meta": "Whisper / Parakeet (100% offline & private)",
    "lang_bias_title": "Speech Language",
    "lang_bias_desc": "Forcibly set transcription language or use automatic detection.",
    "lang_bias_label": "Select Language",
    "lang_opt_auto": "Auto-detect",
    "lang_opt_layout": "Follow Keyboard Layout",
    "streaming_title": "Text Streaming",
    "streaming_desc": "Choose how transcribed text is displayed.",
    "streaming_checkbox": "Real-time streaming typing (experimental)",
    "streaming_subdesc": "If disabled: text is typed as a whole only when you release hotkeys.",
    "vocab_title": "Custom Vocabulary",
    "vocab_desc": "Add specific terms, names, or jargon separated by commas to improve recognition.",
    "vocab_placeholder": "e.g. Aura, commit, repository...",
    "engine_health_parakeet_running": "Parakeet: server running ({provider}, port {port})",
    "engine_health_parakeet_stopped": "Parakeet: server not running",
    "engine_health_whisper_running": "Whisper: server running ({provider}, port {port})",
    "engine_health_whisper_stopped": "Whisper: server not running",
    "engine_starting": "Starting engine…",
    "local_model_title": "Local Recognition",
    "local_model_desc": "Configure a local speech-to-text engine for complete privacy.",
    "local_model_label": "Model Size",
    "model_meta_tiny": "~75 MB — superfast",
    "model_meta_base": "~145 MB — recommended",
    "model_meta_small": "~465 MB — accurate",
    "model_meta_medium": "~1.5 GB — advanced",
    "model_meta_turbo": "~1.6 GB — best accuracy for RU/EN",
    "model_meta_turbo_q5": "~550 MB — near-Turbo, half the size",
    "model_cancel_download": "Cancel download",
    "model_download_cancelled": "Download cancelled",
    "update_available": "Update available",
    "hotkey_title": "Global Hotkey",
    "hotkey_desc": "Hold down the selected hotkey to record, release to transcribe.",
    "hotkey_label": "Combination",
    "hotkey_toggle_mode": "Toggle mode (short tap)",
    "hotkey_toggle_mode_desc": "Short tap starts/stops recording without holding key down.",
    "sound_title": "Overlay Audio Feedback",
    "sound_desc": "Audio sound effects when recording states change.",
    "sound_enable": "Enable overlay sounds",
    "sound_volume_label": "Sound Volume",
    "sound_theme_label": "Sound Theme",
    "sound_theme_zen": "Zen",
    "sound_theme_rhodes": "Rhodes",
    "sound_theme_scifi": "Sci-Fi",
    "sound_theme_classic": "Bell",
    "api_title": "API Keys Authorization",
    "api_desc": "Provide API keys for Gemini, OpenAI, or Groq cloud services.",
    "api_provider": "API Provider",
    "section_cloud_ai": "Cloud AI",
    "api_key": "API Key",
    "api_key_placeholder": "Enter your API key...",
    "hotkey_prompt": "Press keys...",
    "key_saved_placeholder": "•••••••• (saved securely)",
    "key_placeholder": "Enter API key",
    "api_get_key": "Get API Key",
    "history_title": "Transcription History",
    "history_clear": "Clear History",
    "history_desc": "Your latest transcribed phrases are cached locally.",
    "history_empty": "History is empty. Dictated text fragments will appear here.",
    "history_badge_cloud": "Cloud",
    "history_badge_local": "Local",
    "history_engine_whisper": "Whisper",
    "history_engine_parakeet": "NVIDIA Parakeet",
    "history_unit_ms": "ms",
    "history_unit_sec": "s",
    "about_app_title": "Aura Voice Input",
    "about_version": "v1.0.10",
    "about_description": "Global voice input tool for Windows. The program transcribes speech to text and inserts it into any active window with automatic formatting and punctuation.",
    "status_ready": "Ready",
    "btn_save": "Save Settings",
    "confirm_title": "Confirmation",
    "confirm_message": "Are you sure you want to perform this action?",
    "confirm_cancel": "Cancel",
    "confirm_ok": "Confirm",
    "status_loading": "Loading settings...",
    "status_modified": "Settings changed (unsaved)",
    "status_saving": "Saving settings...",
    "status_saved": "Settings saved successfully!",
    "status_error": "Error: ",
    "model_status_ready": "Installed",
    "model_action_download": "Download",
    "model_action_delete": "Delete",
    "api_get_key_pattern": "Get key on {name}",
    "status_loaded": "Settings loaded",
    "status_load_error": "Failed to load settings: ",
    "status_save_error": "Failed to save settings: ",
    "status_streaming_degraded": "Live Parakeet preview disabled — the final text is being prepared in batch mode",
    "model_downloading_pattern": "Starting download for model '{model}'...",
    "model_download_error_pattern": "Download error: {err}",
    "delete_model_title": "Delete model",
    "delete_model_confirm_pattern": "Are you sure you want to delete the local model '{model}'?",
    "delete_model_btn": "Delete",
    "model_deleting_pattern": "Deleting model '{model}'...",
    "model_deleted_success": "Model deleted successfully",
    "model_delete_error_pattern": "Delete error: {err}",
    "model_downloaded_success_pattern": "Model '{model}' downloaded!",
    "confirm_clear_history_title": "Clear history",
    "confirm_clear_history_msg": "Are you sure you want to clear all transcription history?",
    "general_ui_lang_title": "Interface Language",
    "general_ui_lang_desc": "Select the language for settings and application notifications.",
    "update_checks_title": "Update checks",
    "update_checks_desc": "Aura contacts GitHub only when you check manually or enable automatic checks.",
    "update_checks_checkbox": "Check for updates automatically at startup",
    "update_check_now": "Check for updates",
    "update_current": "Aura is up to date.",
    "update_available_pattern": "Aura v{version} is available.",
    "update_check_error_pattern": "Could not check for updates: {error}",
    "update_installing": "Downloading, verifying the signature, and installing the update...",
    "update_installed_restarting": "Update installed. Restarting...",
    "update_install_error_open_release": "Could not install the update. Opening the release page...",
    "hotkey_reset_title": "Reset to Alt+V",
    "local_engine_label": "ASR Engine",
    "local_engine_whisper": "Whisper.cpp",
    "local_engine_parakeet": "NVIDIA Parakeet",
    "parakeet_model_label": "Parakeet Model",
    "model_meta_parakeet": "~670 MB — optimized by NVIDIA",
    "fallback_title": "Automatic fallback when cloud is unavailable",
    "fallback_desc": "If cloud AI is unavailable (VPN, region block, no network), automatically use the already downloaded local model for this recording.",
    "fallback_checkbox": "Enable automatic fallback to local model",
    "copy_context_title": "Edit selected text",
    "copy_context_desc": "When enabled, Aura sends Ctrl+C and passes the selected text to the chosen cloud provider as context for an editing command. Disable this feature when working in a terminal.",
    "copy_context_checkbox": "Allow selection capture and cloud editing",
    "gpu_accel_label": "Local hardware acceleration",
    "gpu_accel_cpu_title": "CPU (no acceleration)",
    "gpu_accel_cpu_desc": "Standard mode. Reliable, but uses more CPU.",
    "gpu_accel_cuda_title": "NVIDIA CUDA (maximum speed)",
    "gpu_accel_cuda_desc": "For GeForce RTX/GTX GPUs. Uses Tensor Cores.",
    "cuda_license_title": "NVIDIA CUDA terms",
    "cuda_license_message": "Aura will download up to 1.52 GiB of acceleration archives and install up to 2.33 GiB of files. Compatible NVIDIA components already installed on this PC are reused; otherwise Aura downloads proprietary CUDA and cuDNN components directly from NVIDIA under NVIDIA's terms.",
    "cuda_license_footnote": "Licenses:",
    "cuda_license_cuda_link": "CUDA Toolkit license",
    "cuda_license_cudnn_link": "cuDNN license",
    "cuda_license_accept": "Download",
    "gpu_accel_dml_title": "DirectML (universal)",
    "gpu_accel_dml_desc": "For AMD, Intel, and NVIDIA GPUs. Basic acceleration.",
    "btn_copy_diagnostics": "Copy Diagnostic Report",
    "toast_diagnostics_copied": "Diagnostic report copied to clipboard!",
    "diag_speech_text_title": "Log Speech Text (Developer Mode)",
    "diag_title": "Diagnostics",
    "diag_speech_text_desc": "Include exact transcribed speech text in diagnostic logs. Disabled by default for privacy.",
    "diag_speech_text_checkbox": "Include speech text in logs",
    "api_custom_url": "Server API URL",
    "api_custom_model": "Model Name",
    "provider_opt_custom": "Custom Server",
    "overlay_timer_title": "Overlay Timer and Status",
    "overlay_timer_desc": "Show duration timer and processing status below voice indicator. Errors are always displayed.",
    "overlay_timer_checkbox": "Show timer and status on overlay",
    "mic_meter_title": "Microphone Check",
    "mic_meter_desc": "Test input signal level and real-time voice detection.",
    "audio_device_label": "Microphone (input device)",
    "audio_device_default": "Default (system)",
    "audio_device_desc": "Select physical microphone for voice recording.",
    "mic_meter_test_btn_start": "Start Test",
    "mic_meter_test_btn_stop": "Stop Test",
    "mic_meter_speech": "Voice",
    "mic_meter_silence": "Silence",
    "mic_start_error": "Microphone start error: ",
    "history_search_placeholder": "Search history...",
    "history_filter_all": "All",
    "history_filter_cloud": "Cloud",
    "history_filter_local": "Local",
    "gpu_status_installing": "Installing..."
  },
  "de": {
    "gpu_accel_label": "Lokale Hardware-Beschleunigung",
    "gpu_accel_cpu_title": "CPU (keine Beschleunigung)",
    "gpu_accel_cpu_desc": "Standardmodus. Zuverlässig, beansprucht aber die CPU.",
    "gpu_accel_cuda_title": "NVIDIA CUDA (maximale Geschwindigkeit)",
    "gpu_accel_cuda_desc": "Für GeForce RTX/GTX-Grafikkarten. Nutzt Tensor Cores.",
    "cuda_license_title": "NVIDIA-CUDA-Bedingungen",
    "cuda_license_message": "Aura lädt bis zu 1,52 GiB Beschleunigungsarchive herunter und installiert bis zu 2,33 GiB Dateien. Bereits installierte kompatible NVIDIA-Komponenten werden wiederverwendet; andernfalls lädt Aura proprietäre CUDA- und cuDNN-Komponenten gemäß den NVIDIA-Bedingungen direkt von NVIDIA herunter.",
    "cuda_license_footnote": "Lizenzen:",
    "cuda_license_cuda_link": "CUDA-Toolkit-Lizenz",
    "cuda_license_cudnn_link": "cuDNN-Lizenz",
    "cuda_license_accept": "Herunterladen",
    "gpu_accel_dml_title": "DirectML (universell)",
    "gpu_accel_dml_desc": "Für AMD-, Intel- und NVIDIA-Grafikkarten. Basisbeschleunigung.",
    "title_settings": "Einstellungen",
    "tab_general": "Allgemein",
    "tab_speech": "Diktat",
    "tab_hotkeys": "Tastenkombinationen",
    "section_engine": "Erkennungsmodul",
    "section_recognition": "Spracherkennung",
    "section_input": "Eingabe",
    "section_dictionary": "Wörterbuch",
    "tab_history": "Verlauf",
    "tab_about": "Über Aura",
    "general_autostart_title": "Aura Autostart",
    "general_autostart_desc": "Startet die App automatisch beim Anmelden in Windows.",
    "general_autostart_checkbox": "Aura beim Systemstart starten",
    "engine_title": "Verarbeitungstyp",
    "engine_desc": "Wählen Sie zwischen Cloud-Transkription oder vollständig lokaler Spracherkennung.",
    "engine_cloud": "Cloud-KI",
    "engine_cloud_meta": "Gemini / OpenAI / Groq (API-Schlüssel erforderlich)",
    "engine_local": "Lokale KI",
    "engine_local_meta": "Whisper / Parakeet (100% offline & privat)",
    "lang_bias_title": "Sprache",
    "lang_bias_desc": "Wählen Sie eine feste Sprache für das Diktat oder aktivieren Sie die Auto-Erkennung.",
    "lang_bias_label": "Sprache auswählen",
    "lang_opt_auto": "Auto-Erkennung",
    "lang_opt_layout": "Tastaturlayout folgen",
    "streaming_title": "Text-Streaming",
    "streaming_desc": "Wählen Sie, wie die Transkription eingegeben wird.",
    "streaming_checkbox": "Echtzeit-Streaming-Eingabe (experimentell)",
    "streaming_subdesc": "Wenn deaktiviert: Text wird als Ganzes eingefügt, wenn die Taste losgelassen wird.",
    "vocab_title": "Eigenes Wörterbuch",
    "vocab_desc": "Tragen Sie Begriffe, Namen oder Fachbegriffe durch Komma getrennt ein, um die Erkennung zu verbessern.",
    "vocab_placeholder": "z.B. Aura, Commit, Repository...",
    "engine_health_parakeet_running": "Parakeet: Server läuft ({provider}, Port {port})",
    "engine_health_parakeet_stopped": "Parakeet: Server läuft nicht",
    "engine_health_whisper_running": "Whisper: Server läuft ({provider}, Port {port})",
    "engine_health_whisper_stopped": "Whisper: Server läuft nicht",
    "engine_starting": "Engine wird gestartet…",
    "local_model_title": "Lokales Whisper-Modell",
    "local_model_desc": "Konfigurieren Sie eine lokale Spracherkennungs-Engine für vollständige Privatsphäre.",
    "local_model_label": "Modellgröße",
    "model_meta_tiny": "~75 MB — superschnell",
    "model_meta_base": "~145 MB — empfohlen",
    "model_meta_small": "~465 MB — präzise",
    "model_meta_medium": "~1.5 GB — fortgeschritten",
    "model_meta_turbo": "~1.6 GB — beste Genauigkeit für RU/EN",
    "model_meta_turbo_q5": "~550 MB — fast wie Turbo, halb so groß",
    "hotkey_title": "Globale Taste",
    "hotkey_desc": "Tastenkombination gedrückt halten, um aufzunehmen, loslassen zur Transkription.",
    "hotkey_label": "Kombination",
    "hotkey_toggle_mode": "Umschaltmodus (kurzes Tippen)",
    "hotkey_toggle_mode_desc": "Kurzes Antippen startet/stoppt Aufnahme ohne Halten.",
    "sound_title": "Audio-Rückmeldung",
    "sound_desc": "Soundeffekte des Overlays während der Aufnahme.",
    "sound_enable": "Overlay-Sounds aktivieren",
    "sound_volume_label": "Tonlautstärke",
    "sound_theme_label": "Sound-Theme",
    "sound_theme_zen": "Zen",
    "sound_theme_rhodes": "Rhodes",
    "sound_theme_scifi": "Sci-Fi",
    "sound_theme_classic": "Glocke",
    "api_title": "API-Schlüssel Autorisierung",
    "api_desc": "Geben Sie Ihre API-Schlüssel für Gemini, OpenAI oder Groq Cloud-Dienste ein.",
    "api_provider": "API-Provider",
    "section_cloud_ai": "Cloud-KI",
    "api_key": "API-Schlüssel",
    "api_key_placeholder": "Geben Sie Ihren API-Schlüssel ein...",
    "hotkey_prompt": "Tasten drücken...",
    "key_saved_placeholder": "•••••••• (sicher gespeichert)",
    "key_placeholder": "API-Schlüssel eingeben",
    "api_get_key": "API-Schlüssel erhalten",
    "history_title": "Diktatverlauf",
    "history_clear": "Verlauf löschen",
    "history_desc": "Die letzten aufgezeichneten Sätze werden lokal gespeichert.",
    "history_empty": "Der Verlauf ist leer. Transkribierte Texte werden hier angezeigt.",
    "history_badge_cloud": "Cloud",
    "history_badge_local": "Lokal",
    "history_engine_whisper": "Whisper",
    "history_engine_parakeet": "NVIDIA Parakeet",
    "history_unit_ms": "ms",
    "history_unit_sec": "s",
    "about_app_title": "Aura Spracheingabe",
    "about_version": "v1.0.10",
    "about_description": "Globales Spracheingabe-Tool für Windows. Die Anwendung überträgt Sprache in Text und fügt ihn mit automatischer Formatierung und Zeichensetzung in jedes aktive Fenster ein.",
    "status_ready": "Bereit",
    "btn_save": "Einstellungen speichern",
    "confirm_title": "Bestätigung",
    "confirm_message": "Sind Sie sicher, dass Sie diese Aktion ausführen möchten?",
    "confirm_cancel": "Abbrechen",
    "confirm_ok": "Bestätigen",
    "status_loading": "Einstellungen werden geladen...",
    "status_modified": "Einstellungen geändert (ungespeichert)",
    "status_saving": "Einstellungen werden gespeichert...",
    "status_saved": "Einstellungen erfolgreich gespeichert!",
    "status_error": "Fehler: ",
    "model_status_ready": "Installiert",
    "model_action_download": "Herunterladen",
    "model_action_delete": "Löschen",
    "api_get_key_pattern": "Schlüssel erhalten auf {name}",
    "status_loaded": "Einstellungen geladen",
    "status_load_error": "Fehler beim Laden der Einstellungen: ",
    "status_save_error": "Fehler beim Speichern der Einstellungen: ",
    "status_streaming_degraded": "Live-Parakeet-Vorschau deaktiviert — der finale Text wird im Batch-Modus erstellt",
    "model_downloading_pattern": "Download für Modell '{model}' wird gestartet...",
    "model_download_error_pattern": "Download-Fehler: {err}",
    "delete_model_title": "Modell löschen",
    "delete_model_confirm_pattern": "Möchten Sie das lokale Modell '{model}' wirklich löschen?",
    "delete_model_btn": "Löschen",
    "model_deleting_pattern": "Modell '{model}' wird gelöscht...",
    "model_deleted_success": "Modell erfolgreich gelöscht",
    "model_delete_error_pattern": "Fehler beim Löschen: {err}",
    "model_downloaded_success_pattern": "Modell '{model}' heruntergeladen!",
    "confirm_clear_history_title": "Verlauf löschen",
    "confirm_clear_history_msg": "Möchten Sie den gesamten Transkriptionsverlauf wirklich löschen?",
    "general_ui_lang_title": "Sprache der Benutzeroberfläche",
    "general_ui_lang_desc": "Wählen Sie die Sprache für Einstellungen und Benachrichtigungen.",
    "update_checks_title": "Update-Prüfung",
    "update_checks_desc": "Aura kontaktiert GitHub nur bei einer manuellen Prüfung oder wenn Sie automatische Prüfungen aktivieren.",
    "update_checks_checkbox": "Beim Start automatisch nach Updates suchen",
    "update_check_now": "Nach Updates suchen",
    "update_current": "Aura ist auf dem neuesten Stand.",
    "update_available_pattern": "Aura v{version} ist verfügbar.",
    "update_check_error_pattern": "Updates konnten nicht geprüft werden: {error}",
    "update_installing": "Update wird heruntergeladen, die Signatur geprüft und die Installation ausgeführt...",
    "update_installed_restarting": "Update installiert. Neustart...",
    "update_install_error_open_release": "Update konnte nicht installiert werden. Die Release-Seite wird geöffnet...",
    "hotkey_reset_title": "Auf Alt+V zurücksetzen",
    "local_engine_label": "Erkennungsmodul",
    "local_engine_whisper": "Whisper.cpp",
    "local_engine_parakeet": "NVIDIA Parakeet",
    "parakeet_model_label": "Parakeet-Modell",
    "model_meta_parakeet": "~670 MB — optimiert von NVIDIA",
    "model_cancel_download": "Download abbrechen",
    "model_download_cancelled": "Download abgebrochen",
    "update_available": "Update verfügbar",
    "fallback_title": "Automatischer Wechsel bei nicht verfügbarer Cloud",
    "fallback_desc": "Wenn die Cloud-KI nicht erreichbar ist (VPN, Regionssperre, kein Netzwerk), automatisch das bereits heruntergeladene lokale Modell für diese Aufnahme verwenden.",
    "fallback_checkbox": "Automatischen Fallback auf lokales Modell aktivieren",
    "copy_context_title": "Ausgewählten Text bearbeiten",
    "copy_context_desc": "Wenn aktiviert, sendet Aura Strg+C und übermittelt den ausgewählten Text als Kontext für einen Bearbeitungsbefehl an den gewählten Cloud-Anbieter. Deaktivieren Sie diese Funktion bei der Arbeit im Terminal.",
    "copy_context_checkbox": "Erfassen der Auswahl und Cloud-Bearbeitung zulassen",
    "btn_copy_diagnostics": "Diagnosebericht kopieren",
    "toast_diagnostics_copied": "Diagnosebericht in Zwischenablage kopiert!",
    "diag_speech_text_title": "Sprachtext protokollieren (Entwicklermodus)",
    "diag_title": "Diagnose",
    "diag_speech_text_desc": "Exakten transkribierten Sprachtext in Diagnoseprotokollen speichern. Aus Datenschutzgründen standardmäßig deaktiviert.",
    "diag_speech_text_checkbox": "Sprachtext in Protokolle aufnehmen",
    "api_custom_url": "Server-API-Adresse",
    "api_custom_model": "Modellname",
    "provider_opt_custom": "Benutzerdefinierter Server",
    "overlay_timer_title": "Overlay-Timer und Status",
    "overlay_timer_desc": "Zeitanzeige und Verarbeitungsstatus unter der Stimmanzeige anzeigen. Fehler werden immer angezeigt.",
    "overlay_timer_checkbox": "Timer und Status im Overlay anzeigen",
    "mic_meter_title": "Mikrofontest",
    "mic_meter_desc": "Testen Sie den Eingangssignalpegel und die Spracherkennung in Echtzeit.",
    "audio_device_label": "Mikrofon (Eingabegerät)",
    "audio_device_default": "Standard (System)",
    "audio_device_desc": "Wählen Sie ein physisches Mikrofon für die Sprachaufnahme aus.",
    "mic_meter_test_btn_start": "Test starten",
    "mic_meter_test_btn_stop": "Test stoppen",
    "mic_meter_speech": "Sprache",
    "mic_meter_silence": "Stille",
    "mic_start_error": "Fehler beim Starten des Mikrofons: ",
    "history_search_placeholder": "Verlauf durchsuchen...",
    "history_filter_all": "Alle",
    "history_filter_cloud": "Cloud",
    "history_filter_local": "Lokal",
    "gpu_status_installing": "Installieren..."
  },
  "es": {
    "title_settings": "Ajustes",
    "tab_general": "General",
    "tab_speech": "Voz",
    "tab_hotkeys": "Accesos rápidos",
    "section_engine": "Motor de reconocimiento",
    "section_recognition": "Reconocimiento",
    "section_input": "Entrada",
    "section_dictionary": "Diccionario",
    "tab_history": "Historial",
    "tab_about": "Acerca de",
    "general_autostart_title": "Inicio automático",
    "general_autostart_desc": "Iniciar la aplicación de forma automática al arrancar Windows.",
    "general_autostart_checkbox": "Iniciar Aura con el sistema",
    "engine_title": "Tipo de procesamiento",
    "engine_desc": "Seleccione entre el procesamiento en la nube de alta calidad o el reconocimiento local totalmente autónomo.",
    "engine_cloud": "IA en la nube",
    "engine_cloud_meta": "Gemini / OpenAI / Groq (requiere clave API)",
    "engine_local": "IA local",
    "engine_local_meta": "Whisper / Parakeet (100% offline y privado)",
    "lang_bias_title": "Idioma de dictado",
    "lang_bias_desc": "Forzar un idioma específico para la transcripción o usar detección automática.",
    "lang_bias_label": "Seleccionar idioma",
    "lang_opt_auto": "Autodetectar",
    "lang_opt_layout": "Según teclado activo",
    "streaming_title": "Escritura fluida",
    "streaming_desc": "Seleccione el método para mostrar el texto transcrito.",
    "streaming_checkbox": "Escritura en tiempo real (experimental)",
    "streaming_subdesc": "Si está desactivado: el texto se inserta completo tras soltar el atajo.",
    "vocab_title": "Vocabulario personalizado",
    "vocab_desc": "Añada términos específicos, nombres o siglas separados por comas para mejorar el dictado.",
    "vocab_placeholder": "ej. Aura, commit, repositorio...",
    "engine_health_parakeet_running": "Parakeet: servidor en ejecución ({provider}, puerto {port})",
    "engine_health_parakeet_stopped": "Parakeet: servidor no en ejecución",
    "engine_health_whisper_running": "Whisper: servidor en ejecución ({provider}, puerto {port})",
    "engine_health_whisper_stopped": "Whisper: servidor no en ejecución",
    "engine_starting": "Iniciando motor…",
    "local_model_title": "Modelo Whisper local",
    "local_model_desc": "Configure un motor local de reconocimiento de voz para mantener la privacidad.",
    "local_model_label": "Tamaño del modelo",
    "model_meta_tiny": "~75 MB — superrápido",
    "model_meta_base": "~145 MB — recomendado",
    "model_meta_small": "~465 MB — preciso",
    "model_meta_medium": "~1.5 GB — avanzado",
    "model_meta_turbo": "~1.6 GB — mejor precisión para RU/EN",
    "model_meta_turbo_q5": "~550 MB — casi como Turbo, mitad de tamaño",
    "hotkey_title": "Acceso rápido global",
    "hotkey_desc": "Mantenga presionadas las teclas seleccionadas para grabar, suéltelas para transcribir.",
    "hotkey_label": "Combinación",
    "hotkey_toggle_mode": "Modo alternar (pulsación corta)",
    "hotkey_toggle_mode_desc": "Una pulsación corta inicia/detiene la grabación sin mantener la tecla.",
    "sound_title": "Efectos de audio",
    "sound_desc": "Efectos sonoros del overlay al grabar.",
    "sound_enable": "Activar sonidos del overlay",
    "sound_volume_label": "Volumen del sonido",
    "sound_theme_label": "Tema sonoro",
    "sound_theme_zen": "Zen",
    "sound_theme_rhodes": "Rhodes",
    "sound_theme_scifi": "Sci-Fi",
    "sound_theme_classic": "Campana",
    "api_title": "Autorización de claves API",
    "api_desc": "Introduzca sus claves API para los servicios en la nube de Gemini, OpenAI o Groq.",
    "api_provider": "Proveedor de API",
    "section_cloud_ai": "IA en la nube",
    "api_key": "Clave API",
    "api_key_placeholder": "Introduzca su clave API...",
    "hotkey_prompt": "Pulse las teclas...",
    "key_saved_placeholder": "•••••••• (guardado de forma segura)",
    "key_placeholder": "Introduzca la clave API",
    "api_get_key": "Obtener clave API",
    "history_title": "Historial de transcripción",
    "history_clear": "Limpiar historial",
    "history_desc": "Las últimas frases dictadas se guardan de forma local.",
    "history_empty": "El historial está vacío. Los textos dictados se mostrarán aquí.",
    "history_badge_cloud": "Nube",
    "history_badge_local": "Local",
    "history_engine_whisper": "Whisper",
    "history_engine_parakeet": "Parakeet de NVIDIA",
    "history_unit_ms": "ms",
    "history_unit_sec": "s",
    "about_app_title": "Dictado por voz Aura",
    "about_version": "v1.0.10",
    "about_description": "Herramienta de entrada de voz global para Windows. El programa transcribe el habla en texto y lo inserta en cualquier ventana activa con formato y puntuación automáticos.",
    "status_ready": "Listo",
    "btn_save": "Guardar ajustes",
    "confirm_title": "Confirmación",
    "confirm_message": "¿Está seguro de realizar esta acción?",
    "confirm_cancel": "Cancelar",
    "confirm_ok": "Confirmar",
    "status_loading": "Cargando ajustes...",
    "status_modified": "Ajustes modificados (sin guardar)",
    "status_saving": "Guardando ajustes...",
    "status_saved": "¡Ajustes guardados correctamente!",
    "status_error": "Error: ",
    "model_status_ready": "Instalado",
    "model_action_download": "Descargar",
    "model_action_delete": "Eliminar",
    "api_get_key_pattern": "Obtener clave en {name}",
    "status_loaded": "Ajustes cargados",
    "status_load_error": "Error al cargar los ajustes: ",
    "status_save_error": "Error al guardar los ajustes: ",
    "status_streaming_degraded": "Vista previa en vivo de Parakeet desactivada: el texto final se prepara por lotes",
    "model_downloading_pattern": "Iniciando descarga para el modelo '{model}'...",
    "model_download_error_pattern": "Error de descarga: {err}",
    "delete_model_title": "Eliminar modelo",
    "delete_model_confirm_pattern": "¿Está seguro de que desea eliminar el modelo local '{model}'?",
    "delete_model_btn": "Eliminar",
    "model_deleting_pattern": "Eliminando modelo '{model}'...",
    "model_deleted_success": "Modelo eliminado correctamente",
    "model_delete_error_pattern": "Error al eliminar: {err}",
    "model_downloaded_success_pattern": "¡Modelo '{model}' descargado!",
    "confirm_clear_history_title": "Limpiar historial",
    "confirm_clear_history_msg": "¿Está seguro de que desea limpiar todo el historial de transcripciones?",
    "general_ui_lang_title": "Idioma de la interfaz",
    "general_ui_lang_desc": "Seleccione el idioma para los ajustes y las notificaciones.",
    "update_checks_title": "Comprobación de actualizaciones",
    "update_checks_desc": "Aura se conecta a GitHub solo al comprobar manualmente o al activar las comprobaciones automáticas.",
    "update_checks_checkbox": "Buscar actualizaciones automáticamente al iniciar",
    "update_check_now": "Buscar actualizaciones",
    "update_current": "Aura está actualizada.",
    "update_available_pattern": "Aura v{version} está disponible.",
    "update_check_error_pattern": "No se pudieron buscar actualizaciones: {error}",
    "update_installing": "Descargando, verificando la firma e instalando la actualización...",
    "update_installed_restarting": "Actualización instalada. Reiniciando...",
    "update_install_error_open_release": "No se pudo instalar la actualización. Abriendo la página de la versión...",
    "gpu_accel_label": "Aceleración de hardware local",
    "gpu_accel_cpu_title": "CPU (sin aceleración)",
    "gpu_accel_cpu_desc": "Modo estándar. Es fiable, pero aumenta la carga del procesador.",
    "gpu_accel_cuda_title": "NVIDIA CUDA (velocidad máxima)",
    "gpu_accel_cuda_desc": "Para GPU GeForce RTX/GTX. Utiliza Tensor Cores.",
    "cuda_license_title": "Términos de NVIDIA CUDA",
    "cuda_license_message": "Aura descargará hasta 1,52 GiB de archivos de aceleración e instalará hasta 2,33 GiB de archivos. Se reutilizarán los componentes NVIDIA compatibles ya instalados; de lo contrario, Aura descargará componentes propietarios de CUDA y cuDNN directamente de NVIDIA bajo sus términos.",
    "cuda_license_footnote": "Licencias:",
    "cuda_license_cuda_link": "Licencia de CUDA Toolkit",
    "cuda_license_cudnn_link": "Licencia de cuDNN",
    "cuda_license_accept": "Descargar",
    "gpu_accel_dml_title": "DirectML (universal)",
    "gpu_accel_dml_desc": "Para GPU AMD, Intel y NVIDIA. Aceleración básica.",
    "hotkey_reset_title": "Restablecer a Alt+V",
    "local_engine_label": "Motor de reconocimiento",
    "local_engine_whisper": "Whisper.cpp",
    "local_engine_parakeet": "NVIDIA Parakeet",
    "parakeet_model_label": "Modelo Parakeet",
    "model_meta_parakeet": "~670 MB — optimizado por NVIDIA",
    "model_cancel_download": "Cancelar descarga",
    "model_download_cancelled": "Descarga cancelada",
    "update_available": "Actualización disponible",
    "fallback_title": "Cambio automático si la nube no está disponible",
    "fallback_desc": "Si la IA en la nube no está disponible (VPN, bloqueo regional, sin red), usar automáticamente el modelo local ya descargado para esta grabación.",
    "fallback_checkbox": "Activar cambio automático al modelo local",
    "copy_context_title": "Editar texto seleccionado",
    "copy_context_desc": "Cuando está activado, Aura envía Ctrl+C y pasa el texto seleccionado al proveedor en la nube elegido como contexto para una orden de edición. Desactive esta función al trabajar en una terminal.",
    "copy_context_checkbox": "Permitir captura de selección y edición en la nube",
    "btn_copy_diagnostics": "Copiar informe de diagnóstico",
    "toast_diagnostics_copied": "¡Informe de diagnóstico copiado al portapapeles!",
    "diag_speech_text_title": "Registrar texto de voz (Modo desarrollador)",
    "diag_title": "Diagnóstico",
    "diag_speech_text_desc": "Incluir texto de voz transcrito exacto en los registros de diagnóstico. Desactivado por defecto por privacidad.",
    "diag_speech_text_checkbox": "Incluir texto de voz en los registros",
    "api_custom_url": "Dirección API del servidor",
    "api_custom_model": "Nombre del modelo",
    "provider_opt_custom": "Servidor personalizado",
    "overlay_timer_title": "Temporizador y estado del overlay",
    "overlay_timer_desc": "Mostrar temporizador de duración y estado de procesamiento debajo del indicador de voz. Los errores siempre se muestran.",
    "overlay_timer_checkbox": "Mostrar temporizador y estado en el overlay",
    "mic_meter_title": "Comprobación de micrófono",
    "mic_meter_desc": "Pruebe el nivel de señal de entrada y la detección de voz en tiempo real.",
    "audio_device_label": "Micrófono (dispositivo de entrada)",
    "audio_device_default": "Predeterminado (sistema)",
    "audio_device_desc": "Seleccione el micrófono físico para la grabación de voz.",
    "mic_meter_test_btn_start": "Iniciar prueba",
    "mic_meter_test_btn_stop": "Detener prueba",
    "mic_meter_speech": "Voz",
    "mic_meter_silence": "Silencio",
    "mic_start_error": "Error al iniciar el micrófono: ",
    "history_search_placeholder": "Buscar en el historial...",
    "history_filter_all": "Todos",
    "history_filter_cloud": "Nube",
    "history_filter_local": "Local",
    "gpu_status_installing": "Instalando..."
  },
  "fr": {
    "title_settings": "Paramètres",
    "tab_general": "Général",
    "tab_speech": "Dictée",
    "tab_hotkeys": "Raccourcis",
    "section_engine": "Moteur de reconnaissance",
    "section_recognition": "Reconnaissance",
    "section_input": "Saisie",
    "section_dictionary": "Dictionnaire",
    "tab_history": "Historique",
    "tab_about": "À propos",
    "general_autostart_title": "Lancement automatique",
    "general_autostart_desc": "Lancer l'application automatiquement au démarrage de Windows.",
    "general_autostart_checkbox": "Démarrer Aura avec Windows",
    "engine_title": "Type de traitement",
    "engine_desc": "Choisissez entre un traitement cloud de haute qualité ou une reconnaissance locale 100% hors ligne.",
    "engine_cloud": "IA Cloud",
    "engine_cloud_meta": "Gemini / OpenAI / Groq (clé API requise)",
    "engine_local": "IA Locale",
    "engine_local_meta": "Whisper / Parakeet (100% hors ligne et privé)",
    "lang_bias_title": "Langue de dictée",
    "lang_bias_desc": "Forcer une langue spécifique pour la dictée ou utiliser la détection automatique.",
    "lang_bias_label": "Sélectionner la langue",
    "lang_opt_auto": "Détection automatique",
    "lang_opt_layout": "Selon le clavier actif",
    "streaming_title": "Saisie en continu",
    "streaming_desc": "Sélectionnez le mode d'affichage du texte transcrit.",
    "streaming_checkbox": "Affichage du texte en temps réel (expérimental)",
    "streaming_subdesc": "Si désactivé: le texte est inséré en une fois lorsque vous relâchez le raccourci.",
    "vocab_title": "Vocabulaire personnalisé",
    "vocab_desc": "Ajoutez des termes spécifiques, noms propres ou sigles séparés par des virgules pour améliorer la dictée.",
    "vocab_placeholder": "ex. Aura, commit, dépôt...",
    "engine_health_parakeet_running": "Parakeet : serveur en cours d'exécution ({provider}, port {port})",
    "engine_health_parakeet_stopped": "Parakeet : serveur non démarré",
    "engine_health_whisper_running": "Whisper : serveur en cours d'exécution ({provider}, port {port})",
    "engine_health_whisper_stopped": "Whisper : serveur non démarré",
    "engine_starting": "Démarrage du moteur…",
    "local_model_title": "Modèle Whisper local",
    "local_model_desc": "Configurez un moteur local de reconnaissance vocale pour préserver entièrement votre confidentialité.",
    "local_model_label": "Taille du modèle",
    "model_meta_tiny": "~75 Mo — super rapide",
    "model_meta_base": "~145 Mo — recommandé",
    "model_meta_small": "~465 Mo — précis",
    "model_meta_medium": "~1.5 Go — avancé",
    "model_meta_turbo": "~1.6 Go — meilleure précision RU/EN",
    "model_meta_turbo_q5": "~550 Mo — proche de Turbo, deux fois plus léger",
    "hotkey_title": "Raccourci global",
    "hotkey_desc": "Maintenez le raccourci pour enregistrer, relâchez pour transcrire.",
    "hotkey_label": "Combinaison",
    "hotkey_toggle_mode": "Mode alterné (appui court)",
    "hotkey_toggle_mode_desc": "Un appui court démarre/arrête l'enregistrement sans maintenir la touche.",
    "sound_title": "Retours audio",
    "sound_desc": "Effets sonores de l'overlay lors de l'enregistrement.",
    "sound_enable": "Activer les sons de l'overlay",
    "sound_volume_label": "Volume du son",
    "sound_theme_label": "Thème sonore",
    "sound_theme_zen": "Zen",
    "sound_theme_rhodes": "Rhodes",
    "sound_theme_scifi": "Sci-Fi",
    "sound_theme_classic": "Cloche",
    "api_title": "Clés d'API",
    "api_desc": "Saisissez vos clés d'API pour les services Gemini, OpenAI ou Groq.",
    "api_provider": "Fournisseur d'API",
    "section_cloud_ai": "IA cloud",
    "api_key": "Clé d'API",
    "api_key_placeholder": "Saisissez votre clé d'API...",
    "hotkey_prompt": "Appuyez sur les touches...",
    "key_saved_placeholder": "•••••••• (enregistré en toute sécurité)",
    "key_placeholder": "Saisissez la clé API",
    "api_get_key": "Obtenir une clé d'API",
    "history_title": "Historique de dictée",
    "history_clear": "Effacer l'historique",
    "history_desc": "Les dernières phrases dictées sont enregistrées localement.",
    "history_empty": "Historique vide. Vos textes transcrits s'afficheront ici.",
    "history_badge_cloud": "Cloud",
    "history_badge_local": "Local",
    "history_engine_whisper": "Whisper",
    "history_engine_parakeet": "Parakeet NVIDIA",
    "history_unit_ms": "ms",
    "history_unit_sec": "s",
    "about_app_title": "Dictée vocale Aura",
    "about_version": "v1.0.10",
    "about_description": "Outil de saisie vocale globale pour Windows. Le programme transcrit la parole en texte et l'insère dans n'importe quelle fenêtre active avec un formatage et une ponctuation automatiques.",
    "status_ready": "Prêt",
    "btn_save": "Enregistrer",
    "confirm_title": "Confirmation",
    "confirm_message": "Voulez-vous vraiment effectuer cette action?",
    "confirm_cancel": "Annuler",
    "confirm_ok": "Confirmer",
    "status_loading": "Chargement...",
    "status_modified": "Modifications non enregistrées",
    "status_saving": "Enregistrement...",
    "status_saved": "Paramètres enregistrés !",
    "status_error": "Erreur: ",
    "model_status_ready": "Installé",
    "model_action_download": "Télécharger",
    "model_action_delete": "Supprimer",
    "api_get_key_pattern": "Obtenir la clé sur {name}",
    "status_loaded": "Paramètres chargés",
    "status_load_error": "Échec du chargement des paramètres : ",
    "status_save_error": "Échec de l'enregistrement des paramètres : ",
    "status_streaming_degraded": "Aperçu live Parakeet désactivé — le texte final est préparé par lot",
    "model_downloading_pattern": "Démarrage du téléchargement pour le modèle '{model}'...",
    "model_download_error_pattern": "Erreur de téléchargement: {err}",
    "delete_model_title": "Supprimer le modèle",
    "delete_model_confirm_pattern": "Voulez-vous vraiment supprimer le modèle local '{model}' ?",
    "delete_model_btn": "Supprimer",
    "model_deleting_pattern": "Suppression du modèle '{model}'...",
    "model_deleted_success": "Modèle supprimé avec succès",
    "model_delete_error_pattern": "Erreur de suppression: {err}",
    "model_downloaded_success_pattern": "Modèle '{model}' téléchargé !",
    "confirm_clear_history_title": "Effacer l'historique",
    "confirm_clear_history_msg": "Voulez-vous vraiment effacer tout l'historique des transcriptions ?",
    "general_ui_lang_title": "Langue de l'interface",
    "general_ui_lang_desc": "Sélectionnez la langue pour les paramètres et les notifications de l'application.",
    "update_checks_title": "Recherche de mises à jour",
    "update_checks_desc": "Aura contacte GitHub uniquement lors d’une vérification manuelle ou si vous activez les vérifications automatiques.",
    "update_checks_checkbox": "Rechercher automatiquement les mises à jour au démarrage",
    "update_check_now": "Rechercher les mises à jour",
    "update_current": "Aura est à jour.",
    "update_available_pattern": "Aura v{version} est disponible.",
    "update_check_error_pattern": "Impossible de rechercher les mises à jour : {error}",
    "update_installing": "Téléchargement, vérification de la signature et installation de la mise à jour…",
    "update_installed_restarting": "Mise à jour installée. Redémarrage…",
    "update_install_error_open_release": "Impossible d’installer la mise à jour. Ouverture de la page de la version…",
    "gpu_accel_label": "Accélération matérielle locale",
    "gpu_accel_cpu_title": "CPU (sans accélération)",
    "gpu_accel_cpu_desc": "Mode standard. Fiable, mais sollicite le processeur.",
    "gpu_accel_cuda_title": "NVIDIA CUDA (vitesse maximale)",
    "gpu_accel_cuda_desc": "Pour les GPU GeForce RTX/GTX. Utilise les Tensor Cores.",
    "cuda_license_title": "Conditions NVIDIA CUDA",
    "cuda_license_message": "Aura téléchargera jusqu'à 1,52 Gio d'archives d'accélération et installera jusqu'à 2,33 Gio de fichiers. Les composants NVIDIA compatibles déjà installés seront réutilisés ; sinon, Aura téléchargera les composants propriétaires CUDA et cuDNN directement depuis NVIDIA selon ses conditions.",
    "cuda_license_footnote": "Licences :",
    "cuda_license_cuda_link": "Licence CUDA Toolkit",
    "cuda_license_cudnn_link": "Licence cuDNN",
    "cuda_license_accept": "Télécharger",
    "gpu_accel_dml_title": "DirectML (universel)",
    "gpu_accel_dml_desc": "Pour les GPU AMD, Intel et NVIDIA. Accélération de base.",
    "hotkey_reset_title": "Réinitialiser à Alt+V",
    "local_engine_label": "Moteur de reconnaissance",
    "local_engine_whisper": "Whisper.cpp",
    "local_engine_parakeet": "NVIDIA Parakeet",
    "parakeet_model_label": "Modèle Parakeet",
    "model_meta_parakeet": "~670 Mo — optimisé par NVIDIA",
    "model_cancel_download": "Annuler le téléchargement",
    "model_download_cancelled": "Téléchargement annulé",
    "update_available": "Mise à jour disponible",
    "fallback_title": "Basculement automatique si le cloud est indisponible",
    "fallback_desc": "Si l'IA cloud est indisponible (VPN, blocage régional, pas de réseau), utiliser automatiquement le modèle local déjà téléchargé pour cet enregistrement.",
    "fallback_checkbox": "Activer le basculement automatique vers le modèle local",
    "copy_context_title": "Modifier le texte sélectionné",
    "copy_context_desc": "Lorsque cette option est activée, Aura envoie Ctrl+C et transmet le texte sélectionné au fournisseur cloud choisi comme contexte d’une commande de modification. Désactivez-la lorsque vous travaillez dans un terminal.",
    "copy_context_checkbox": "Autoriser la capture de la sélection et la modification dans le cloud",
    "btn_copy_diagnostics": "Copier le rapport de diagnostic",
    "toast_diagnostics_copied": "Rapport de diagnostic copié dans le presse-papiers !",
    "diag_speech_text_title": "Consigner le texte vocal (Mode développeur)",
    "diag_title": "Diagnostic",
    "diag_speech_text_desc": "Inclure le texte vocal transcrit exact dans les journaux de diagnostic. Désactivé par défaut par confidentialité.",
    "diag_speech_text_checkbox": "Inclure le texte vocal dans les journaux",
    "api_custom_url": "Adresse API du serveur",
    "api_custom_model": "Nom du modèle",
    "provider_opt_custom": "Serveur personnalisé",
    "overlay_timer_title": "Minuteur et statut de l'overlay",
    "overlay_timer_desc": "Afficher le compteur de temps et l'état du traitement sous l'indicateur vocal. Les erreurs sont toujours affichées.",
    "overlay_timer_checkbox": "Afficher le minuteur et l'état sur l'overlay",
    "mic_meter_title": "Vérification du microphone",
    "mic_meter_desc": "Testez le niveau du signal d'entrée et la détection vocale en temps réel.",
    "audio_device_label": "Microphone (périphérique d'entrée)",
    "audio_device_default": "Par défaut (système)",
    "audio_device_desc": "Sélectionnez le microphone physique pour l'enregistrement vocal.",
    "mic_meter_test_btn_start": "Démarrer le test",
    "mic_meter_test_btn_stop": "Arrêter le test",
    "mic_meter_speech": "Voix",
    "mic_meter_silence": "Silence",
    "mic_start_error": "Erreur de démarrage du microphone : ",
    "history_search_placeholder": "Rechercher dans l'historique...",
    "history_filter_all": "Tous",
    "history_filter_cloud": "Nuage",
    "history_filter_local": "Local",
    "gpu_status_installing": "Installation..."
  },
  "it": {
    "title_settings": "Impostazioni",
    "tab_general": "Generale",
    "tab_speech": "Dettatura",
    "tab_hotkeys": "Scorciatoie",
    "section_engine": "Motore di riconoscimento",
    "section_recognition": "Riconoscimento",
    "section_input": "Digitazione",
    "section_dictionary": "Dizionario",
    "tab_history": "Cronologia",
    "tab_about": "Informazioni",
    "general_autostart_title": "Avvio automatico",
    "general_autostart_desc": "Avvia l'app automaticamente all'accesso di Windows.",
    "general_autostart_checkbox": "Avvia Aura con il sistema",
    "engine_title": "Tipo di elaborazione",
    "engine_desc": "Scegli tra l'elaborazione cloud di alta qualità o il riconoscimento locale offline.",
    "engine_cloud": "IA Cloud",
    "engine_cloud_meta": "Gemini / OpenAI / Groq (chiave API richiesta)",
    "engine_local": "IA Locale",
    "engine_local_meta": "Whisper / Parakeet (100% offline e privato)",
    "lang_bias_title": "Lingua dettatura",
    "lang_bias_desc": "Imposta una lingua fissa per la transrizione o usa il rilevamento automatico.",
    "lang_bias_label": "Seleziona lingua",
    "lang_opt_auto": "Rilevamento automatico",
    "lang_opt_layout": "In base alla tastiera",
    "streaming_title": "Dattilografia a scorrimento",
    "streaming_desc": "Seleziona come visualizzare il testo trascritto.",
    "streaming_checkbox": "Inserimento del testo in tempo reale (sperimentale)",
    "streaming_subdesc": "Se disattivato: il testo viene inserito interamente solo quando rilasci la scorciatoia.",
    "vocab_title": "Vocabolario personalizzato",
    "vocab_desc": "Aggiungi parole specifiche, nomi o acronimi separati da virgole per migliorare la precisione.",
    "vocab_placeholder": "es. Aura, commit, repository...",
    "engine_health_parakeet_running": "Parakeet: server in esecuzione ({provider}, porta {port})",
    "engine_health_parakeet_stopped": "Parakeet: server non in esecuzione",
    "engine_health_whisper_running": "Whisper: server in esecuzione ({provider}, porta {port})",
    "engine_health_whisper_stopped": "Whisper: server non in esecuzione",
    "engine_starting": "Avvio del motore…",
    "local_model_title": "Modello Whisper locale",
    "local_model_desc": "Configura un motore locale di riconoscimento vocale per la massima privacy.",
    "local_model_label": "Dimensione modello",
    "model_meta_tiny": "~75 MB — superveloce",
    "model_meta_base": "~145 MB — consigliato",
    "model_meta_small": "~465 MB — preciso",
    "model_meta_medium": "~1.5 GB — avanzato",
    "model_meta_turbo": "~1.6 GB — massima precisione RU/EN",
    "model_meta_turbo_q5": "~550 MB — quasi come Turbo, metà del peso",
    "hotkey_title": "Tasto di scelta rapida",
    "hotkey_desc": "Tieni premuto il tasto per registrare, rilascelo per trascrivere.",
    "hotkey_label": "Scorciatoia",
    "hotkey_toggle_mode": "Modalità alternata (tocco breve)",
    "hotkey_toggle_mode_desc": "Un tocco breve avvia/ferma la registrazione senza tenere premuto.",
    "sound_title": "Feedback sonori",
    "sound_desc": "Effetti acustici dell'overlay durante la registrazione.",
    "sound_enable": "Attiva i suoni dell'overlay",
    "sound_volume_label": "Volume del suono",
    "sound_theme_label": "Tema sonoro",
    "sound_theme_zen": "Zen",
    "sound_theme_rhodes": "Rhodes",
    "sound_theme_scifi": "Sci-Fi",
    "sound_theme_classic": "Campanella",
    "api_title": "Autorizzazione chiavi API",
    "api_desc": "Inserisci le tue chiavi API per Gemini, OpenAI o Groq.",
    "api_provider": "Provider API",
    "section_cloud_ai": "IA nel cloud",
    "api_key": "Chiave API",
    "api_key_placeholder": "Inserisci la tua chiave API...",
    "hotkey_prompt": "Premi i tasti...",
    "key_saved_placeholder": "•••••••• (salvato in modo sicuro)",
    "key_placeholder": "Inserisci la chiave API",
    "api_get_key": "Ottieni chiave API",
    "history_title": "Cronologia dettati",
    "history_clear": "Cancella cronologia",
    "history_desc": "Le ultime frasi dettate vengono salvate in locale.",
    "history_empty": "La cronologia è vuota. I testi dettati appariranno qui.",
    "history_badge_cloud": "Cloud",
    "history_badge_local": "Locale",
    "history_engine_whisper": "Whisper",
    "history_engine_parakeet": "Parakeet NVIDIA",
    "history_unit_ms": "ms",
    "history_unit_sec": "s",
    "about_app_title": "Dettatura vocale Aura",
    "about_version": "v1.0.10",
    "about_description": "Strumento di inserimento vocale globale per Windows. Il programma trascrive la voce in testo e la inserisce in qualsiasi finestra attiva con formattazione e punteggiatura automatiche.",
    "status_ready": "Pronto",
    "btn_save": "Salva impostazioni",
    "confirm_title": "Conferma",
    "confirm_message": "Sei sicuro di voler procedere?",
    "confirm_cancel": "Annulla",
    "confirm_ok": "Conferma",
    "status_loading": "Caricamento...",
    "status_modified": "Impostazioni modificate (non salvate)",
    "status_saving": "Salvataggio...",
    "status_saved": "Impostazioni salvate con successo!",
    "status_error": "Errore: ",
    "model_status_ready": "Installato",
    "model_action_download": "Scarica",
    "model_action_delete": "Elimina",
    "api_get_key_pattern": "Ottieni la chiave su {name}",
    "status_loaded": "Impostazioni caricate",
    "status_load_error": "Impossibile caricare le impostazioni: ",
    "status_save_error": "Impossibile salvare le impostazioni: ",
    "status_streaming_degraded": "Anteprima live di Parakeet disattivata — il testo finale viene elaborato in batch",
    "model_downloading_pattern": "Avvio del download per il modello '{model}'...",
    "model_download_error_pattern": "Errore di download: {err}",
    "delete_model_title": "Elimina modello",
    "delete_model_confirm_pattern": "Sei sicuro di voler eliminare il modello locale '{model}'?",
    "delete_model_btn": "Elimina",
    "model_deleting_pattern": "Eliminazione del modello '{model}'...",
    "model_deleted_success": "Modello eliminato con successo",
    "model_delete_error_pattern": "Errore di eliminazione: {err}",
    "model_downloaded_success_pattern": "Modello '{model}' scaricato!",
    "confirm_clear_history_title": "Cancella cronologia",
    "confirm_clear_history_msg": "Sei sicuro di voler cancellare tutta la cronologia delle trascrizioni?",
    "general_ui_lang_title": "Lingua dell'interfaccia",
    "general_ui_lang_desc": "Seleziona la lingua per le impostazioni e le notifiche dell'applicazione.",
    "update_checks_title": "Controllo aggiornamenti",
    "update_checks_desc": "Aura contatta GitHub solo durante un controllo manuale o se abiliti i controlli automatici.",
    "update_checks_checkbox": "Controlla automaticamente gli aggiornamenti all’avvio",
    "update_check_now": "Controlla aggiornamenti",
    "update_current": "Aura è aggiornata.",
    "update_available_pattern": "È disponibile Aura v{version}.",
    "update_check_error_pattern": "Impossibile verificare gli aggiornamenti: {error}",
    "update_installing": "Download, verifica della firma e installazione dell’aggiornamento...",
    "update_installed_restarting": "Aggiornamento installato. Riavvio...",
    "update_install_error_open_release": "Impossibile installare l’aggiornamento. Apertura della pagina della versione...",
    "gpu_accel_label": "Accelerazione hardware locale",
    "gpu_accel_cpu_title": "CPU (senza accelerazione)",
    "gpu_accel_cpu_desc": "Modalità standard. Affidabile, ma utilizza maggiormente la CPU.",
    "gpu_accel_cuda_title": "NVIDIA CUDA (velocità massima)",
    "gpu_accel_cuda_desc": "Per GPU GeForce RTX/GTX. Usa i Tensor Core.",
    "cuda_license_title": "Condizioni NVIDIA CUDA",
    "cuda_license_message": "Aura scaricherà fino a 1,52 GiB di archivi di accelerazione e installerà fino a 2,33 GiB di file. I componenti NVIDIA compatibili già installati verranno riutilizzati; altrimenti Aura scaricherà componenti CUDA e cuDNN proprietari direttamente da NVIDIA secondo i relativi termini.",
    "cuda_license_footnote": "Licenze:",
    "cuda_license_cuda_link": "Licenza CUDA Toolkit",
    "cuda_license_cudnn_link": "Licenza cuDNN",
    "cuda_license_accept": "Scarica",
    "gpu_accel_dml_title": "DirectML (universale)",
    "gpu_accel_dml_desc": "Per GPU AMD, Intel e NVIDIA. Accelerazione di base.",
    "hotkey_reset_title": "Ripristina ad Alt+V",
    "local_engine_label": "Motore di riconoscimento",
    "local_engine_whisper": "Whisper.cpp",
    "local_engine_parakeet": "NVIDIA Parakeet",
    "parakeet_model_label": "Modello Parakeet",
    "model_meta_parakeet": "~670 MB — ottimizzato da NVIDIA",
    "model_cancel_download": "Annulla download",
    "model_download_cancelled": "Download annullato",
    "update_available": "Aggiornamento disponibile",
    "fallback_title": "Passaggio automatico quando il cloud non è disponibile",
    "fallback_desc": "Se l'IA cloud non è disponibile (VPN, blocco regionale, nessuna rete), utilizza automaticamente il modello locale già scaricato per questa registrazione.",
    "fallback_checkbox": "Attiva il fallback automatico al modello locale",
    "copy_context_title": "Modifica testo selezionato",
    "copy_context_desc": "Quando è abilitata, Aura invia Ctrl+C e passa il testo selezionato al provider cloud scelto come contesto per un comando di modifica. Disattiva questa funzione quando lavori in un terminale.",
    "copy_context_checkbox": "Consenti acquisizione della selezione e modifica nel cloud",
    "btn_copy_diagnostics": "Copia rapporto diagnostico",
    "toast_diagnostics_copied": "Rapporto diagnostico copiato negli appunti!",
    "diag_speech_text_title": "Registra testo vocale (Modalità sviluppatore)",
    "diag_title": "Diagnostica",
    "diag_speech_text_desc": "Include il testo vocale trascritto esatto nei log di diagnostica. Disattivato di default per la privacy.",
    "diag_speech_text_checkbox": "Includi testo vocale nei log",
    "api_custom_url": "Indirizzo API del server",
    "api_custom_model": "Nome del modello",
    "provider_opt_custom": "Server personalizzato",
    "overlay_timer_title": "Timer e stato dell'overlay",
    "overlay_timer_desc": "Mostra il timer di durata e lo stato di elaborazione sotto l'indicatore vocale. Gli errori vengono sempre visualizzati.",
    "overlay_timer_checkbox": "Mostra timer e stato nell'overlay",
    "mic_meter_title": "Controllo microfono",
    "mic_meter_desc": "Testa il livello del segnale di ingresso e il rilevamento vocale in tempo reale.",
    "audio_device_label": "Microfono (dispositivo di input)",
    "audio_device_default": "Predefinito (sistema)",
    "audio_device_desc": "Seleziona il microfono fisico per la registrazione vocale.",
    "mic_meter_test_btn_start": "Avvia test",
    "mic_meter_test_btn_stop": "Arresta test",
    "mic_meter_speech": "Voce",
    "mic_meter_silence": "Silenzio",
    "mic_start_error": "Errore di avvio del microfono: ",
    "history_search_placeholder": "Cerca nella cronologia...",
    "history_filter_all": "Tutti",
    "history_filter_cloud": "Cloud",
    "history_filter_local": "Locale",
    "gpu_status_installing": "Installazione..."
  },
  "zh": {
    "title_settings": "设置",
    "tab_general": "常规",
    "tab_speech": "语音",
    "tab_hotkeys": "快捷键",
    "section_engine": "识别引擎",
    "section_recognition": "识别",
    "section_input": "输入",
    "section_dictionary": "词典",
    "tab_history": "历史记录",
    "tab_about": "关于我们",
    "general_autostart_title": "自启动设置",
    "general_autostart_desc": "在Windows启动时自动运行此应用程序。",
    "general_autostart_checkbox": "系统启动时运行 Aura",
    "engine_title": "处理类型",
    "engine_desc": "选择高品质云端识别，或完全离线的本地语音识别。",
    "engine_cloud": "云端智能 AI",
    "engine_cloud_meta": "Gemini / OpenAI / Groq (需要 API 密钥)",
    "engine_local": "本地 AI (离线)",
    "engine_local_meta": "Whisper / Parakeet (100% 离线和私密)",
    "lang_bias_title": "识别语言",
    "lang_bias_desc": "强制设定特定的听写语言，或使用自动检测。",
    "lang_bias_label": "选择语言",
    "lang_opt_auto": "自动检测",
    "lang_opt_layout": "遵循当前键盘布局",
    "streaming_title": "输入模式",
    "streaming_desc": "选择转换后文本的录入方式。",
    "streaming_checkbox": "实时流式文本录入 (实验性)",
    "streaming_subdesc": "如果关闭: 只有松开按键后，文字才会一次性录入。",
    "vocab_title": "自定义词典",
    "vocab_desc": "以逗号分隔输入专用术语、人名或品牌，以便提高识别精度。",
    "vocab_placeholder": "例如：Aura, commit, 仓库...",
    "engine_health_parakeet_running": "Parakeet：服务器运行中（{provider}，端口 {port}）",
    "engine_health_parakeet_stopped": "Parakeet：服务器未运行",
    "engine_health_whisper_running": "Whisper：服务器运行中（{provider}，端口 {port}）",
    "engine_health_whisper_stopped": "Whisper：服务器未运行",
    "engine_starting": "引擎启动中…",
    "local_model_title": "本地 Whisper 模型",
    "local_model_desc": "配置本地语音识别引擎，确保数据完全私密。",
    "local_model_label": "模型大小",
    "model_meta_tiny": "~75 MB — 超快速",
    "model_meta_base": "~145 MB — 推荐",
    "model_meta_small": "~465 MB — 精准",
    "model_meta_medium": "~1.5 GB — 高级",
    "model_meta_turbo": "~1.6 GB — RU/EN 最佳精度",
    "model_meta_turbo_q5": "~550 MB — 接近 Turbo，体积减半",
    "hotkey_title": "全局快捷键",
    "hotkey_desc": "按住选择的组合键开始录音，松开即可完成转文字并录入。",
    "hotkey_label": "组合按键",
    "hotkey_toggle_mode": "触发模式 (短按切换)",
    "hotkey_toggle_mode_desc": "短按启动/停止录音，无需一直按住按键。",
    "sound_title": "声音反馈",
    "sound_desc": "录音状态切换时播放提示音。",
    "sound_enable": "启用悬浮条声音反馈",
    "sound_volume_label": "音量",
    "sound_theme_label": "声音主题",
    "sound_theme_zen": "禅宗",
    "sound_theme_rhodes": "Rhodes",
    "sound_theme_scifi": "科幻",
    "sound_theme_classic": "铃声",
    "api_title": "API 密钥授权",
    "api_desc": "输入您在 Gemini、OpenAI 或 Groq 云端服务的 API 密钥。",
    "api_provider": "API 供应商",
    "section_cloud_ai": "云端 AI",
    "api_key": "API 密钥",
    "api_key_placeholder": "在此输入您的 API 密钥...",
    "hotkey_prompt": "按键...",
    "key_saved_placeholder": "•••••••• (已安全保存)",
    "key_placeholder": "输入 API 密钥",
    "api_get_key": "获取 API 密钥",
    "history_title": "听写历史记录",
    "history_clear": "清空历史",
    "history_desc": "您最近转换出的文字将缓存在本地。",
    "history_empty": "历史记录为空。您听写的文字会显示在这里。",
    "history_badge_cloud": "云端",
    "history_badge_local": "本地",
    "history_engine_whisper": "Whisper",
    "history_engine_parakeet": "NVIDIA Parakeet",
    "history_unit_ms": "毫秒",
    "history_unit_sec": "秒",
    "about_app_title": "Aura 智能语音输入",
    "about_version": "v1.0.10",
    "about_description": "适用于 Windows 的全局语音输入工具。本程序可以将语音转录为文本，并以自动格式和标点符号插入到任何活动窗口中。",
    "status_ready": "就绪",
    "btn_save": "保存设置",
    "confirm_title": "确认",
    "confirm_message": "您确定要执行此操作吗？",
    "confirm_cancel": "取消",
    "confirm_ok": "确认",
    "status_loading": "正在加载设置...",
    "status_modified": "设置已更改 (未保存)",
    "status_saving": "正在保存设置...",
    "status_saved": "设置保存成功！",
    "status_error": "发生错误: ",
    "model_status_ready": "已安装",
    "model_action_download": "下载",
    "model_action_delete": "删除",
    "api_get_key_pattern": "在 {name} 获取密钥",
    "status_loaded": "设置已加载",
    "status_load_error": "加载设置失败: ",
    "status_save_error": "保存设置失败: ",
    "status_streaming_degraded": "Parakeet 实时预览已停用——最终文本将批量生成",
    "model_downloading_pattern": "正在启动模型 '{model}' 的下载...",
    "model_download_error_pattern": "下载错误: {err}",
    "delete_model_title": "删除模型",
    "delete_model_confirm_pattern": "您确定要删除本地模型 '{model}' 吗？",
    "delete_model_btn": "删除",
    "model_deleting_pattern": "正在删除模型 '{model}'...",
    "model_deleted_success": "模型删除成功",
    "model_delete_error_pattern": "删除错误: {err}",
    "model_downloaded_success_pattern": "模型 '{model}' 已下载！",
    "confirm_clear_history_title": "清空历史",
    "confirm_clear_history_msg": "您确定要清空所有听写历史记录吗？",
    "general_ui_lang_title": "界面语言",
    "general_ui_lang_desc": "选择设置和应用程序通知的语言。",
    "update_checks_title": "更新检查",
    "update_checks_desc": "Aura 仅在您手动检查或启用自动检查时连接 GitHub。",
    "update_checks_checkbox": "启动时自动检查更新",
    "update_check_now": "检查更新",
    "update_current": "Aura 已是最新版本。",
    "update_available_pattern": "Aura v{version} 可用。",
    "update_check_error_pattern": "无法检查更新：{error}",
    "update_installing": "正在下载、验证签名并安装更新...",
    "update_installed_restarting": "更新已安装。正在重启...",
    "update_install_error_open_release": "无法安装更新。正在打开发布页面...",
    "gpu_accel_label": "本地硬件加速",
    "gpu_accel_cpu_title": "CPU（无加速）",
    "gpu_accel_cpu_desc": "标准模式。稳定可靠，但会占用更多处理器资源。",
    "gpu_accel_cuda_title": "NVIDIA CUDA（最高速度）",
    "gpu_accel_cuda_desc": "适用于 GeForce RTX/GTX 显卡。使用 Tensor Core。",
    "cuda_license_title": "NVIDIA CUDA 条款",
    "cuda_license_message": "Aura 将下载最多 1.52 GiB 的加速组件包，并安装最多 2.33 GiB 的文件。系统中已有的兼容 NVIDIA 组件将被重复使用；否则 Aura 将根据 NVIDIA 条款直接下载其专有 CUDA 和 cuDNN 组件。",
    "cuda_license_footnote": "许可协议：",
    "cuda_license_cuda_link": "CUDA Toolkit 许可证",
    "cuda_license_cudnn_link": "cuDNN 许可证",
    "cuda_license_accept": "下载",
    "gpu_accel_dml_title": "DirectML（通用）",
    "gpu_accel_dml_desc": "适用于 AMD、Intel 和 NVIDIA 显卡。基础加速。",
    "hotkey_reset_title": "重置为 Alt+V",
    "local_engine_label": "识别引擎",
    "local_engine_whisper": "Whisper.cpp",
    "local_engine_parakeet": "NVIDIA Parakeet",
    "parakeet_model_label": "Parakeet 模型",
    "model_meta_parakeet": "~670 MB — NVIDIA 优化",
    "model_cancel_download": "取消下载",
    "model_download_cancelled": "下载已取消",
    "update_available": "有可用更新",
    "fallback_title": "云端不可用时自动切换",
    "fallback_desc": "当云端 AI 不可用时（VPN、地区限制、无网络），自动使用已下载的本地模型进行本次录音识别。",
    "fallback_checkbox": "启用自动回退至本地模型",
    "copy_context_title": "编辑选中文本",
    "copy_context_desc": "启用后，Aura 会发送 Ctrl+C，并将选中文本作为编辑指令的上下文传给所选云服务提供商。在终端中工作时请关闭此功能。",
    "copy_context_checkbox": "允许捕获选区并在云端编辑",
    "btn_copy_diagnostics": "复制诊断报告",
    "toast_diagnostics_copied": "诊断报告已复制到剪贴板！",
    "diag_speech_text_title": "记录语音文本（开发者模式）",
    "diag_title": "诊断",
    "diag_speech_text_desc": "在诊断日志中包含精确的语音转写文本。出于隐私原因默认禁用。",
    "diag_speech_text_checkbox": "在日志中包含语音文本",
    "api_custom_url": "服务器 API 地址",
    "api_custom_model": "模型名称",
    "provider_opt_custom": "自定义服务器",
    "overlay_timer_title": "悬浮窗计时器与状态",
    "overlay_timer_desc": "在语音指示器下方显示录音计时器和处理状态。错误信息将始终显示。",
    "overlay_timer_checkbox": "在悬浮窗上显示计时器和状态",
    "mic_meter_title": "麦克风检测",
    "mic_meter_desc": "实时测试输入信号强度和语音检测。",
    "audio_device_label": "麦克风（输入设备）",
    "audio_device_default": "默认（系统）",
    "audio_device_desc": "选择用于语音录制的物理麦克风。",
    "mic_meter_test_btn_start": "开始测试",
    "mic_meter_test_btn_stop": "停止测试",
    "mic_meter_speech": "语音",
    "mic_meter_silence": "静音",
    "mic_start_error": "麦克风启动失败：",
    "history_search_placeholder": "搜索历史记录...",
    "history_filter_all": "全部",
    "history_filter_cloud": "云端",
    "history_filter_local": "本地",
    "gpu_status_installing": "正在安装..."
  },
  "pt": {
    "title_settings": "Configurações",
    "tab_general": "Geral",
    "tab_speech": "Voz",
    "tab_hotkeys": "Teclas de atalho",
    "section_engine": "Motor de reconhecimento",
    "section_recognition": "Reconhecimento",
    "section_input": "Entrada",
    "section_dictionary": "Dicionário",
    "tab_history": "Histórico",
    "tab_about": "Sobre",
    "general_autostart_title": "Inicialização",
    "general_autostart_desc": "Iniciar o aplicativo automaticamente com o Windows.",
    "general_autostart_checkbox": "Iniciar o Aura com o Windows",
    "engine_title": "Tipo de processamento",
    "engine_desc": "Escolha entre processamento na nuvem de alta qualidade ou reconhecimento de voz local 100% offline.",
    "engine_cloud": "IA na Nuvem",
    "engine_cloud_meta": "Gemini / OpenAI / Groq (chave API necessária)",
    "engine_local": "IA Local",
    "engine_local_meta": "Whisper / Parakeet (100% offline e privado)",
    "lang_bias_title": "Idioma do Diktat",
    "lang_bias_desc": "Forçar um idioma específico para a transcrição ou usar detecção automática.",
    "lang_bias_label": "Selecionar idioma",
    "lang_opt_auto": "Auto-detectar",
    "lang_opt_layout": "Seguir o teclado ativo",
    "streaming_title": "Fluxo de texto",
    "streaming_desc": "Escolha o método para exibir o texto transcrito.",
    "streaming_checkbox": "Escrita em tempo real (experimental)",
    "streaming_subdesc": "Se desativado: o texto é colado inteiro apenas ao soltar o atalho.",
    "vocab_title": "Dicionário personalizado",
    "vocab_desc": "Adicione termos específicos, nomes ou siglas separados por vírgula para melhorar o reconhecimento.",
    "vocab_placeholder": "ex. Aura, commit, repositório...",
    "engine_health_parakeet_running": "Parakeet: servidor em execução ({provider}, porta {port})",
    "engine_health_parakeet_stopped": "Parakeet: servidor não em execução",
    "engine_health_whisper_running": "Whisper: servidor em execução ({provider}, porta {port})",
    "engine_health_whisper_stopped": "Whisper: servidor não em execução",
    "engine_starting": "Iniciando o motor…",
    "local_model_title": "Modelo Whisper local",
    "local_model_desc": "Configure um mecanismo local de reconhecimento de voz para manter total privacidade.",
    "local_model_label": "Tamanho do modelo",
    "model_meta_tiny": "~75 MB — super-rápido",
    "model_meta_base": "~145 MB — recomendado",
    "model_meta_small": "~465 MB — preciso",
    "model_meta_medium": "~1.5 GB — avançado",
    "model_meta_turbo": "~1.6 GB — melhor precisão RU/EN",
    "model_meta_turbo_q5": "~550 MB — quase como Turbo, metade do tamanho",
    "hotkey_title": "Teclas globais",
    "hotkey_desc": "Segure o atalho para gravar, solte para transcrever.",
    "hotkey_label": "Combinação",
    "hotkey_toggle_mode": "Modo alternar (toque rápido)",
    "hotkey_toggle_mode_desc": "Um toque rápido inicia/para a gravação sem segurar o botão.",
    "sound_title": "Feedback sonoro",
    "sound_desc": "Efeitos sonoros do overlay ao gravar.",
    "sound_enable": "Habilitar sons do overlay",
    "sound_volume_label": "Volume do som",
    "sound_theme_label": "Tema sonoro",
    "sound_theme_zen": "Zen",
    "sound_theme_rhodes": "Rhodes",
    "sound_theme_scifi": "Sci-Fi",
    "sound_theme_classic": "Sino",
    "api_title": "Autorização de chaves API",
    "api_desc": "Insira suas chaves API para os serviços Gemini, OpenAI ou Groq.",
    "api_provider": "Provedor de API",
    "section_cloud_ai": "IA na nuvem",
    "api_key": "Chave API",
    "api_key_placeholder": "Insira sua chave API...",
    "hotkey_prompt": "Pressione as teclas...",
    "key_saved_placeholder": "•••••••• (salvo com segurança)",
    "key_placeholder": "Insira a chave da API",
    "api_get_key": "Obter chave API",
    "history_title": "Histórico de transcrição",
    "history_clear": "Limpar histórico",
    "history_desc": "As últimas frases ditadas são armazenadas localmente.",
    "history_empty": "O histórico está vazio. Seus textos ditados aparecerão aqui.",
    "history_badge_cloud": "Nuvem",
    "history_badge_local": "Local",
    "history_engine_whisper": "Whisper",
    "history_engine_parakeet": "Parakeet NVIDIA",
    "history_unit_ms": "ms",
    "history_unit_sec": "s",
    "about_app_title": "Ditado de voz Aura",
    "about_version": "v1.0.10",
    "about_description": "Ferramenta de entrada de voz global para Windows. O programa transcreve a fala em texto e a insere em qualquer janela ativa com formatação e pontuação automáticas.",
    "status_ready": "Pronto",
    "btn_save": "Salvar configurações",
    "confirm_title": "Confirmação",
    "confirm_message": "Tem certeza de que deseja executar esta ação?",
    "confirm_cancel": "Cancelar",
    "confirm_ok": "Confirmar",
    "status_loading": "Carregando...",
    "status_modified": "Configurações alteradas (não salvas)",
    "status_saving": "Salvando...",
    "status_saved": "Configurações salvas com sucesso!",
    "status_error": "Erro: ",
    "model_status_ready": "Instalado",
    "model_action_download": "Baixar",
    "model_action_delete": "Excluir",
    "api_get_key_pattern": "Obter chave em {name}",
    "status_loaded": "Configurações carregadas",
    "status_load_error": "Falha ao carregar configurações: ",
    "status_save_error": "Falha ao salvar configurações: ",
    "status_streaming_degraded": "Pré-visualização ao vivo do Parakeet desativada — o texto final será gerado em lote",
    "model_downloading_pattern": "Iniciando download para o modelo '{model}'...",
    "model_download_error_pattern": "Erro de download: {err}",
    "delete_model_title": "Excluir modelo",
    "delete_model_confirm_pattern": "Tem certeza de que deseja excluir o modelo local '{model}'?",
    "delete_model_btn": "Excluir",
    "model_deleting_pattern": "Excluindo modelo '{model}'...",
    "model_deleted_success": "Modelo excluído com sucesso",
    "model_delete_error_pattern": "Erro ao excluir: {err}",
    "model_downloaded_success_pattern": "Modelo '{model}' baixado!",
    "confirm_clear_history_title": "Limpar histórico",
    "confirm_clear_history_msg": "Tem certeza de que deseja limpar todo o histórico de transcrições?",
    "general_ui_lang_title": "Idioma da interface",
    "general_ui_lang_desc": "Selecione o idioma para as configurações e notificações do aplicativo.",
    "update_checks_title": "Verificação de atualizações",
    "update_checks_desc": "A Aura só acessa o GitHub quando você verifica manualmente ou ativa as verificações automáticas.",
    "update_checks_checkbox": "Verificar atualizações automaticamente ao iniciar",
    "update_check_now": "Verificar atualizações",
    "update_current": "A Aura está atualizada.",
    "update_available_pattern": "A versão {version} da Aura está disponível.",
    "update_check_error_pattern": "Não foi possível verificar atualizações: {error}",
    "update_installing": "Baixando, verificando a assinatura e instalando a atualização...",
    "update_installed_restarting": "Atualização instalada. Reiniciando...",
    "update_install_error_open_release": "Não foi possível instalar a atualização. Abrindo a página da versão...",
    "gpu_accel_label": "Aceleração de hardware local",
    "gpu_accel_cpu_title": "CPU (sem aceleração)",
    "gpu_accel_cpu_desc": "Modo padrão. Seguro, mas exige mais do processador.",
    "gpu_accel_cuda_title": "NVIDIA CUDA (velocidade máxima)",
    "gpu_accel_cuda_desc": "Para GPUs GeForce RTX/GTX. Usa Tensor Cores.",
    "cuda_license_title": "Termos do NVIDIA CUDA",
    "cuda_license_message": "O Aura baixará até 1,52 GiB de arquivos de aceleração e instalará até 2,33 GiB de arquivos. Componentes NVIDIA compatíveis já instalados serão reutilizados; caso contrário, o Aura baixará componentes CUDA e cuDNN proprietários diretamente da NVIDIA sob os termos da NVIDIA.",
    "cuda_license_footnote": "Licenças:",
    "cuda_license_cuda_link": "Licença do CUDA Toolkit",
    "cuda_license_cudnn_link": "Licença do cuDNN",
    "cuda_license_accept": "Baixar",
    "gpu_accel_dml_title": "DirectML (universal)",
    "gpu_accel_dml_desc": "Para GPUs AMD, Intel e NVIDIA. Aceleração básica.",
    "hotkey_reset_title": "Redefinir para Alt+V",
    "local_engine_label": "Motor de reconhecimento",
    "local_engine_whisper": "Whisper.cpp",
    "local_engine_parakeet": "NVIDIA Parakeet",
    "parakeet_model_label": "Modelo Parakeet",
    "model_meta_parakeet": "~670 MB — otimizado pela NVIDIA",
    "model_cancel_download": "Cancelar download",
    "model_download_cancelled": "Download cancelado",
    "update_available": "Atualização disponível",
    "fallback_title": "Alternância automática quando a nuvem estiver indisponível",
    "fallback_desc": "Se a IA na nuvem estiver indisponível (VPN, bloqueio regional, sem rede), usar automaticamente o modelo local já baixado para esta gravação.",
    "fallback_checkbox": "Ativar fallback automático para modelo local",
    "copy_context_title": "Editar texto selecionado",
    "copy_context_desc": "Quando ativado, o Aura envia Ctrl+C e encaminha o texto selecionado ao provedor de nuvem escolhido como contexto para um comando de edição. Desative este recurso ao trabalhar em um terminal.",
    "copy_context_checkbox": "Permitir captura da seleção e edição na nuvem",
    "btn_copy_diagnostics": "Copiar relatório de diagnóstico",
    "toast_diagnostics_copied": "Relatório de diagnóstico copiado para a área de transferência!",
    "diag_speech_text_title": "Registrar texto de voz (Modo desenvolvedor)",
    "diag_title": "Diagnóstico",
    "diag_speech_text_desc": "Incluir texto de voz transcrito exato nos logs de diagnóstico. Desativado por padrão por privacidade.",
    "diag_speech_text_checkbox": "Incluir texto de voz nos logs",
    "api_custom_url": "Endereço da API do servidor",
    "api_custom_model": "Nome do modelo",
    "provider_opt_custom": "Servidor personalizado",
    "overlay_timer_title": "Temporizador e status do overlay",
    "overlay_timer_desc": "Exibir contador de tempo e status de processamento abaixo do indicador de voz. Erros são sempre exibidos.",
    "overlay_timer_checkbox": "Exibir temporizador e status no overlay",
    "mic_meter_title": "Verificação de microfone",
    "mic_meter_desc": "Teste o nível do sinal de entrada e a detecção de fala em tempo real.",
    "audio_device_label": "Microfone (dispositivo de entrada)",
    "audio_device_default": "Padrão (sistema)",
    "audio_device_desc": "Selecione o microfone físico para gravação de voz.",
    "mic_meter_test_btn_start": "Iniciar teste",
    "mic_meter_test_btn_stop": "Parar teste",
    "mic_meter_speech": "Voz",
    "mic_meter_silence": "Silêncio",
    "mic_start_error": "Erro ao iniciar o microfone: ",
    "history_search_placeholder": "Pesquisar no histórico...",
    "history_filter_all": "Todos",
    "history_filter_cloud": "Nuvem",
    "history_filter_local": "Local",
    "gpu_status_installing": "Instalando..."
  },
  "tr": {
    "title_settings": "Ayarlar",
    "tab_general": "Genel",
    "tab_speech": "Ses",
    "tab_hotkeys": "Kısayollar",
    "section_engine": "Tanıma motoru",
    "section_recognition": "Tanıma",
    "section_input": "Giriş",
    "section_dictionary": "Sözlük",
    "tab_history": "Geçmiş",
    "tab_about": "Hakkında",
    "general_autostart_title": "Başlangıçta Çalıştır",
    "general_autostart_desc": "Windows açıldığında uygulamayı otomatik olarak başlat.",
    "general_autostart_checkbox": "Aura'yı sistem açılışında başlat",
    "engine_title": "İşlem Türü",
    "engine_desc": "Yüksek kaliteli bulut işleme veya tamamen çevrimdışı yerel konuşma tanıma arasında seçim yapın.",
    "engine_cloud": "Bulut Yapay Zekası",
    "engine_cloud_meta": "Gemini / OpenAI / Groq (API anahtarı gerekli)",
    "engine_local": "Yerel Yapay Zeka",
    "engine_local_meta": "Whisper / Parakeet (100% çevrimdışı ve gizli)",
    "lang_bias_title": "Yazım Dili",
    "lang_bias_desc": "Transkripsiyon için belirli bir dili zorlayın veya otomatik algılamayı kullanın.",
    "lang_bias_label": "Dil Seçin",
    "lang_opt_auto": "Otomatik Algıla",
    "lang_opt_layout": "Klavye Düzenine Göre",
    "streaming_title": "Yazım Modu",
    "streaming_desc": "Transkripsiyonu ekleme yöntemini seçin.",
    "streaming_checkbox": "Gerçek zamanlı akışlı metin girişi (deneysel)",
    "streaming_subdesc": "Kapatılırsa: metin sadece tuşu bıraktığınızda bir bütün olarak eklenir.",
    "vocab_title": "Özel Sözlük",
    "vocab_desc": "Algılama kalitesini artırmak için özel terimleri, isimleri virgülle ayırarak girin.",
    "vocab_placeholder": "örn. Aura, commit, depo...",
    "engine_health_parakeet_running": "Parakeet: sunucu çalışıyor ({provider}, port {port})",
    "engine_health_parakeet_stopped": "Parakeet: sunucu çalışmıyor",
    "engine_health_whisper_running": "Whisper: sunucu çalışıyor ({provider}, port {port})",
    "engine_health_whisper_stopped": "Whisper: sunucu çalışmıyor",
    "engine_starting": "Motor başlatılıyor…",
    "local_model_title": "Yerel Whisper Modülü",
    "local_model_desc": "Tam gizlilik için yerel bir konuşma tanıma motoru yapılandırın.",
    "local_model_label": "Model Boyutu",
    "model_meta_tiny": "~75 MB — süper hızlı",
    "model_meta_base": "~145 MB — önerilen",
    "model_meta_small": "~465 MB — hassas",
    "model_meta_medium": "~1.5 GB — gelişmiş",
    "model_meta_turbo": "~1.6 GB — RU/EN için en iyi doğruluk",
    "model_meta_turbo_q5": "~550 MB — Turbo'ya yakın, yarı boyut",
    "hotkey_title": "Global Kısayol",
    "hotkey_desc": "Kayda başlamak için seçilen kombinasyonu basılı tutun, transkripsiyon için bırakın.",
    "hotkey_label": "Kombinasyon",
    "hotkey_toggle_mode": "Geçiş modu (kısa basma)",
    "hotkey_toggle_mode_desc": "Kısa bir basış, basılı tutmadan kaydı başlatır veya durdurur.",
    "sound_title": "Ses Geri Bildirimi",
    "sound_desc": "Kayıt durumları değiştiğinde çalınacak ses efektleri.",
    "sound_enable": "Overlay seslerini etkinleştir",
    "sound_volume_label": "Ses Seviyesi",
    "sound_theme_label": "Ses Teması",
    "sound_theme_zen": "Zen",
    "sound_theme_rhodes": "Rhodes",
    "sound_theme_scifi": "Sci-Fi",
    "sound_theme_classic": "Zil",
    "api_title": "API Anahtarları Yetkilendirme",
    "api_desc": "Gemini, OpenAI veya Groq bulut hizmetleri için API anahtarlarınızı girin.",
    "api_provider": "API Sağlayıcısı",
    "section_cloud_ai": "Bulut yapay zekâsı",
    "api_key": "API Anahtarı",
    "api_key_placeholder": "API anahtarınızı buraya girin...",
    "hotkey_prompt": "Tuşlara basın...",
    "key_saved_placeholder": "•••••••• (güvenle kaydedildi)",
    "key_placeholder": "API anahtarını girin",
    "api_get_key": "API Anahtarı Al",
    "history_title": "Yazım Geçmişi",
    "history_clear": "Geçmişi Temizle",
    "history_desc": "Son sesli yazımlarınız yerel olarak saklanır.",
    "history_empty": "Geçmiş boş. Yazdığınız metinler burada görünecektir.",
    "history_badge_cloud": "Bulut",
    "history_badge_local": "Yerel",
    "history_engine_whisper": "Whisper",
    "history_engine_parakeet": "NVIDIA Parakeet",
    "history_unit_ms": "ms",
    "history_unit_sec": "sn",
    "about_app_title": "Aura Sesli Giriş",
    "about_version": "v1.0.10",
    "about_description": "Windows için genel sesli giriş aracı. Program, konuşmayı metne dönüştürür ve otomatik biçimlendirme ve noktalama işaretleriyle herhangi bir aktif pencereye ekler.",
    "status_ready": "Hazır",
    "btn_save": "Ayarları Kaydet",
    "confirm_title": "Onay",
    "confirm_message": "Bu işlemi gerçekleştirmek istediğinizden emin misiniz?",
    "confirm_cancel": "İptal",
    "confirm_ok": "Onayla",
    "status_loading": "Ayarlar yükleniyor...",
    "status_modified": "Ayarlar değiştirildi (kaydedilmedi)",
    "status_saving": "Ayarlar kaydediliyor...",
    "status_saved": "Ayarlar başarıyla kaydedildi!",
    "status_error": "Hata: ",
    "model_status_ready": "Yüklendi",
    "model_action_download": "İndir",
    "model_action_delete": "Sil",
    "api_get_key_pattern": "{name} üzerinden anahtar al",
    "status_loaded": "Ayarlar yüklendi",
    "status_load_error": "Ayarlar yüklenemedi: ",
    "status_save_error": "Ayarlar kaydedilemedi: ",
    "status_streaming_degraded": "Canlı Parakeet önizlemesi devre dışı — nihai metin toplu olarak hazırlanıyor",
    "model_downloading_pattern": "'{model}' modeli için indirme başlatılıyor...",
    "model_download_error_pattern": "İndirme hatası: {err}",
    "delete_model_title": "Modeli sil",
    "delete_model_confirm_pattern": "Yerel '{model}' modelini silmek istediğinizden emin misiniz?",
    "delete_model_btn": "Sil",
    "model_deleting_pattern": "'{model}' modeli siliniyor...",
    "model_deleted_success": "Model başarıyla silindi",
    "model_delete_error_pattern": "Silme hatası: {err}",
    "model_downloaded_success_pattern": "'{model}' modeli indirildi!",
    "confirm_clear_history_title": "Geçmişi Temizle",
    "confirm_clear_history_msg": "Tüm transkripsiyon geçmişini temizlemek istediğinizden emin misiniz?",
    "general_ui_lang_title": "Arayüz Dili",
    "general_ui_lang_desc": "Ayarlar ve uygulama bildirimleri için dili seçin.",
    "update_checks_title": "Güncelleme denetimi",
    "update_checks_desc": "Aura, GitHub’a yalnızca elle denetlediğinizde veya otomatik denetimleri etkinleştirdiğinizde bağlanır.",
    "update_checks_checkbox": "Başlangıçta güncellemeleri otomatik olarak denetle",
    "update_check_now": "Güncellemeleri denetle",
    "update_current": "Aura güncel.",
    "update_available_pattern": "Aura v{version} kullanılabilir.",
    "update_check_error_pattern": "Güncellemeler denetlenemedi: {error}",
    "update_installing": "Güncelleme indiriliyor, imza doğrulanıyor ve kuruluyor...",
    "update_installed_restarting": "Güncelleme kuruldu. Yeniden başlatılıyor...",
    "update_install_error_open_release": "Güncelleme kurulamadı. Sürüm sayfası açılıyor...",
    "gpu_accel_label": "Yerel donanım hızlandırma",
    "gpu_accel_cpu_title": "CPU (hızlandırma yok)",
    "gpu_accel_cpu_desc": "Standart mod. Güvenlidir ancak işlemciyi daha fazla kullanır.",
    "gpu_accel_cuda_title": "NVIDIA CUDA (en yüksek hız)",
    "gpu_accel_cuda_desc": "GeForce RTX/GTX ekran kartları için. Tensor çekirdeklerini kullanır.",
    "cuda_license_title": "NVIDIA CUDA koşulları",
    "cuda_license_message": "Aura en fazla 1,52 GiB hızlandırma arşivi indirecek ve en fazla 2,33 GiB dosya kuracaktır. Bilgisayarda yüklü uyumlu NVIDIA bileşenleri yeniden kullanılır; aksi halde Aura tescilli CUDA ve cuDNN bileşenlerini NVIDIA koşulları kapsamında doğrudan NVIDIA'dan indirir.",
    "cuda_license_footnote": "Lisanslar:",
    "cuda_license_cuda_link": "CUDA Toolkit lisansı",
    "cuda_license_cudnn_link": "cuDNN lisansı",
    "cuda_license_accept": "İndir",
    "gpu_accel_dml_title": "DirectML (evrensel)",
    "gpu_accel_dml_desc": "AMD, Intel ve NVIDIA ekran kartları için. Temel hızlandırma.",
    "hotkey_reset_title": "Alt+V'ye Sıfırla",
    "local_engine_label": "Tanıma Motoru",
    "local_engine_whisper": "Whisper.cpp",
    "local_engine_parakeet": "NVIDIA Parakeet",
    "parakeet_model_label": "Parakeet Modeli",
    "model_meta_parakeet": "~670 MB — NVIDIA tarafından optimize edildi",
    "model_cancel_download": "İndirmeyi iptal et",
    "model_download_cancelled": "İndirme iptal edildi",
    "update_available": "Güncelleme mevcut",
    "fallback_title": "Bulut kullanılamadığında otomatik geçiş",
    "fallback_desc": "Bulut yapay zekası kullanılamıyorsa (VPN, bölge engeli, ağ yok), bu kayıt için önceden indirilmiş yerel modeli otomatik olarak kullan.",
    "fallback_checkbox": "Yerel modele otomatik geçişi etkinleştir",
    "copy_context_title": "Seçili metni düzenle",
    "copy_context_desc": "Etkinleştirildiğinde Aura, Ctrl+C gönderir ve seçili metni bir düzenleme komutu için bağlam olarak seçilen bulut sağlayıcısına iletir. Terminalde çalışırken bu özelliği devre dışı bırakın.",
    "copy_context_checkbox": "Seçimi yakalamaya ve bulutta düzenlemeye izin ver",
    "btn_copy_diagnostics": "Teşhis Raporunu Kopyala",
    "toast_diagnostics_copied": "Teşhis raporu panoya kopyalandı!",
    "diag_speech_text_title": "Konuşma Metnini Günlüğe Kaydet (Geliştirici Modu)",
    "diag_title": "Teşhis",
    "diag_speech_text_desc": "Teşhis günlüklerine tam transkribe edilmiş konuşma metnini dahil et. Gizlilik nedeniyle varsayılan olarak devre dışıdır.",
    "diag_speech_text_checkbox": "Konuşma metnini günlüklere dahil et",
    "api_custom_url": "Sunucu API Adresi",
    "api_custom_model": "Model Adı",
    "provider_opt_custom": "Özel Sunucu",
    "overlay_timer_title": "Arayüz Zamanlayıcısı ve Durumu",
    "overlay_timer_desc": "Ses göstergesinin altında süre sayacını ve işleme durumunu göster. Hatalar her zaman görünür kalır.",
    "overlay_timer_checkbox": "Arayüzde zamanlayıcıyı ve durumu göster",
    "mic_meter_title": "Mikrofon Kontrolü",
    "mic_meter_desc": "Giriş sinyali seviyesini ve gerçek zamanlı ses algılamayı test edin.",
    "audio_device_label": "Mikrofon (giriş cihazı)",
    "audio_device_default": "Varsayılan (sistem)",
    "audio_device_desc": "Ses kaydı için fiziksel mikrofonu seçin.",
    "mic_meter_test_btn_start": "Testi Başlat",
    "mic_meter_test_btn_stop": "Testi Durdur",
    "mic_meter_speech": "Konuşma",
    "mic_meter_silence": "Sessizlik",
    "mic_start_error": "Mikrofon başlatma hatası: ",
    "history_search_placeholder": "Geçmişte ara...",
    "history_filter_all": "Tümü",
    "history_filter_cloud": "Bulut",
    "history_filter_local": "Yerel",
    "gpu_status_installing": "Yükleniyor..."
  }
};

let currentLanguage = "ru";

function syncPanelSelection(select) {
  const wrap = select.closest(".select-wrap");
  const panel = wrap && wrap.querySelector(".select-panel");
  if (!panel) return;
  const value = select.value;
  panel.querySelectorAll(".select-panel-item").forEach((item) => {
    item.classList.toggle("is-selected", item.dataset.value === value);
  });
}

function buildSelectPanel(select) {
  const wrap = select.closest(".select-wrap");
  const panel = wrap && wrap.querySelector(".select-panel");
  if (!panel) return;
  panel.textContent = "";
  for (const option of select.options) {
    if (option.disabled) continue;
    const item = document.createElement("div");
    item.className = "select-panel-item";
    item.dataset.value = option.value;
    item.setAttribute("role", "option");

    const main = document.createElement("div");
    main.className = "select-panel-item-main";

    const name = document.createElement("span");
    name.className = "select-panel-item-name";
    name.textContent = option.textContent.trim();
    main.appendChild(name);

    const check = document.createElement("span");
    check.className = "select-panel-item-check";
    check.setAttribute("aria-hidden", "true");
    check.innerHTML =
      '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3" stroke-linecap="round" stroke-linejoin="round"><polyline points="20 6 9 17 4 12"></polyline></svg>';

    item.appendChild(main);
    item.appendChild(check);
    item.addEventListener("click", () => {
      pickSelectValue(select, option.value);
    });
    panel.appendChild(item);
  }
  syncPanelSelection(select);
}

function rebuildSelectPanels() {
  document.querySelectorAll("select.custom-select").forEach(buildSelectPanel);
}

function closeSelectPanels() {
  document.querySelectorAll(".select-panel.open").forEach((panel) => {
    panel.classList.remove("open");
  });
  document.querySelectorAll("select.custom-select[aria-expanded]").forEach((el) => {
    el.setAttribute("aria-expanded", "false");
  });
}

function toggleSelectPanel(select) {
  const wrap = select.closest(".select-wrap");
  const panel = wrap && wrap.querySelector(".select-panel");
  if (!panel) return;
  const willOpen = !panel.classList.contains("open");
  closeSelectPanels();
  if (willOpen) {
    // Values can change programmatically (loadSettings, engine switching)
    // after the panel was built, so the accent mark must be re-synced
    // against the current value every time the panel opens.
    syncPanelSelection(select);
    panel.classList.add("open");
    select.setAttribute("aria-expanded", "true");
  }
}

function pickSelectValue(select, value) {
  if (select.value === value) {
    closeSelectPanels();
    return;
  }
  select.value = value;
  select.dispatchEvent(new Event("change", { bubbles: true }));
  syncPanelSelection(select);
  closeSelectPanels();
}

function movePanelFocus(select, direction) {
  const panel = select.closest(".select-wrap").querySelector(".select-panel");
  if (!panel) return;
  const items = Array.from(panel.querySelectorAll(".select-panel-item"));
  const currentIndex = items.findIndex((item) => item.classList.contains("is-focused"));
  let next = currentIndex === -1 ? 0 : currentIndex + direction;
  next = (next + items.length) % items.length;
  items.forEach((item) => {
    item.classList.toggle("is-focused", item === items[next]);
    if (item === items[next] && typeof item.scrollIntoView === "function") {
      item.scrollIntoView({ block: "nearest" });
    }
  });
  return items[next];
}

function handleSelectMousedown(event) {
  if (event.button !== 0) return;
  event.preventDefault();
  const select = event.currentTarget
    .closest(".select-wrap")
    ?.querySelector("select.custom-select");
  if (select) toggleSelectPanel(select);
}

function handleSelectKeydown(event) {
  const select = event.currentTarget;
  const panel = select.closest(".select-wrap")?.querySelector(".select-panel");
  const isOpen = panel && panel.classList.contains("open");
  const KEY_OPEN = ["Enter", " ", "ArrowDown", "ArrowUp"];
  if (!isOpen) {
    if (KEY_OPEN.includes(event.key)) {
      event.preventDefault();
      toggleSelectPanel(select);
      movePanelFocus(select, 1);
    }
    return;
  }
  if (event.key === "Escape") {
    event.preventDefault();
    closeSelectPanels();
    select.focus();
    return;
  }
  if (event.key === "ArrowDown" || event.key === "ArrowUp") {
    event.preventDefault();
    movePanelFocus(select, event.key === "ArrowDown" ? 1 : -1);
    return;
  }
  if (event.key === "Enter" || event.key === " ") {
    event.preventDefault();
    const focused = panel.querySelector(".select-panel-item.is-focused");
    if (focused) pickSelectValue(select, focused.dataset.value);
    else closeSelectPanels();
  }
}

function initSelectPanels() {
  document.querySelectorAll("select.custom-select").forEach((select) => {
    const wrap = select.closest(".select-wrap");
    if (!wrap) return;
    const catcher = document.createElement("div");
    catcher.className = "select-catcher";
    catcher.setAttribute("aria-hidden", "true");
    wrap.insertBefore(catcher, select);
    catcher.addEventListener("mousedown", handleSelectMousedown);
    select.addEventListener("keydown", handleSelectKeydown);

    // Clicking the associated <label for="…"> activates the native select
    // directly (the OS dropdown would open over the custom panel), so the
    // label must open the panel itself instead.
    const label = document.querySelector(`label[for="${select.id}"]`);
    if (label) {
      label.addEventListener("mousedown", (event) => {
        event.preventDefault();
      });
      label.addEventListener("click", (event) => {
        event.preventDefault();
        toggleSelectPanel(select);
      });
      label.addEventListener("keydown", (event) => {
        if (event.key === "Enter" || event.key === " " || event.key === "Spacebar") {
          event.preventDefault();
          toggleSelectPanel(select);
        }
      });
    }
  });
  document.addEventListener("mousedown", (event) => {
    if (!event.target.closest(".select-wrap")) {
      closeSelectPanels();
    }
  });
  rebuildSelectPanels();
}

function getTranslation(key, params = {}) {
  const dict = i18nDict[currentLanguage] || i18nDict.ru;
  let template = dict[key] || i18nDict.ru[key] || key;
  for (const [k, v] of Object.entries(params)) {
    template = template.replaceAll(`{${k}}`, v);
  }
  return template;
}

document.addEventListener("DOMContentLoaded", () => {
  // Navigation tabs follow the WAI-ARIA tab pattern and remain native buttons.
  const tabs = document.querySelectorAll(".nav-tab");
  const panels = document.querySelectorAll(".tab-panel");

  function activateTab(tab) {
    tabs.forEach((item) => {
      const selected = item === tab;
      item.classList.toggle("active", selected);
      item.setAttribute("aria-selected", String(selected));
      item.tabIndex = selected ? 0 : -1;
    });
    panels.forEach((panel) => {
      panel.style.display = panel.id === "panel-" + tab.dataset.tab ? "flex" : "none";
    });
    if (tab.dataset.tab === "history") loadHistoryList();
    if (tab.dataset.tab !== "speech" && isMicTesting) {
      stopMicTesting();
    }
  }

  tabs.forEach((tab) => {
    tab.addEventListener("click", () => activateTab(tab));
  });
  bindTabKeyboardNavigation(tabs, activateTab);

  // Toggle API Key visibility
  const apiKeyInput = document.getElementById("input-api-key");
  const toggleKeyBtn = document.getElementById("btn-toggle-key");
  toggleKeyBtn.addEventListener("click", () => {
    if (apiKeyInput.type === "password") {
      apiKeyInput.type = "text";
      toggleKeyBtn.classList.add("visible");
    } else {
      apiKeyInput.type = "password";
      toggleKeyBtn.classList.remove("visible");
    }
  });

  // Engine change (Cloud vs Local) toggling Whisper card visibility
const radioCloud = document.getElementById("radio-cloud");
  const radioLocal = document.getElementById("radio-local");
  const selectLocalEngine = document.getElementById("select-local-engine");
  const groupWhisperModels = document.getElementById("group-whisper-models");
  const groupParakeetModels = document.getElementById("group-parakeet-models");

  function updateEngineUI() {
    const vocabCard = document.getElementById("card-vocabulary");
    const fallbackCard = document.getElementById("card-cloud-fallback");
    const streamingCard = document.getElementById("card-streaming");
    const langCard = document.getElementById("card-language");
    const localEngineSection = document.getElementById("local-engine-section");
    const apiKeysCard = document.getElementById("card-api-keys");
    const cloudFunctionsCard = document.getElementById("card-cloud-functions");

    if (radioLocal.checked) {
      if (localEngineSection) localEngineSection.style.display = "flex";
      updateLocalEngineUI();
      if (fallbackCard) fallbackCard.style.display = "none";
      if (apiKeysCard) apiKeysCard.style.display = "none";
      if (cloudFunctionsCard) cloudFunctionsCard.style.display = "none";
    } else {
      if (localEngineSection) localEngineSection.style.display = "none";
      if (apiKeysCard) apiKeysCard.style.display = "flex";
      if (fallbackCard) fallbackCard.style.display = "flex";
      if (langCard) langCard.style.display = "none";
      if (vocabCard) vocabCard.style.display = "none";
      if (streamingCard) streamingCard.style.display = "none";

      const isHuggingFace = selectProvider && selectProvider.value === "huggingface";
      if (cloudFunctionsCard) cloudFunctionsCard.style.display = isHuggingFace ? "none" : "flex";
    }
  }

  function updateLocalEngineUI() {
    if (!selectLocalEngine || !groupWhisperModels || !groupParakeetModels) return;
    const isParakeet = selectLocalEngine.value === "parakeet";
    const vocabCard = document.getElementById("card-vocabulary");
    if (vocabCard) vocabCard.style.display = isParakeet ? "none" : "flex";
    const langCard = document.getElementById("card-language");
    if (langCard) langCard.style.display = isParakeet ? "none" : "flex";
    const streamingCard = document.getElementById("card-streaming");
    if (streamingCard) streamingCard.style.display = isParakeet ? "flex" : "none";
    const gpuSettings = document.getElementById("gpu-acceleration-settings");
    if (gpuSettings) {
      gpuSettings.style.display = "block";
    }
    if (isParakeet) {
      groupWhisperModels.style.display = "none";
      groupParakeetModels.style.display = "block";
      selectModelCard("parakeet-v3");
    } else {
      groupWhisperModels.style.display = "block";
      groupParakeetModels.style.display = "none";
      if (selectedModelName === "parakeet-v3") {
        selectModelCard(lastSelectedWhisperModel || "large-v3-turbo-q5_0");
      }
    }
  }

async function refreshEngineHealth() {
    const chip = document.getElementById("engine-health-chip");
    if (!chip) return;
    try {
      const health = await invoke("get_engine_health");
      lastHealthState = health;
      chip.classList.remove("health-ok", "health-warn");
      if (health.engine === "parakeet-local-fallback") {
        chip.textContent = "";
        chip.style.display = "none";
        return;
      }
      chip.style.display = "";
      if (health.running) {
        const runningKey = health.engine === "whisper"
          ? "engine_health_whisper_running"
          : "engine_health_parakeet_running";
        chip.textContent = getTranslation(runningKey, {
          provider: health.provider || "cpu",
          port: health.port ?? "?",
        });
        chip.classList.add("health-ok");
        return;
      }
      if (Date.now() < engineStartNoticeUntil) {
        chip.innerHTML = `<span class="spinner-inline" aria-hidden="true"></span> ${getTranslation("engine_starting")}`;
        chip.classList.add("health-warn");
        return;
      }
      const stoppedKey = health.engine === "whisper"
        ? "engine_health_whisper_stopped"
        : "engine_health_parakeet_stopped";
      chip.textContent = getTranslation(stoppedKey);
      chip.classList.add("health-warn");
    } catch (e) {
      console.error(e);
    }
  }

  // Right after the user switches the local engine (or its acceleration) the
  // resident server needs seconds to boot and warm up. Poll fast until the
  // server actually reports running (or the hard cap passes) and WITHOUT the
  // focus gate: the settings window is usually unfocused while the user
  // dictates, and a frozen "starting…" chip is worse than a cheap IPC poll.
  function kickEngineStatusPolling(windowMs = 60000) {
    engineStartNoticeUntil = Date.now() + windowMs;
    if (engineFastPollTimer) return;
    engineFastPollTimer = setInterval(() => {
      const running = Boolean(lastHealthState && lastHealthState.running);
      if (running || Date.now() >= engineStartNoticeUntil) {
        clearInterval(engineFastPollTimer);
        engineFastPollTimer = null;
      }
      refreshEngineHealth();
    }, 700);
  }

  if (selectLocalEngine) {
    selectLocalEngine.addEventListener("change", () => {
      updateLocalEngineUI();
      markSettingsModified(true);
      refreshEngineHealth();
      kickEngineStatusPolling();
    });
  }

  radioCloud.addEventListener("change", updateEngineUI);
  radioLocal.addEventListener("change", updateEngineUI);

  // Dynamic API Key Links
  const linkGetKey = document.getElementById("link-get-key");
  const providerLinks = {
    gemini: { url: "https://aistudio.google.com/", name: "Google AI Studio" },
    openai: { url: "https://platform.openai.com/api-keys", name: "OpenAI Platform" },
    groq: { url: "https://console.groq.com/keys", name: "Groq Console" },
    huggingface: { url: "https://huggingface.co/settings/tokens", name: "Hugging Face" },
    custom: { url: "https://deepinfra.com/dash/api_keys", name: "DeepInfra / Custom" }
  };
  function updateApiKeyLink() {
    const prov = selectProvider ? selectProvider.value : "gemini";
    const info = providerLinks[prov] || providerLinks.gemini;
    if (linkGetKey) {
      linkGetKey.href = info.url;
      linkGetKey.textContent = getTranslation("api_get_key_pattern", { name: info.name });
    }
  }

  function updateCustomProviderUI() {
    const isCustom = selectProvider && selectProvider.value === "custom";
    const urlGroup = document.getElementById("custom-provider-url-group");
    const modelGroup = document.getElementById("custom-provider-model-group");
    if (urlGroup) urlGroup.style.display = isCustom ? "flex" : "none";
    if (modelGroup) modelGroup.style.display = isCustom ? "flex" : "none";
  }

  document.addEventListener("click", (e) => {
    const anchor = e.target.closest("a[href^='http://'], a[href^='https://']");
    if (anchor) {
      e.preventDefault();
      invoke("open_url", { url: anchor.href }).catch((err) => console.error("Failed to open URL:", err));
    }
  });

  // Settings elements
  const selectProvider = document.getElementById("select-provider");
  const selectHotkey = document.getElementById("input-hotkey");
  const selectLanguage = document.getElementById("select-language");
  const textareaDictionary = document.getElementById("textarea-dictionary");
  const checkboxToggle = document.getElementById("checkbox-toggle");
  const checkboxCloudFallback = document.getElementById("checkbox-cloud-fallback");
  const checkboxAutostart = document.getElementById("checkbox-autostart");
  const checkboxAutomaticUpdateChecks = document.getElementById("checkbox-automatic-update-checks");
const btnSaveSettings = document.getElementById("btn-save-settings");
  // NOTE: the click binding for the Save button lives once, in the "Bind Events"
  // block below — it must not be re-registered here (would double-save).
  
  const checkboxSounds = document.getElementById("checkbox-sounds");
  const checkboxCopyContext = document.getElementById("checkbox-selection-edit-enabled");
  const selectSoundTheme = document.getElementById("select-sound-theme");
  const rangeVolume = document.getElementById("range-sound-volume");
  const volumeLabel = document.getElementById("volume-value-label");
  const selectAudioDevice = document.getElementById("select-audio-device");

  if (selectAudioDevice) {
    selectAudioDevice.addEventListener("change", () => {
      markSettingsModified(true);
    });
  }

  async function loadAudioDevices(selectedDevice = "default") {
    if (!selectAudioDevice) return;
    try {
      const devices = await invoke("get_audio_input_devices");
      const currentVal = selectedDevice || selectAudioDevice.value || "default";
      const dict = i18nDict[currentLanguage] || i18nDict.ru;
      selectAudioDevice.innerHTML = `<option value="default" data-i18n="audio_device_default">${dict.audio_device_default || "По умолчанию (системное)"}</option>`;
      if (Array.isArray(devices)) {
        devices.forEach((dev) => {
          if (dev && dev.trim() && dev !== "default") {
            const opt = document.createElement("option");
            opt.value = dev;
            opt.textContent = dev;
            selectAudioDevice.appendChild(opt);
          }
        });
      }
      selectAudioDevice.value = currentVal;
      buildSelectPanel(selectAudioDevice);
      syncPanelSelection(selectAudioDevice);
    } catch (e) {
      console.error("Failed to load audio input devices", e);
    }
  }

  if (rangeVolume) {
    rangeVolume.addEventListener("input", () => {
      if (volumeLabel) {
        volumeLabel.textContent = `${rangeVolume.value}%`;
      }
    });
  }

  // Hotkey Recorder Widget Events
  const btnResetHotkey = document.getElementById("btn-reset-hotkey");
  let isRecordingHotkey = false;
  let hasRecordedThisSession = false;
  const recordedHotkeyModifiers = new Set();
  const hotkeyModifierOrder = ["Ctrl", "Alt", "Shift", "Win"];

  const hotkeyModifierName = (event) => {
    const modifierCodes = {
      "ControlLeft": "Ctrl", "ControlRight": "Ctrl",
      "AltLeft": "Alt", "AltRight": "Alt",
      "ShiftLeft": "Shift", "ShiftRight": "Shift",
      "MetaLeft": "Win", "MetaRight": "Win",
      "OSLeft": "Win", "OSRight": "Win"
    };
    return modifierCodes[event.code] || {
      "Control": "Ctrl", "Alt": "Alt", "Shift": "Shift", "Meta": "Win", "OS": "Win"
    }[event.key] || null;
  };

  const orderedHotkeyModifiers = () =>
    hotkeyModifierOrder.filter(modifier => recordedHotkeyModifiers.has(modifier));

  if (selectHotkey) {
    selectHotkey.addEventListener("focus", () => {
      isRecordingHotkey = true;
      hasRecordedThisSession = false;
      recordedHotkeyModifiers.clear();
      selectHotkey.value = getTranslation("hotkey_prompt") || "Press keys...";
      selectHotkey.classList.add("recording");
    });

    selectHotkey.addEventListener("blur", () => {
      isRecordingHotkey = false;
      recordedHotkeyModifiers.clear();
      selectHotkey.classList.remove("recording");
      // Restore current settings value on blur ONLY if user didn't record a new combination
      if (!hasRecordedThisSession) {
        invoke("get_settings").then(settings => {
          if (settings) {
            selectHotkey.value = settings.hotkey || "Alt+V";
          }
        }).catch(() => {
          selectHotkey.value = "Alt+V";
        });
      }
    });

    selectHotkey.addEventListener("keydown", (e) => {
      if (!isRecordingHotkey) return;
      e.preventDefault();
      e.stopPropagation();

      const key = e.key;
      const code = e.code;

      const modifierName = hotkeyModifierName(e);
      if (modifierName) {
        recordedHotkeyModifiers.add(modifierName);
        selectHotkey.value = orderedHotkeyModifiers().join("+");
        return;
      }

      if (e.ctrlKey) recordedHotkeyModifiers.add("Ctrl");
      if (e.altKey) recordedHotkeyModifiers.add("Alt");
      if (e.shiftKey) recordedHotkeyModifiers.add("Shift");
      if (e.metaKey) recordedHotkeyModifiers.add("Win");

      let keyName = "";
      if (code.startsWith("Key")) {
        // Physical letter keys, e.g. "KeyV" -> "V"
        keyName = code.substring(3).toUpperCase();
      } else if (code.startsWith("Digit")) {
        // Physical number keys, e.g. "Digit1" -> "1"
        keyName = code.substring(5);
      } else if (code.startsWith("F") && code.length >= 2 && !isNaN(code.substring(1))) {
        // Function keys, e.g. "F8" -> "F8"
        keyName = code;
      } else {
        // Map common physical layout codes
        const codeMap = {
          "Space": "Space",
          "CapsLock": "Caps Lock",
          "Tab": "Tab"
        };
        if (codeMap[code]) {
          keyName = codeMap[code];
        } else {
          // If e.code is empty or unrecognized, fallback to e.key for basic alphanumeric
          if (key.length === 1 && /[a-zA-Z0-9]/.test(key)) {
            keyName = key.toUpperCase();
          } else {
            return;
          }
        }
      }

      const hotkeyStr = [...orderedHotkeyModifiers(), keyName].join("+");
      hasRecordedThisSession = true; // Mark as successfully recorded
      selectHotkey.value = hotkeyStr;
      isRecordingHotkey = false;
      selectHotkey.classList.remove("recording");
      selectHotkey.blur();

      // Trigger modified state
      selectHotkey.dispatchEvent(new Event("change", { bubbles: true }));
    });

    selectHotkey.addEventListener("keyup", (e) => {
      if (!isRecordingHotkey || !hotkeyModifierName(e) || recordedHotkeyModifiers.size === 0) return;
      e.preventDefault();
      e.stopPropagation();

      if (recordedHotkeyModifiers.size >= 2) {
        hasRecordedThisSession = true;
        selectHotkey.value = orderedHotkeyModifiers().join("+");
        isRecordingHotkey = false;
        selectHotkey.classList.remove("recording");
        selectHotkey.blur();
        selectHotkey.dispatchEvent(new Event("change", { bubbles: true }));
      } else {
        recordedHotkeyModifiers.delete(hotkeyModifierName(e));
        selectHotkey.value = getTranslation("hotkey_prompt") || "Press keys...";
      }
    });
  }

  if (btnResetHotkey) {
    btnResetHotkey.addEventListener("click", () => {
      if (selectHotkey) {
        selectHotkey.value = "Alt+V";
        selectHotkey.dispatchEvent(new Event("change", { bubbles: true }));
      }
    });
  }


  function updateSoundUI() {
    const themeGroup = document.getElementById("sound-theme-group");
    const volumeGroup = document.getElementById("sound-volume-group");
    const show = (checkboxSounds && checkboxSounds.checked) ? "flex" : "none";
    if (themeGroup) {
      themeGroup.style.display = show;
    }
    if (volumeGroup) {
      volumeGroup.style.display = show;
    }
  }
  if (checkboxSounds) {
    checkboxSounds.addEventListener("change", updateSoundUI);
  }
  
  let apiKeys = {
    gemini: "",
    openai: "",
    groq: "",
    huggingface: "",
    custom: ""
  };
  let apiKeyPresent = {
    gemini: false,
    openai: false,
    groq: false,
    huggingface: false,
    custom: false
  };
  let apiKeyDirty = {
    gemini: false,
    openai: false,
    groq: false,
    huggingface: false,
    custom: false
  };
  let previousSelProvider = selectProvider.value;

  function renderProviderKeyInput() {
    const provider = selectProvider.value;
    apiKeyInput.value = apiKeys[provider] || "";
const providerDict = i18nDict[currentLanguage] || i18nDict.ru;
    apiKeyInput.placeholder = apiKeyPresent[provider]
      ? providerDict.key_saved_placeholder || "•••••••• (saved securely)"
      : providerDict.key_placeholder || "Enter API key";
  }

  apiKeyInput.addEventListener("input", () => {
    const provider = selectProvider.value;
    apiKeys[provider] = apiKeyInput.value;
    apiKeyDirty[provider] = true;
  });

  if (selectProvider) {
    selectProvider.addEventListener("change", () => {
      updateApiKeyLink();
      updateCustomProviderUI();
      renderProviderKeyInput();
      updateEngineUI();
    });
  }
  
  const footerStatusText = document.getElementById("footer-status-text");

  let selectedModelName = "base";
  let lastSelectedWhisperModel = "large-v3-turbo-q5_0";
  let activeLocalAcceleration = "cpu";

  let settingsModified = false;
  let isSettingsLoaded = false;
  let autoSaveTimeout = null;
  let settingsRevision = 0;
  let engineHealthTimer = null;
  let engineFastPollTimer = null;
  let engineStartNoticeUntil = 0;
  let lastHealthState = null;

  function markSettingsModified(immediate = false) {
    if (!isSettingsLoaded) return;
    settingsModified = true;
    settingsRevision += 1;
    showStatus(getTranslation("status_modified"), false, true);

    if (autoSaveTimeout) {
      clearTimeout(autoSaveTimeout);
      autoSaveTimeout = null;
    }

    const delay = immediate ? 80 : 700;
    autoSaveTimeout = setTimeout(() => {
      autoSaveTimeout = null;
      if (settingsModified) {
        saveSettings();
      }
    }, delay);
  }

  function bindSettingsChangeListeners() {
    const checkboxStreaming = document.getElementById("checkbox-streaming");
    const checkboxLogSpeechText = document.getElementById("setting-log-speech-text");
    const checkboxOverlayTimer = document.getElementById("checkbox-overlay-show-timer");
    const inputCustomUrl = document.getElementById("input-custom-url");
    const inputCustomModel = document.getElementById("input-custom-model");
    const inputs = [
      radioCloud, radioLocal, selectProvider, apiKeyInput, inputCustomUrl, inputCustomModel, selectHotkey,
      selectLanguage, textareaDictionary, checkboxToggle, checkboxCloudFallback,
      checkboxAutostart, checkboxAutomaticUpdateChecks, checkboxStreaming, checkboxSounds,
      selectSoundTheme, rangeVolume, selectLocalEngine, checkboxCopyContext, checkboxLogSpeechText,
      checkboxOverlayTimer
    ];
    inputs.forEach(input => {
      if (input) {
        const isImmediate = input.tagName === "SELECT" || input.type === "checkbox" || input.type === "radio";
        input.addEventListener("change", () => markSettingsModified(isImmediate));
        if (input.tagName === "INPUT" || input.tagName === "TEXTAREA") {
          input.addEventListener("input", () => markSettingsModified(false));
        }
      }
    });
  }
  const modelCards = document.querySelectorAll(".model-card[data-model]");

  // WAI-ARIA radio group: arrow keys move between cards within the same group
  const arrowDirection = { ArrowUp: -1, ArrowLeft: -1, ArrowRight: 1, ArrowDown: 1 };

  modelCards.forEach(card => {
    card.addEventListener("click", async (e) => {
      if (e.target.closest("[data-static]")) {
        return;
      }
      // Prevent selection trigger when clicking delete/download buttons inside the card
      if (e.target.closest(".btn-delete-card-model") || e.target.closest(".btn-download-card-model") || e.target.closest(".btn-cancel-download")) {
        return;
      }
      const model = card.dataset.model;
      const downloaded = await invoke("get_downloaded_models").catch(() => []);
      if (downloaded.includes(model)) {
        selectModelCard(model);
      }
    });

    card.addEventListener("keydown", async (e) => {
      if (e.target.closest("[data-static]")) {
        return;
      }
      // Inner buttons (delete/download/cancel) handle their own keys
      if (e.target.closest("button")) {
        return;
      }
      if (e.key === "Enter" || e.key === " ") {
        e.preventDefault();
        const model = card.dataset.model;
        const downloaded = await invoke("get_downloaded_models").catch(() => []);
        if (downloaded.includes(model)) {
          selectModelCard(model);
        }
        return;
      }
      const direction = arrowDirection[e.key];
      if (direction === undefined) {
        return;
      }
      e.preventDefault();
      const group = card.parentElement;
      if (!group) {
        return;
      }
      const groupCards = Array.from(group.querySelectorAll(".model-card[data-model]"));
      const index = groupCards.indexOf(card);
      if (index === -1) {
        return;
      }
      const next = groupCards[(index + direction + groupCards.length) % groupCards.length];
      next.focus();
      const model = next.dataset.model;
      const downloaded = await invoke("get_downloaded_models").catch(() => []);
      if (downloaded.includes(model)) {
        selectModelCard(model);
      }
    });
  });

  function selectModelCard(model) {
    if (model === "punctuation") {
      return;
    }
    if (model !== "parakeet-v3") {
      lastSelectedWhisperModel = model;
    }
    if (selectedModelName !== model) {
      selectedModelName = model;
      markSettingsModified(true);
    }
    modelCards.forEach(c => {
      const isCurrent = c.dataset.model === model;
      c.classList.toggle("active", isCurrent);
      c.setAttribute("aria-checked", isCurrent ? "true" : "false");
    });
  }

  // Load Settings from Backend
  async function loadSettings(preFetchedSettings = null) {
    try {
      const dict = i18nDict[currentLanguage] || i18nDict.ru;
      showStatus(dict.status_loading || "Загрузка настроек...");
      const settings = preFetchedSettings || await invoke("get_settings");
      
      if (settings) {
        if (settings.transcription_mode === "local") {
          radioLocal.checked = true;
        } else {
          radioCloud.checked = true;
        }
        
        if (settings.model_name && settings.model_name !== "parakeet-v3" && settings.model_name !== "punctuation") {
          lastSelectedWhisperModel = settings.model_name;
        }
        selectModelCard(settings.model_name || "base");
        apiKeys = { gemini: "", openai: "", groq: "", huggingface: "", custom: "" };
        apiKeyDirty = { gemini: false, openai: false, groq: false, huggingface: false, custom: false };
        apiKeyPresent = {
          gemini: !!settings.has_api_key_gemini,
          openai: !!settings.has_api_key_openai,
          groq: !!settings.has_api_key_groq,
          huggingface: !!settings.has_api_key_huggingface,
          custom: !!settings.has_api_key_custom
        };

        selectProvider.value = settings.api_provider || "gemini";
        previousSelProvider = selectProvider.value;
        renderProviderKeyInput();
        updateApiKeyLink();
        const inputCustomUrl = document.getElementById("input-custom-url");
        if (inputCustomUrl) {
          inputCustomUrl.value = settings.custom_api_url || "https://api.deepinfra.com/v1/openai";
        }
        const inputCustomModel = document.getElementById("input-custom-model");
        if (inputCustomModel) {
          inputCustomModel.value = settings.custom_model_name || "openai/whisper-large-v3-turbo";
        }
        updateCustomProviderUI();
        if (selectHotkey) {
          selectHotkey.value = settings.hotkey || "Alt+V";
        }
        if (selectLanguage) {
          selectLanguage.value = settings.language || "auto";
        }
        if (selectLocalEngine) {
          selectLocalEngine.value = settings.local_engine || "whisper";
        }
        updateEngineUI();
        if (textareaDictionary) {
          textareaDictionary.value = settings.dictionary || "";
        }
        if (checkboxToggle) {
          checkboxToggle.checked = !!settings.toggle_enabled;
        }
        if (checkboxCloudFallback) {
          checkboxCloudFallback.checked = settings.cloud_fallback_enabled !== false;
        }
        if (checkboxAutostart) {
          checkboxAutostart.checked = !!settings.autostart;
        }        if (checkboxAutomaticUpdateChecks) {
          checkboxAutomaticUpdateChecks.checked = !!settings.automatic_update_checks;
        }

 
        const checkboxStreaming = document.getElementById("checkbox-streaming");
        if (checkboxStreaming) {
          checkboxStreaming.checked = !!settings.streaming_enabled;
        }
 
  if (checkboxSounds) {
    checkboxSounds.checked = settings.overlay_sounds !== false;
  }
  if (checkboxCopyContext) {

    checkboxCopyContext.checked = !!settings.copy_context_on_start;
  }
        const checkboxLogSpeechText = document.getElementById("setting-log-speech-text");
        if (checkboxLogSpeechText) {
          checkboxLogSpeechText.checked = !!settings.log_speech_text;
        }
        const checkboxOverlayTimer = document.getElementById("checkbox-overlay-show-timer");
        if (checkboxOverlayTimer) {
          checkboxOverlayTimer.checked = settings.overlay_show_timer !== false;
        }
        activeLocalAcceleration = settings.local_acceleration || "cpu";
        if (selectSoundTheme) {
          selectSoundTheme.value = settings.overlay_sound_theme || "zen";
        }
        if (rangeVolume) {
          const volumeVal = typeof settings.overlay_sound_volume === "number" ? Math.round(settings.overlay_sound_volume * 100) : 80;
          rangeVolume.value = volumeVal;
          if (volumeLabel) {
            volumeLabel.textContent = `${volumeVal}%`;
          }
        }
        updateSoundUI();
 
        updateEngineUI();
        await loadAudioDevices(settings.audio_input_device || "default");
        await refreshDownloadedModels();

activeLocalAcceleration = settings.local_acceleration || "cpu";
        selectGpuProvider(activeLocalAcceleration);
        await updateGpuCardStates();
        
        isSettingsLoaded = true;
        settingsModified = false;
        
        refreshEngineHealth();
        if (radioLocal.checked) {
          kickEngineStatusPolling();
        }
        if (!engineHealthTimer) {
          engineHealthTimer = setInterval(() => {
            if (document.hasFocus()) {
              refreshEngineHealth();
            }
          }, 10000);
        }
        
        showStatus(getTranslation("status_loaded"));
        
        bindSettingsChangeListeners();
      }
    } catch (err) {
      console.error(err);
      showStatus(`${getTranslation("status_load_error")}${err}`, true);
    }
  }

  async function refreshDownloadedModels() {
    try {
      const downloaded = await invoke("get_downloaded_models");
      const dict = i18nDict[currentLanguage] || i18nDict.ru;
modelCards.forEach(card => {
        const model = card.dataset.model;
        // Never tear down the UI of a download that is still running
        // (a refresh triggered by a sibling download must not do it either).
        if (inFlightModelDownloads.has(model)) {
          return;
        }
        const isDownloaded = downloaded.includes(model);
        const actionEl = document.getElementById(`action-${model}`);

        // Always restore a clean, non-downloading state. Without this, a cancelled
        // download leaves the progress bar frozen and the action button hidden.
        const progressEl = document.getElementById(`progress-${model}`);
        if (progressEl) {
          const cancelBtn = progressEl.querySelector(".btn-cancel-download");
          if (cancelBtn) cancelBtn.remove();
          progressEl.style.display = "none";
        }
        if (actionEl) actionEl.style.display = "flex";

        if (isDownloaded) {
          actionEl.innerHTML = `
            <span class="status-ready-badge">
              <svg class="status-ready-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3" stroke-linecap="round" stroke-linejoin="round">
                <polyline points="20 6 9 17 4 12"></polyline>
              </svg>
              <span data-i18n="model_status_ready">${dict.model_status_ready || "Установлено"}</span>
            </span>
            <button type="button" class="btn-delete-card-model" title="${dict.model_action_delete || "Удалить"}" data-model="${model}">
              <svg class="btn-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="3 6 5 6 21 6"></polyline><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"></path><line x1="10" y1="11" x2="10" y2="17"></line><line x1="14" y1="11" x2="14" y2="17"></line></svg>
            </button>
          `;
          // Bind click to the delete button
          actionEl.querySelector(".btn-delete-card-model").addEventListener("click", () => deleteModelCard(model));
        } else {
          actionEl.innerHTML = `
            <button type="button" class="btn-download-card-model" data-model="${model}">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="btn-icon"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"></path><polyline points="7 10 12 15 17 10"></polyline><line x1="12" y1="15" x2="12" y2="3"></line></svg>
              ${dict.model_action_download || "Скачать"}
            </button>
          `;
// Bind click to the download button
          actionEl.querySelector(".btn-download-card-model").addEventListener("click", () => downloadModelCard(model));
        }
      });
      refreshEngineHealth();
    } catch (err) {
      console.error("Failed to check downloaded models", err);
    }
  }

  // Save Settings to Backend
  let saveInFlight = false;
  let savePending = false;

  async function saveSettings() {
    // Guard against re-entrancy: rapid clicks or duplicate bindings must not
    // start a second save while one is already running. A change arriving
    // mid-flight re-schedules itself instead of being dropped.
    if (saveInFlight) {
      savePending = true;
      return;
    }
    saveInFlight = true;
    const revisionAtStart = settingsRevision;
    if (btnSaveSettings) {
      btnSaveSettings.disabled = true;
    }
    try {
      const dict = i18nDict[currentLanguage] || i18nDict.ru;
      showStatus(dict.status_saving || "Сохранение настроек...");
      
      const checkboxStreaming = document.getElementById("checkbox-streaming");
      
      // Update apiKeys cache from active input first:
      apiKeys[selectProvider.value] = apiKeyInput.value.trim();

      const soundVolFloat = rangeVolume ? parseFloat(rangeVolume.value) / 100 : 0.8;

      const settings = {
        transcription_mode: radioLocal.checked ? "local" : "cloud",
        ui_language: currentLanguage,
        api_provider: selectProvider.value,
        custom_api_url: (() => {
          const el = document.getElementById("input-custom-url");
          return el ? el.value.trim() : "https://api.deepinfra.com/v1/openai";
        })(),
        custom_model_name: (() => {
          const el = document.getElementById("input-custom-model");
          return el ? el.value.trim() : "openai/whisper-large-v3-turbo";
        })(),

        model_name: selectedModelName,
        hotkey: selectHotkey ? selectHotkey.value : "Alt+V",
        streaming_enabled: checkboxStreaming ? checkboxStreaming.checked : false,
        toggle_enabled: checkboxToggle ? checkboxToggle.checked : false,
        language: selectLanguage ? selectLanguage.value : "auto",
        dictionary: textareaDictionary ? textareaDictionary.value : "",
        cloud_fallback_enabled: checkboxCloudFallback ? checkboxCloudFallback.checked : true,
        autostart: checkboxAutostart ? checkboxAutostart.checked : false,
        automatic_update_checks: checkboxAutomaticUpdateChecks ? checkboxAutomaticUpdateChecks.checked : false,
        local_engine: selectLocalEngine ? selectLocalEngine.value : "whisper",
        local_acceleration: activeLocalAcceleration,
        overlay_sounds: checkboxSounds ? checkboxSounds.checked : true,
    overlay_sound_theme: selectSoundTheme ? selectSoundTheme.value : "zen",
    overlay_sound_volume: soundVolFloat,
    copy_context_on_start: checkboxCopyContext ? checkboxCopyContext.checked : false,
    audio_input_device: selectAudioDevice ? selectAudioDevice.value : "default",
    log_speech_text: (() => {
      const el = document.getElementById("setting-log-speech-text");
      return el ? el.checked : false;
    })(),
    overlay_show_timer: (() => {
      const el = document.getElementById("checkbox-overlay-show-timer");
      return el ? el.checked : true;
    })()
  };

await invoke("set_settings", { settings });
      const failedProviders = [];
      for (const provider of ["gemini", "openai", "groq", "huggingface", "custom"]) {
        if (apiKeyDirty[provider]) {
          const key = apiKeys[provider].trim();
          try {
            await invoke("set_provider_key", { provider, key });
            apiKeyPresent[provider] = key.length > 0;
            apiKeyDirty[provider] = false;
            apiKeys[provider] = "";
          } catch (keyErr) {
            // Save the rest of the keys anyway; only the failed provider
            // stays dirty so the next save retries it.
            console.error(`Failed to save ${provider} key:`, keyErr);
            failedProviders.push(provider);
          }
        }
      }
      renderProviderKeyInput();
      if (radioLocal.checked) {
        kickEngineStatusPolling();
      }
      if (settingsRevision === revisionAtStart) {
        settingsModified = false;
      }
      if (failedProviders.length > 0) {
        showStatus(
          `${getTranslation("status_save_error")} (${failedProviders.join(", ")})`,
          true
        );
      } else {
        showStatus(dict.status_saved || "Настройки успешно сохранены!");
      }
      
      // Temporary success animation in footer status
      setTimeout(() => {
        if (!settingsModified) {
          const currentDict = i18nDict[currentLanguage] || i18nDict.ru;
          showStatus(currentDict.status_ready || "Готово");
        }
      }, 3000);
} catch (err) {
      console.error(err);
      showStatus(`${getTranslation("status_save_error")}${err}`, true);
    } finally {
      saveInFlight = false;
      if (btnSaveSettings) {
        btnSaveSettings.disabled = false;
      }
      if ((savePending || settingsModified) && isSettingsLoaded) {
        savePending = false;
        markSettingsModified(true);
      } else {
        savePending = false;
      }
    }
  }

async function downloadModelCard(model) {
    if (inFlightModelDownloads.has(model)) {
      return;
    }
    inFlightModelDownloads.add(model);
    const actionEl = document.getElementById(`action-${model}`);
    const progressEl = document.getElementById(`progress-${model}`);
    const fillEl = document.getElementById(`fill-${model}`);
    const pctEl = document.getElementById(`pct-${model}`);
    const modelAtStart = selectedModelName;

    try {
      showStatus(getTranslation("model_downloading_pattern", { model }));

      // Hide actions, show progress
      if (actionEl) actionEl.style.display = "none";
      if (progressEl) {
        progressEl.classList.remove("installing");
        progressEl.style.display = "flex";
        const oldBtn = progressEl.querySelector(".btn-cancel-download");
        if (oldBtn) oldBtn.remove();
        const cancelBtn = document.createElement("button");
        cancelBtn.type = "button";
        cancelBtn.className = "btn-cancel-download";
        const cancelLabel = getTranslation("model_cancel_download") || "Отменить загрузку";
        cancelBtn.title = cancelLabel;
        cancelBtn.setAttribute("aria-label", cancelLabel);
        cancelBtn.innerHTML = '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><line x1="18" y1="6" x2="6" y2="18"></line><line x1="6" y1="6" x2="18" y2="18"></line></svg>';
        cancelBtn.addEventListener("click", (e) => {
          e.stopPropagation();
          cancelBtn.disabled = true;
          invoke("cancel_model_download", { modelName: model }).catch((e2) => {
            console.error("Cancel model download error:", e2);
            cancelBtn.disabled = false;
          });
        });
        progressEl.appendChild(cancelBtn);
      }
      if (fillEl) {
        fillEl.style.width = "0%";
      }
      if (pctEl) {
        pctEl.textContent = "0%";
      }

      await invoke("download_model_command", { modelName: model });
      // Only auto-select when the user has not picked a different model
      // during the (potentially long) download.
      if (model !== "punctuation" && selectedModelName === modelAtStart) {
        selectModelCard(model);
      }
    } catch (err) {
      console.error(err);
      const errStr = String(err).toLowerCase();
      if (errStr.includes("cancel")) {
        showStatus(getTranslation("model_download_cancelled") || "Загрузка отменена");
      } else {
        showStatus(getTranslation("model_download_error_pattern", { err }), true);
      }
    } finally {
      inFlightModelDownloads.delete(model);
      if (progressEl) {
        progressEl.classList.remove("installing");
        const cancelBtn = progressEl.querySelector(".btn-cancel-download");
        if (cancelBtn) cancelBtn.remove();
        progressEl.style.display = "none";
      }
      if (actionEl) {
        actionEl.style.display = "flex";
      }
      await refreshDownloadedModels();
    }
  }

  async function deleteModelCard(model) {
    const confirmTitle = getTranslation("delete_model_title");
    const confirmMsg = getTranslation("delete_model_confirm_pattern", { model });
    const confirmBtn = getTranslation("delete_model_btn");
    const cancelBtn = getTranslation("confirm_cancel");

    const confirmed = await showConfirm(
      confirmTitle,
      confirmMsg,
      confirmBtn,
      cancelBtn
    );
    if (!confirmed) {
      return;
    }
    try {
      showStatus(getTranslation("model_deleting_pattern", { model }));
      await invoke("delete_model_command", { modelName: model });
      
showStatus(getTranslation("model_deleted_success"));
      if (model === "parakeet-v3" && selectLocalEngine?.value === "parakeet") {
        selectLocalEngine.value = "whisper";
        updateLocalEngineUI();
        markSettingsModified();
      }
      await refreshDownloadedModels();
    } catch (err) {
      console.error(err);
      showStatus(getTranslation("model_delete_error_pattern", { err }), true);
    }
  }

  // Cross-window sync: tray-driven changes, live history updates and
  // streaming degradation notices emitted by the Rust side.
  listen("settings-changed", () => {
    if (!isSettingsLoaded || saveInFlight || settingsModified) return;
    loadSettings();
  });

  listen("history-updated", () => {
    loadHistoryList();
  });

  listen("streaming-degraded", (event) => {
    console.warn("Streaming degraded:", event.payload);
    showStatus(getTranslation("status_streaming_degraded"), true);
  });

  // Listen to model-download-progress events from Rust
  listen("model-download-progress", (event) => {
    const payload = event.payload;
    if (!payload) return;

    const model = payload.model;
    const percent = typeof payload.percentage === "number" ? Math.round(payload.percentage) : 0;
    
    const fillEl = document.getElementById(`fill-${model}`);
    const pctEl = document.getElementById(`pct-${model}`);
    const progressEl = document.getElementById(`progress-${model}`);
    const actionEl = document.getElementById(`action-${model}`);
    
    if (fillEl && pctEl) {
      if (payload.status === "installing" || (percent >= 100 && !payload.done)) {
        if (progressEl) progressEl.classList.add("installing");
        fillEl.style.width = "100%";
        const installText = getTranslation("gpu_status_installing") || "Установка...";
        pctEl.innerHTML = `<span class="spinner-inline" aria-hidden="true"></span> ${installText}`;
      } else {
        if (progressEl) progressEl.classList.remove("installing");
        fillEl.style.width = `${percent}%`;
        pctEl.textContent = `${percent}%`;
      }
    }

    if (payload.done) {
      showStatus(getTranslation("model_downloaded_success_pattern", { model }));
      if (progressEl) {
        progressEl.classList.remove("installing");
        const cancelBtn = progressEl.querySelector(".btn-cancel-download");
        if (cancelBtn) cancelBtn.remove();
        progressEl.style.display = "none";
      }
      if (actionEl) actionEl.style.display = "flex";
      refreshDownloadedModels();
    }
  });

  // --- Local GPU Acceleration Logic ---
  async function checkGpuInstalled(provider) {
    if (provider === "cpu") return true;
    try {
      return await invoke("check_gpu_downloaded", { provider });
    } catch (e) {
      console.error(e);
      return false;
    }
  }

  function selectGpuProvider(provider) {
    if (activeLocalAcceleration !== provider) {
      activeLocalAcceleration = provider;
      markSettingsModified();
    }
    document.querySelectorAll("[data-gpu]").forEach(card => {
      const isSelected = card.getAttribute("data-gpu") === provider;
      card.setAttribute("aria-checked", isSelected ? "true" : "false");
      card.classList.toggle("active", isSelected);
    });
  }

  const activeGpuDownloads = new Set();
  // Whisper-model downloads in flight; refreshDownloadedModels must leave
  // their progress UI untouched until they finish or fail.
  const inFlightModelDownloads = new Set();

  async function updateGpuCardStates() {
    const providers = ["cuda"];
    const dict = i18nDict[currentLanguage] || i18nDict.ru;
    for (const provider of providers) {
      if (activeGpuDownloads.has(provider)) {
        continue;
      }
      const isDownloaded = await checkGpuInstalled(provider);
      const actionEl = document.getElementById(`action-gpu-${provider}`);
      const progressEl = document.getElementById(`progress-gpu-${provider}`);
      if (!actionEl) continue;
      if (progressEl) {
        const cancelBtn = progressEl.querySelector(".btn-cancel-download");
        if (cancelBtn) cancelBtn.remove();
        progressEl.style.display = "none";
      }
      actionEl.style.display = "flex";

      if (isDownloaded) {
        actionEl.innerHTML = `
          <span class="status-ready-badge">
            <svg class="status-ready-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3" stroke-linecap="round" stroke-linejoin="round"><polyline points="20 6 9 17 4 12"></polyline></svg>
            <span data-i18n="model_status_ready">${dict.model_status_ready || "Установлено"}</span>
          </span>
          <button type="button" class="btn-delete-card-model" title="${dict.model_action_delete || "Удалить"}" data-gpu="${provider}">
            <svg class="btn-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="3 6 5 6 21 6"></polyline><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"></path><line x1="10" y1="11" x2="10" y2="17"></line><line x1="14" y1="11" x2="14" y2="17"></line></svg>
          </button>
        `;
        actionEl.querySelector(".btn-delete-card-model").addEventListener("click", (e) => {
          e.stopPropagation();
          deleteGpuBinaries(provider);
        });
      } else {
        actionEl.innerHTML = `
          <button type="button" class="btn-download-card-model" data-gpu="${provider}">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="btn-icon"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"></path><polyline points="7 10 12 15 17 10"></polyline><line x1="12" y1="15" x2="12" y2="3"></line></svg>
            ${dict.model_action_download || "Скачать"}
          </button>
        `;
        actionEl.querySelector(".btn-download-card-model").addEventListener("click", (e) => {
          e.stopPropagation();
          downloadGpuBinaries(provider);
        });
      }
    }
  }

  async function downloadGpuBinaries(provider) {
    if (activeGpuDownloads.has(provider)) {
      return;
    }

    let acceptedNvidiaTerms = false;
    if (provider === "cuda") {
      let nvidiaRuntimeOnPath = false;
      try {
        nvidiaRuntimeOnPath = await invoke("check_nvidia_runtime_on_path");
      } catch (error) {
        console.error("Failed to check NVIDIA runtime on PATH", error);
      }
      if (!nvidiaRuntimeOnPath) {
        const dict = i18nDict[currentLanguage] || i18nDict.ru;
        acceptedNvidiaTerms = await showConfirm(
          dict.cuda_license_title,
          dict.cuda_license_message,
          dict.cuda_license_accept,
          dict.confirm_cancel,
          [
            {
              label: dict.cuda_license_cuda_link,
              url: "https://docs.nvidia.com/cuda/archive/11.8.0/eula/index.html"
            },
            {
              label: dict.cuda_license_cudnn_link,
              url: "https://docs.nvidia.com/deeplearning/cudnn/archives/cudnn-850/sla/index.html"
            }
          ]
        );
        if (!acceptedNvidiaTerms) return;
      }
    }

    const actionEl = document.getElementById(`action-gpu-${provider}`);
    const progressEl = document.getElementById(`progress-gpu-${provider}`);
    const fillEl = document.getElementById(`fill-gpu-${provider}`);
    const percentEl = document.getElementById(`pct-gpu-${provider}`);
    
    activeGpuDownloads.add(provider);
    if (actionEl) actionEl.style.display = "none";
    if (progressEl) {
      progressEl.style.display = "flex";
      const oldBtn = progressEl.querySelector(".btn-cancel-download");
      if (oldBtn) oldBtn.remove();
      const cancelBtn = document.createElement("button");
      cancelBtn.type = "button";
      cancelBtn.className = "btn-cancel-download";
      const cancelLabel = getTranslation("model_cancel_download") || "Отменить загрузку";
      cancelBtn.title = cancelLabel;
      cancelBtn.setAttribute("aria-label", cancelLabel);
      cancelBtn.innerHTML = '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><line x1="18" y1="6" x2="6" y2="18"></line><line x1="6" y1="6" x2="18" y2="18"></line></svg>';
cancelBtn.addEventListener("click", (e) => {
        e.stopPropagation();
        cancelBtn.disabled = true;
        invoke("cancel_gpu_download", { provider }).catch((e2) => {
          console.error(e2);
          cancelBtn.disabled = false;
        });
      });
      progressEl.appendChild(cancelBtn);
    }
    if (fillEl) fillEl.style.width = "0%";
    if (percentEl) percentEl.textContent = "0%";

    try {
      showStatus(getTranslation("model_downloading_pattern", { model: provider.toUpperCase() }));
      await invoke("download_gpu_binaries", { provider, acceptedNvidiaTerms });
    } catch (err) {
      console.error(err);
      const errStr = String(err).toLowerCase();
      if (errStr.includes("cancel")) {
        showStatus(getTranslation("model_download_cancelled") || "Загрузка отменена");
      } else {
        showStatus(`${getTranslation("status_error")}${err}`, true);
      }
    } finally {
      activeGpuDownloads.delete(provider);
      await updateGpuCardStates();
    }
  }

  async function deleteGpuBinaries(provider) {
    const dict = i18nDict[currentLanguage] || i18nDict.ru;
    const title = dict.delete_model_title || "Удаление";
    const message = dict.confirm_message || "Вы действительно хотите выполнить это действие?";
    const confirmText = dict.confirm_ok || "Удалить";
    const cancelText = dict.confirm_cancel || "Отмена";
    
    const confirmed = await showConfirm(title, message, confirmText, cancelText);
    if (confirmed) {
      try {
        await invoke("delete_gpu_binaries", { provider });
        if (activeLocalAcceleration === provider) {
          selectGpuProvider("cpu");
        }
        await updateGpuCardStates();
      } catch (err) {
        console.error(err);
        showStatus(getTranslation("model_delete_error_pattern", { err: String(err) }), true);
      }
    }
  }

  // Listen to GPU download progress events
  listen("gpu-download-progress", event => {
    const progress = event.payload;
    if (!progress) return;
    const fillEl = document.getElementById(`fill-gpu-${progress.provider}`);
    const percentEl = document.getElementById(`pct-gpu-${progress.provider}`);
    const progressEl = document.getElementById(`progress-gpu-${progress.provider}`);
    const actionEl = document.getElementById(`action-gpu-${progress.provider}`);

    if (fillEl && percentEl) {
      if (progress.status === "installing") {
        if (progressEl) progressEl.classList.add("installing");
        fillEl.style.width = "100%";
        const installText = getTranslation("gpu_status_installing") || "Установка...";
        percentEl.innerHTML = `<span class="spinner-inline" aria-hidden="true"></span> ${installText}`;
      } else {
        if (progressEl) progressEl.classList.remove("installing");
        const percentage = typeof progress.percentage === 'number' ? Math.round(progress.percentage) : 0;
        fillEl.style.width = `${percentage}%`;
        percentEl.textContent = `${percentage}%`;
      }
    }

    if (progress.done) {
      if (progressEl) {
        progressEl.classList.remove("installing");
        const cancelBtn = progressEl.querySelector(".btn-cancel-download");
        if (cancelBtn) cancelBtn.remove();
        progressEl.style.display = "none";
      }
      if (actionEl) actionEl.style.display = "flex";
      updateGpuCardStates();
    }
  });

  // Bind GPU card event listeners
  document.querySelectorAll("[data-gpu]").forEach(card => {
    card.addEventListener("click", async (e) => {
      // Prevent selection trigger when clicking delete/download buttons inside the card
      if (e.target.closest(".btn-delete-card-model") || e.target.closest(".btn-download-card-model")) {
        return;
      }
      const provider = card.getAttribute("data-gpu");
      const installed = await checkGpuInstalled(provider);
      if (installed) {
        selectGpuProvider(provider);
        kickEngineStatusPolling();
      }
    });

card.addEventListener("keydown", async (e) => {
      if (e.target.tagName === "BUTTON" || e.target.closest("button")) {
        return;
      }
      if (e.key === "Enter" || e.key === " ") {
        e.preventDefault();
        const provider = card.getAttribute("data-gpu");
        const installed = await checkGpuInstalled(provider);
        if (installed) {
          selectGpuProvider(provider);
          kickEngineStatusPolling();
        }
        return;
      }
      const direction = arrowDirection[e.key];
      if (direction === undefined) {
        return;
      }
      e.preventDefault();
      const group = card.parentElement;
      if (!group) {
        return;
      }
      const groupCards = Array.from(group.querySelectorAll("[data-gpu]"));
      const index = groupCards.indexOf(card);
      if (index === -1) {
        return;
      }
      let next = groupCards[(index + direction + groupCards.length) % groupCards.length];
      // Skip disabled/hidden cards (e.g. DirectML)
      while (next !== card && (next.hasAttribute("hidden") || next.getAttribute("aria-disabled") === "true")) {
        const nextIndex = groupCards.indexOf(next);
        next = groupCards[(nextIndex + direction + groupCards.length) % groupCards.length];
      }
      if (next === card) {
        return;
      }
      next.focus();
      const provider = next.getAttribute("data-gpu");
      const installed = await checkGpuInstalled(provider);
      if (installed) {
        selectGpuProvider(provider);
      }
    });
  });

  // --- Asynchronous native confirmation dialog ---
  function showConfirm(title, message, confirmText = "ОК", cancelText = "Отмена", links = []) {
    return new Promise((resolve) => {
      const modal = document.getElementById("custom-confirm-modal");
      const titleEl = document.getElementById("confirm-modal-title");
      const msgEl = document.getElementById("confirm-modal-message");
      const linksEl = document.getElementById("confirm-modal-links");
      const btnOk = document.getElementById("btn-confirm-ok");
      const btnCancel = document.getElementById("btn-confirm-cancel");
      if (!(modal instanceof HTMLDialogElement) || !titleEl || !msgEl || !linksEl || !btnOk || !btnCancel) {
        resolve(false);
        return;
      }

      titleEl.textContent = title;
      msgEl.textContent = message;
      btnOk.textContent = confirmText;
      btnCancel.textContent = cancelText;
      linksEl.replaceChildren();
      linksEl.hidden = links.length === 0;
      if (links.length > 0) {
        const dict = i18nDict[currentLanguage] || i18nDict.ru;
        const prefix = document.createElement("span");
        prefix.className = "modal-links-prefix";
        prefix.textContent = dict.cuda_license_footnote || "Лицензии:";
        linksEl.appendChild(prefix);

        links.forEach(({ label, url }, idx) => {
          if (idx > 0) {
            const sep = document.createElement("span");
            sep.className = "modal-links-sep";
            sep.textContent = "•";
            linksEl.appendChild(sep);
          }
          const button = document.createElement("button");
          button.type = "button";
          button.className = "modal-link-button";
          button.innerHTML = `<span>${label}</span><svg class="link-icon" width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6"></path><polyline points="15 3 21 3 21 9"></polyline><line x1="10" y1="14" x2="21" y2="3"></line></svg>`;
          button.addEventListener("click", () => {
            invoke("open_url", { url }).catch(console.error);
          });
          linksEl.appendChild(button);
        });
      }
      if (modal.open) modal.close();
      modal.showModal();
      requestAnimationFrame(() => modal.classList.add("active"));

      let settled = false;
      function cleanUp(result) {
        if (settled) return;
        settled = true;
        modal.classList.remove("active");
        btnOk.removeEventListener("click", onOk);
        btnCancel.removeEventListener("click", onCancel);
        modal.removeEventListener("cancel", onDialogCancel);
        linksEl.replaceChildren();
        linksEl.hidden = true;
        setTimeout(() => {
          if (modal.open) modal.close();
          resolve(result);
        }, 200);
      }
      function onOk() { cleanUp(true); }
      function onCancel() { cleanUp(false); }
      function onDialogCancel(event) {
        event.preventDefault();
        cleanUp(false);
      }
      btnOk.addEventListener("click", onOk);
      btnCancel.addEventListener("click", onCancel);
      modal.addEventListener("cancel", onDialogCancel);
    });
  }
  function showStatus(msg, isError = false, isModified = false) {
    footerStatusText.textContent = msg;
    const footerStatus = footerStatusText.closest(".footer-status");
    
    if (footerStatus) {
      footerStatus.classList.remove("modified", "error", "success");
      const iconEl = document.getElementById("footer-status-icon");
      if (isError) {
        footerStatus.classList.add("error");
        footerStatusText.style.color = "var(--status-error)";
        if (iconEl) {
          iconEl.innerHTML = `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3" stroke-linecap="round" stroke-linejoin="round"><line x1="12" y1="9" x2="12" y2="13"></line><line x1="12" y1="17" x2="12.01" y2="17"></line><circle cx="12" cy="12" r="10"></circle></svg>`;
        }
      } else if (isModified) {
        footerStatus.classList.add("modified");
        footerStatusText.style.color = "var(--status-modified)";
        if (iconEl) {
          iconEl.innerHTML = `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><path d="M12 20h9"></path><path d="M16.5 3.5a2.121 2.121 0 0 1 3 3L7 19l-4 1 1-4L16.5 3.5z"></path></svg>`;
        }
      } else {
        footerStatus.classList.add("success");
        footerStatusText.style.color = "";
        if (iconEl) {
          iconEl.innerHTML = `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3" stroke-linecap="round" stroke-linejoin="round"><polyline points="20 6 9 17 4 12"></polyline></svg>`;
        }
      }
    }
  }

  // Bind Events
  if (btnSaveSettings) {
    btnSaveSettings.addEventListener("click", saveSettings);
  }
  selectProvider.addEventListener("change", () => {
    // Keep only unsaved drafts in memory; stored keys are never returned by IPC.
    apiKeys[previousSelProvider] = apiKeyInput.value;
    previousSelProvider = selectProvider.value;
    renderProviderKeyInput();
    updateApiKeyLink();
    updateCustomProviderUI();
  });

  // Window controls via Tauri IPC commands
  const btnWindowMinimize = document.getElementById("btn-window-minimize");
  const btnWindowClose = document.getElementById("btn-window-close");
  
  if (btnWindowMinimize) {
btnWindowMinimize.addEventListener("click", () => invoke("minimize_window").catch(e => console.error(e)));
  }
  if (btnWindowClose) {
    btnWindowClose.addEventListener("click", () => invoke("close_window").catch(e => console.error(e)));
  }

  // Window dragging via mousedown on header (bypasses click-through/drag bugs in Webview2)
  const appHeader = document.querySelector(".app-header");
  if (appHeader) {
    appHeader.addEventListener("mousedown", (e) => {
      // Only trigger drag on left click and avoid dragging when clicking on control buttons or select elements
if (e.button === 0 && !e.target.closest(".window-control-btn") && !e.target.closest("button") && !e.target.closest("select")) {
        invoke("start_dragging_command").catch(e => console.error(e));
      }
    });
  }

  // Translations Helper
  function applyLanguage(lang) {
    currentLanguage = lang;
    document.documentElement.lang = i18nDict[lang] ? lang : "ru";
    const dict = i18nDict[lang] || i18nDict.ru;
    
    // Update data-i18n elements
    const elements = document.querySelectorAll("[data-i18n]");
    elements.forEach(el => {
      const key = el.getAttribute("data-i18n");
      const text = dict[key] || i18nDict.ru[key];
      if (text) {
        if (el.tagName === "INPUT" || el.tagName === "TEXTAREA") {
          el.placeholder = text;
        } else {
          el.textContent = text;
        }
      }
    });

    const selectUiLang = document.getElementById("select-ui-lang");
    if (selectUiLang) {
      selectUiLang.setAttribute("aria-label", dict.general_ui_lang_title || "UI Language");
    }

    const btnReset = document.getElementById("btn-reset-hotkey");
    if (btnReset) {
      btnReset.setAttribute("title", dict.hotkey_reset_title || "Сбросить на Alt+V");
    }

    
    // Update inputs and placeholders
    const apiInput = document.getElementById("input-api-key");
    if (apiInput) {
      apiInput.placeholder = dict.api_key_placeholder || "";
    }
    const dictionaryTextarea = document.getElementById("textarea-dictionary");
    if (dictionaryTextarea) {
      dictionaryTextarea.placeholder = dict.vocab_placeholder || "";
    }
    const hotkeyInput = document.getElementById("input-hotkey");
    if (hotkeyInput) {
      hotkeyInput.placeholder = dict.hotkey_prompt || "";
    }
    const historySearchInput = document.getElementById("input-history-search");
    if (historySearchInput) {
      historySearchInput.placeholder = dict.history_search_placeholder || "";
    }
    
    // Update dynamic link text
    updateApiKeyLink();
    
    // Refresh model cards status/actions
    refreshDownloadedModels();
    updateGpuCardStates();
    
    // If settings modified status is showing, update it
    if (settingsModified) {
      showStatus(dict.status_modified, false, true);
    }
    
    // Reload history list if the active panel is panel-history
    const historyTab = document.getElementById("tab-btn-history");
    if (historyTab && historyTab.classList.contains("active")) {
      loadHistoryList();
    }

    rebuildSelectPanels();
  }


  // Microphone Meter Test
  const btnToggleMicMeter = document.getElementById("btn-toggle-mic-meter");
  const micMeterFill = document.getElementById("mic-meter-fill");
  const micMeterPeak = document.getElementById("mic-meter-peak");
  const micVadIndicator = document.getElementById("mic-vad-indicator");
  const micVadText = document.getElementById("mic-vad-text");
  let isMicTesting = false;
  let rawVolumeTarget = 0;
  let smoothedVolume = 0;
  let peakVolume = 0;
  let peakHoldFrames = 0;
  let micMeterAnimId = null;
  let speechHangoverTimer = null;
  let isSpeechActive = false;

  function setSpeechState(speechDetected) {
    if (speechDetected) {
      if (speechHangoverTimer) {
        clearTimeout(speechHangoverTimer);
        speechHangoverTimer = null;
      }
      if (!isSpeechActive) {
        isSpeechActive = true;
        if (micVadIndicator) micVadIndicator.classList.add("speaking");
        if (micVadText) micVadText.textContent = getTranslation("mic_meter_speech") || "Речь";
      }
    } else {
      if (isSpeechActive && !speechHangoverTimer) {
        speechHangoverTimer = setTimeout(() => {
          isSpeechActive = false;
          speechHangoverTimer = null;
          if (micVadIndicator) micVadIndicator.classList.remove("speaking");
          if (micVadText) micVadText.textContent = getTranslation("mic_meter_silence") || "Тишина";
        }, 220);
      }
    }
  }

  function runMicMeterPhysics() {
    if (!isMicTesting) return;

    // Fast responsive attack (0.35), smooth natural decay (0.08)
    if (rawVolumeTarget > smoothedVolume) {
      smoothedVolume += (rawVolumeTarget - smoothedVolume) * 0.35;
    } else {
      smoothedVolume += (rawVolumeTarget - smoothedVolume) * 0.08;
    }
    if (smoothedVolume < 0.3) smoothedVolume = 0;

    // Peak hold & gentle smooth falloff
    if (smoothedVolume > peakVolume) {
      peakVolume = smoothedVolume;
      peakHoldFrames = 25; // ~400ms hold
    } else if (peakHoldFrames > 0) {
      peakHoldFrames--;
    } else {
      peakVolume = Math.max(0, peakVolume - 0.7);
    }

    if (micMeterFill) {
      micMeterFill.style.width = `${smoothedVolume.toFixed(1)}%`;
    }
    if (micMeterPeak) {
      micMeterPeak.style.left = `${peakVolume.toFixed(1)}%`;
      micMeterPeak.style.opacity = peakVolume > 1.0 ? "1" : "0";
    }

    micMeterAnimId = requestAnimationFrame(runMicMeterPhysics);
  }

  async function stopMicTesting() {
    if (!isMicTesting) return;
    isMicTesting = false;
    if (micMeterAnimId) {
      cancelAnimationFrame(micMeterAnimId);
      micMeterAnimId = null;
    }
    if (speechHangoverTimer) {
      clearTimeout(speechHangoverTimer);
      speechHangoverTimer = null;
    }
    isSpeechActive = false;
    rawVolumeTarget = 0;
    smoothedVolume = 0;
    peakVolume = 0;
    peakHoldFrames = 0;

    try {
      await invoke("stop_mic_meter");
    } catch (e) {
      console.error("Failed to stop mic meter:", e);
    }
    if (btnToggleMicMeter) {
      btnToggleMicMeter.textContent = getTranslation("mic_meter_test_btn_start") || "Запустить тест";
      btnToggleMicMeter.classList.remove("active");
    }
    if (micMeterFill) micMeterFill.style.width = "0%";
    if (micMeterPeak) {
      micMeterPeak.style.left = "0%";
      micMeterPeak.style.opacity = "0";
    }
    if (micVadIndicator) {
      micVadIndicator.classList.remove("speaking");
    }
    if (micVadText) {
      micVadText.textContent = getTranslation("mic_meter_silence") || "Тишина";
    }
  }

  if (btnToggleMicMeter) {
    btnToggleMicMeter.addEventListener("click", async () => {
      if (isMicTesting) {
        await stopMicTesting();
      } else {
        isMicTesting = true;
        rawVolumeTarget = 0;
        smoothedVolume = 0;
        peakVolume = 0;
        peakHoldFrames = 0;
        isSpeechActive = false;
        if (speechHangoverTimer) {
          clearTimeout(speechHangoverTimer);
          speechHangoverTimer = null;
        }

        try {
          await invoke("start_mic_meter");
          btnToggleMicMeter.textContent = getTranslation("mic_meter_test_btn_stop") || "Остановить тест";
          btnToggleMicMeter.classList.add("active");
          micMeterAnimId = requestAnimationFrame(runMicMeterPhysics);
        } catch (err) {
          console.error("Failed to start mic test:", err);
          await stopMicTesting();
          showStatus(`${getTranslation("mic_start_error") || "Ошибка микрофона: "}${err}`, true);
        }
      }
    });

    listen("mic-meter-level", (event) => {
      if (!isMicTesting) return;
      const { volume, is_speech } = event.payload || {};
      const perceptual = Math.pow(Math.min(1.0, (volume || 0) * 2.0), 0.6);
      rawVolumeTarget = perceptual * 100;

      setSpeechState(!!is_speech);
    });
  }

  window.addEventListener("beforeunload", () => {
    if (isMicTesting) {
      invoke("stop_mic_meter").catch(() => {});
    }
  });

  // --- History List, Search, Filter & Clear Interactions ---
  const historyContainer = document.getElementById("history-items-container");
  const btnClearHistory = document.getElementById("btn-clear-history");
  const inputHistorySearch = document.getElementById("input-history-search");
  const btnClearHistorySearch = document.getElementById("btn-clear-history-search");
  const historyFilterButtons = document.querySelectorAll(".history-filter-btn");

  let cachedHistoryList = [];
  let historySearchQuery = "";
  let historyActiveFilter = "all";

  function renderHistoryItems() {
    if (!historyContainer) return;
    const dict = i18nDict[currentLanguage] || i18nDict.ru;

    if (!cachedHistoryList || cachedHistoryList.length === 0) {
      historyContainer.innerHTML = `<div class="history-empty-state" id="history-empty-text" data-i18n="history_empty">${dict.history_empty}</div>`;
      return;
    }

    const q = (historySearchQuery || "").trim().toLowerCase();
    const filtered = cachedHistoryList.filter(entry => {
      // Filter by mode
      if (historyActiveFilter === "cloud" && entry.mode !== "cloud") return false;
      if (historyActiveFilter === "local" && entry.mode === "cloud") return false;

      // Filter by search query
      if (q && !(entry.text || "").toLowerCase().includes(q)) {
        return false;
      }
      return true;
    });

    if (filtered.length === 0) {
      historyContainer.innerHTML = `<div class="history-empty-state" id="history-empty-text" data-i18n="history_empty">${dict.history_empty}</div>`;
      return;
    }

    historyContainer.innerHTML = "";
    const fragment = document.createDocumentFragment();

    function formatHistoryDuration(ms) {
      if (!ms) return "";
      if (ms < 1000) return `${ms} ${dict.history_unit_ms || "ms"}`;
      const secs = ms >= 10000 ? Math.round(ms / 1000) : Math.round((ms / 1000) * 10) / 10;
      return `${secs} ${dict.history_unit_sec || "s"}`;
    }

    filtered.forEach(entry => {
      const date = new Date(entry.timestamp_ms);
      const timeStr = date.toLocaleTimeString(currentLanguage, { hour: '2-digit', minute: '2-digit', second: '2-digit' });
      const dateStr = date.toLocaleDateString(currentLanguage, { month: 'short', day: 'numeric' });
      const displayTime = `${dateStr}, ${timeStr}`;

      const itemEl = document.createElement("div");
      itemEl.className = "history-item";

      let badgeHtml;
      const engineLabel = dict[`history_engine_${entry.engine}`];
      if (engineLabel) {
        const durationHtml = entry.processing_ms
          ? `<span class="history-item-duration">${formatHistoryDuration(entry.processing_ms)}</span>`
          : "";
        badgeHtml =
          `<span class="history-item-badge badge-local">${escapeHtml(engineLabel)}</span>${durationHtml}`;
      } else if (entry.mode === "cloud") {
        badgeHtml = `<span class="history-item-badge badge-cloud">${escapeHtml(dict.history_badge_cloud || "Cloud")}</span>`;
      } else {
        badgeHtml = `<span class="history-item-badge badge-local">${escapeHtml(dict.history_badge_local || "Local")}</span>`;
      }

      itemEl.innerHTML = `
        <div class="history-item-body">
          <div class="history-item-meta">
            <span class="history-item-time">${displayTime}</span>
            ${badgeHtml}
          </div>
          <div class="history-item-text">${escapeHtml(entry.text)}</div>
        </div>
        <button type="button" class="btn-copy-history" title="Copy to clipboard">
          <svg class="copy-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <rect x="9" y="9" width="13" height="13" rx="2" ry="2"></rect>
            <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"></path>
          </svg>
          <svg class="check-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" style="display: none; color: var(--accent-color);">
            <polyline points="20 6 9 17 4 12"></polyline>
          </svg>
        </button>
      `;

      // Bind copy event
      const btnCopy = itemEl.querySelector(".btn-copy-history");
      const copyIcon = itemEl.querySelector(".copy-icon");
      const checkIcon = itemEl.querySelector(".check-icon");

      btnCopy.addEventListener("click", async () => {
        try {
          await invoke("copy_to_clipboard", { text: entry.text });
          
          // Hide copy icon, show checkmark SVG
          copyIcon.style.display = "none";
          checkIcon.style.display = "block";
          
          if (btnCopy._copyTimeout) {
            clearTimeout(btnCopy._copyTimeout);
          }
          
          btnCopy._copyTimeout = setTimeout(() => {
            checkIcon.style.display = "none";
            copyIcon.style.display = "block";
            btnCopy._copyTimeout = null;
          }, 1500);
        } catch (err) {
          console.error("Failed to copy", err);
        }
      });

      fragment.appendChild(itemEl);
    });
    historyContainer.appendChild(fragment);
  }

  async function loadHistoryList() {
    if (!historyContainer) return;
    try {
      cachedHistoryList = (await invoke("get_history")) || [];
      renderHistoryItems();
    } catch (err) {
      console.error("Failed to load history", err);
    }
  }

  if (inputHistorySearch) {
    inputHistorySearch.addEventListener("input", () => {
      historySearchQuery = inputHistorySearch.value;
      if (btnClearHistorySearch) {
        btnClearHistorySearch.style.display = historySearchQuery ? "flex" : "none";
      }
      renderHistoryItems();
    });
  }

  if (btnClearHistorySearch) {
    btnClearHistorySearch.addEventListener("click", () => {
      if (inputHistorySearch) {
        inputHistorySearch.value = "";
        inputHistorySearch.focus();
      }
      historySearchQuery = "";
      btnClearHistorySearch.style.display = "none";
      renderHistoryItems();
    });
  }

  historyFilterButtons.forEach(btn => {
    btn.addEventListener("click", () => {
      historyFilterButtons.forEach(b => b.classList.remove("active"));
      btn.classList.add("active");
      historyActiveFilter = btn.dataset.filter || "all";
      renderHistoryItems();
    });
  });

  function escapeHtml(text) {
    return (text || "")
      .replace(/&/g, "&amp;")
      .replace(/</g, "&lt;")
      .replace(/>/g, "&gt;")
      .replace(/"/g, "&quot;")
      .replace(/'/g, "&#039;");
  }

  const btnCopyDiagnostics = document.getElementById("btn-copy-diagnostics");
  if (btnCopyDiagnostics) {
    btnCopyDiagnostics.addEventListener("click", async () => {
      try {
        const report = await invoke("get_diagnostic_report");
        try {
          await invoke("copy_to_clipboard", { text: report });
        } catch (e) {
          if (navigator.clipboard && navigator.clipboard.writeText) {
            await navigator.clipboard.writeText(report);
          } else {
            throw e;
          }
        }
        showStatus(getTranslation("toast_diagnostics_copied"));
      } catch (err) {
        console.error("Failed to copy diagnostic report", err);
        showStatus(`${getTranslation("status_error")}${err}`, true);
      }
    });
  }

  if (btnClearHistory) {
    btnClearHistory.addEventListener("click", async () => {
      const dict = i18nDict[currentLanguage] || i18nDict.ru;
      const confirmed = await showConfirm(
        dict.confirm_clear_history_title,
        dict.confirm_clear_history_msg,
        dict.confirm_ok,
        dict.confirm_cancel
      );
      if (confirmed) {
        try {
          await invoke("clear_history");
          loadHistoryList();
        } catch (err) {
          console.error("Failed to clear history", err);
        }
      }
    });
  }

  let updateAvailable = false;

  async function checkForUpdates(announceNoUpdate = false) {
    const checkButton = document.getElementById("btn-check-updates");
    const badge = document.getElementById("update-badge");
    const badgeText = document.getElementById("update-badge-text");
    const navDot = document.getElementById("update-dot");
    if (checkButton) checkButton.disabled = true;
    try {
      const info = await invoke("check_for_app_update");
      updateAvailable = !!info;
      if (!info) {
        if (badge) badge.style.display = "none";
        if (navDot) navDot.style.display = "none";
        if (announceNoUpdate) showStatus(getTranslation("update_current"));
        return;
      }
      const label = getTranslation("update_available") || "Доступно обновление";
      if (badgeText) badgeText.textContent = label + " (v" + info.version + ")";
      if (badge) badge.style.display = "inline-flex";
      if (navDot) navDot.style.display = "inline-block";
      if (announceNoUpdate) showStatus(getTranslation("update_available_pattern", { version: info.version }));
    } catch (error) {
      console.error("Update check failed", error);
      if (announceNoUpdate) showStatus(getTranslation("update_check_error_pattern", { error: String(error) }), true);
    } finally {
      if (checkButton) checkButton.disabled = false;
    }
  }

  async function installAvailableUpdate() {
    if (!updateAvailable) {
      await checkForUpdates(true);
      if (!updateAvailable) return;
    }
    try {
      showStatus(getTranslation("update_installing"));
      await invoke("install_app_update");
      showStatus(getTranslation("update_installed_restarting"));
      await invoke("relaunch_app");
    } catch (error) {
      console.error("Update installation failed", error);
      showStatus(getTranslation("update_install_error_open_release"), true);
      invoke("open_url", {
        url: "https://github.com/malashkadev/aura/releases/latest"
      }).catch((openError) => console.error("Failed to open release page", openError));
    }
  }

  const checkUpdatesButton = document.getElementById("btn-check-updates");
  if (checkUpdatesButton) {
    checkUpdatesButton.addEventListener("click", () => checkForUpdates(true));
  }
  const updateBadge = document.getElementById("update-badge");
  if (updateBadge) {
    updateBadge.addEventListener("click", installAvailableUpdate);
  }
  // Initialize UI language and Settings
  (async () => {
    const supportedLangs = ["ru", "en", "de", "es", "fr", "it", "zh", "pt", "tr"];
    let legacyUiLang = localStorage.getItem("aura_ui_lang");
    if (legacyUiLang === null) {
      legacyUiLang = localStorage.getItem("ui-language");
    }
    if (!supportedLangs.includes(legacyUiLang)) {
      legacyUiLang = null;
    }
    const browserUiLang = navigator.language.toLowerCase().split(/[-_]/)[0];
    const provisionalUiLang = legacyUiLang || (supportedLangs.includes(browserUiLang) ? browserUiLang : "en");
    applyLanguage(provisionalUiLang);

    let settings = null;
    try {
      settings = await invoke("get_settings");
    } catch (err) {
      console.error(err);
    }

    const backendUiLang = supportedLangs.includes(settings?.ui_language) ? settings.ui_language : null;
    const savedUiLang = legacyUiLang || backendUiLang || provisionalUiLang;
    localStorage.setItem("aura_ui_lang", savedUiLang);
    localStorage.setItem("ui-language", savedUiLang);
    if (settings && settings.ui_language !== savedUiLang) {
      invoke("set_ui_language", { uiLanguage: savedUiLang }).catch(error => {
        console.error("Failed to migrate interface language", error);
      });
    }

    // UI Language Selector Setup
    const selectUiLang = document.getElementById("select-ui-lang");
    if (selectUiLang) {
      selectUiLang.value = savedUiLang;
      
      selectUiLang.addEventListener("change", async (e) => {
        const selectedLang = e.target.value;
        localStorage.setItem("aura_ui_lang", selectedLang);
        localStorage.setItem("ui-language", selectedLang);
        applyLanguage(selectedLang);
        try {
          await invoke("set_ui_language", { uiLanguage: selectedLang });
        } catch (error) {
          console.error("Failed to persist interface language", error);
        }
      });
    }
    
    // Apply initial language choice outside the if block so translations initialize even if #select-ui-lang is missing
    applyLanguage(savedUiLang);

    initSelectPanels();

    // Initialize Settings
    await loadSettings(settings);

    if (settings?.automatic_update_checks) {
      await checkForUpdates(false);
    }

  })();
});
