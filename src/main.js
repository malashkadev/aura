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
    "tab_text": "Текст",
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
    "engine_health_parakeet_gaming": "Parakeet: сервер выгружен (игровой режим)",
    "engine_health_whisper_running": "Whisper: сервер запущен ({provider}, порт {port})",
    "engine_health_whisper_stopped": "Whisper: сервер не запущен",
    "engine_health_whisper_gaming": "Whisper: сервер выгружен (игровой режим)",
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
    "sound_theme_bubble": "Капля воды",
    "sound_theme_haptic": "Тактильный свитч",
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
    "history_group_today": "Сегодня",
    "history_group_yesterday": "Вчера",
    "about_app_title": "Голосовой ввод Aura",
    "about_version": "v1.2.0",
    "about_description": "Инструмент глобального голосового ввода для Windows. Программа переводит речь в текст и вставляет его в любое активное окно с автоматическим форматированием и расстановкой пунктуации.",
    "about_gratitude_text": "Спасибо, что пользуетесь Aura! Если программа делает вашу работу быстрее и удобнее — поддержите проект звездой на GitHub.",
    "btn_star_github": "Поставить звезду на GitHub",
    "about_row_tour_title": "Вводный экскурс по возможностям",
    "about_row_tour_desc": "Интерактивный тур с подсказками по основным функциям",
    "about_row_changelog_title": "Что нового в этой версии",
    "about_row_changelog_desc": "Список изменений, улучшения производительности и релизы",
    "about_row_report_issue_title": "Сообщить об ошибке",
    "about_row_report_issue_desc": "Написать о проблеме разработчикам на GitHub",
    "about_row_star_title": "Поставить звезду на GitHub",
    "about_row_star_desc": "Поддержите проект в открытом репозитории",
    "about_btn_exit_app": "Выйти из Aura",
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
    "status_streaming_degraded": "Живой ввод приостановлен — полный текст появится после завершения записи",
    "model_downloading_pattern": "Запуск скачивания {model}...",
    "model_download_error_pattern": "Ошибка скачивания: {err}",
    "delete_model_title": "Удаление модели",
    "delete_model_confirm_pattern": "Вы действительно хотите удалить локальную модель {model}?",
    "delete_model_btn": "Удалить",
    "model_deleting_pattern": "Удаление модели {model}...",
    "model_deleted_success": "Модель успешно удалена",
    "model_delete_error_pattern": "Ошибка удаления: {err}",
    "model_downloaded_success_pattern": "Модель {model} скачана",
    "confirm_clear_history_title": "Очистить историю",
    "confirm_clear_history_msg": "Вы действительно хотите очистить всю историю транскрипций?",
    "general_ui_lang_title": "Язык интерфейса",
    "general_ui_lang_desc": "Выберите язык для отображения настроек и уведомлений приложения.",
    "update_checks_title": "Проверка обновлений",
    "update_checks_desc": "Aura обращается к GitHub только при ручной проверке или если вы включили автоматическую проверку.",
    "update_checks_checkbox": "Автоматически проверять обновления при запуске",
    "update_check_now": "Проверить обновления",
    "update_current": "Установлена актуальная версия Aura",
    "update_available_pattern": "Доступна Aura v{version}",
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
    "copy_context_desc": "Если включено, Aura отправляет Ctrl+C и передаёт выделенный текст провайдеру или локальной модели как контекст для команды редактирования. Отключите эту функцию при работе в терминале.",
    "copy_context_checkbox": "Разрешить редактирование выделенного текста",
    "ai_edit_title": "Команды над выделением",
    "ai_edit_desc": "Голосовые команды для выделенного текста: «переведи», «исправь грамматику». Команда применяется к выделению, обычная диктовка вставляется без изменений.",
    "ai_edit_mode_label": "Режим редактирования",
    "ai_edit_mode_auto": "Авто",
    "ai_edit_mode_cloud": "Облачный",
    "ai_edit_mode_local": "Локальный",
    "ai_edit_mode_off": "Выключен",
    "ai_edit_mode_sub": "Авто использует локальную модель, если она установлена, иначе облачный провайдер. Для команд над выделением включите «Разрешить захват выделения и команды редактирования».",
    "translate_target_label": "Язык перевода по умолчанию",
    "translate_target_sub": "Используется при голосовой команде «переведи» без указания языка.",
    "edit_cloud_provider_label": "Облачный сервис",
    "edit_cloud_provider_sub": "Модель, выполняющая команды перевода и исправления грамматики текста.",
    "edit_cloud_provider_fallback_sub": "Используется как резервный сервис, если локальная модель не установлена или недоступна.",
    "llm_model_label": "Локальная модель",
    "model_meta_llm_0_6b": "~460 МБ — минимальная, простые команды",
    "model_meta_llm_1_7b": "~1,2 ГБ — сбалансированная",
    "model_meta_llm_4b": "~2,3 ГБ — максимальное качество",
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
    "gpu_status_installing": "Установка...",
    "replacements_title": "Автозамена",
    "replacements_desc": "Автоматически заменяет надиктованные слова и фразы на точные термины, адреса почты или готовый текст.",
    "replacements_trigger_ph": "Слово или фраза (напр. почта)",
    "replacements_value_ph": "Заменить на (напр. name@mail.ru)",
    "replacements_caret_hint": "Символ | в тексте замены указывает, куда переместить курсор после вставки (напр. Здравствуйте, |!).",
    "replacements_macro_hint": "Доступные макросы: {date} (ГГГГ-ММ-ДД), {date_ru} (ДД.ММ.ГГГГ), {time} (ЧЧ:ММ), {datetime}.",
    "replacements_macros_label": "Вставить шаблон:",
    "macro_chip_date_ru": "+ Дата РФ",
    "macro_chip_date_iso": "+ Дата ISO",
    "macro_chip_time": "+ Время",
    "macro_chip_datetime": "+ Дата и время",
    "btn_add": "Добавить",
    "replacements_empty": "Правила автозамены пока не настроены",
    "replacements_delete_aria": "Удалить правило автозамены",
    "gaming_title": "Игровой режим",
    "gaming_desc": "Автоматическая пауза перехвата клавиш и выгрузка моделей для максимального FPS в играх.",
    "gaming_mode_label": "Включить игровой режим вручную",
    "gaming_mode_subdesc": "Отключает перехват клавиатуры и выгружает модели из видеопамяти (VRAM).",
    "gaming_auto_label": "Автоматический игровой режим",
    "gaming_auto_desc": "Приостанавливает горячие клавиши и освобождает видеопамять при запуске полноэкранных 3D-игр.",
    "gaming_status_label": "Текущее состояние",
    "gaming_status_inactive": "Не активен",
    "gaming_status_active": "Активен (пауза)",
    "gaming_btn_toggle_on": "Включить вручную",
    "gaming_btn_toggle_off": "Выключить",
    "gaming_tray_hint": "Ручное переключение также доступно через меню Aura в трее Windows.",
    "standby_title": "Выгрузка моделей при бездействии (Сон)",
    "standby_timeout_label": "Время бездействия до перехода в сон",
    "standby_desc": "Освобождает видеопамять (VRAM), если голосовой ввод не используется указанное время.",
    "standby_15m": "Через 15 минут",
    "standby_30m": "Через 30 минут (рекомендуется)",
    "standby_60m": "Через 1 час",
    "standby_never": "Никогда (держать в VRAM)",
    "window_minimize": "Свернуть",
    "window_close": "Закрыть",
    "history_copy_title": "Скопировать в буфер обмена",
    "history_search_clear": "Очистить поиск",
    "api_key_toggle_visibility": "Показать/скрыть ключ",
    "history_filters_label": "Фильтры истории",
    "onboarding_skip": "Пропустить обучение",
    "onboarding_prev": "Назад",
    "onboarding_next": "Далее",
    "onboarding_finish": "Начать работу",
    "onboarding_replay": "Вводный экскурс по возможностям",
    "onboarding_step_tag": "{current} / {total}",
    "onboarding_step_1_title": "Способы распознавания речи",
    "onboarding_step_1_desc": "Локальный ИИ обеспечивает 100% приватность без передачи аудио в сеть и ускоряется видеокартой (NVIDIA CUDA) или CPU. Облачный ИИ не требует загрузки моделей и быстро расшифровывает через API без нагрузки на систему.",
    "onboarding_step_2_title": "Язык распознавания и словарь",
    "onboarding_step_2_desc": "Аура умеет автоматически подстраиваться под язык активной раскладки клавиатуры. В словаре можно задать правила автозамены опечаток и расшифровки профессиональных терминов.",
    "onboarding_step_3_title": "Умная вставка и редактирование выделенного",
    "onboarding_step_3_desc": "Выделите текст в любой программе, нажмите хоткей и надиктуйте команду: «исправь ошибки» или «переведи» — Аура выполнит редактирование прямо на месте.",
    "onboarding_step_4_title": "Горячие клавиши, оверлей и звуковой отклик",
    "onboarding_step_4_desc": "Главная клавиша запускает запись в любой программе. При старте и завершении звучит приятный щелчок, а на экране появляется плавающий таймер со статусом распознавания.",
    "onboarding_step_5_title": "История записей и тихая работа в трее",
    "onboarding_step_5_desc": "Все надиктованные тексты сохраняются в истории для повторного копирования. Аура тихо сворачивается в системный трей и всегда готова к мгновенной работе по горячей клавише.",
    "commands_guide_title": "Поддерживаемые голосовые команды",
    "commands_guide_desc": "Доступные действия над выделенным текстом.",
    "commands_guide_label": "Поддерживаемые команды",
    "commands_guide_sub": "Голосовые фразы для действий над выделенным текстом",
    "commands_guide_btn": "Справочник команд",
    "commands_guide_close": "Закрыть",
    "commands_guide_active_summary": "{active} из {total} команд активно",
    "smart_context_title": "Умная контекстная вставка",
    "smart_context_desc": "Автоматически адаптирует регистр букв, знаки препинания и пробелы под положение курсора в документе.",
    "smart_context_checkbox": "Учитывать контекст вокруг курсора",
    "smart_context_subdesc": "Начинает со строчной буквы при вставке в середину предложения, убирает лишние точки и аккуратно расставляет пробелы.",
    "smart_context_trailing_period_checkbox": "Завершать точкой при дополнении в конце предложения",
    "smart_context_trailing_period_subdesc": "Если курсор стоит в конце незавершенной фразы, завершает фразу точкой. При выключении оставляет конец открытым для дальнейшей речи.",
    "sound_preview_btn": "Прослушать тему",
    "sound_preview_label": "Тест",
    "api_key_status_saved": "Сохранен в Windows DPAPI",
    "api_key_shared_hint": "Единый ключ Aura: действует для диктовки и AI-редактирования.",
    "api_key_btn_change": "Изменить"
  },
  "en": {
    "title_settings": "Settings",
    "tab_general": "General",
    "tab_speech": "Speech",
    "tab_text": "Text",
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
    "engine_health_parakeet_gaming": "Parakeet: server unloaded (Gaming Mode active)",
    "engine_health_whisper_running": "Whisper: server running ({provider}, port {port})",
    "engine_health_whisper_stopped": "Whisper: server not running",
    "engine_health_whisper_gaming": "Whisper: server unloaded (Gaming Mode active)",
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
    "sound_theme_bubble": "Water Drop",
    "sound_theme_haptic": "Haptic Switch",
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
    "history_group_today": "Today",
    "history_group_yesterday": "Yesterday",
    "about_app_title": "Aura Voice Input",
    "about_version": "v1.2.0",
    "about_description": "Global voice input tool for Windows. The program transcribes speech to text and inserts it into any active window with automatic formatting and punctuation.",
    "about_gratitude_text": "Thank you for using Aura! If it makes your work faster and easier, please consider supporting the project with a star on GitHub.",
    "btn_star_github": "Star on GitHub",
    "about_row_tour_title": "Feature Tour",
    "about_row_tour_desc": "Interactive walkthrough of core features",
    "about_row_changelog_title": "What's New in this Version",
    "about_row_changelog_desc": "Changelog, performance improvements and releases",
    "about_row_report_issue_title": "Report an Issue",
    "about_row_report_issue_desc": "File a bug report or feedback on GitHub",
    "about_row_star_title": "Star on GitHub",
    "about_row_star_desc": "Support the open-source project",
    "about_btn_exit_app": "Exit Aura",
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
    "status_streaming_degraded": "Live preview paused — full text will appear when recording ends",
    "model_downloading_pattern": "Starting download for {model}...",
    "model_download_error_pattern": "Download error: {err}",
    "delete_model_title": "Delete model",
    "delete_model_confirm_pattern": "Are you sure you want to delete the local model {model}?",
    "delete_model_btn": "Delete",
    "model_deleting_pattern": "Deleting model {model}...",
    "model_deleted_success": "Model deleted successfully",
    "model_delete_error_pattern": "Delete error: {err}",
    "model_downloaded_success_pattern": "Model {model} downloaded",
    "confirm_clear_history_title": "Clear history",
    "confirm_clear_history_msg": "Are you sure you want to clear all transcription history?",
    "general_ui_lang_title": "Interface Language",
    "general_ui_lang_desc": "Select the language for settings and application notifications.",
    "update_checks_title": "Update checks",
    "update_checks_desc": "Aura contacts GitHub only when you check manually or enable automatic checks.",
    "update_checks_checkbox": "Check for updates automatically at startup",
    "update_check_now": "Check for updates",
    "update_current": "Aura is up to date",
    "update_available_pattern": "Aura v{version} is available",
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
    "copy_context_desc": "When enabled, Aura sends Ctrl+C and passes the selected text to the provider or local model as context for an editing command. Disable this feature when working in a terminal.",
    "copy_context_checkbox": "Enable selected text editing",
    "ai_edit_title": "Commands on selection",
    "ai_edit_desc": "Voice commands for the selected text: \"translate\", \"fix grammar\". A command is applied to the selection; plain dictation is inserted unchanged.",
    "ai_edit_mode_label": "Editing mode",
    "ai_edit_mode_auto": "Auto",
    "ai_edit_mode_cloud": "Cloud",
    "ai_edit_mode_local": "Local",
    "ai_edit_mode_off": "Off",
    "ai_edit_mode_sub": "Auto uses the local model when it is installed, otherwise the cloud provider. For commands on selection, enable \"Allow selection capture and edit commands\".",
    "translate_target_label": "Default translation language",
    "translate_target_sub": "Used when saying \"translate\" without specifying a target language.",
    "edit_cloud_provider_label": "Cloud AI Service",
    "edit_cloud_provider_sub": "Model executing translation and grammar correction commands.",
    "edit_cloud_provider_fallback_sub": "Used as a fallback service if the local model is not installed or unavailable.",
    "llm_model_label": "Local model",
    "model_meta_llm_0_6b": "~460 MB — smallest, basic commands",
    "model_meta_llm_1_7b": "~1.2 GB — balanced",
    "model_meta_llm_4b": "~2.3 GB — best quality",
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
    "gpu_status_installing": "Installing...",
    "replacements_title": "Auto-replacements",
    "replacements_desc": "Automatically replaces dictated words and phrases with exact terms, email addresses, or custom text.",
    "replacements_trigger_ph": "Word or phrase (e.g. email)",
    "replacements_value_ph": "Replace with (e.g. name@mail.com)",
    "replacements_caret_hint": "The | symbol in the replacement text marks where to place the cursor after insertion (e.g. Hello, |!).",
    "replacements_macro_hint": "Available macros: {date} (YYYY-MM-DD), {date_ru} (DD.MM.YYYY), {time} (HH:MM), {datetime}.",
    "replacements_macros_label": "Insert template:",
    "macro_chip_date_ru": "+ Date (RU)",
    "macro_chip_date_iso": "+ Date (ISO)",
    "macro_chip_time": "+ Time",
    "macro_chip_datetime": "+ Date & Time",
    "btn_add": "Add",
    "replacements_empty": "No replacement rules configured yet",
    "replacements_delete_aria": "Delete replacement rule",
    "gaming_title": "Gaming Mode",
    "gaming_desc": "Automatically pauses hotkey hooks and unloads models from video memory for maximum FPS in games.",
    "gaming_mode_label": "Enable Gaming Mode manually",
    "gaming_mode_subdesc": "Disables keyboard hooks and unloads models from video memory (VRAM).",
    "gaming_auto_label": "Automatic Gaming Mode",
    "gaming_auto_desc": "Suspends hotkeys and frees video memory (VRAM) when launching fullscreen 3D games.",
    "gaming_status_label": "Current status",
    "gaming_status_inactive": "Inactive",
    "gaming_status_active": "Active (paused)",
    "gaming_btn_toggle_on": "Enable manually",
    "gaming_btn_toggle_off": "Disable",
    "gaming_tray_hint": "Manual toggle is also available via the Aura system tray menu.",
    "standby_title": "Standby Model Unloader (Sleep)",
    "standby_timeout_label": "Idle timeout before sleep",
    "standby_desc": "Releases video memory (VRAM) when voice typing is unused for the selected duration.",
    "standby_15m": "After 15 minutes",
    "standby_30m": "After 30 minutes (recommended)",
    "standby_60m": "After 1 hour",
    "standby_never": "Never (keep in VRAM)",
    "window_minimize": "Minimize",
    "window_close": "Close",
    "history_copy_title": "Copy to clipboard",
    "history_search_clear": "Clear search",
    "api_key_toggle_visibility": "Show/hide key",
    "history_filters_label": "History filters",
    "onboarding_skip": "Skip tutorial",
    "onboarding_prev": "Back",
    "onboarding_next": "Next",
    "onboarding_finish": "Get started",
    "onboarding_replay": "Feature walkthrough tour",
    "onboarding_step_tag": "{current} / {total}",
    "onboarding_step_1_title": "Speech Recognition Methods",
    "onboarding_step_1_desc": "Local AI guarantees 100% privacy without sending audio to the network, accelerated by GPU (NVIDIA CUDA) or CPU. Cloud AI requires no model downloads and delivers fast API processing with zero system load.",
    "onboarding_step_2_title": "Recognition Language & Dictionary",
    "onboarding_step_2_desc": "Aura can automatically adapt to the active keyboard layout language. In the dictionary, you can configure text replacement rules for typos and domain-specific terminology.",
    "onboarding_step_3_title": "Smart Insertion & Selected Text Editing",
    "onboarding_step_3_desc": "Select text in any application, press your hotkey, and speak a command: 'fix grammar' or 'translate' — Aura edits and replaces the text directly in place.",
    "onboarding_step_4_title": "Global Hotkeys, Overlay & Audio Feedback",
    "onboarding_step_4_desc": "The primary hotkey triggers speech recording anywhere. Gentle clicks confirm start and stop, and a floating overlay displays live recording status and elapsed time.",
    "onboarding_step_5_title": "Transcription History & Silent Tray Operation",
    "onboarding_step_5_desc": "All dictated texts are securely preserved in history for quick reuse. Aura stays quietly in the Windows tray, instantly ready for voice input whenever you press your hotkey.",
    "commands_guide_title": "Supported Voice Commands",
    "commands_guide_desc": "Available actions for selected text.",
    "commands_guide_label": "Supported Commands",
    "commands_guide_sub": "Voice phrases for actions on selected text",
    "commands_guide_btn": "Commands Guide",
    "commands_guide_close": "Close",
    "commands_guide_active_summary": "{active} of {total} commands active",
    "smart_context_title": "Smart Context-Aware Insertion",
    "smart_context_desc": "Automatically adapts letter case, punctuation, and boundary whitespace based on cursor position.",
    "smart_context_checkbox": "Adapt to surrounding cursor context",
    "smart_context_subdesc": "Starts with lowercase when inserting mid-sentence, strips trailing periods, and ensures clean spacing.",
    "smart_context_trailing_period_checkbox": "End with a period when continuing at the end of a sentence",
    "smart_context_trailing_period_subdesc": "When cursor is at the end of an open sentence, completes with a period. When unchecked, leaves open for further dictation.",
    "sound_preview_btn": "Preview theme",
    "sound_preview_label": "Test",
    "api_key_status_saved": "Saved in Windows DPAPI",
    "api_key_shared_hint": "Unified Aura key: applies to voice dictation and AI editing.",
    "api_key_btn_change": "Change"
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
    "tab_text": "Text",
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
    "engine_health_parakeet_gaming": "Parakeet: Server entladen (Spielmodus aktiv)",
    "engine_health_whisper_running": "Whisper: Server läuft ({provider}, Port {port})",
    "engine_health_whisper_stopped": "Whisper: Server läuft nicht",
    "engine_health_whisper_gaming": "Whisper: Server entladen (Spielmodus aktiv)",
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
    "sound_theme_bubble": "Wassertropfen",
    "sound_theme_haptic": "Haptischer Schalter",
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
    "history_group_today": "Heute",
    "history_group_yesterday": "Gestern",
    "about_app_title": "Aura Spracheingabe",
    "about_version": "v1.2.0",
    "about_description": "Globales Spracheingabe-Tool für Windows. Die Anwendung überträgt Sprache in Text und fügt ihn mit automatischer Formatierung und Zeichensetzung in jedes aktive Fenster ein.",
    "about_gratitude_text": "Danke, dass Sie Aura nutzen! Wenn das Tool Ihre Arbeit erleichtert, unterstützen Sie das Projekt gerne mit einem Stern auf GitHub.",
    "btn_star_github": "Stern auf GitHub geben",
    "about_row_tour_title": "Funktionstour",
    "about_row_tour_desc": "Interaktive Einführung in die wichtigsten Funktionen",
    "about_row_changelog_title": "Was ist neu in dieser Version",
    "about_row_changelog_desc": "Versionshinweise, Verbesserungen und Veröffentlichungen",
    "about_row_report_issue_title": "Problem melden",
    "about_row_report_issue_desc": "Fehlerbericht oder Feedback auf GitHub einreichen",
    "about_row_star_title": "Auf GitHub mit Stern bewerten",
    "about_row_star_desc": "Unterstützen Sie das Open-Source-Projekt",
    "about_btn_exit_app": "Aura beenden",
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
    "status_streaming_degraded": "Live-Vorschau pausiert — der vollständige Text erscheint nach Aufnahmeende",
    "model_downloading_pattern": "Download für {model} wird gestartet...",
    "model_download_error_pattern": "Download-Fehler: {err}",
    "delete_model_title": "Modell löschen",
    "delete_model_confirm_pattern": "Möchten Sie das lokale Modell {model} wirklich löschen?",
    "delete_model_btn": "Löschen",
    "model_deleting_pattern": "Modell {model} wird gelöscht...",
    "model_deleted_success": "Modell erfolgreich gelöscht",
    "model_delete_error_pattern": "Fehler beim Löschen: {err}",
    "model_downloaded_success_pattern": "Modell {model} heruntergeladen",
    "confirm_clear_history_title": "Verlauf löschen",
    "confirm_clear_history_msg": "Möchten Sie den gesamten Transkriptionsverlauf wirklich löschen?",
    "general_ui_lang_title": "Sprache der Benutzeroberfläche",
    "general_ui_lang_desc": "Wählen Sie die Sprache für Einstellungen und Benachrichtigungen.",
    "update_checks_title": "Update-Prüfung",
    "update_checks_desc": "Aura kontaktiert GitHub nur bei einer manuellen Prüfung oder wenn Sie automatische Prüfungen aktivieren.",
    "update_checks_checkbox": "Beim Start automatisch nach Updates suchen",
    "update_check_now": "Nach Updates suchen",
    "update_current": "Aura ist auf dem neuesten Stand",
    "update_available_pattern": "Aura v{version} ist verfügbar",
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
    "copy_context_desc": "Wenn aktiviert, sendet Aura Strg+C und übermittelt den ausgewählten Text als Kontext für einen Bearbeitungsbefehl an den Anbieter oder das lokale Modell. Deaktivieren Sie diese Funktion bei der Arbeit im Terminal.",
    "copy_context_checkbox": "Bearbeitung des ausgewählten Texts aktivieren",
    "ai_edit_title": "Befehle für die Auswahl",
    "ai_edit_desc": "Sprachbefehle für den ausgewählten Text: „übersetzen“, „Grammatik korrigieren“. Ein Befehl wird auf die Auswahl angewendet; normales Diktat wird unverändert eingefügt.",
    "ai_edit_mode_label": "Bearbeitungsmodus",
    "ai_edit_mode_auto": "Auto",
    "ai_edit_mode_cloud": "Cloud",
    "ai_edit_mode_local": "Lokal",
    "ai_edit_mode_off": "Aus",
    "ai_edit_mode_sub": "Auto verwendet das lokale Modell, wenn es installiert ist, sonst den Cloud-Anbieter. Für Befehle auf die Auswahl aktivieren Sie „Erfassen der Auswahl und Bearbeitungsbefehle zulassen“.",
    "translate_target_label": "Standard-Zielsprache für Übersetzungen",
    "translate_target_sub": "Wird verwendet, wenn „übersetze“ ohne Zielangabe gesprochen wird.",
    "edit_cloud_provider_label": "Cloud-KI-Dienst",
    "edit_cloud_provider_sub": "Modell zur Ausführung von Befehlen wie Übersetzung und Textkorrektur.",
    "edit_cloud_provider_fallback_sub": "Wird als Fallback verwendet, wenn das lokale Modell nicht installiert oder nicht verfügbar ist.",
    "llm_model_label": "Lokales Modell",
    "model_meta_llm_0_6b": "~460 MB — kleinste, einfache Befehle",
    "model_meta_llm_1_7b": "~1,2 GB — ausgewogen",
    "model_meta_llm_4b": "~2,3 GB — beste Qualität",
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
    "gpu_status_installing": "Installieren...",
    "replacements_title": "Auto-Ersetzung",
    "replacements_desc": "Ersetzt diktierte Wörter und Ausdrücke automatisch durch genaue Begriffe, E-Mail-Adressen oder Text.",
    "replacements_trigger_ph": "Wort oder Satz (z.B. E-Mail)",
    "replacements_value_ph": "Ersetzen durch (z.B. name@mail.com)",
    "replacements_caret_hint": "Das Symbol | im Ersatztext markiert die Cursorposition nach dem Einfügen (z.B. Hallo, |!).",
    "replacements_macro_hint": "Verfügbare Makros: {date} (JJJJ-MM-TT), {date_ru} (TT.MM.JJJJ), {time} (HH:MM), {datetime}.",
    "replacements_macros_label": "Vorlage einfügen:",
    "macro_chip_date_ru": "+ Datum (RU)",
    "macro_chip_date_iso": "+ Datum (ISO)",
    "macro_chip_time": "+ Uhrzeit",
    "macro_chip_datetime": "+ Datum & Zeit",
    "btn_add": "Hinzufügen",
    "replacements_empty": "Noch keine Ersetzungsregeln eingerichtet",
    "replacements_delete_aria": "Ersetzungsregel löschen",
    "gaming_title": "Spielemodus",
    "gaming_desc": "Pausiert Tastatur-Hooks automatisch und gibt Videospeicher frei für maximale FPS in Spielen.",
    "gaming_mode_label": "Spielemodus manuell aktivieren",
    "gaming_mode_subdesc": "Deaktiviert Tastatur-Hooks und entlädt Modelle aus dem Grafikspeicher (VRAM).",
    "gaming_auto_label": "Automatischer Spielemodus",
    "gaming_auto_desc": "Pausiert Hotkeys und gibt Videospeicher (VRAM) frei, wenn Vollbild-3D-Spiele gestartet werden.",
    "gaming_status_label": "Aktueller Status",
    "gaming_status_inactive": "Inaktiv",
    "gaming_status_active": "Aktiv (pausiert)",
    "gaming_btn_toggle_on": "Manuell aktivieren",
    "gaming_btn_toggle_off": "Deaktivieren",
    "gaming_tray_hint": "Manuelles Umschalten ist auch über das Aura-Taskleistenmenü möglich.",
    "standby_title": "Modell-Entladung im Leerlauf (Schlafmodus)",
    "standby_timeout_label": "Inaktivitätszeit vor dem Schlafmodus",
    "standby_desc": "Gibt Videospeicher (VRAM) frei, wenn die Spracheingabe für die gewählte Zeit nicht genutzt wird.",
    "standby_15m": "Nach 15 Minuten",
    "standby_30m": "Nach 30 Minuten (empfohlen)",
    "standby_60m": "Nach 1 Stunde",
    "standby_never": "Nie (im VRAM behalten)",
    "window_minimize": "Minimieren",
    "window_close": "Schließen",
    "history_copy_title": "In die Zwischenablage kopieren",
    "history_search_clear": "Suche löschen",
    "api_key_toggle_visibility": "Schlüssel anzeigen/ausblenden",
    "history_filters_label": "Verlaufsfilter",
    "onboarding_skip": "Tutorial überspringen",
    "onboarding_prev": "Zurück",
    "onboarding_next": "Weiter",
    "onboarding_finish": "Loslegen",
    "onboarding_replay": "Funktionsüberblick starten",
    "onboarding_step_tag": "{current} / {total}",
    "onboarding_step_1_title": "Spracherkennungsmethoden",
    "onboarding_step_1_desc": "Lokale KI bietet 100% Privatsphäre ohne Datenübertragung und wird per GPU (NVIDIA CUDA) oder CPU beschleunigt. Cloud-KI benötigt keine Downloads und verarbeitet blitzschnell per API ohne Systemlast.",
    "onboarding_step_2_title": "Erkennungssprache & Wörterbuch",
    "onboarding_step_2_desc": "Aura kann sich automatisch an die aktive Tastatursprache anpassen. Im Wörterbuch können Textersetzungsregeln für Tippfehler und Fachbegriffe festgelegt werden.",
    "onboarding_step_3_title": "Intelligentes Einfügen & Textbearbeitung",
    "onboarding_step_3_desc": "Markieren Sie Text in einem beliebigen Programm, drücken Sie den Hotkey und sprechen Sie einen Befehl: „Fehler korrigieren“ oder „übersetzen“ — Aura ersetzt den Text direkt an Ort und Stelle.",
    "onboarding_step_4_title": "Globale Hotkeys, Overlay & Audio-Feedback",
    "onboarding_step_4_desc": "Der Haupt-Hotkey startet die Aufnahme überall. Sanfte Klicks bestätigen Start und Stopp, und ein schwebendes Overlay zeigt Status und Aufnahmezeit an.",
    "onboarding_step_5_title": "Verlauf & leiser Betrieb im System-Tray",
    "onboarding_step_5_desc": "Alle diktierten Texte werden im Verlauf gespeichert. Aura bleibt leise im Windows-Tray und ist per Tastendruck jederzeit sofort einsatzbereit.",
    "commands_guide_title": "Unterstützte Sprachbefehle",
    "commands_guide_desc": "Verfügbare Aktionen für markierten Text.",
    "commands_guide_label": "Unterstützte Befehle",
    "commands_guide_sub": "Sprachphrasen für Aktionen auf markiertem Text",
    "commands_guide_btn": "Befehlsübersicht",
    "commands_guide_close": "Schließen",
    "commands_guide_active_summary": "{active} von {total} Befehlen aktiv",
    "smart_context_title": "Intelligente kontextbezogene Einfügung",
    "smart_context_desc": "Passt Groß-/Kleinschreibung, Zeichensetzung und Leerzeichen automatisch an die Cursorposition an.",
    "smart_context_checkbox": "Kontext um den Cursor berücksichtigen",
    "smart_context_subdesc": "Beginnt mitten im Satz mit Kleinbuchstaben, entfernt überflüssige Punkte und formatiert Leerzeichen sauber.",
    "smart_context_trailing_period_checkbox": "Bei Fortsetzung am Satzende mit Punkt abschließen",
    "smart_context_trailing_period_subdesc": "Schließt den Satz mit einem Punkt ab, wenn der Cursor am Ende steht. Deaktiviert lassen, um weitersprechen zu können.",
    "sound_preview_btn": "Thema anhören",
    "sound_preview_label": "Testen",
    "api_key_status_saved": "In Windows DPAPI gespeichert",
    "api_key_shared_hint": "Einheitlicher Aura-Schlüssel: gilt für Sprache und KI-Bearbeitung.",
    "api_key_btn_change": "Ändern"
  },
  "es": {
    "title_settings": "Ajustes",
    "tab_general": "General",
    "tab_speech": "Voz",
    "tab_text": "Texto",
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
    "engine_health_parakeet_gaming": "Parakeet: servidor descargado (Modo de juego activo)",
    "engine_health_whisper_running": "Whisper: servidor en ejecución ({provider}, puerto {port})",
    "engine_health_whisper_stopped": "Whisper: servidor no en ejecución",
    "engine_health_whisper_gaming": "Whisper: servidor descargado (Modo de juego activo)",
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
    "sound_theme_bubble": "Gota de agua",
    "sound_theme_haptic": "Interruptor háptico",
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
    "history_group_today": "Hoy",
    "history_group_yesterday": "Ayer",
    "about_app_title": "Dictado por voz Aura",
    "about_version": "v1.2.0",
    "about_description": "Herramienta de entrada de voz global para Windows. El programa transcribe el habla en texto y lo inserta en cualquier ventana activa con formato y puntuación automáticos.",
    "about_gratitude_text": "¡Gracias por usar Aura! Si hace tu trabajo más rápido y cómodo, apoya el proyecto con una estrella en GitHub.",
    "btn_star_github": "Dar una estrella en GitHub",
    "about_row_tour_title": "Paseo de funciones",
    "about_row_tour_desc": "Guía interactiva de las funciones principales",
    "about_row_changelog_title": "Novedades en esta versión",
    "about_row_changelog_desc": "Registro de cambios, mejoras y lanzamientos",
    "about_row_report_issue_title": "Informar de un error",
    "about_row_report_issue_desc": "Enviar un reporte o sugerencia en GitHub",
    "about_row_star_title": "Dar una estrella en GitHub",
    "about_row_star_desc": "Apoya el proyecto de código abierto",
    "about_btn_exit_app": "Salir de Aura",
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
    "status_streaming_degraded": "Vista previa en vivo en pausa: el texto completo aparecerá al finalizar la grabación",
    "model_downloading_pattern": "Iniciando descarga para {model}...",
    "model_download_error_pattern": "Error de descarga: {err}",
    "delete_model_title": "Eliminar modelo",
    "delete_model_confirm_pattern": "¿Está seguro de que desea eliminar el modelo local {model}?",
    "delete_model_btn": "Eliminar",
    "model_deleting_pattern": "Eliminando modelo {model}...",
    "model_deleted_success": "Modelo eliminado correctamente",
    "model_delete_error_pattern": "Error al eliminar: {err}",
    "model_downloaded_success_pattern": "Modelo {model} descargado",
    "confirm_clear_history_title": "Limpiar historial",
    "confirm_clear_history_msg": "¿Está seguro de que desea limpiar todo el historial de transcripciones?",
    "general_ui_lang_title": "Idioma de la interfaz",
    "general_ui_lang_desc": "Seleccione el idioma para los ajustes y las notificaciones.",
    "update_checks_title": "Comprobación de actualizaciones",
    "update_checks_desc": "Aura se conecta a GitHub solo al comprobar manualmente o al activar las comprobaciones automáticas.",
    "update_checks_checkbox": "Buscar actualizaciones automáticamente al iniciar",
    "update_check_now": "Buscar actualizaciones",
    "update_current": "Aura está actualizada",
    "update_available_pattern": "Aura v{version} está disponible",
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
    "copy_context_desc": "Cuando está activado, Aura envía Ctrl+C y pasa el texto seleccionado al proveedor o al modelo local como contexto para una orden de edición. Desactive esta función al trabajar en una terminal.",
    "copy_context_checkbox": "Permitir edición del texto seleccionado",
    "ai_edit_title": "Órdenes sobre la selección",
    "ai_edit_desc": "Órdenes por voz para el texto seleccionado: «traducir», «corregir gramática». La orden se aplica a la selección; el dictado normal se inserta sin cambios.",
    "ai_edit_mode_label": "Modo de edición",
    "ai_edit_mode_auto": "Auto",
    "ai_edit_mode_cloud": "Nube",
    "ai_edit_mode_local": "Local",
    "ai_edit_mode_off": "Desactivado",
    "ai_edit_mode_sub": "Auto usa el modelo local si está instalado; si no, el proveedor en la nube. Para órdenes sobre la selección, active «Permitir captura de selección y órdenes de edición».",
    "translate_target_label": "Idioma de traducción predeterminado",
    "translate_target_sub": "Se utiliza al decir «traduce» sin especificar un idioma de destino.",
    "edit_cloud_provider_label": "Servicio de IA en la nube",
    "edit_cloud_provider_sub": "Modelo que ejecuta comandos de traducción y corrección de texto.",
    "edit_cloud_provider_fallback_sub": "Se utiliza como reserva si el modelo local no está instalado o disponible.",
    "llm_model_label": "Modelo local",
    "model_meta_llm_0_6b": "~460 MB — la más pequeña, órdenes básicas",
    "model_meta_llm_1_7b": "~1,2 GB — equilibrada",
    "model_meta_llm_4b": "~2,3 GB — máxima calidad",
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
    "gpu_status_installing": "Instalando...",
    "replacements_title": "Autoreemplazo",
    "replacements_desc": "Reemplaza automáticamente palabras y frases dictadas por términos exactos, correos o texto personalizado.",
    "replacements_trigger_ph": "Palabra o frase (ej. correo)",
    "replacements_value_ph": "Reemplazar con (ej. name@mail.com)",
    "replacements_caret_hint": "El símbolo | en el texto de reemplazo indica dónde colocar el cursor tras la inserción (ej. Hola, |!).",
    "replacements_macro_hint": "Macros disponibles: {date} (AAAA-MM-DD), {date_ru} (DD.MM.AAAA), {time} (HH:MM), {datetime}.",
    "replacements_macros_label": "Insertar plantilla:",
    "macro_chip_date_ru": "+ Fecha (RU)",
    "macro_chip_date_iso": "+ Fecha (ISO)",
    "macro_chip_time": "+ Hora",
    "macro_chip_datetime": "+ Fecha y hora",
    "btn_add": "Añadir",
    "replacements_empty": "Aún no hay reglas de sustitución configuradas",
    "replacements_delete_aria": "Eliminar regla de sustitución",
    "gaming_title": "Modo de juego",
    "gaming_desc": "Pausa automáticamente los ganchos de teclado y descarga modelos de la VRAM para el máximo FPS en juegos.",
    "gaming_mode_label": "Activar modo de juego manualmente",
    "gaming_mode_subdesc": "Desactiva los ganchos de teclado y descarga modelos de la memoria de video (VRAM).",
    "gaming_auto_label": "Modo de juego automático",
    "gaming_auto_desc": "Suspende las teclas de acceso rápido y libera memoria de video (VRAM) al iniciar juegos 3D a pantalla completa.",
    "gaming_status_label": "Estado actual",
    "gaming_status_inactive": "Inactivo",
    "gaming_status_active": "Activo (en pausa)",
    "gaming_btn_toggle_on": "Activar manualmente",
    "gaming_btn_toggle_off": "Desactivar",
    "gaming_tray_hint": "El cambio manual también está disponible en el menú de Aura en la bandeja del sistema.",
    "standby_title": "Descarga de modelos por inactividad (Reposo)",
    "standby_timeout_label": "Tiempo de inactividad antes del reposo",
    "standby_desc": "Libera memoria de video (VRAM) cuando el dictado por voz no se utiliza durante el tiempo seleccionado.",
    "standby_15m": "Tras 15 minutos",
    "standby_30m": "Tras 30 minutos (recomendado)",
    "standby_60m": "Tras 1 hora",
    "standby_never": "Nunca (mantener en VRAM)",
    "window_minimize": "Minimizar",
    "window_close": "Cerrar",
    "history_copy_title": "Copiar al portapapeles",
    "history_search_clear": "Borrar búsqueda",
    "api_key_toggle_visibility": "Mostrar/ocultar clave",
    "history_filters_label": "Filtros del historial",
    "onboarding_skip": "Omitir aprendizaje",
    "onboarding_prev": "Atrás",
    "onboarding_next": "Siguiente",
    "onboarding_finish": "Comenzar",
    "onboarding_replay": "Recorrido de funciones",
    "onboarding_step_tag": "{current} / {total}",
    "onboarding_step_1_title": "Métodos de reconocimiento de voz",
    "onboarding_step_1_desc": "La IA local garantiza un 100% de privacidad sin enviar audio a la red, acelerada por GPU (NVIDIA CUDA) o CPU. La IA en la nube no requiere descargas y procesa al instante mediante API sin carga en el sistema.",
    "onboarding_step_2_title": "Idioma de reconocimiento y diccionario",
    "onboarding_step_2_desc": "Aura puede adaptarse automáticamente al idioma del teclado activo. En el diccionario puede definir reglas de reemplazo para erratas y términos especializados.",
    "onboarding_step_3_title": "Inserción inteligente y edición del texto seleccionado",
    "onboarding_step_3_desc": "Seleccione texto en cualquier aplicación, presione el atajo y dicte una orden: «corrige errores» o «traduce» — Aura reemplazará el texto directamente en el lugar.",
    "onboarding_step_4_title": "Atajos globales, overlay y confirmación de audio",
    "onboarding_step_4_desc": "La tecla principal inicia la grabación en cualquier lugar. Clics suaves confirman el inicio y fin, y un overlay flotante muestra el tiempo y estado de la grabación.",
    "onboarding_step_5_title": "Historial de transcripciones y bandeja del sistema",
    "onboarding_step_5_desc": "Todos los textos dictados se guardan en el historial para su reutilización. Aura permanece en la bandeja del sistema lista para activarse con su atajo.",
    "commands_guide_title": "Comandos de voz compatibles",
    "commands_guide_desc": "Acciones disponibles sobre el texto seleccionado.",
    "commands_guide_label": "Comandos compatibles",
    "commands_guide_sub": "Frases de voz para acciones en texto seleccionado",
    "commands_guide_btn": "Guía de comandos",
    "commands_guide_close": "Cerrar",
    "commands_guide_active_summary": "{active} de {total} comandos activos",
    "smart_context_title": "Inserción contextual inteligente",
    "smart_context_desc": "Adapta automáticamente mayúsculas/minúsculas, puntuación y espacios según la posición del cursor.",
    "smart_context_checkbox": "Tener en cuenta el contexto del cursor",
    "smart_context_subdesc": "Comienza con minúscula en mitad de una frase, elimina puntos finales y ajusta los espacios limpios.",
    "smart_context_trailing_period_checkbox": "Finalizar con punto al continuar al final de la oración",
    "smart_context_trailing_period_subdesc": "Completa la frase con un punto si el cursor está al final. Desactívelo para seguir dictando.",
    "sound_preview_btn": "Probar tema",
    "sound_preview_label": "Probar",
    "api_key_status_saved": "Guardado en Windows DPAPI",
    "api_key_shared_hint": "Clave única de Aura: se aplica al dictado y a la edición con IA.",
    "api_key_btn_change": "Cambiar"
  },
  "fr": {
    "title_settings": "Paramètres",
    "tab_general": "Général",
    "tab_speech": "Dictée",
    "tab_text": "Texte",
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
    "engine_health_parakeet_gaming": "Parakeet : serveur déchargé (Mode jeu actif)",
    "engine_health_whisper_running": "Whisper : serveur en cours d'exécution ({provider}, port {port})",
    "engine_health_whisper_stopped": "Whisper : serveur non démarré",
    "engine_health_whisper_gaming": "Whisper : serveur déchargé (Mode jeu actif)",
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
    "sound_theme_bubble": "Goutte d'eau",
    "sound_theme_haptic": "Interrupteur haptique",
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
    "history_group_today": "Aujourd'hui",
    "history_group_yesterday": "Hier",
    "about_app_title": "Dictée vocale Aura",
    "about_version": "v1.2.0",
    "about_description": "Outil de saisie vocale globale pour Windows. Le programme transcrit la parole en texte et l'insère dans n'importe quelle fenêtre active avec un formatage et une ponctuation automatiques.",
    "about_gratitude_text": "Merci d'utiliser Aura ! Si le programme vous fait gagner du temps, soutenez le projet avec une étoile sur GitHub.",
    "btn_star_github": "Mettre une étoile sur GitHub",
    "about_row_tour_title": "Visite guidée",
    "about_row_tour_desc": "Présentation interactive des fonctionnalités clés",
    "about_row_changelog_title": "Nouveautés de cette version",
    "about_row_changelog_desc": "Journal des modifications, améliorations et versions",
    "about_row_report_issue_title": "Signaler un problème",
    "about_row_report_issue_desc": "Signaler un bug ou faire part de vos retours sur GitHub",
    "about_row_star_title": "Mettre une étoile sur GitHub",
    "about_row_star_desc": "Soutenez le projet open source",
    "about_btn_exit_app": "Quitter Aura",
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
    "status_streaming_degraded": "Aperçu en direct suspendu — le texte complet apparaîtra à la fin de l'enregistrement",
    "model_downloading_pattern": "Démarrage du téléchargement pour {model}...",
    "model_download_error_pattern": "Erreur de téléchargement: {err}",
    "delete_model_title": "Supprimer le modèle",
    "delete_model_confirm_pattern": "Voulez-vous vraiment supprimer le modèle local {model} ?",
    "delete_model_btn": "Supprimer",
    "model_deleting_pattern": "Suppression du modèle {model}...",
    "model_deleted_success": "Modèle supprimé avec succès",
    "model_delete_error_pattern": "Erreur de suppression: {err}",
    "model_downloaded_success_pattern": "Modèle {model} téléchargé",
    "confirm_clear_history_title": "Effacer l'historique",
    "confirm_clear_history_msg": "Voulez-vous vraiment effacer tout l'historique des transcriptions ?",
    "general_ui_lang_title": "Langue de l'interface",
    "general_ui_lang_desc": "Sélectionnez la langue pour les paramètres et les notifications de l'application.",
    "update_checks_title": "Recherche de mises à jour",
    "update_checks_desc": "Aura contacte GitHub uniquement lors d’une vérification manuelle ou si vous activez les vérifications automatiques.",
    "update_checks_checkbox": "Rechercher automatiquement les mises à jour au démarrage",
    "update_check_now": "Rechercher les mises à jour",
    "update_current": "Aura est à jour",
    "update_available_pattern": "Aura v{version} est disponible",
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
    "copy_context_desc": "Lorsque cette option est activée, Aura envoie Ctrl+C et transmet le texte sélectionné au fournisseur ou au modèle local comme contexte d’une commande de modification. Désactivez-la lorsque vous travaillez dans un terminal.",
    "copy_context_checkbox": "Activer la modification du texte sélectionné",
    "ai_edit_title": "Commandes sur la sélection",
    "ai_edit_desc": "Commandes vocales pour le texte sélectionné : « traduire », « corriger la grammaire ». La commande s’applique à la sélection ; la dictée simple est insérée inchangée.",
    "ai_edit_mode_label": "Mode de modification",
    "ai_edit_mode_auto": "Auto",
    "ai_edit_mode_cloud": "Cloud",
    "ai_edit_mode_local": "Local",
    "ai_edit_mode_off": "Désactivé",
    "ai_edit_mode_sub": "Auto utilise le modèle local s’il est installé, sinon le fournisseur cloud. Pour les commandes sur la sélection, activez « Autoriser la capture de la sélection et les commandes de modification ».",
    "translate_target_label": "Langue de traduction par défaut",
    "translate_target_sub": "Utilisée lorsque vous dites « traduis » sans préciser de langue.",
    "edit_cloud_provider_label": "Service d'IA cloud",
    "edit_cloud_provider_sub": "Modèle exécutant les commandes de traduction et de correction de texte.",
    "edit_cloud_provider_fallback_sub": "Utilisé en secours si le modèle local n'est pas installé ou indisponible.",
    "llm_model_label": "Modèle local",
    "model_meta_llm_0_6b": "~460 Mo — la plus légère, commandes simples",
    "model_meta_llm_1_7b": "~1,2 Go — équilibrée",
    "model_meta_llm_4b": "~2,3 Go — qualité maximale",
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
    "gpu_status_installing": "Installation...",
    "replacements_title": "Remplacement automatique",
    "replacements_desc": "Remplace automatiquement les mots et expressions dictés par des termes exacts, adresses e-mail ou texte personnalisé.",
    "replacements_trigger_ph": "Mot ou phrase (ex. e-mail)",
    "replacements_value_ph": "Remplacer par (ex. name@mail.com)",
    "replacements_caret_hint": "Le symbole | dans le texte de remplacement indique où placer le curseur après insertion (ex. Bonjour, |!).",
    "replacements_macro_hint": "Macros disponibles : {date} (AAAA-MM-JJ), {date_ru} (JJ.MM.AAAA), {time} (HH:MM), {datetime}.",
    "replacements_macros_label": "Insérer un modèle :",
    "macro_chip_date_ru": "+ Date (RU)",
    "macro_chip_date_iso": "+ Date (ISO)",
    "macro_chip_time": "+ Heure",
    "macro_chip_datetime": "+ Date et heure",
    "btn_add": "Ajouter",
    "replacements_empty": "Aucune règle de remplacement configurée pour l'instant",
    "replacements_delete_aria": "Supprimer la règle de remplacement",
    "gaming_title": "Mode Jeu",
    "gaming_desc": "Met en pause automatiquement les hooks clavier et décharge les modèles de la VRAM pour un FPS maximal en jeu.",
    "gaming_mode_label": "Activer le mode Jeu manuellement",
    "gaming_mode_subdesc": "Désactive les hooks clavier et décharge les modèles de la mémoire vidéo (VRAM).",
    "gaming_auto_label": "Mode Jeu automatique",
    "gaming_auto_desc": "Suspend les raccourcis clavier et libère la mémoire vidéo (VRAM) lors du lancement de jeux 3D en plein écran.",
    "gaming_status_label": "État actuel",
    "gaming_status_inactive": "Inactif",
    "gaming_status_active": "Actif (en pause)",
    "gaming_btn_toggle_on": "Activer manuellement",
    "gaming_btn_toggle_off": "Désactiver",
    "gaming_tray_hint": "Le basculement manuel est également disponible via le menu Aura dans la barre des tâches.",
    "standby_title": "Mise en veille des modèles en cas d'inactivité",
    "standby_timeout_label": "Délai d'inactivité avant la mise en veille",
    "standby_desc": "Libère la mémoire vidéo (VRAM) lorsque la saisie vocale n'est pas utilisée pendant la durée choisie.",
    "standby_15m": "Après 15 minutes",
    "standby_30m": "Après 30 minutes (recommandé)",
    "standby_60m": "Après 1 heure",
    "standby_never": "Jamais (garder en VRAM)",
    "window_minimize": "Réduire",
    "window_close": "Fermer",
    "history_copy_title": "Copier dans le presse-papiers",
    "history_search_clear": "Effacer la recherche",
    "api_key_toggle_visibility": "Afficher/masquer la clé",
    "history_filters_label": "Filtres de l'historique",
    "onboarding_skip": "Ignorer le tutoriel",
    "onboarding_prev": "Retour",
    "onboarding_next": "Suivant",
    "onboarding_finish": "Commencer",
    "onboarding_replay": "Visite guidée des fonctionnalités",
    "onboarding_step_tag": "{current} / {total}",
    "onboarding_step_1_title": "Méthodes de reconnaissance vocale",
    "onboarding_step_1_desc": "L'IA locale offre 100% de confidentialité sans envoi audio sur le réseau, accélérée par GPU (NVIDIA CUDA) ou CPU. L'IA cloud ne nécessite aucun téléchargement et traite via API sans charge système.",
    "onboarding_step_2_title": "Langue de reconnaissance et dictionnaire",
    "onboarding_step_2_desc": "Aura peut s'adapter automatiquement à la langue du clavier actif. Dans le dictionnaire, définissez des règles de remplacement pour les coquilles et termes spécialisés.",
    "onboarding_step_3_title": "Insertion intelligente et édition du texte sélectionné",
    "onboarding_step_3_desc": "Sélectionnez du texte dans n'importe quel logiciel, appuyez sur le raccourci et dictez un ordre : « corrige les erreurs » ou « traduis » — Aura modifie le texte directement sur place.",
    "onboarding_step_4_title": "Raccourcis globaux, overlay et retour sonore",
    "onboarding_step_4_desc": "La touche principale déclenche l'enregistrement n'importe où. De discrets clics confirment le début et la fin, et un overlay flottant indique l'état et la durée.",
    "onboarding_step_5_title": "Historique des dictées et zone de notification",
    "onboarding_step_5_desc": "Tous les textes dictés sont conservés dans l'historique pour être réutilisés. Aura reste discrètement dans la barre des tâches, prête à être activée par votre raccourci.",
    "commands_guide_title": "Commandes vocales prises en charge",
    "commands_guide_desc": "Actions disponibles sur le texte sélectionné.",
    "commands_guide_label": "Commandes prises en charge",
    "commands_guide_sub": "Phrases vocales pour les actions sur le texte sélectionné",
    "commands_guide_btn": "Guide des commandes",
    "commands_guide_close": "Fermer",
    "commands_guide_active_summary": "{active} sur {total} commandes actives",
    "smart_context_title": "Insertion contextuelle intelligente",
    "smart_context_desc": "Adapte automatiquement la casse, la ponctuation et les espaces selon la position du curseur.",
    "smart_context_checkbox": "Prendre en compte le contexte autour du curseur",
    "smart_context_subdesc": "Commence par une minuscule au milieu d'une phrase, supprime les points inutiles et ajuste les espaces.",
    "smart_context_trailing_period_checkbox": "Terminer par un point lors de la continuation en fin de phrase",
    "smart_context_trailing_period_subdesc": "Termine par un point si le curseur est en fin de phrase. Désactivez pour poursuivre la dictée.",
    "sound_preview_btn": "Écouter le thème",
    "sound_preview_label": "Tester",
    "api_key_status_saved": "Enregistré dans Windows DPAPI",
    "api_key_shared_hint": "Clé unique Aura : s'applique à la dictée et à l'édition IA.",
    "api_key_btn_change": "Modifier"
  },
  "it": {
    "title_settings": "Impostazioni",
    "tab_general": "Generale",
    "tab_speech": "Dettatura",
    "tab_text": "Testo",
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
    "engine_health_parakeet_gaming": "Parakeet: server scaricato (Modalità gioco attiva)",
    "engine_health_whisper_running": "Whisper: server in esecuzione ({provider}, porta {port})",
    "engine_health_whisper_stopped": "Whisper: server non in esecuzione",
    "engine_health_whisper_gaming": "Whisper: server scaricato (Modalità gioco attiva)",
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
    "sound_theme_bubble": "Goccia d'acqua",
    "sound_theme_haptic": "Interruttore tattile",
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
    "history_group_today": "Oggi",
    "history_group_yesterday": "Ieri",
    "about_app_title": "Dettatura vocale Aura",
    "about_version": "v1.2.0",
    "about_description": "Strumento di inserimento vocale globale per Windows. Il programma trascrive la voce in testo e la inserisce in qualsiasi finestra attiva con formattazione e punteggiatura automatiche.",
    "about_gratitude_text": "Grazie per aver scelto Aura! Se rende il tuo lavoro più veloce e comodo, sostieni il progetto con una stella su GitHub.",
    "btn_star_github": "Lascia una stella su GitHub",
    "about_row_tour_title": "Tour delle funzionalità",
    "about_row_tour_desc": "Guida interattiva alle funzionalità principali",
    "about_row_changelog_title": "Novità in questa versione",
    "about_row_changelog_desc": "Registro modifiche, miglioramenti e versioni",
    "about_row_report_issue_title": "Segnala un problema",
    "about_row_report_issue_desc": "Invia una segnalazione di bug su GitHub",
    "about_row_star_title": "Metti una stella su GitHub",
    "about_row_star_desc": "Sostieni il progetto open source",
    "about_btn_exit_app": "Esci da Aura",
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
    "status_streaming_degraded": "Anteprima live in pausa — il testo completo apparirà al termine della registrazione",
    "model_downloading_pattern": "Avvio del download per {model}...",
    "model_download_error_pattern": "Errore di download: {err}",
    "delete_model_title": "Elimina modello",
    "delete_model_confirm_pattern": "Sei sicuro di voler eliminare il modello locale {model}?",
    "delete_model_btn": "Elimina",
    "model_deleting_pattern": "Eliminazione del modello {model}...",
    "model_deleted_success": "Modello eliminato con successo",
    "model_delete_error_pattern": "Errore di eliminazione: {err}",
    "model_downloaded_success_pattern": "Modello {model} scaricato",
    "confirm_clear_history_title": "Cancella cronologia",
    "confirm_clear_history_msg": "Sei sicuro di voler cancellare tutta la cronologia delle trascrizioni?",
    "general_ui_lang_title": "Lingua dell'interfaccia",
    "general_ui_lang_desc": "Seleziona la lingua per le impostazioni e le notifiche dell'applicazione.",
    "update_checks_title": "Controllo aggiornamenti",
    "update_checks_desc": "Aura contatta GitHub solo durante un controllo manuale o se abiliti i controlli automatici.",
    "update_checks_checkbox": "Controlla automaticamente gli aggiornamenti all’avvio",
    "update_check_now": "Controlla aggiornamenti",
    "update_current": "Aura è aggiornata",
    "update_available_pattern": "È disponibile Aura v{version}",
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
    "copy_context_desc": "Quando è abilitata, Aura invia Ctrl+C e passa il testo selezionato al provider o al modello locale come contesto per un comando di modifica. Disattiva questa funzione quando lavori in un terminale.",
    "copy_context_checkbox": "Abilita modifica del testo selezionato",
    "ai_edit_title": "Comandi sulla selezione",
    "ai_edit_desc": "Comandi vocali per il testo selezionato: «traduci», «correggi la grammatica». Il comando si applica alla selezione; la dettatura semplice viene inserita invariata.",
    "ai_edit_mode_label": "Modalità di modifica",
    "ai_edit_mode_auto": "Auto",
    "ai_edit_mode_cloud": "Cloud",
    "ai_edit_mode_local": "Locale",
    "ai_edit_mode_off": "Disattivo",
    "ai_edit_mode_sub": "Auto usa il modello locale se installato, altrimenti il provider cloud. Per i comandi sulla selezione, attiva «Consenti acquisizione della selezione e comandi di modifica».",
    "translate_target_label": "Lingua di traduzione predefinita",
    "translate_target_sub": "Utilizzata quando pronunci «traduci» senza specificare una lingua.",
    "edit_cloud_provider_label": "Servizio IA cloud",
    "edit_cloud_provider_sub": "Modello che esegue comandi di traduzione e correzione del testo.",
    "edit_cloud_provider_fallback_sub": "Utilizzato come fallback se il modello locale non è installato o disponibile.",
    "llm_model_label": "Modello locale",
    "model_meta_llm_0_6b": "~460 MB — più leggera, comandi semplici",
    "model_meta_llm_1_7b": "~1,2 GB — bilanciata",
    "model_meta_llm_4b": "~2,3 GB — qualità massima",
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
    "gpu_status_installing": "Installazione...",
    "replacements_title": "Sostituzione automatica",
    "replacements_desc": "Sostituisce automaticamente parole e frasi dettate con termini esatti, indirizzi email o testo personalizzato.",
    "replacements_trigger_ph": "Parola o frase (es. email)",
    "replacements_value_ph": "Sostituisci con (es. name@mail.com)",
    "replacements_caret_hint": "Il simbolo | nel testo sostitutivo indica dove posizionare il cursore dopo l'inserimento (es. Ciao, |!).",
    "replacements_macro_hint": "Macro disponibili: {date} (AAAA-MM-GG), {date_ru} (GG.MM.AAAA), {time} (OO:MM), {datetime}.",
    "replacements_macros_label": "Inserisci modello:",
    "macro_chip_date_ru": "+ Data (RU)",
    "macro_chip_date_iso": "+ Data (ISO)",
    "macro_chip_time": "+ Ora",
    "macro_chip_datetime": "+ Data e ora",
    "btn_add": "Aggiungi",
    "replacements_empty": "Nessuna regola di sostituzione configurata",
    "replacements_delete_aria": "Elimina regola di sostituzione",
    "gaming_title": "Modalità Gioco",
    "gaming_desc": "Mette in pausa automaticamente gli hook della tastiera e scarica i modelli dalla VRAM per il massimo FPS nei giochi.",
    "gaming_mode_label": "Attiva modalità Gioco manualmente",
    "gaming_mode_subdesc": "Disattiva gli hook della tastiera e scarica i modelli dalla memoria video (VRAM).",
    "gaming_auto_label": "Modalità Gioco automatica",
    "gaming_auto_desc": "Sospende i tasti di scelta rapida e libera la memoria video (VRAM) all'avvio di giochi 3D a schermo intero.",
    "gaming_status_label": "Stato attuale",
    "gaming_status_inactive": "Inattivo",
    "gaming_status_active": "Attivo (in pausa)",
    "gaming_btn_toggle_on": "Attiva manualmente",
    "gaming_btn_toggle_off": "Disattiva",
    "gaming_tray_hint": "La commutazione manuale è disponibile anche dal menu della barra delle applicazioni di Aura.",
    "standby_title": "Sospensione modelli per inattività",
    "standby_timeout_label": "Tempo di inattività prima della sospensione",
    "standby_desc": "Libera la memoria video (VRAM) se la digitazione vocale non viene utilizzata per il tempo indicato.",
    "standby_15m": "Dopo 15 minuti",
    "standby_30m": "Dopo 30 minuti (consigliato)",
    "standby_60m": "Dopo 1 ora",
    "standby_never": "Mai (mantieni in VRAM)",
    "window_minimize": "Riduci a icona",
    "window_close": "Chiudi",
    "history_copy_title": "Copia negli appunti",
    "history_search_clear": "Cancella ricerca",
    "api_key_toggle_visibility": "Mostra/nascondi chiave",
    "history_filters_label": "Filtri della cronologia",
    "onboarding_skip": "Salta introduzione",
    "onboarding_prev": "Indietro",
    "onboarding_next": "Avanti",
    "onboarding_finish": "Inizia",
    "onboarding_replay": "Tour guidato delle funzionalità",
    "onboarding_step_tag": "{current} / {total}",
    "onboarding_step_1_title": "Metodi di riconoscimento vocale",
    "onboarding_step_1_desc": "L'IA locale garantisce il 100% di privacy senza invio di dati alla rete, accelerata da GPU (NVIDIA CUDA) o CPU. L'IA cloud non richiede download e offre elaborazione rapida via API senza carico sul sistema.",
    "onboarding_step_2_title": "Lingua di riconoscimento e dizionario",
    "onboarding_step_2_desc": "Aura può adattarsi automaticamente alla lingua della tastiera attiva. Nel dizionario puoi configurare regole di sostituzione per refusi e terminologia specifica.",
    "onboarding_step_3_title": "Inserimento intelligente e modifica del testo selezionato",
    "onboarding_step_3_desc": "Seleziona il testo in qualsiasi programma, premi la scorciatoia e pronuncia un comando: 'correggi errori' o 'traduci' — Aura modificherà il testo direttamente sul posto.",
    "onboarding_step_4_title": "Scorciatoie globali, overlay e feedback audio",
    "onboarding_step_4_desc": "Il tasto principale avvia la registrazione ovunque. Delicati clic confermano l'inizio e la fine, e un overlay mobile mostra stato e tempo trascorso.",
    "onboarding_step_5_title": "Cronologia trascrizioni e barra delle applicazioni",
    "onboarding_step_5_desc": "Tutti i testi dettati sono salvati nella cronologia per un rapido riutilizzo. Aura resta silenziosa nella barra delle applicazioni, pronta all'uso con la scorciatoia.",
    "commands_guide_title": "Comandi vocali supportati",
    "commands_guide_desc": "Azioni disponibili per il testo selezionato.",
    "commands_guide_label": "Comandi supportati",
    "commands_guide_sub": "Frasi vocali per azioni sul testo selezionato",
    "commands_guide_btn": "Guida comandi",
    "commands_guide_close": "Chiudi",
    "commands_guide_active_summary": "{active} su {total} comandi attivi",
    "smart_context_title": "Inserimento contestuale intelligente",
    "smart_context_desc": "Adatta automaticamente maiuscole/minuscole, punteggiatura e spaziatura in base alla posizione del cursore.",
    "smart_context_checkbox": "Adatta al contesto attorno al cursore",
    "smart_context_subdesc": "Inizia con la minuscola a metà frase, rimuove i punti finali superflui e gestisce gli spazi in modo pulito.",
    "smart_context_trailing_period_checkbox": "Termina con un punto quando si continua alla fine della frase",
    "smart_context_trailing_period_subdesc": "Se il cursore si trova alla fine di una frase aperta, termina con un punto. Se disattivato, lascia la frase aperta per continuare la dettatura.",
    "sound_preview_btn": "Ascolta tema",
    "sound_preview_label": "Test",
    "api_key_status_saved": "Salvato in Windows DPAPI",
    "api_key_shared_hint": "Chiave unificata Aura: valida per dettatura e modifica IA.",
    "api_key_btn_change": "Modifica"
  },
  "zh": {
    "title_settings": "设置",
    "tab_general": "常规",
    "tab_speech": "语音",
    "tab_text": "文本",
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
    "engine_health_parakeet_gaming": "Parakeet：服务器已卸载（游戏模式已启用）",
    "engine_health_whisper_running": "Whisper：服务器运行中（{provider}，端口 {port}）",
    "engine_health_whisper_stopped": "Whisper：服务器未运行",
    "engine_health_whisper_gaming": "Whisper：服务器已卸载（游戏模式已启用）",
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
    "sound_theme_bubble": "水滴音",
    "sound_theme_haptic": "触感微动",
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
    "history_group_today": "今天",
    "history_group_yesterday": "昨天",
    "about_app_title": "Aura 智能语音输入",
    "about_version": "v1.2.0",
    "about_description": "适用于 Windows 的全局语音输入工具。本程序可以将语音转录为文本，并以自动格式和标点符号插入到任何活动窗口中。",
    "about_gratitude_text": "感谢您使用 Aura！如果它让您的输入更高效便捷，欢迎在 GitHub 上为本项目点亮一颗星。",
    "btn_star_github": "在 GitHub 上点亮 Star",
    "about_row_tour_title": "功能导览",
    "about_row_tour_desc": "核心功能交互式引导介绍",
    "about_row_changelog_title": "此版本的新功能",
    "about_row_changelog_desc": "更新日志、性能提升及版本发布",
    "about_row_report_issue_title": "反馈问题",
    "about_row_report_issue_desc": "在 GitHub 上提交问题或建议",
    "about_row_star_title": "在 GitHub 上点赞",
    "about_row_star_desc": "在开源社区支持该项目",
    "about_btn_exit_app": "退出 Aura",
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
    "status_streaming_degraded": "实时预览已暂停——完整文本将在录音结束后显示",
    "model_downloading_pattern": "正在启动模型 {model} 的下载...",
    "model_download_error_pattern": "下载错误: {err}",
    "delete_model_title": "删除模型",
    "delete_model_confirm_pattern": "您确定要删除本地模型 {model} 吗？",
    "delete_model_btn": "删除",
    "model_deleting_pattern": "正在删除模型 {model}...",
    "model_deleted_success": "模型删除成功",
    "model_delete_error_pattern": "删除错误: {err}",
    "model_downloaded_success_pattern": "模型 {model} 已下载",
    "confirm_clear_history_title": "清空历史",
    "confirm_clear_history_msg": "您确定要清空所有听写历史记录吗？",
    "general_ui_lang_title": "界面语言",
    "general_ui_lang_desc": "选择设置和应用程序通知的语言。",
    "update_checks_title": "更新检查",
    "update_checks_desc": "Aura 仅在您手动检查或启用自动检查时连接 GitHub。",
    "update_checks_checkbox": "启动时自动检查更新",
    "update_check_now": "检查更新",
    "update_current": "Aura 已是最新版本",
    "update_available_pattern": "Aura v{version} 可用",
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
    "copy_context_desc": "启用后，Aura 会发送 Ctrl+C，并将选中文本作为编辑指令的上下文传给服务提供商或本地模型。在终端中工作时请关闭此功能。",
    "copy_context_checkbox": "启用选中文本编辑",
    "ai_edit_title": "选区指令",
    "ai_edit_desc": "针对选中文本的语音指令：“翻译”、“修正语法”。指令作用于选区；普通听写原样插入。",
    "ai_edit_mode_label": "编辑模式",
    "ai_edit_mode_auto": "自动",
    "ai_edit_mode_cloud": "云端",
    "ai_edit_mode_local": "本地",
    "ai_edit_mode_off": "关闭",
    "ai_edit_mode_sub": "自动模式在已安装本地模型时使用本地模型，否则使用云端服务。要使用选区指令，请开启“允许捕获选区和编辑指令”。",
    "translate_target_label": "默认翻译目标语言",
    "translate_target_sub": "当语音指令仅为“翻译”且未指定语言时使用。",
    "edit_cloud_provider_label": "云端 AI 服务",
    "edit_cloud_provider_sub": "执行翻译和文本纠错指令的云端模型。",
    "edit_cloud_provider_fallback_sub": "当本地模型未安装或不可用时作为备用服务。",
    "llm_model_label": "本地模型",
    "model_meta_llm_0_6b": "~460 MB — 最小，基础指令",
    "model_meta_llm_1_7b": "~1.2 GB — 均衡",
    "model_meta_llm_4b": "~2.3 GB — 最佳质量",
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
    "gpu_status_installing": "正在安装...",
    "replacements_title": "自动替换",
    "replacements_desc": "自动将口述词语和短语替换为精准术语、电子邮件地址或自定义文本。",
    "replacements_trigger_ph": "口述词或短语 (例如: 邮箱)",
    "replacements_value_ph": "替换为 (例如: name@mail.com)",
    "replacements_caret_hint": "替换文本中的 | 符号指示插入后光标的位置（例如：你好，|！）。",
    "replacements_macro_hint": "可用宏: {date} (YYYY-MM-DD), {date_ru} (DD.MM.YYYY), {time} (HH:MM), {datetime}。",
    "replacements_macros_label": "插入模板:",
    "macro_chip_date_ru": "+ 日期 (RU)",
    "macro_chip_date_iso": "+ 日期 (ISO)",
    "macro_chip_time": "+ 时间",
    "macro_chip_datetime": "+ 日期与时间",
    "btn_add": "添加",
    "replacements_empty": "暂未配置自动替换规则",
    "replacements_delete_aria": "删除替换规则",
    "gaming_title": "游戏模式",
    "gaming_desc": "自动暂停按键拦截并从显存中卸载模型，以保证游戏中的最高帧率。",
    "gaming_mode_label": "手动开启游戏模式",
    "gaming_mode_subdesc": "禁用键盘钩子并从显存中卸载模型。",
    "gaming_auto_label": "全屏 3D 游戏自动检测",
    "gaming_auto_desc": "启动全屏 3D 游戏时暂停快捷键并释放显存 (VRAM)。",
    "gaming_status_label": "当前状态",
    "gaming_status_inactive": "未激活",
    "gaming_status_active": "已激活 (已暂停)",
    "gaming_btn_toggle_on": "手动开启",
    "gaming_btn_toggle_off": "关闭",
    "gaming_tray_hint": "也可以通过 Windows 系统托盘菜单手动切换。",
    "standby_title": "闲置时释放模型（睡眠模式）",
    "standby_timeout_label": "进入睡眠前的闲置时间",
    "standby_desc": "在选定时间内未使用语音输入时释放显存 (VRAM)。",
    "standby_15m": "15 分钟后",
    "standby_30m": "30 分钟后 (推荐)",
    "standby_60m": "1 小时后",
    "standby_never": "从不 (常驻显存)",
    "window_minimize": "最小化",
    "window_close": "关闭",
    "history_copy_title": "复制到剪贴板",
    "history_search_clear": "清除搜索",
    "api_key_toggle_visibility": "显示/隐藏密钥",
    "history_filters_label": "历史记录筛选",
    "onboarding_skip": "跳过教程",
    "onboarding_prev": "上一步",
    "onboarding_next": "下一步",
    "onboarding_finish": "开始使用",
    "onboarding_replay": "功能入门导览",
    "onboarding_step_tag": "{current} / {total}",
    "onboarding_step_1_title": "语音识别方式",
    "onboarding_step_1_desc": "本地 AI 提供 100% 隐私保护，无需将音频上传至网络，并支持 GPU（NVIDIA CUDA）或 CPU 硬件加速。云端 AI 无需下载模型，通过 API 快速处理且不占用系统资源。",
    "onboarding_step_2_title": "识别语言与自定义词汇表",
    "onboarding_step_2_desc": "Aura 可根据当前键盘布局自动匹配识别语言。您可以在词汇表中设置文本替换规则，以纠正错别字或识别专业术语。",
    "onboarding_step_3_title": "智能上下文插入与选中文本编辑",
    "onboarding_step_3_desc": "在任何应用中选中文本，按快捷键并说出指令（如“纠正错误”或“翻译”），Aura 即可就地完成文本编辑与替换。",
    "onboarding_step_4_title": "全局快捷键、悬浮层与声音反馈",
    "onboarding_step_4_desc": "主快捷键可在任意界面快速启动录音。开始与结束伴有轻柔的点击提示音，屏幕上会显示录音状态与时间的悬浮层。",
    "onboarding_step_5_title": "听写历史记录与静默托盘运行",
    "onboarding_step_5_desc": "所有听写文本均安全保存在历史记录中以便快速复用。Aura 静默常驻 Windows 托盘，随时按快捷键即可开始听写。",
    "commands_guide_title": "支持的语音指令",
    "commands_guide_desc": "选中文本可用的操作。",
    "commands_guide_label": "支持的指令",
    "commands_guide_sub": "针对选中文本执行操作的语音短语",
    "commands_guide_btn": "指令指南",
    "commands_guide_close": "关闭",
    "commands_guide_active_summary": "{active} / {total} 个指令已启用",
    "smart_context_title": "智能光标上下文插入",
    "smart_context_desc": "根据光标在文档中的位置，自动调整大小写、标点符号及前后空格。",
    "smart_context_checkbox": "根据光标上下文智能调整",
    "smart_context_subdesc": "在句中插入时自动使用小写并去除结尾句号，精确处理两端空格。",
    "smart_context_trailing_period_checkbox": "句末续写时自动添加句号",
    "smart_context_trailing_period_subdesc": "若光标处于未完成句子末尾，则以句号结束。关闭时保持开放以便继续语音输入。",
    "sound_preview_btn": "试听音效",
    "sound_preview_label": "测试",
    "api_key_status_saved": "已保存在 Windows DPAPI",
    "api_key_shared_hint": "Aura 统一密钥：适用于语音听写和 AI 编辑。",
    "api_key_btn_change": "更改"
  },
  "pt": {
    "title_settings": "Configurações",
    "tab_general": "Geral",
    "tab_speech": "Voz",
    "tab_text": "Texto",
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
    "engine_health_parakeet_gaming": "Parakeet: servidor descarregado (Modo de Jogo ativo)",
    "engine_health_whisper_running": "Whisper: servidor em execução ({provider}, porta {port})",
    "engine_health_whisper_stopped": "Whisper: servidor não em execução",
    "engine_health_whisper_gaming": "Whisper: servidor descarregado (Modo de Jogo ativo)",
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
    "sound_theme_bubble": "Gota de água",
    "sound_theme_haptic": "Interruptor tátil",
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
    "history_group_today": "Hoje",
    "history_group_yesterday": "Ontem",
    "about_app_title": "Ditado de voz Aura",
    "about_version": "v1.2.0",
    "about_description": "Ferramenta de entrada de voz global para Windows. O programa transcreve a fala em texto e a insere em qualquer janela ativa com formatação e pontuação automáticas.",
    "about_gratitude_text": "Obrigado por usar o Aura! Se ele torna seu dia a dia mais ágil, apoie o projeto com uma estrela no GitHub.",
    "btn_star_github": "Dar uma estrela no GitHub",
    "about_row_tour_title": "Tour de recursos",
    "about_row_tour_desc": "Guia interativo dos principais recursos",
    "about_row_changelog_title": "Novidades desta versão",
    "about_row_changelog_desc": "Registro de alterações, melhorias e lançamentos",
    "about_row_report_issue_title": "Relatar um problema",
    "about_row_report_issue_desc": "Enviar relatório de bug ou feedback no GitHub",
    "about_row_star_title": "Dar uma estrela no GitHub",
    "about_row_star_desc": "Apoie o projeto de código aberto",
    "about_btn_exit_app": "Sair do Aura",
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
    "status_streaming_degraded": "Pré-visualização ao vivo pausada — o texto completo aparecerá ao encerrar a gravação",
    "model_downloading_pattern": "Iniciando download para {model}...",
    "model_download_error_pattern": "Erro de download: {err}",
    "delete_model_title": "Excluir modelo",
    "delete_model_confirm_pattern": "Tem certeza de que deseja excluir o modelo local {model}?",
    "delete_model_btn": "Excluir",
    "model_deleting_pattern": "Excluindo modelo {model}...",
    "model_deleted_success": "Modelo excluído com sucesso",
    "model_delete_error_pattern": "Erro ao excluir: {err}",
    "model_downloaded_success_pattern": "Modelo {model} baixado",
    "confirm_clear_history_title": "Limpar histórico",
    "confirm_clear_history_msg": "Tem certeza de que deseja limpar todo o histórico de transcrições?",
    "general_ui_lang_title": "Idioma da interface",
    "general_ui_lang_desc": "Selecione o idioma para as configurações e notificações do aplicativo.",
    "update_checks_title": "Verificação de atualizações",
    "update_checks_desc": "A Aura só acessa o GitHub quando você verifica manualmente ou ativa as verificações automáticas.",
    "update_checks_checkbox": "Verificar atualizações automaticamente ao iniciar",
    "update_check_now": "Verificar atualizações",
    "update_current": "A Aura está atualizada",
    "update_available_pattern": "A versão {version} da Aura está disponível",
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
    "copy_context_desc": "Quando ativado, o Aura envia Ctrl+C e encaminha o texto selecionado ao provedor ou ao modelo local como contexto para um comando de edição. Desative este recurso ao trabalhar em um terminal.",
    "copy_context_checkbox": "Ativar edição de texto selecionado",
    "ai_edit_title": "Comandos sobre a seleção",
    "ai_edit_desc": "Comandos de voz para o texto selecionado: «traduzir», «corrigir gramática». O comando é aplicado à seleção; a ditado comum é inserida sem alterações.",
    "ai_edit_mode_label": "Modo de edição",
    "ai_edit_mode_auto": "Auto",
    "ai_edit_mode_cloud": "Nuvem",
    "ai_edit_mode_local": "Local",
    "ai_edit_mode_off": "Desativado",
    "ai_edit_mode_sub": "Auto usa o modelo local se estiver instalado; caso contrário, o provedor de nuvem. Para comandos sobre a seleção, ative «Permitir captura da seleção e comandos de edição».",
    "translate_target_label": "Idioma padrão para tradução",
    "translate_target_sub": "Usado ao dizer «traduza» sem especificar um idioma de destino.",
    "edit_cloud_provider_label": "Serviço de IA na nuvem",
    "edit_cloud_provider_sub": "Modelo que executa comandos de tradução e correção de texto.",
    "edit_cloud_provider_fallback_sub": "Usado como contingência se o modelo local não estiver instalado ou disponível.",
    "llm_model_label": "Modelo local",
    "model_meta_llm_0_6b": "~460 MB — o menor, comandos básicos",
    "model_meta_llm_1_7b": "~1,2 GB — equilibrada",
    "model_meta_llm_4b": "~2,3 GB — qualidade máxima",
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
    "gpu_status_installing": "Instalando...",
    "replacements_title": "Substituição automática",
    "replacements_desc": "Substitui automaticamente palavras e frases ditadas por termos exatos, endereços de email ou texto personalizado.",
    "replacements_trigger_ph": "Palavra ou frase (ex: email)",
    "replacements_value_ph": "Substituir por (ex: name@mail.com)",
    "replacements_caret_hint": "O símbolo | no texto de substituição indica onde posicionar o cursor após a inserção (ex: Olá, |!).",
    "replacements_macro_hint": "Macros disponíveis: {date} (AAAA-MM-DD), {date_ru} (DD.MM.AAAA), {time} (HH:MM), {datetime}.",
    "replacements_macros_label": "Inserir modelo:",
    "macro_chip_date_ru": "+ Data (RU)",
    "macro_chip_date_iso": "+ Data (ISO)",
    "macro_chip_time": "+ Hora",
    "macro_chip_datetime": "+ Data e hora",
    "btn_add": "Adicionar",
    "replacements_empty": "Nenhuma regra de substituição configurada ainda",
    "replacements_delete_aria": "Excluir regra de substituição",
    "gaming_title": "Modo de Jogo",
    "gaming_desc": "Pausa automaticamente os ganchos de teclado e descarrega modelos da VRAM para FPS máximo em jogos.",
    "gaming_mode_label": "Ativar Modo de Jogo manualmente",
    "gaming_mode_subdesc": "Desativa ganchos de teclado e descarrega modelos da memória de vídeo (VRAM).",
    "gaming_auto_label": "Modo de Jogo Automático",
    "gaming_auto_desc": "Suspende as teclas de atalho e libera memória de vídeo (VRAM) ao iniciar jogos 3D em tela cheia.",
    "gaming_status_label": "Status atual",
    "gaming_status_inactive": "Inativo",
    "gaming_status_active": "Ativo (pausado)",
    "gaming_btn_toggle_on": "Ativar manualmente",
    "gaming_btn_toggle_off": "Desativar",
    "gaming_tray_hint": "A alternância manual também está disponível no menu do Aura na bandeja do sistema.",
    "standby_title": "Liberação de modelos por inatividade (Repouso)",
    "standby_timeout_label": "Tempo de inatividade antes do repouso",
    "standby_desc": "Libera memória de vídeo (VRAM) se a digitação por voz não for usada durante o tempo selecionado.",
    "standby_15m": "Após 15 minutos",
    "standby_30m": "Após 30 minutos (recomendado)",
    "standby_60m": "Após 1 hora",
    "standby_never": "Nunca (manter na VRAM)",
    "window_minimize": "Minimizar",
    "window_close": "Fechar",
    "history_copy_title": "Copiar para a área de transferência",
    "history_search_clear": "Limpar pesquisa",
    "api_key_toggle_visibility": "Mostrar/ocultar chave",
    "history_filters_label": "Filtros do histórico",
    "onboarding_skip": "Pular tutorial",
    "onboarding_prev": "Voltar",
    "onboarding_next": "Avançar",
    "onboarding_finish": "Começar",
    "onboarding_replay": "Visita guiada de recursos",
    "onboarding_step_tag": "{current} / {total}",
    "onboarding_step_1_title": "Métodos de reconhecimento de voz",
    "onboarding_step_1_desc": "A IA local garante 100% de privacidade sem envio de áudio para a rede, acelerada por GPU (NVIDIA CUDA) ou CPU. A IA em nuvem não requer downloads e processa rapidamente via API sem carga no sistema.",
    "onboarding_step_2_title": "Idioma de reconhecimento e dicionário",
    "onboarding_step_2_desc": "O Aura pode se adaptar automaticamente ao idioma do teclado ativo. No dicionário, configure regras de substituição para termos técnicos e erros de digitação.",
    "onboarding_step_3_title": "Inserção inteligente e edição do texto selecionado",
    "onboarding_step_3_desc": "Selecione o texto em qualquer aplicativo, pressione o atalho e dite um comando: 'corrija erros' ou 'traduza' — o Aura editará o texto diretamente no local.",
    "onboarding_step_4_title": "Atalhos globais, overlay e resposta sonora",
    "onboarding_step_4_desc": "A tecla principal inicia a gravação em qualquer lugar. Cliques suaves confirmam início e parada, e um overlay flutuante exibe status e tempo de gravação.",
    "onboarding_step_5_title": "Histórico de transcrições e bandeja do sistema",
    "onboarding_step_5_desc": "Todos os textos ditados são salvos no histórico para reutilização. O Aura opera discretamente na bandeja do sistema, pronto para uso com seu atalho.",
    "commands_guide_title": "Comandos de voz suportados",
    "commands_guide_desc": "Ações disponíveis para o texto selecionado.",
    "commands_guide_label": "Comandos suportados",
    "commands_guide_sub": "Frases de voz para ações no texto selecionado",
    "commands_guide_btn": "Guia de comandos",
    "commands_guide_close": "Fechar",
    "commands_guide_active_summary": "{active} de {total} comandos ativos",
    "smart_context_title": "Inserção contextual inteligente",
    "smart_context_desc": "Adapta automaticamente maiúsculas/minúsculas, pontuação e espaçamento de acordo com a posição do cursor.",
    "smart_context_checkbox": "Adaptar ao contexto ao redor do cursor",
    "smart_context_subdesc": "Inicia com letra minúscula no meio da frase, remove pontos finais extras e ajusta os espaços de forma limpa.",
    "smart_context_trailing_period_checkbox": "Finalizar com ponto ao continuar no fim da frase",
    "smart_context_trailing_period_subdesc": "Se o cursor estiver no fim de uma frase aberta, finaliza com ponto. Quando desativado, mantém aberto para mais ditado.",
    "sound_preview_btn": "Ouvir tema",
    "sound_preview_label": "Testar",
    "api_key_status_saved": "Salvo no Windows DPAPI",
    "api_key_shared_hint": "Chave unificada do Aura: aplica-se ao ditado e à edição por IA.",
    "api_key_btn_change": "Alterar"
  },
  "tr": {
    "title_settings": "Ayarlar",
    "tab_general": "Genel",
    "tab_speech": "Ses",
    "tab_text": "Metin",
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
    "engine_health_parakeet_gaming": "Parakeet: sunucu bellekten kaldırıldı (Oyun Modu aktif)",
    "engine_health_whisper_running": "Whisper: sunucu çalışıyor ({provider}, port {port})",
    "engine_health_whisper_stopped": "Whisper: sunucu çalışmıyor",
    "engine_health_whisper_gaming": "Whisper: sunucu bellekten kaldırıldı (Oyun Modu aktif)",
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
    "sound_theme_bubble": "Su Damlası",
    "sound_theme_haptic": "Haptik Anahtar",
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
    "history_group_today": "Bugün",
    "history_group_yesterday": "Dün",
    "about_app_title": "Aura Sesli Giriş",
    "about_version": "v1.2.0",
    "about_description": "Windows için genel sesli giriş aracı. Program, konuşmayı metne dönüştürür ve otomatik biçimlendirme ve noktalama işaretleriyle herhangi bir aktif pencereye ekler.",
    "about_gratitude_text": "Aura'yı kullandığınız için teşekkürler! Çalışmanızı hızlandırıyorsa, projeyi GitHub'da yıldızlayarak destekleyebilirsiniz.",
    "btn_star_github": "GitHub'da Yıldız Ver",
    "about_row_tour_title": "Özellik Turu",
    "about_row_tour_desc": "Temel özelliklere ilişkin etkileşimli rehber",
    "about_row_changelog_title": "Bu Sürümdeki Yenilikler",
    "about_row_changelog_desc": "Değişiklik günlüğü, performans iyileştirmeleri ve sürümler",
    "about_row_report_issue_title": "Hata Bildir",
    "about_row_report_issue_desc": "GitHub üzerinden hata veya geri bildirim gönderin",
    "about_row_star_title": "GitHub'da Yıldız Ver",
    "about_row_star_desc": "Açık kaynaklı projeyi destekleyin",
    "about_btn_exit_app": "Aura'dan Çık",
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
    "status_streaming_degraded": "Canlı önizleme duraklatıldı — tam metin kayıt bittiğinde görünecek",
    "model_downloading_pattern": "{model} modeli için indirme başlatılıyor...",
    "model_download_error_pattern": "İndirme hatası: {err}",
    "delete_model_title": "Modeli sil",
    "delete_model_confirm_pattern": "Yerel {model} modelini silmek istediğinizden emin misiniz?",
    "delete_model_btn": "Sil",
    "model_deleting_pattern": "{model} modeli siliniyor...",
    "model_deleted_success": "Model başarıyla silindi",
    "model_delete_error_pattern": "Silme hatası: {err}",
    "model_downloaded_success_pattern": "{model} modeli indirildi",
    "confirm_clear_history_title": "Geçmişi Temizle",
    "confirm_clear_history_msg": "Tüm transkripsiyon geçmişini temizlemek istediğinizden emin misiniz?",
    "general_ui_lang_title": "Arayüz Dili",
    "general_ui_lang_desc": "Ayarlar ve uygulama bildirimleri için dili seçin.",
    "update_checks_title": "Güncelleme denetimi",
    "update_checks_desc": "Aura, GitHub’a yalnızca elle denetlediğinizde veya otomatik denetimleri etkinleştirdiğinizde bağlanır.",
    "update_checks_checkbox": "Başlangıçta güncellemeleri otomatik olarak denetle",
    "update_check_now": "Güncellemeleri denetle",
    "update_current": "Aura güncel",
    "update_available_pattern": "Aura v{version} kullanılabilir",
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
    "copy_context_desc": "Etkinleştirildiğinde Aura, Ctrl+C gönderir ve seçili metni bir düzenleme komutu için bağlam olarak seçilen bulut sağlayıcısına veya yerel modele iletir. Terminalde çalışırken bu özelliği devre dışı bırakın.",
    "copy_context_checkbox": "Seçili metni düzenlemeyi etkinleştir",
    "ai_edit_title": "Seçim üzerinde komutlar",
    "ai_edit_desc": "Seçili metin için sesli komutlar: «çevir», «dilbilgisini düzelt». Komut seçime uygulanır; normal dikte olduğu gibi eklenir.",
    "ai_edit_mode_label": "Düzenleme modu",
    "ai_edit_mode_auto": "Otomatik",
    "ai_edit_mode_cloud": "Bulut",
    "ai_edit_mode_local": "Yerel",
    "ai_edit_mode_off": "Kapalı",
    "ai_edit_mode_sub": "Otomatik, yerel model kuruluysa onu kullanır; değilse bulut sağlayıcısını kullanır. Seçim komutları için «Seçim yakalamaya ve düzenleme komutlarına izin ver» seçeneğini etkinleştirin.",
    "translate_target_label": "Varsayılan çeviri dili",
    "translate_target_sub": "Hedef dil belirtilmeden «çevir» komutu verildiğinde kullanılır.",
    "edit_cloud_provider_label": "Bulut Yapay Zeka Servisi",
    "edit_cloud_provider_sub": "Metin çeviri ve düzeltme komutlarını yürüten model.",
    "edit_cloud_provider_fallback_sub": "Yerel model kurulu değilse veya kullanılamıyorsa yedek olarak kullanılır.",
    "llm_model_label": "Yerel model",
    "model_meta_llm_0_6b": "~460 MB — en küçük, temel komutlar",
    "model_meta_llm_1_7b": "~1,2 GB — dengeli",
    "model_meta_llm_4b": "~2,3 GB — en iyi kalite",
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
    "gpu_status_installing": "Yükleniyor...",
    "replacements_title": "Otomatik Değiştirme",
    "replacements_desc": "Dikte edilen kelimeleri ve ifadeleri otomatik olarak kesin terimlerle, e-posta adresleriyle veya özel metinle değiştirir.",
    "replacements_trigger_ph": "Kelime veya ifade (örn. e-posta)",
    "replacements_value_ph": "Şununla değiştir (örn. name@mail.com)",
    "replacements_caret_hint": "Değiştirme metnindeki | simgesi, eklemeden sonra imlecin nereye yerleştirileceğini belirtir (örn. Merhaba, |!).",
    "replacements_macro_hint": "Kullanılabilir makrolar: {date} (YYYY-AA-GG), {date_ru} (GG.AA.YYYY), {time} (SS:DD), {datetime}.",
    "replacements_macros_label": "Şablon ekle:",
    "macro_chip_date_ru": "+ Tarih (RU)",
    "macro_chip_date_iso": "+ Tarih (ISO)",
    "macro_chip_time": "+ Saat",
    "macro_chip_datetime": "+ Tarih ve saat",
    "btn_add": "Ekle",
    "replacements_empty": "Henüz yapılandırılmış değiştirme kuralı yok",
    "replacements_delete_aria": "Değiştirme kuralını sil",
    "gaming_title": "Oyun Modu",
    "gaming_desc": "Oyunlarda maksimum FPS için klavye kancalarını otomatik duraklatır ve modelleri video belleğinden kaldırır.",
    "gaming_mode_label": "Oyun Modunu manuel etkinleştir",
    "gaming_mode_subdesc": "Klavye kancalarını devre dışı bırakır ve modelleri video belleğinden (VRAM) kaldırır.",
    "gaming_auto_label": "Otomatik Oyun Modu",
    "gaming_auto_desc": "Tam ekran 3D oyunlar başlatıldığında kısayol tuşlarını askıya alır ve video belleğini (VRAM) serbest bırakır.",
    "gaming_status_label": "Mevcut durum",
    "gaming_status_inactive": "Etkin değil",
    "gaming_status_active": "Aktif (duraklatıldı)",
    "gaming_btn_toggle_on": "Manuel olarak etkinleştir",
    "gaming_btn_toggle_off": "Devre dışı bırak",
    "gaming_tray_hint": "Manuel geçişe Windows sistem tepsisindeki Aura menüsünden de erişilebilir.",
    "standby_title": "Boşta Model Boşaltma (Uyku Modu)",
    "standby_timeout_label": "Uyku öncesi boşta kalma süresi",
    "standby_desc": "Sesli yazma seçilen süre boyunca kullanılmadığında video belleğini (VRAM) serbest bırakır.",
    "standby_15m": "15 dakika sonra",
    "standby_30m": "30 dakika sonra (önerilen)",
    "standby_60m": "1 saat sonra",
    "standby_never": "Asla (VRAM'de tut)",
    "window_minimize": "Küçült",
    "window_close": "Kapat",
    "history_copy_title": "Panoya kopyala",
    "history_search_clear": "Aramayı temizle",
    "api_key_toggle_visibility": "Anahtarı göster/gizle",
    "history_filters_label": "Geçmiş filtreleri",
    "onboarding_skip": "Öğreticiyi atla",
    "onboarding_prev": "Geri",
    "onboarding_next": "İleri",
    "onboarding_finish": "Başla",
    "onboarding_replay": "Özellik tanıtım turu",
    "onboarding_step_tag": "{current} / {total}",
    "onboarding_step_1_title": "Konuşma Tanıma Yöntemleri",
    "onboarding_step_1_desc": "Yerel Yapay Zeka, ses verilerini ağa aktarmadan %100 gizlilik sağlar ve GPU (NVIDIA CUDA) veya CPU ile hızlandırılır. Bulut Yapay Zeka ise model indirmeye gerek kalmadan API üzerinden sıfır sistem yüküyle anında çalışır.",
    "onboarding_step_2_title": "Tanıma Dili ve Sözlük",
    "onboarding_step_2_desc": "Aura, aktif klavye düzeninin diline otomatik olarak uyum sağlayabilir. Sözlükte yazım hatalarını düzeltmek ve teknik terimleri tanımlamak için metin değiştirme kuralları belirleyebilirsiniz.",
    "onboarding_step_3_title": "Akıllı Ekleme ve Seçili Metni Düzenleme",
    "onboarding_step_3_desc": "Herhangi bir uygulamada metni seçin, kısayola basın ve bir komut söyleyin: 'hataları düzelt' veya 'çevir' — Aura metni doğrudan yerinde düzenler.",
    "onboarding_step_4_title": "Genel Kısayollar, Overlay ve Sesli Geri Bildirim",
    "onboarding_step_4_desc": "Ana kısayol tuşu herhangi bir yerde kaydı başlatır. Hafif sesli tıklamalar başlangıç ve duruşu doğrular, hareketli bir overlay ise kayıt süresini ve durumunu gösterir.",
    "onboarding_step_5_title": "Metin Geçmişi ve Sistem Tepsisinde Sessiz Çalışma",
    "onboarding_step_5_desc": "Dikte edilen tüm metinler yeniden kopyalanmak üzere geçmişte saklanır. Aura sistem tepsisinde sessizce bekler ve kısayolunuza anında yanıt verir.",
    "commands_guide_title": "Desteklenen Sesli Komutlar",
    "commands_guide_desc": "Seçilen metin üzerindeki kullanılabilir eylemler.",
    "commands_guide_label": "Desteklenen Komutlar",
    "commands_guide_sub": "Seçili metin üzerinde eylemler için sesli ifadeler",
    "commands_guide_btn": "Komut Rehberi",
    "commands_guide_close": "Kapat",
    "commands_guide_active_summary": "{active} / {total} komut aktif",
    "smart_context_title": "Akıllı Bağlamsal Ekleme",
    "smart_context_desc": "İmleç konumuna göre harf boyutunu, noktalama işaretlerini ve boşlukları otomatik olarak uyarlar.",
    "smart_context_checkbox": "İmleç çevresindeki bağlama göre uyarla",
    "smart_context_subdesc": "Cümle ortasında küçük harfle başlar, fazladan noktaları kaldırır ve boşlukları düzenler.",
    "smart_context_trailing_period_checkbox": "Cümle sonunda devam ederken nokta ile bitir",
    "smart_context_trailing_period_subdesc": "İmleç tamamlanmamış cümlenin sonundaysa nokta ekler. Kapatılırsa dikteye devam etmek için açık bırakır.",
    "sound_preview_btn": "Temayı dinle",
    "sound_preview_label": "Test et",
    "api_key_status_saved": "Windows DPAPI'de kayıtlı",
    "api_key_shared_hint": "Birleşik Aura anahtarı: sesli dikte ve yapay zeka düzenlemesi için geçerlidir.",
    "api_key_btn_change": "Değiştir"
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
    panel.classList.add("closing");
    if (panel._closeTimer) clearTimeout(panel._closeTimer);
    panel._closeTimer = setTimeout(() => {
      panel.classList.remove("closing");
      panel._closeTimer = null;
    }, 120);
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
    if (panel._closeTimer) {
      clearTimeout(panel._closeTimer);
      panel._closeTimer = null;
    }
    panel.classList.remove("closing");
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

const COMMANDS_METADATA = [
  {
    id: "fix_grammar",
    names: {
      ru: "Исправление грамматики",
      en: "Fix Grammar & Typos",
      de: "Grammatik & Tippfehler korrigieren",
      es: "Corregir gramática y erratas",
      fr: "Corriger la grammaire et les fautes",
      it: "Correggi grammatica e refusi",
      zh: "修正语法和错别字",
      pt: "Corrigir gramática e digitação",
      tr: "Dilbilgisi ve Yazım Hatalarını Düzelt"
    },
    descriptions: {
      ru: "Исправляет орфографию, пунктуацию и опечатки в выделенном фрагменте, сохраняя стиль и смысл автора.",
      en: "Corrects spelling, punctuation, and typos in the selection while preserving original meaning and voice.",
      de: "Korrigiert Rechtschreibung, Zeichensetzung und Tippfehler unter Beibehaltung von Ton und Sinn.",
      es: "Corrige ortografía, puntuación y erratas en el texto seleccionado manteniendo el estilo y significado.",
      fr: "Corrige l'orthographe, la ponctuation et les coquilles dans la sélection en conservant le sens et le ton.",
      it: "Corregge ortografia, punteggiatura ed errori di battitura preservando stile e significato originari.",
      zh: "纠正选中文本中的拼写、标点和错别字，同时保留原有语调和含义。",
      pt: "Corrige ortografia, pontuação e erros de digitação mantendo o tom e sentido originais.",
      tr: "Seçilen metindeki yazım, noktalama ve yazım hatalarını yazarın üslubunu koruyarak düzeltir."
    },
    chips: {
      ru: ["«исправь ошибки»", "«проверь текст»", "«исправь пунктуацию»"],
      en: ['"fix grammar"', '"fix errors"', '"proofread"'],
      de: ['„Grammatik korrigieren“', '„Fehler beheben“', '„Text prüfen“'],
      es: ['«corrige errores»', '«revisa la gramática»', '«corrige el texto»'],
      fr: ['« corrige les fautes »', '« vérifie le texte »', '« corrige la grammaire »'],
      it: ['"correggi errori"', '"controlla grammatica"', '"correggi il testo"'],
      zh: ['“修正语法”', '“检查错误”', '“修改语病”'],
      pt: ['«corrija erros»', '«corrija a gramática»', '«revise o texto»'],
      tr: ['"hataları düzelt"', '"dilbilgisini düzelt"', '"metni kontrol et"']
    }
  },
  {
    id: "translate",
    names: {
      ru: "Перевод текста",
      en: "Translate",
      de: "Übersetzen",
      es: "Traducir",
      fr: "Traduire",
      it: "Traduci",
      zh: "翻译文本",
      pt: "Traduzir",
      tr: "Metni Çevir"
    },
    descriptions: {
      ru: "Переводит выделенный текст на указанный язык («на немецкий») или на язык перевода по умолчанию.",
      en: "Translates the selection to the specified language (e.g. 'to German') or default translation target.",
      de: "Übersetzt den ausgewählten Text in die gewünschte Sprache (z. B. „auf Deutsch“) oder Standardsprache.",
      es: "Traduce el texto seleccionado al idioma indicado (p. ej. «al alemán») o al idioma predeterminado.",
      fr: "Traduit la sélection dans la langue demandée (ex. « en allemand ») ou la langue par défaut.",
      it: "Traduce la selezione nella lingua richiesta (es. 'in tedesco') o nella lingua predefinita.",
      zh: "将选中文本翻译成指定语言（如“翻译成德语”）或默认目标语言。",
      pt: "Traduz o texto selecionado para o idioma especificado (ex.: «para alemão») ou idioma padrão.",
      tr: "Seçilen metni belirtilen dile (ör. 'Almancaya çevir') veya varsayılan çeviri diline çevirir."
    },
    chips: {
      ru: ["«переведи»", "«переведи на английский»", "«переведи на немецкий»"],
      en: ['"translate"', '"translate to english"', '"translate to german"'],
      de: ['„übersetzen“', '„auf Englisch übersetzen“', '„auf Spanisch übersetzen“'],
      es: ['«traduce»', '«traduce al inglés»', '«traduce al francés»'],
      fr: ['« traduis »', '« traduis en anglais »', '« traduis en espagnol »'],
      it: ['"traduci"', '"traduci in inglese"', '"traduci in tedesco"'],
      zh: ['“翻译”', '“翻译成英文”', '“翻译成中文”'],
      pt: ['«traduza»', '«traduza para inglês»', '«traduza para espanhol»'],
      tr: ['"çevir"', '"İngilizceye çevir"', '"Almancaya çevir"']
    }
  }
];

function getTranslation(key, params = {}) {
  const dict = i18nDict[currentLanguage] || i18nDict.ru;
  let template = dict[key] || i18nDict.ru[key] || key;
  for (const [k, v] of Object.entries(params)) {
    template = template.replaceAll(`{${k}}`, v);
  }
  return template;
}

document.addEventListener("DOMContentLoaded", () => {
  let isMicTesting = false;
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

  const initialTabMatch = (location.hash || "").replace("#", "") || new URLSearchParams(location.search).get("tab");
  if (initialTabMatch) {
    const targetTab = Array.from(tabs).find((t) => t.dataset.tab === initialTabMatch);
    if (targetTab) activateTab(targetTab);
  }

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

    const animateIn = (el) => {
      if (!el) return;
      el.classList.remove("engine-switch-fade");
      void el.offsetWidth;
      el.classList.add("engine-switch-fade");
    };

    if (radioLocal.checked) {
      if (localEngineSection) {
        localEngineSection.style.display = "flex";
        animateIn(localEngineSection);
      }
      updateLocalEngineUI();
      if (fallbackCard) fallbackCard.style.display = "none";
      if (apiKeysCard) apiKeysCard.style.display = "none";
    } else {
      if (localEngineSection) localEngineSection.style.display = "none";
      if (apiKeysCard) {
        apiKeysCard.style.display = "flex";
        animateIn(apiKeysCard);
      }
      if (fallbackCard) {
        fallbackCard.style.display = "flex";
        animateIn(fallbackCard);
      }
      if (langCard) langCard.style.display = "none";
      if (vocabCard) vocabCard.style.display = "none";
      if (streamingCard) streamingCard.style.display = "none";
    }
  }

  function updateLocalEngineUI() {
    if (!selectLocalEngine || !groupWhisperModels || !groupParakeetModels) return;
    const isParakeet = selectLocalEngine.value === "parakeet";
    const vocabCard = document.getElementById("card-vocabulary");
    const langCard = document.getElementById("card-language");
    const streamingCard = document.getElementById("card-streaming");
    const gpuSettings = document.getElementById("gpu-acceleration-settings");

    const animateIn = (el) => {
      if (!el) return;
      el.classList.remove("engine-switch-fade");
      void el.offsetWidth;
      el.classList.add("engine-switch-fade");
    };

    if (vocabCard) {
      vocabCard.style.display = isParakeet ? "none" : "flex";
      if (!isParakeet) animateIn(vocabCard);
    }
    if (langCard) {
      langCard.style.display = isParakeet ? "none" : "flex";
      if (!isParakeet) animateIn(langCard);
    }
    if (streamingCard) {
      streamingCard.style.display = isParakeet ? "flex" : "none";
      if (isParakeet) animateIn(streamingCard);
    }
    if (gpuSettings) {
      gpuSettings.style.display = "block";
    }
    if (isParakeet) {
      groupWhisperModels.style.display = "none";
      groupParakeetModels.style.display = "block";
      animateIn(groupParakeetModels);
      selectModelCard("parakeet-v3");
    } else {
      groupWhisperModels.style.display = "block";
      groupParakeetModels.style.display = "none";
      animateIn(groupWhisperModels);
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
      if (health.starting) {
        chip.innerHTML = `<span class="spinner-inline" aria-hidden="true"></span> ${getTranslation("engine_starting")}`;
        chip.classList.add("health-warn");
        return;
      }
      if (!health.running && health.gaming_mode) {
        const gamingKey = health.engine === "whisper"
          ? "engine_health_whisper_gaming"
          : "engine_health_parakeet_gaming";
        chip.textContent = getTranslation(gamingKey);
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
  // server reports running (or stops starting).
  function kickEngineStatusPolling(maxAttempts = 30) {
    if (engineFastPollTimer) return;
    let attempts = 0;
    engineFastPollTimer = setInterval(async () => {
      attempts++;
      await refreshEngineHealth();
      const running = Boolean(lastHealthState && lastHealthState.running);
      const starting = Boolean(lastHealthState && lastHealthState.starting);
      if (running || (!starting && attempts >= 6) || attempts >= maxAttempts) {
        clearInterval(engineFastPollTimer);
        engineFastPollTimer = null;
      }
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
  const linkGetKeyEdit = document.getElementById("link-get-key-edit");
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
    const text = getTranslation("api_get_key_pattern", { name: info.name });
    if (linkGetKey) {
      linkGetKey.href = info.url;
      linkGetKey.textContent = text;
    }
    if (linkGetKeyEdit) {
      linkGetKeyEdit.href = info.url;
      linkGetKeyEdit.textContent = text;
    }
  }

  function updateCustomProviderUI() {
    const isCustom = (selectProvider && selectProvider.value === "custom") || (selectEditCloudProvider && selectEditCloudProvider.value === "custom");
    const urlGroup = document.getElementById("custom-provider-url-group");
    const modelGroup = document.getElementById("custom-provider-model-group");
    const editUrlGroup = document.getElementById("edit-custom-provider-url-group");
    const editModelGroup = document.getElementById("edit-custom-provider-model-group");
    const animateIn = (el) => {
      if (!el) return;
      el.classList.remove("engine-switch-fade");
      void el.offsetWidth;
      el.classList.add("engine-switch-fade");
    };
    [urlGroup, modelGroup, editUrlGroup, editModelGroup].forEach((group) => {
      if (group) {
        const wasHidden = group.style.display === "none";
        group.style.display = isCustom ? "flex" : "none";
        if (isCustom && wasHidden) {
          animateIn(group);
        }
      }
    });
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
  const selectEditCloudProvider = document.getElementById("select-edit-cloud-provider");
  const apiKeyInputEdit = document.getElementById("input-edit-api-key");
  const toggleKeyEditBtn = document.getElementById("btn-toggle-key-edit");
  const inputCustomUrl = document.getElementById("input-custom-url");
  const inputCustomModel = document.getElementById("input-custom-model");
  const inputEditCustomUrl = document.getElementById("input-edit-custom-url");
  const inputEditCustomModel = document.getElementById("input-edit-custom-model");
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
  const selectAiEditMode = document.getElementById("select-ai-edit-mode");
  const selectTranslateTarget = document.getElementById("select-translate-target");
  const selectionEditInner = document.getElementById("selection-edit-inner");
  const syncSelectionEditInner = () => {
    if (selectionEditInner && checkboxCopyContext) {
      selectionEditInner.classList.toggle("is-expanded", !!checkboxCopyContext.checked);
    }
  };
  const checkboxSmartContext = document.getElementById("checkbox-smart-context-enabled");
  const checkboxSmartContextTrailingPeriod = document.getElementById("checkbox-smart-context-trailing-period");
  const smartContextSuboptions = document.getElementById("smart-context-suboptions");
  const syncSmartContextSuboptions = () => {
    if (smartContextSuboptions && checkboxSmartContext) {
      smartContextSuboptions.classList.toggle("is-expanded", !!checkboxSmartContext.checked);
    }
  };
  const soundSuboptions = document.getElementById("sound-suboptions");
  const syncSoundSuboptions = () => {
    if (soundSuboptions && checkboxSounds) {
      soundSuboptions.classList.toggle("is-expanded", !!checkboxSounds.checked);
    }
  };
  const selectSoundTheme = document.getElementById("select-sound-theme");
  const rangeVolume = document.getElementById("range-sound-volume");
  const volumeLabel = document.getElementById("volume-value-label");
  const selectAudioDevice = document.getElementById("select-audio-device");
  let lastSoundVolume = 80;

  let settingsAudioCtx = null;
  let settingsGlobalGain = null;

  function initSettingsAudioCtx(vol) {
    if (!settingsAudioCtx) {
      settingsAudioCtx = new (window.AudioContext || window.webkitAudioContext)();
      settingsGlobalGain = settingsAudioCtx.createGain();
      settingsGlobalGain.connect(settingsAudioCtx.destination);
    }
    if (settingsAudioCtx.state === "suspended") {
      settingsAudioCtx.resume();
    }
    const safeVol = Math.max(0, Math.min(1.0, vol / 100));
    settingsGlobalGain.gain.setValueAtTime(safeVol, settingsAudioCtx.currentTime);
  }

  function playSettingsBowl(freq, duration, gainStart = 0.08) {
    const now = settingsAudioCtx.currentTime;
    const components = [
      { f: freq, gain: gainStart, decay: duration },
      { f: freq * 1.5, gain: gainStart * 0.45, decay: duration * 0.8 },
      { f: freq * 2.76, gain: gainStart * 0.3, decay: duration * 0.55 }
    ];
    components.forEach(comp => {
      const oscA = settingsAudioCtx.createOscillator();
      const gainA = settingsAudioCtx.createGain();
      oscA.type = "sine";
      oscA.frequency.setValueAtTime(comp.f - 0.7, now);

      const oscB = settingsAudioCtx.createOscillator();
      const gainB = settingsAudioCtx.createGain();
      oscB.type = "sine";
      oscB.frequency.setValueAtTime(comp.f + 0.7, now);

      const attack = 0.015;
      gainA.gain.setValueAtTime(0.0001, now);
      gainA.gain.linearRampToValueAtTime(comp.gain, now + attack);
      gainA.gain.exponentialRampToValueAtTime(0.0001, now + comp.decay);

      gainB.gain.setValueAtTime(0.0001, now);
      gainB.gain.linearRampToValueAtTime(comp.gain, now + attack);
      gainB.gain.exponentialRampToValueAtTime(0.0001, now + comp.decay);

      oscA.connect(gainA);
      gainA.connect(settingsGlobalGain);
      oscB.connect(gainB);
      gainB.connect(settingsGlobalGain);

      oscA.start(now);
      oscA.stop(now + comp.decay + 0.1);
      oscB.start(now);
      oscB.stop(now + comp.decay + 0.1);
    });
  }

  function playSettingsRhodes(freq, duration, gainStart = 0.08) {
    const now = settingsAudioCtx.currentTime;
    const osc1 = settingsAudioCtx.createOscillator();
    const osc2 = settingsAudioCtx.createOscillator();
    const gain1 = settingsAudioCtx.createGain();
    const gain2 = settingsAudioCtx.createGain();

    osc1.type = "triangle";
    osc1.frequency.setValueAtTime(freq, now);
    osc2.type = "sine";
    osc2.frequency.setValueAtTime(freq, now);

    const attack = 0.010;
    gain1.gain.setValueAtTime(0.0001, now);
    gain1.gain.linearRampToValueAtTime(gainStart * 0.4, now + attack);
    gain1.gain.exponentialRampToValueAtTime(0.0001, now + duration);

    gain2.gain.setValueAtTime(0.0001, now);
    gain2.gain.linearRampToValueAtTime(gainStart * 0.7, now + attack);
    gain2.gain.exponentialRampToValueAtTime(0.0001, now + duration);

    osc1.connect(gain1);
    gain1.connect(settingsGlobalGain);
    osc2.connect(gain2);
    gain2.connect(settingsGlobalGain);

    osc1.start(now);
    osc1.stop(now + duration + 0.1);
    osc2.start(now);
    osc2.stop(now + duration + 0.1);
  }

  function playSettingsSciFi(freqStart, freqEnd, duration, gainStart = 0.06) {
    const now = settingsAudioCtx.currentTime;
    const osc1 = settingsAudioCtx.createOscillator();
    const osc2 = settingsAudioCtx.createOscillator();
    const gainNode = settingsAudioCtx.createGain();

    osc1.type = "sine";
    osc1.frequency.setValueAtTime(freqStart, now);
    osc1.frequency.exponentialRampToValueAtTime(freqEnd, now + duration);

    osc2.type = "sine";
    osc2.frequency.setValueAtTime(freqStart + 3.0, now);
    osc2.frequency.exponentialRampToValueAtTime(freqEnd + 3.0, now + duration);

    const attack = 0.012;
    gainNode.gain.setValueAtTime(0.0001, now);
    gainNode.gain.linearRampToValueAtTime(gainStart, now + attack);
    gainNode.gain.exponentialRampToValueAtTime(0.0001, now + duration);

    osc1.connect(gainNode);
    osc2.connect(gainNode);
    gainNode.connect(settingsGlobalGain);

    osc1.start(now);
    osc1.stop(now + duration + 0.1);
    osc2.start(now);
    osc2.stop(now + duration + 0.1);
  }

  function playSettingsBell(freq, duration, gainStart = 0.07) {
    const now = settingsAudioCtx.currentTime;
    const osc1 = settingsAudioCtx.createOscillator();
    const osc2 = settingsAudioCtx.createOscillator();
    const osc3 = settingsAudioCtx.createOscillator();
    const gain1 = settingsAudioCtx.createGain();
    const gain2 = settingsAudioCtx.createGain();
    const gain3 = settingsAudioCtx.createGain();

    osc1.type = "sine";
    osc1.frequency.setValueAtTime(freq, now);
    osc2.type = "sine";
    osc2.frequency.setValueAtTime(freq * 2.0, now);
    osc3.type = "sine";
    osc3.frequency.setValueAtTime(freq * 3.0, now);

    const attack = 0.006;
    gain1.gain.setValueAtTime(0.0001, now);
    gain1.gain.linearRampToValueAtTime(gainStart, now + attack);
    gain1.gain.exponentialRampToValueAtTime(0.0001, now + duration);

    gain2.gain.setValueAtTime(0.0001, now);
    gain2.gain.linearRampToValueAtTime(gainStart * 0.45, now + attack);
    gain2.gain.exponentialRampToValueAtTime(0.0001, now + duration * 0.7);

    gain3.gain.setValueAtTime(0.0001, now);
    gain3.gain.linearRampToValueAtTime(gainStart * 0.25, now + attack);
    gain3.gain.exponentialRampToValueAtTime(0.0001, now + duration * 0.4);

    osc1.connect(gain1);
    gain1.connect(settingsGlobalGain);
    osc2.connect(gain2);
    gain2.connect(settingsGlobalGain);
    osc3.connect(gain3);
    gain3.connect(settingsGlobalGain);

    osc1.start(now);
    osc1.stop(now + duration + 0.1);
    osc2.start(now);
    osc2.stop(now + duration + 0.1);
    osc3.start(now);
    osc3.stop(now + duration + 0.1);
  }

  function playSettingsWaterDrop(fStart, fEnd, duration, gainStart = 0.08) {
    const now = settingsAudioCtx.currentTime;
    const osc = settingsAudioCtx.createOscillator();
    const gain = settingsAudioCtx.createGain();

    osc.type = "sine";
    osc.frequency.setValueAtTime(fStart, now);
    osc.frequency.exponentialRampToValueAtTime(fEnd, now + (duration * 0.65));
    osc.frequency.exponentialRampToValueAtTime(fEnd * 0.9, now + duration);

    gain.gain.setValueAtTime(0.0001, now);
    gain.gain.linearRampToValueAtTime(gainStart, now + 0.008);
    gain.gain.exponentialRampToValueAtTime(0.0001, now + duration);

    osc.connect(gain);
    gain.connect(settingsGlobalGain);

    osc.start(now);
    osc.stop(now + duration + 0.02);
  }

  function playSettingsHapticClick(clickFreq, duration, gainStart = 0.10) {
    const now = settingsAudioCtx.currentTime;
    const osc = settingsAudioCtx.createOscillator();
    const gain = settingsAudioCtx.createGain();

    osc.type = "sine";
    osc.frequency.setValueAtTime(clickFreq * 2.5, now);
    osc.frequency.exponentialRampToValueAtTime(clickFreq * 0.6, now + duration);

    gain.gain.setValueAtTime(0.0001, now);
    gain.gain.linearRampToValueAtTime(gainStart, now + 0.002);
    gain.gain.exponentialRampToValueAtTime(0.0001, now + duration);

    const bufferSize = Math.floor(settingsAudioCtx.sampleRate * 0.008);
    const noiseBuffer = settingsAudioCtx.createBuffer(1, bufferSize, settingsAudioCtx.sampleRate);
    const output = noiseBuffer.getChannelData(0);
    for (let i = 0; i < bufferSize; i++) {
      output[i] = Math.random() * 2 - 1;
    }
    const noise = settingsAudioCtx.createBufferSource();
    noise.buffer = noiseBuffer;
    const filter = settingsAudioCtx.createBiquadFilter();
    filter.type = "bandpass";
    filter.frequency.value = 2800;
    const noiseGain = settingsAudioCtx.createGain();
    noiseGain.gain.setValueAtTime(gainStart * 0.35, now);
    noiseGain.gain.exponentialRampToValueAtTime(0.0001, now + 0.008);

    noise.connect(filter);
    filter.connect(noiseGain);
    noiseGain.connect(settingsGlobalGain);
    noise.start(now);

    osc.connect(gain);
    gain.connect(settingsGlobalGain);
    osc.start(now);
    osc.stop(now + duration);
  }

  function playSettingsThemePreview(vol) {
    if (vol <= 0) return;
    const theme = (selectSoundTheme && selectSoundTheme.value) || "zen";
    try {
      initSettingsAudioCtx(vol);
      if (theme === "rhodes") {
        playSettingsRhodes(440.00, 0.65, 0.07);
        setTimeout(() => { playSettingsRhodes(554.37, 0.80, 0.07); }, 90);
      } else if (theme === "scifi") {
        playSettingsSciFi(440.00, 880.00, 0.22, 0.055);
      } else if (theme === "classic") {
        playSettingsBell(440.00, 0.45, 0.06);
        setTimeout(() => { playSettingsBell(659.25, 0.60, 0.06); }, 90);
      } else if (theme === "bubble") {
        playSettingsWaterDrop(400, 950, 0.12, 0.08);
      } else if (theme === "haptic") {
        playSettingsHapticClick(180, 0.03, 0.12);
      } else {
        // Default: zen
        playSettingsBowl(174.61, 1.2, 0.08);
      }
    } catch (_) {}
  }

  function updateVolumeSliderVisuals(val) {
    const v = Math.max(0, Math.min(100, Math.round(val)));
    if (rangeVolume) {
      rangeVolume.value = v;
      rangeVolume.style.background = `linear-gradient(to right, var(--accent-color) 0%, var(--accent-color) ${v}%, hsl(0, 0%, 20%) ${v}%, hsl(0, 0%, 20%) 100%)`;
    }
    if (volumeLabel) {
      volumeLabel.textContent = `${v}%`;
    }
  }

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
      if (selectAudioDevice.value !== currentVal) {
        selectAudioDevice.value = "default";
      }
      buildSelectPanel(selectAudioDevice);
      syncPanelSelection(selectAudioDevice);
    } catch (e) {
      console.error("Failed to load audio input devices", e);
    }
  }

  if (rangeVolume) {
    rangeVolume.addEventListener("input", () => {
      const v = Number(rangeVolume.value);
      updateVolumeSliderVisuals(v);
    });

    rangeVolume.addEventListener("change", () => {
      const v = Number(rangeVolume.value);
      updateVolumeSliderVisuals(v);
      if (v > 0) {
        lastSoundVolume = v;
      }
      if (checkboxSounds && checkboxSounds.checked && v > 0) {
        playSettingsThemePreview(v);
      }
    });
  }

  if (selectSoundTheme) {
    selectSoundTheme.addEventListener("change", () => {
      const cur = Number(rangeVolume ? rangeVolume.value : 80);
      if (checkboxSounds && checkboxSounds.checked && cur > 0) {
        playSettingsThemePreview(cur);
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
    syncSoundSuboptions();
  }
  if (checkboxSounds) {
    checkboxSounds.addEventListener("change", updateSoundUI);
    updateSoundUI();
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
  let previousSelProvider = selectProvider ? selectProvider.value : "gemini";

  function renderProviderKeyInput() {
    const provider = selectProvider ? selectProvider.value : "gemini";
    const val = apiKeys[provider] || "";
    const providerDict = i18nDict[currentLanguage] || i18nDict.ru;
    const placeholder = apiKeyPresent[provider]
      ? providerDict.key_saved_placeholder || "•••••••• (saved securely)"
      : providerDict.key_placeholder || "Enter API key";

    if (apiKeyInput) {
      apiKeyInput.value = val;
      apiKeyInput.placeholder = placeholder;
    }
    if (apiKeyInputEdit) {
      apiKeyInputEdit.value = val;
      apiKeyInputEdit.placeholder = placeholder;
    }
  }

  function handleProviderChange(newProvider) {
    if (apiKeyInput) apiKeys[previousSelProvider] = apiKeyInput.value;
    previousSelProvider = newProvider;
    if (selectProvider && selectProvider.value !== newProvider) {
      selectProvider.value = newProvider;
      syncPanelSelection(selectProvider);
    }
    if (selectEditCloudProvider && selectEditCloudProvider.value !== newProvider) {
      selectEditCloudProvider.value = newProvider;
      syncPanelSelection(selectEditCloudProvider);
    }
    renderProviderKeyInput();
    updateApiKeyLink();
    updateCustomProviderUI();
    updateEngineUI();
    markSettingsModified(true);
  }

  if (apiKeyInput) {
    apiKeyInput.addEventListener("input", () => {
      const provider = selectProvider ? selectProvider.value : "gemini";
      apiKeys[provider] = apiKeyInput.value;
      apiKeyDirty[provider] = true;
      if (apiKeyInputEdit) apiKeyInputEdit.value = apiKeyInput.value;
    });
  }

  if (apiKeyInputEdit) {
    apiKeyInputEdit.addEventListener("input", () => {
      const provider = selectProvider ? selectProvider.value : (selectEditCloudProvider ? selectEditCloudProvider.value : "gemini");
      apiKeys[provider] = apiKeyInputEdit.value;
      apiKeyDirty[provider] = true;
      if (apiKeyInput) apiKeyInput.value = apiKeyInputEdit.value;
    });
  }

  if (toggleKeyEditBtn && apiKeyInputEdit) {
    toggleKeyEditBtn.addEventListener("click", () => {
      if (apiKeyInputEdit.type === "password") {
        apiKeyInputEdit.type = "text";
        toggleKeyEditBtn.classList.add("visible");
      } else {
        apiKeyInputEdit.type = "password";
        toggleKeyEditBtn.classList.remove("visible");
      }
    });
  }

  if (inputEditCustomUrl && inputCustomUrl) {
    inputEditCustomUrl.addEventListener("input", () => {
      inputCustomUrl.value = inputEditCustomUrl.value;
      markSettingsModified(false);
    });
    inputCustomUrl.addEventListener("input", () => {
      inputEditCustomUrl.value = inputCustomUrl.value;
    });
  }

  if (inputEditCustomModel && inputCustomModel) {
    inputEditCustomModel.addEventListener("input", () => {
      inputCustomModel.value = inputEditCustomModel.value;
      markSettingsModified(false);
    });
    inputCustomModel.addEventListener("input", () => {
      inputEditCustomModel.value = inputCustomModel.value;
    });
  }

  if (selectProvider) {
    selectProvider.addEventListener("change", () => {
      handleProviderChange(selectProvider.value);
    });
  }

  if (selectEditCloudProvider) {
    selectEditCloudProvider.addEventListener("change", () => {
      handleProviderChange(selectEditCloudProvider.value);
    });
  }

  let selectedModelName = "base";
  let lastSelectedWhisperModel = "large-v3-turbo-q5_0";
  let selectedLlmModel = "qwen3-1.7b";
  let disabledEditCommands = [];
  let downloadedModelList = [];
  let activeLocalAcceleration = "cpu";

  let settingsModified = false;
  let isSettingsLoaded = false;
  let hasCompletedOnboarding = false;
  let autoSaveTimeout = null;
  let settingsRevision = 0;
  let engineHealthTimer = null;
  let engineFastPollTimer = null;
  let lastHealthState = null;
  let updateGamingModeUi = () => {};

  function markSettingsModified(immediate = false) {
    if (!isSettingsLoaded) return;
    settingsModified = true;
    settingsRevision += 1;

    if (autoSaveTimeout) {
      clearTimeout(autoSaveTimeout);
      autoSaveTimeout = null;
    }

    const delay = immediate ? 150 : 700;
    autoSaveTimeout = setTimeout(() => {
      autoSaveTimeout = null;
      if (settingsModified) {
        saveSettings();
      }
    }, delay);
  }

  let currentReplacements = [];

  function renderReplacementsList(replacements) {
    const container = document.getElementById("replacements-list");
    if (!container) return;
    container.innerHTML = "";

    if (!replacements || replacements.length === 0) {
      const empty = document.createElement("div");
      empty.className = "replacements-empty";
      empty.setAttribute("data-i18n", "replacements_empty");
      empty.textContent = getTranslation("replacements_empty") || "Правила автозамены не настроены";
      container.appendChild(empty);
      return;
    }

    replacements.forEach((rep, index) => {
      const item = document.createElement("div");
      item.className = "replacement-item";

      const info = document.createElement("div");
      info.className = "replacement-info";

      const trigger = document.createElement("span");
      trigger.className = "replacement-trigger";
      trigger.textContent = rep.trigger;

      const arrow = document.createElement("span");
      arrow.className = "replacement-arrow";
      arrow.setAttribute("aria-hidden", "true");
      arrow.innerHTML = '<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><line x1="5" y1="12" x2="19" y2="12"></line><polyline points="12 5 19 12 12 19"></polyline></svg>';

      const val = document.createElement("span");
      val.className = "replacement-value";
      val.textContent = rep.replacement;

      info.appendChild(trigger);
      info.appendChild(arrow);
      info.appendChild(val);

      const btnDel = document.createElement("button");
      btnDel.type = "button";
      btnDel.className = "replacement-delete";
      const delLabel = getTranslation("replacements_delete_aria") || "Delete replacement";
      btnDel.setAttribute("aria-label", delLabel);
      btnDel.setAttribute("title", delLabel);
      btnDel.innerHTML = '<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><line x1="18" y1="6" x2="6" y2="18"></line><line x1="6" y1="6" x2="18" y2="18"></line></svg>';
      btnDel.addEventListener("click", () => {
        currentReplacements.splice(index, 1);
        renderReplacementsList(currentReplacements);
        markSettingsModified(true);
      });

      item.appendChild(info);
      item.appendChild(btnDel);
      container.appendChild(item);
    });
  }

  function setupReplacementsUI() {
    const btnAdd = document.getElementById("btn-add-replacement");
    const inputTrigger = document.getElementById("input-rep-trigger");
    const inputValue = document.getElementById("input-rep-value");

    function addCurrentReplacement() {
      const trig = inputTrigger ? inputTrigger.value.trim() : "";
      const val = inputValue ? inputValue.value.trim() : "";
      if (!trig || !val) return;

      const existingIdx = currentReplacements.findIndex(r => r.trigger.toLowerCase() === trig.toLowerCase());
      if (existingIdx >= 0) {
        currentReplacements[existingIdx].replacement = val;
      } else {
        currentReplacements.push({ trigger: trig, replacement: val });
      }

      if (inputTrigger) inputTrigger.value = "";
      if (inputValue) inputValue.value = "";
      renderReplacementsList(currentReplacements);
      markSettingsModified(true);
    }

    if (btnAdd) {
      btnAdd.addEventListener("click", addCurrentReplacement);
    }
    if (inputValue) {
      inputValue.addEventListener("keydown", (e) => {
        if (e.key === "Enter") {
          e.preventDefault();
          addCurrentReplacement();
        }
      });
    }
    if (inputTrigger) {
      inputTrigger.addEventListener("keydown", (e) => {
        if (e.key === "Enter") {
          e.preventDefault();
          inputValue?.focus();
        }
      });
    }
  }

  // --- Supported Voice Commands Guide (Inline) ---
  function renderSupportedCommandsGuide() {
    const guideEl = document.getElementById("commands-info-guide");
    if (!guideEl) return;
    guideEl.innerHTML = "";

    const lang = i18nDict[currentLanguage] ? currentLanguage : "ru";

    COMMANDS_METADATA.forEach(cmd => {
      const item = document.createElement("div");
      item.className = "guide-item";

      const title = document.createElement("span");
      title.className = "guide-item-title";
      title.textContent = (cmd.names && (cmd.names[lang] || cmd.names.ru)) || cmd.id;
      item.appendChild(title);

      const chipsContainer = document.createElement("div");
      chipsContainer.className = "command-chips";
      const chipsList = (cmd.chips && (cmd.chips[lang] || cmd.chips.ru)) || [];
      chipsList.forEach(chipText => {
        const chip = document.createElement("span");
        chip.className = "chip";
        chip.textContent = chipText;
        chipsContainer.appendChild(chip);
      });
      item.appendChild(chipsContainer);

      guideEl.appendChild(item);
    });
  }

  let isSettingsListenersBound = false;
  function bindSettingsChangeListeners() {
    if (isSettingsListenersBound) return;
    isSettingsListenersBound = true;

    const checkboxStreaming = document.getElementById("checkbox-streaming");
    const checkboxLogSpeechText = document.getElementById("setting-log-speech-text");
    const checkboxOverlayTimer = document.getElementById("checkbox-overlay-show-timer");
    const inputCustomUrl = document.getElementById("input-custom-url");
    const inputCustomModel = document.getElementById("input-custom-model");
    const checkboxGamingAuto = document.getElementById("checkbox-gaming-mode-auto-detect");
    const selectVramStandby = document.getElementById("select-vram-standby");

    const groupLlmModels = document.getElementById("group-llm-models");
    const inputs = [
      radioCloud, radioLocal, selectProvider, selectEditCloudProvider, apiKeyInput, apiKeyInputEdit,
      inputCustomUrl, inputEditCustomUrl, inputCustomModel, inputEditCustomModel, selectHotkey,
      selectLanguage, textareaDictionary, checkboxToggle, checkboxCloudFallback,
      checkboxAutostart, checkboxAutomaticUpdateChecks, checkboxStreaming, checkboxSounds,
      selectSoundTheme, rangeVolume, selectLocalEngine, checkboxCopyContext, checkboxLogSpeechText,
      checkboxOverlayTimer, checkboxGamingAuto, selectVramStandby, selectAiEditMode, selectTranslateTarget,
      checkboxSmartContext, checkboxSmartContextTrailingPeriod
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

    const syncLlmGroupVisibility = () => {
      const groupCloud = document.getElementById("group-edit-cloud-provider");
      const descCloud = document.getElementById("desc-edit-cloud-provider");
      const animateIn = (el) => {
        if (!el) return;
        el.classList.remove("engine-switch-fade");
        void el.offsetWidth;
        el.classList.add("engine-switch-fade");
      };
      if (selectAiEditMode) {
        const mode = selectAiEditMode.value;
        const needsLocal = mode === "auto" || mode === "local";
        const needsCloud = mode === "auto" || mode === "cloud";
        if (groupLlmModels) {
          const wasHidden = groupLlmModels.style.display === "none";
          groupLlmModels.style.display = needsLocal ? "" : "none";
          if (needsLocal && wasHidden) animateIn(groupLlmModels);
        }
        if (groupCloud) {
          const wasHidden = groupCloud.style.display === "none";
          groupCloud.style.display = needsCloud ? "" : "none";
          if (needsCloud && wasHidden) animateIn(groupCloud);
        }
        if (descCloud) {
          descCloud.textContent = mode === "auto"
            ? getTranslation("edit_cloud_provider_fallback_sub")
            : getTranslation("edit_cloud_provider_sub");
        }
      }
    };
    if (selectAiEditMode) {
      selectAiEditMode.addEventListener("change", syncLlmGroupVisibility);
      syncLlmGroupVisibility();
    }
    if (checkboxCopyContext) {
      checkboxCopyContext.addEventListener("change", syncSelectionEditInner);
      syncSelectionEditInner();
    }
    if (checkboxSmartContext) {
      checkboxSmartContext.addEventListener("change", syncSmartContextSuboptions);
      syncSmartContextSuboptions();
    }

    const checkboxGamingMode = document.getElementById("checkbox-gaming-mode");

    updateGamingModeUi = function(isActive) {
      if (checkboxGamingMode) {
        checkboxGamingMode.checked = isActive;
      }
    };

    let isTogglingGaming = false;
    if (checkboxGamingMode) {
      checkboxGamingMode.addEventListener("change", async () => {
        if (isTogglingGaming) return;
        isTogglingGaming = true;
        checkboxGamingMode.disabled = true;
        try {
          const newState = await invoke("toggle_gaming_mode");
          updateGamingModeUi(!!newState);
          refreshEngineHealth();
        } catch (err) {
          console.error("Failed to toggle gaming mode:", err);
          checkboxGamingMode.checked = !checkboxGamingMode.checked;
        } finally {
          isTogglingGaming = false;
          checkboxGamingMode.disabled = false;
        }
      });
    }

    listen("gaming-mode-changed", (event) => {
      if (typeof event.payload === "boolean") {
        updateGamingModeUi(event.payload);
        refreshEngineHealth();
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

  function selectModelCard(model, markModified = true) {
    if (model === "punctuation") {
      return;
    }
    if (model.startsWith("qwen3-")) {
      // Local edit models are a separate setting (settings.llm_model); they
      // must never overwrite the ASR model_name selection.
      const isDownloaded = downloadedModelList.includes(model);
      if (selectedLlmModel !== model) {
        selectedLlmModel = model;
        if (markModified) {
          markSettingsModified(true);
        }
      }
      modelCards.forEach(c => {
        if (!c.dataset.model.startsWith("qwen3-")) {
          return;
        }
        const isCurrent = c.dataset.model === model && isDownloaded;
        c.classList.toggle("active", isCurrent);
        c.setAttribute("aria-checked", isCurrent ? "true" : "false");
      });
      return;
    }
    if (model !== "parakeet-v3") {
      lastSelectedWhisperModel = model;
    }
    if (selectedModelName !== model) {
      selectedModelName = model;
      if (markModified) {
        markSettingsModified(true);
      }
    }
    modelCards.forEach(c => {
      if (c.dataset.model.startsWith("qwen3-")) {
        return;
      }
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
        if (typeof settings.has_completed_onboarding === "boolean") {
          hasCompletedOnboarding = settings.has_completed_onboarding;
        }
        if (localStorage.getItem("aura_onboarding_completed") === "true") {
          hasCompletedOnboarding = true;
        }
        if (hasCompletedOnboarding) {
          try {
            localStorage.setItem("aura_onboarding_completed", "true");
          } catch (_) {}
        }
        if (settings.transcription_mode === "local") {
          radioLocal.checked = true;
        } else {
          radioCloud.checked = true;
        }
        
        if (settings.model_name && settings.model_name !== "parakeet-v3" && settings.model_name !== "punctuation") {
          lastSelectedWhisperModel = settings.model_name;
        }
        selectModelCard(settings.model_name || "base", false);
        if (typeof settings.ai_edit_mode === "string") {
          const aiEditSelect = document.getElementById("select-ai-edit-mode");
          if (aiEditSelect) {
            aiEditSelect.value = settings.ai_edit_mode === "off" ? "auto" : settings.ai_edit_mode;
            syncPanelSelection(aiEditSelect);
            const groupLlm = document.getElementById("group-llm-models");
            const groupCloud = document.getElementById("group-edit-cloud-provider");
            const descCloud = document.getElementById("desc-edit-cloud-provider");
            const mode = aiEditSelect.value;
            const needsLocal = mode === "auto" || mode === "local";
            const needsCloud = mode === "auto" || mode === "cloud";
            if (groupLlm) {
              groupLlm.style.display = needsLocal ? "" : "none";
            }
            if (groupCloud) {
              groupCloud.style.display = needsCloud ? "" : "none";
            }
            if (descCloud) {
              descCloud.textContent = mode === "auto"
                ? getTranslation("edit_cloud_provider_fallback_sub")
                : getTranslation("edit_cloud_provider_sub");
            }
          }
        }
        if (selectTranslateTarget) {
          selectTranslateTarget.value = settings.translate_default_target || "en";
          syncPanelSelection(selectTranslateTarget);
        }
        if (typeof settings.llm_model === "string" && settings.llm_model.startsWith("qwen3-")) {
          selectedLlmModel = settings.llm_model;
        }
        selectModelCard(selectedLlmModel, false);
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
        if (selectEditCloudProvider) {
          selectEditCloudProvider.value = selectProvider.value;
          syncPanelSelection(selectEditCloudProvider);
        }
        renderProviderKeyInput();
        updateApiKeyLink();
        const inputCustomUrl = document.getElementById("input-custom-url");
        if (inputCustomUrl) {
          inputCustomUrl.value = settings.custom_api_url || "https://api.deepinfra.com/v1/openai";
        }
        const inputEditCustomUrl = document.getElementById("input-edit-custom-url");
        if (inputEditCustomUrl) {
          inputEditCustomUrl.value = settings.custom_api_url || "https://api.deepinfra.com/v1/openai";
        }
        const inputCustomModel = document.getElementById("input-custom-model");
        if (inputCustomModel) {
          inputCustomModel.value = settings.custom_model_name || "openai/whisper-large-v3-turbo";
        }
        const inputEditCustomModel = document.getElementById("input-edit-custom-model");
        if (inputEditCustomModel) {
          inputEditCustomModel.value = settings.custom_model_name || "openai/whisper-large-v3-turbo";
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

    checkboxCopyContext.checked = settings.ai_edit_mode !== "off" && !!settings.copy_context_on_start;
    syncSelectionEditInner();
  }
  if (checkboxSmartContext) {
    checkboxSmartContext.checked = settings.smart_context_enabled !== false;
  }
  if (checkboxSmartContextTrailingPeriod) {
    checkboxSmartContextTrailingPeriod.checked = settings.smart_context_trailing_period !== false;
  }
  syncSmartContextSuboptions();
        const checkboxLogSpeechText = document.getElementById("setting-log-speech-text");
        if (checkboxLogSpeechText) {
          checkboxLogSpeechText.checked = !!settings.log_speech_text;
        }
        const checkboxOverlayTimer = document.getElementById("checkbox-overlay-show-timer");
        if (checkboxOverlayTimer) {
          checkboxOverlayTimer.checked = settings.overlay_show_timer !== false;
        }
        const checkboxGamingMode = document.getElementById("checkbox-gaming-mode");
        if (checkboxGamingMode) {
          checkboxGamingMode.checked = !!settings.gaming_mode_enabled;
        }
        try {
          const isGamingActive = await invoke("get_gaming_mode_state");
          updateGamingModeUi(isGamingActive);
        } catch (_) {
          updateGamingModeUi(!!settings.gaming_mode_enabled);
        }
        const checkboxGamingAuto = document.getElementById("checkbox-gaming-mode-auto-detect");
        if (checkboxGamingAuto) {
          checkboxGamingAuto.checked = !!settings.gaming_mode_auto_detect;
        }
        const selectVramStandby = document.getElementById("select-vram-standby");
        if (selectVramStandby) {
          selectVramStandby.value = String(settings.vram_standby_timeout_mins !== undefined ? settings.vram_standby_timeout_mins : 30);
          syncPanelSelection(selectVramStandby);
        }
        currentReplacements = Array.isArray(settings.text_replacements) ? [...settings.text_replacements] : [];
        renderReplacementsList(currentReplacements);
        activeLocalAcceleration = settings.local_acceleration || "cpu";
        if (selectSoundTheme) {
          selectSoundTheme.value = settings.overlay_sound_theme || "zen";
          syncPanelSelection(selectSoundTheme);
        }
        if (rangeVolume) {
          const volumeVal = typeof settings.overlay_sound_volume === "number" ? Math.round(settings.overlay_sound_volume * 100) : 80;
          updateVolumeSliderVisuals(volumeVal);
          if (volumeVal > 0) lastSoundVolume = volumeVal;
        }
        updateSoundUI();
 
        updateEngineUI();
        await loadAudioDevices(settings.audio_input_device || "default");
        await refreshDownloadedModels();

        activeLocalAcceleration = settings.local_acceleration || "cpu";
        selectGpuProvider(activeLocalAcceleration);
        await updateGpuCardStates();

        disabledEditCommands = Array.isArray(settings.disabled_edit_commands) ? [...settings.disabled_edit_commands] : [];
        renderSupportedCommandsGuide();
        
        isSettingsLoaded = true;
        settingsModified = false;
        
        refreshEngineHealth().then(() => {
          if (lastHealthState && lastHealthState.starting) {
            kickEngineStatusPolling();
          }
        });
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
      downloadedModelList = Array.isArray(downloaded) ? downloaded : [];
      const dict = i18nDict[currentLanguage] || i18nDict.ru;
modelCards.forEach(card => {
        const model = card.dataset.model;
        // Never tear down the UI of a download that is still running
        // (a refresh triggered by a sibling download must not do it either).
        if (inFlightModelDownloads.has(model)) {
          return;
        }
        const isDownloaded = downloadedModelList.includes(model);
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
            <span class="status-ready-badge" title="${dict.model_status_ready || "Установлено"}" aria-label="${dict.model_status_ready || "Установлено"}" data-i18n-title="model_status_ready">
              <svg class="status-ready-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3" stroke-linecap="round" stroke-linejoin="round">
                <polyline points="20 6 9 17 4 12"></polyline>
              </svg>
            </span>
            <button type="button" class="btn-delete-card-model" title="${dict.model_action_delete || "Удалить"}" aria-label="${dict.model_action_delete || "Удалить"}" data-model="${model}">
              <svg class="btn-icon" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="3 6 5 6 21 6"></polyline><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"></path><line x1="10" y1="11" x2="10" y2="17"></line><line x1="14" y1="11" x2="14" y2="17"></line></svg>
            </button>
          `;
          // Bind click to the delete button
          actionEl.querySelector(".btn-delete-card-model").addEventListener("click", (e) => {
            e.stopPropagation();
            deleteModelCard(model);
          });
        } else {
          actionEl.innerHTML = `
            <button type="button" class="btn-download-card-model" data-model="${model}">
              <svg class="btn-icon" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"></path><polyline points="7 10 12 15 17 10"></polyline><line x1="12" y1="15" x2="12" y2="3"></line></svg>
              ${dict.model_action_download || "Скачать"}
            </button>
          `;
// Bind click to the download button
          actionEl.querySelector(".btn-download-card-model").addEventListener("click", (e) => {
            e.stopPropagation();
            downloadModelCard(model);
          });
        }
      });

      // Reconcile Qwen3 LLM cards active state against downloaded models:
      const downloadedLlm = downloadedModelList.filter(id => id.startsWith("qwen3-"));
      if (downloadedLlm.length === 0) {
        modelCards.forEach(c => {
          if (c.dataset.model.startsWith("qwen3-")) {
            c.classList.remove("active");
            c.setAttribute("aria-checked", "false");
          }
        });
      } else {
        if (downloadedLlm.includes(selectedLlmModel)) {
          selectModelCard(selectedLlmModel, false);
        } else {
          selectModelCard(downloadedLlm[0], false);
        }
      }

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
    })(),
    gaming_mode_enabled: (() => {
      const el = document.getElementById("checkbox-gaming-mode");
      return el ? el.checked : false;
    })(),
    gaming_mode_auto_detect: (() => {
      const el = document.getElementById("checkbox-gaming-mode-auto-detect");
      return el ? el.checked : false;
    })(),
    vram_standby_timeout_mins: (() => {
      const el = document.getElementById("select-vram-standby");
      return el ? (parseInt(el.value, 10) || 0) : 30;
    })(),
    text_replacements: currentReplacements,
    ai_edit_mode: (() => {
      const el = document.getElementById("select-ai-edit-mode");
      if (!checkboxCopyContext || !checkboxCopyContext.checked) return "off";
      return el ? el.value : "auto";
    })(),
    translate_default_target: selectTranslateTarget ? selectTranslateTarget.value : "en",
    llm_model: selectedLlmModel,
    has_completed_onboarding: hasCompletedOnboarding || localStorage.getItem("aura_onboarding_completed") === "true",
    disabled_edit_commands: disabledEditCommands,
    smart_context_enabled: (() => {
      const el = document.getElementById("checkbox-smart-context-enabled");
      return el ? el.checked : true;
    })(),
    smart_context_trailing_period: (() => {
      const el = document.getElementById("checkbox-smart-context-trailing-period");
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
        refreshEngineHealth().then(() => {
          if (lastHealthState && lastHealthState.starting) {
            kickEngineStatusPolling();
          }
        });
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
        const currentDict = i18nDict[currentLanguage] || i18nDict.ru;
        showStatus(currentDict.status_ready || "Готово");
      }
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
  function getModelDisplayName(model) {
    if (!model) return "";
    const card = document.querySelector(`.model-card[data-model="${model}"]`);
    if (card) {
      const nameEl = card.querySelector(".model-name");
      if (nameEl && nameEl.textContent) {
        return nameEl.textContent.trim();
      }
    }
    const names = {
      "tiny": "Whisper Tiny",
      "base": "Whisper Base",
      "small": "Whisper Small",
      "medium": "Whisper Medium",
      "large-v3-turbo-q5_0": "Whisper Large v3 Turbo",
      "parakeet-v3": "NVIDIA Parakeet v3",
      "qwen3-0.6b": "Qwen3 0.6B",
      "qwen3-1.7b": "Qwen3 1.7B",
      "qwen3-4b": "Qwen3 4B",
      "punctuation": "Silero Punctuation",
    };
    return names[model] || model;
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
    const llmAtStart = selectedLlmModel;

    try {
      showStatus(getTranslation("model_downloading_pattern", { model: getModelDisplayName(model) }));

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
      // during the (potentially long) download. LLM cards write to a
      // different setting (llm_model), so they compare against their own
      // snapshot instead of the speech model.
      const isLlmCard = model.startsWith("qwen3-");
      const selectionUntouched = isLlmCard
        ? selectedLlmModel === llmAtStart
        : selectedModelName === modelAtStart;
      if (model !== "punctuation" && selectionUntouched) {
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
    const modelDisplayName = getModelDisplayName(model);
    const confirmTitle = getTranslation("delete_model_title");
    const confirmMsg = getTranslation("delete_model_confirm_pattern", { model: modelDisplayName });
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
      showStatus(getTranslation("model_deleting_pattern", { model: modelDisplayName }));
      await invoke("delete_model_command", { modelName: model });
      
      showStatus(getTranslation("model_deleted_success"));
      if (model === "parakeet-v3" && selectLocalEngine?.value === "parakeet") {
        selectLocalEngine.value = "whisper";
        updateLocalEngineUI();
        markSettingsModified();
      }
      // A deleted local edit model can still be the selected one. Move the
      // selection to a model that is still installed so the saved setting
      // never points at a file that no longer exists.
      if (model.startsWith("qwen3-") && selectedLlmModel === model) {
        try {
          const downloaded = await invoke("get_downloaded_models");
          const replacement = (Array.isArray(downloaded) ? downloaded : []).find(
            id => id.startsWith("qwen3-") && id !== model
          );
          if (replacement) {
            selectModelCard(replacement);
            markSettingsModified(true);
          } else {
            selectedLlmModel = "qwen3-1.7b";
            markSettingsModified(true);
          }
        } catch (err) {
          console.error("Failed to reselect local edit model", err);
        }
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

  listen("audio-devices-changed", (event) => {
    console.log("Audio devices changed:", event.payload);
    loadAudioDevices(selectAudioDevice ? selectAudioDevice.value : "default");
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
      const isInternalPayload = typeof model === "string" && (model.startsWith("llm-runtime-") || model === "punctuation");
      if (!isInternalPayload) {
        showStatus(getTranslation("model_downloaded_success_pattern", { model: getModelDisplayName(model) }));
      }
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
          <span class="status-ready-badge" title="${dict.model_status_ready || "Установлено"}" aria-label="${dict.model_status_ready || "Установлено"}" data-i18n-title="model_status_ready">
            <svg class="status-ready-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3" stroke-linecap="round" stroke-linejoin="round"><polyline points="20 6 9 17 4 12"></polyline></svg>
          </span>
          <button type="button" class="btn-delete-card-model" title="${dict.model_action_delete || "Удалить"}" aria-label="${dict.model_action_delete || "Удалить"}" data-gpu="${provider}">
            <svg class="btn-icon" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="3 6 5 6 21 6"></polyline><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"></path><line x1="10" y1="11" x2="10" y2="17"></line><line x1="14" y1="11" x2="14" y2="17"></line></svg>
          </button>
        `;
        actionEl.querySelector(".btn-delete-card-model").addEventListener("click", (e) => {
          e.stopPropagation();
          deleteGpuBinaries(provider);
        });
      } else {
        actionEl.innerHTML = `
          <button type="button" class="btn-download-card-model" data-gpu="${provider}">
            <svg class="btn-icon" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"></path><polyline points="7 10 12 15 17 10"></polyline><line x1="12" y1="15" x2="12" y2="3"></line></svg>
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
  let toastTimer = null;
  let toastHideTimer = null;

  function showToast(msg, type = "info", duration = 3000) {
    const toastEl = document.getElementById("app-toast");
    const toastText = document.getElementById("toast-text");
    const toastIcon = document.getElementById("toast-icon");
    if (!toastEl || !toastText) return;

    if (toastTimer) {
      clearTimeout(toastTimer);
      toastTimer = null;
    }
    if (toastHideTimer) {
      clearTimeout(toastHideTimer);
      toastHideTimer = null;
    }

    toastText.textContent = msg;
    toastEl.classList.remove("toast-error", "toast-success", "toast-info", "toast-hiding");

    if (toastIcon) {
      if (type === "error") {
        toastEl.classList.add("toast-error");
        toastIcon.innerHTML = `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round" width="16" height="16"><circle cx="12" cy="12" r="10"></circle><line x1="12" y1="8" x2="12" y2="12"></line><line x1="12" y1="16" x2="12.01" y2="16"></line></svg>`;
      } else if (type === "success") {
        toastEl.classList.add("toast-success");
        toastIcon.innerHTML = `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round" width="16" height="16"><polyline points="20 6 9 17 4 12"></polyline></svg>`;
      } else {
        toastEl.classList.add("toast-info");
        toastIcon.innerHTML = `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round" width="16" height="16"><circle cx="12" cy="12" r="10"></circle><line x1="12" y1="16" x2="12" y2="12"></line><line x1="12" y1="8" x2="12.01" y2="8"></line></svg>`;
      }
    }

    toastEl.hidden = false;
    void toastEl.offsetWidth;

    if (duration > 0) {
      toastTimer = setTimeout(() => {
        toastEl.classList.add("toast-hiding");
        toastHideTimer = setTimeout(() => {
          toastEl.hidden = true;
          toastEl.classList.remove("toast-hiding");
        }, 180);
      }, duration);
    }
  }

  const SILENT_STATUS_KEYS = new Set([
    "status_ready",
    "status_loading",
    "status_loaded",
    "status_saving",
    "status_saved",
    "status_modified",
  ]);

  function isSilentStatus(msg) {
    if (!msg || typeof msg !== "string" || !msg.trim()) return true;
    const trimmed = msg.trim();
    if (trimmed === "Готово" || trimmed === "Ready") return true;
    for (const lang of Object.keys(i18nDict)) {
      const dict = i18nDict[lang];
      if (!dict) continue;
      for (const key of SILENT_STATUS_KEYS) {
        if (dict[key] && dict[key].trim() === trimmed) {
          return true;
        }
      }
    }
    return false;
  }

  function showStatus(msg, isError = false, isModified = false) {
    if (isModified || isSilentStatus(msg)) {
      return;
    }

    if (isError) {
      showToast(msg, "error", 4500);
      return;
    }

    const isSuccess =
      /скопирован|успешно|удален|copied|success|deleted|erfolgreich|kopiert|succès|copié|copiado|éxito|eliminado|copiato|successo|sucesso|eliminato|excluído|kopyalandı|başarılı|silindi|已复制|成功|已删除|gekopieerd|geslaagd|verwijderd/i.test(
        msg
      );
    showToast(msg, isSuccess ? "success" : "info", 3000);
  }

  // Bind Events
  if (btnSaveSettings) {
    btnSaveSettings.addEventListener("click", saveSettings);
  }


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

    // Update data-i18n-title elements (tooltips and aria-labels)
    const titleElements = document.querySelectorAll("[data-i18n-title]");
    titleElements.forEach(el => {
      const key = el.getAttribute("data-i18n-title");
      const text = dict[key] || i18nDict.ru[key];
      if (text) {
        el.setAttribute("title", text);
        el.setAttribute("aria-label", text);
      }
    });

    const selectUiLang = document.getElementById("select-ui-lang");
    if (selectUiLang) {
      selectUiLang.setAttribute("aria-label", dict.general_ui_lang_title || "UI Language");
    }

    const btnReset = document.getElementById("btn-reset-hotkey");
    if (btnReset) {
      const resetTitle = dict.hotkey_reset_title || "Сбросить на Alt+V";
      btnReset.setAttribute("title", resetTitle);
      btnReset.setAttribute("aria-label", resetTitle);
    }

    const btnMin = document.getElementById("btn-window-minimize");
    if (btnMin) {
      const minText = dict.window_minimize || (lang === "ru" ? "Свернуть" : "Minimize");
      btnMin.setAttribute("title", minText);
      btnMin.setAttribute("aria-label", minText);
    }

    const btnClose = document.getElementById("btn-window-close");
    if (btnClose) {
      const closeText = dict.window_close || (lang === "ru" ? "Закрыть" : "Close");
      btnClose.setAttribute("title", closeText);
      btnClose.setAttribute("aria-label", closeText);
    }

    const btnClearSearch = document.getElementById("btn-clear-history-search");
    if (btnClearSearch) {
      const clearSearchText = dict.history_search_clear || "Clear search";
      btnClearSearch.setAttribute("title", clearSearchText);
      btnClearSearch.setAttribute("aria-label", clearSearchText);
    }

    const btnToggleKey = document.getElementById("btn-toggle-key");
    if (btnToggleKey) {
      const toggleKeyText = dict.api_key_toggle_visibility || "Show/hide key";
      btnToggleKey.setAttribute("aria-label", toggleKeyText);
      btnToggleKey.setAttribute("title", toggleKeyText);
    }

    const btnToggleKeyEdit = document.getElementById("btn-toggle-key-edit");
    if (btnToggleKeyEdit) {
      const toggleKeyText = dict.api_key_toggle_visibility || "Show/hide key";
      btnToggleKeyEdit.setAttribute("aria-label", toggleKeyText);
      btnToggleKeyEdit.setAttribute("title", toggleKeyText);
    }

    const historyFiltersGroup = document.querySelector(".history-filter-pills");
    if (historyFiltersGroup) {
      historyFiltersGroup.setAttribute("aria-label", dict.history_filters_label || "History filters");
    }

    const gpuModesGroup = document.querySelector(".gpu-modes-grid");
    if (gpuModesGroup) {
      gpuModesGroup.setAttribute("aria-label", dict.gpu_accel_label || "Hardware acceleration");
    }

    const llmModelsGroup = document.getElementById("llm-model-cards-container");
    if (llmModelsGroup) {
      llmModelsGroup.setAttribute("aria-label", dict.llm_model_label || "Local model");
    }

    const modelsGridGroup = document.querySelector(".models-grid");
    if (modelsGridGroup) {
      modelsGridGroup.setAttribute("aria-label", dict.model_title || "Whisper Models");
    }

    const settingsNav = document.querySelector(".settings-nav");
    if (settingsNav) {
      settingsNav.setAttribute("aria-label", dict.title_settings || "Settings");
    }

    
    // Update inputs and placeholders
    const apiInput = document.getElementById("input-api-key");
    if (apiInput) {
      apiInput.placeholder = dict.api_key_placeholder || "";
    }
    const apiInputEdit = document.getElementById("input-edit-api-key");
    if (apiInputEdit) {
      apiInputEdit.placeholder = dict.api_key_placeholder || "";
    }
    const descCloud = document.getElementById("desc-edit-cloud-provider");
    const aiEditSelect = document.getElementById("select-ai-edit-mode");
    if (descCloud && aiEditSelect) {
      descCloud.textContent = aiEditSelect.value === "auto"
        ? (dict.edit_cloud_provider_fallback_sub || getTranslation("edit_cloud_provider_fallback_sub"))
        : (dict.edit_cloud_provider_sub || getTranslation("edit_cloud_provider_sub"));
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
    const inputRepTrigger = document.getElementById("input-rep-trigger");
    if (inputRepTrigger) {
      inputRepTrigger.placeholder = dict.replacements_trigger_ph || "";
    }
    const inputRepValue = document.getElementById("input-rep-value");
    if (inputRepValue) {
      inputRepValue.placeholder = dict.replacements_value_ph || "";
    }
    renderReplacementsList(currentReplacements);
    const checkboxGamingMode = document.getElementById("checkbox-gaming-mode");
    if (typeof updateGamingModeUi === "function" && checkboxGamingMode) {
      updateGamingModeUi(checkboxGamingMode.checked);
    }
    
    // Update dynamic link text and provider key placeholders
    updateApiKeyLink();
    renderProviderKeyInput();
    
    // Refresh model cards status/actions
    refreshDownloadedModels();
    updateGpuCardStates();
    renderSupportedCommandsGuide();
    
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
          btnToggleMicMeter.textContent = getTranslation("mic_meter_test_btn_stop");
          btnToggleMicMeter.classList.add("active");
          micMeterAnimId = requestAnimationFrame(runMicMeterPhysics);
        } catch (err) {
          console.error("Failed to start mic test:", err);
          await stopMicTesting();
          showStatus(`${getTranslation("mic_start_error")}${err}`, true);
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

    const now = new Date();
    const startOfToday = new Date(now.getFullYear(), now.getMonth(), now.getDate()).getTime();
    const startOfYesterday = startOfToday - 86400000;
    const endOfToday = startOfToday + 86400000;

    function getDayGroup(timestampMs) {
      if (!timestampMs) return { key: "today", title: dict.history_group_today || "Сегодня" };
      if (timestampMs >= startOfToday && timestampMs < endOfToday) {
        return { key: "today", title: dict.history_group_today || "Сегодня" };
      }
      if (timestampMs >= startOfYesterday && timestampMs < startOfToday) {
        return { key: "yesterday", title: dict.history_group_yesterday || "Вчера" };
      }
      const d = new Date(timestampMs);
      const isCurrentYear = d.getFullYear() === now.getFullYear();
      const dateTitle = d.toLocaleDateString(currentLanguage || "ru", {
        day: "numeric",
        month: "long",
        year: isCurrentYear ? undefined : "numeric"
      });
      const dateKey = `${d.getFullYear()}-${d.getMonth()}-${d.getDate()}`;
      return { key: dateKey, title: dateTitle };
    }

    function formatHistoryDuration(ms) {
      if (!ms) return "";
      if (ms < 1000) return `${ms} ${dict.history_unit_ms || "ms"}`;
      const secs = ms >= 10000 ? Math.round(ms / 1000) : Math.round((ms / 1000) * 10) / 10;
      return `${secs} ${dict.history_unit_sec || "s"}`;
    }

    let lastDayKey = null;

    filtered.forEach(entry => {
      const dayGroup = getDayGroup(entry.timestamp_ms);
      if (dayGroup.title && dayGroup.key !== lastDayKey) {
        lastDayKey = dayGroup.key;
        const headerEl = document.createElement("div");
        headerEl.className = "history-day-header";
        headerEl.innerHTML = `
          <span class="history-day-title">${escapeHtml(dayGroup.title)}</span>
          <span class="history-day-line"></span>
        `;
        fragment.appendChild(headerEl);
      }

      const date = new Date(entry.timestamp_ms || Date.now());
      const displayTime = date.toLocaleTimeString(currentLanguage, { hour: '2-digit', minute: '2-digit', second: '2-digit' });

      const itemEl = document.createElement("div");
      itemEl.className = "history-item";

      let badgeHtml;
      const engineLabel = dict[`history_engine_${entry.engine}`];
      const durationHtml = entry.processing_ms
        ? `<span class="history-item-duration">${formatHistoryDuration(entry.processing_ms)}</span>`
        : "";
      if (engineLabel) {
        badgeHtml = `<span class="history-item-badge badge-local">${escapeHtml(engineLabel)}</span>${durationHtml}`;
      } else if (entry.mode === "cloud") {
        badgeHtml = `<span class="history-item-badge badge-cloud">${escapeHtml(dict.history_badge_cloud || "Cloud")}</span>${durationHtml}`;
      } else {
        badgeHtml = `<span class="history-item-badge badge-local">${escapeHtml(dict.history_badge_local || "Local")}</span>${durationHtml}`;
      }

      itemEl.innerHTML = `
        <div class="history-item-body">
          <div class="history-item-meta">
            <span class="history-item-time">${displayTime}</span>
            ${badgeHtml}
          </div>
          <div class="history-item-text">${escapeHtml(entry.text)}</div>
        </div>
        <button type="button" class="btn-copy-history" title="${escapeHtml(dict.history_copy_title || "Copy to clipboard")}" aria-label="${escapeHtml(dict.history_copy_title || "Copy to clipboard")}">
          <svg class="copy-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <rect x="9" y="9" width="13" height="13" rx="2" ry="2"></rect>
            <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"></path>
          </svg>
          <svg class="check-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <polyline points="20 6 9 17 4 12"></polyline>
          </svg>
        </button>
      `;

      // Bind copy event
      const btnCopy = itemEl.querySelector(".btn-copy-history");

      btnCopy.addEventListener("click", async () => {
        try {
          await invoke("copy_to_clipboard", { text: entry.text });
          
          btnCopy.classList.add("copied");
          
          if (btnCopy._copyTimeout) {
            clearTimeout(btnCopy._copyTimeout);
          }
          
          btnCopy._copyTimeout = setTimeout(() => {
            btnCopy.classList.remove("copied");
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

  const btnExitApp = document.getElementById("btn-exit-app");
  if (btnExitApp) {
    btnExitApp.addEventListener("click", async () => {
      try {
        await invoke("exit_app");
      } catch (err) {
        console.error("Failed to exit app:", err);
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

    if (settings?.has_completed_onboarding === true || localStorage.getItem("aura_onboarding_completed") === "true") {
      hasCompletedOnboarding = true;
      try {
        localStorage.setItem("aura_onboarding_completed", "true");
      } catch (_) {}
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

    setupReplacementsUI();
    initSelectPanels();

    // Spotlight Tour Controller (Docked Footer + Spotlight Cutout)
    const TOUR_STEPS = [
      {
        tab: "speech",
        targetSelector: "#card-engine",
        titleKey: "onboarding_step_1_title",
        descKey: "onboarding_step_1_desc"
      },
      {
        tab: "text",
        targetSelector: "#card-language",
        titleKey: "onboarding_step_2_title",
        descKey: "onboarding_step_2_desc"
      },
      {
        tab: "text",
        targetSelector: "#card-selection-edit",
        titleKey: "onboarding_step_3_title",
        descKey: "onboarding_step_3_desc"
      },
      {
        tab: "hotkeys",
        targetSelector: "#card-hotkey",
        titleKey: "onboarding_step_4_title",
        descKey: "onboarding_step_4_desc"
      },
      {
        tab: "history",
        targetSelector: ".card-history",
        titleKey: "onboarding_step_5_title",
        descKey: "onboarding_step_5_desc"
      }
    ];

    let currentTourStep = 0;
    let isTourActive = false;

    function initSpotlightTour() {
      const overlay = document.getElementById("spotlight-tour-overlay");
      const cutout = document.getElementById("spotlight-cutout");
      const popover = document.getElementById("spotlight-popover");
      const stepIndicator = document.getElementById("spotlight-step-indicator");
      const segmentsContainer = document.getElementById("spotlight-segments");
      const titleEl = document.getElementById("spotlight-title");
      const descEl = document.getElementById("spotlight-desc");
      const btnPrev = document.getElementById("btn-spotlight-prev");
      const btnNext = document.getElementById("btn-spotlight-next");
      const btnSkip = document.getElementById("btn-spotlight-skip");
      const btnReplay = document.getElementById("btn-replay-onboarding");
      const backdrop = document.getElementById("spotlight-backdrop");
      const mainContent = document.getElementById("settings-main-content");

      if (!overlay || !cutout || !popover || !btnPrev || !btnNext || !btnSkip) {
        return {
          startTour: () => {},
          checkAutoStart: () => {}
        };
      }

      function renderSegments() {
        if (!segmentsContainer) return;
        segmentsContainer.innerHTML = "";
        TOUR_STEPS.forEach((_, idx) => {
          const seg = document.createElement("span");
          seg.className = "spotlight-seg" + (idx <= currentTourStep ? " filled" : "");
          segmentsContainer.appendChild(seg);
        });
      }

      function updateTourPosition() {
        if (!isTourActive) return;
        const step = TOUR_STEPS[currentTourStep];
        let target = document.querySelector(step.targetSelector);

        // If the target element is hidden (e.g. #card-language is display:none in cloud recognition mode),
        // fallback to the first visible card in the same tab (e.g. #card-replacements)
        if (target && target.offsetParent === null) {
          target = document.querySelector(`#panel-${step.tab} .card:not([style*="display: none"])`);
        }

        if (!target || target.offsetParent === null) {
          cutout.style.opacity = "0";
          return;
        }

        target.scrollIntoView({ block: "center", inline: "nearest", behavior: "smooth" });

        const rect = target.getBoundingClientRect();
        if (rect.width === 0 || rect.height === 0) {
          cutout.style.opacity = "0";
          return;
        }

        cutout.style.opacity = "1";
        const padding = 6;
        const cutoutTop = Math.max(0, rect.top - padding);
        const cutoutLeft = Math.max(0, rect.left - padding);
        const cutoutWidth = rect.width + padding * 2;
        const cutoutHeight = rect.height + padding * 2;

        cutout.style.top = `${Math.round(cutoutTop)}px`;
        cutout.style.left = `${Math.round(cutoutLeft)}px`;
        cutout.style.width = `${Math.round(cutoutWidth)}px`;
        cutout.style.height = `${Math.round(cutoutHeight)}px`;
      }

      function showTourStep(index) {
        currentTourStep = index;
        const step = TOUR_STEPS[currentTourStep];

        const tabBtn = document.getElementById(`tab-btn-${step.tab}`);
        if (tabBtn) {
          activateTab(tabBtn);
        }

        const total = TOUR_STEPS.length;
        stepIndicator.textContent = getTranslation("onboarding_step_tag", {
          current: String(currentTourStep + 1),
          total: String(total)
        });
        titleEl.textContent = getTranslation(step.titleKey);
        descEl.textContent = getTranslation(step.descKey);

        btnPrev.style.display = currentTourStep === 0 ? "none" : "inline-flex";
        btnNext.textContent = currentTourStep === total - 1
          ? getTranslation("onboarding_finish")
          : getTranslation("onboarding_next");

        renderSegments();

        updateTourPosition();
        setTimeout(updateTourPosition, 80);
      }

      function startTour() {
        isTourActive = true;
        overlay.hidden = false;
        if (mainContent) mainContent.classList.add("tour-active-padding");
        currentTourStep = 0;
        showTourStep(0);
      }

      function finishTour() {
        isTourActive = false;
        overlay.hidden = true;
        if (mainContent) mainContent.classList.remove("tour-active-padding");
        hasCompletedOnboarding = true;
        try {
          localStorage.setItem("aura_onboarding_completed", "true");
        } catch (_) {}
        invoke("complete_onboarding").catch(err => {
          console.error("Failed to mark onboarding as completed", err);
        });
      }

      btnNext.addEventListener("click", () => {
        if (currentTourStep < TOUR_STEPS.length - 1) {
          showTourStep(currentTourStep + 1);
        } else {
          finishTour();
        }
      });

      btnPrev.addEventListener("click", () => {
        if (currentTourStep > 0) {
          showTourStep(currentTourStep - 1);
        }
      });

      btnSkip.addEventListener("click", () => {
        finishTour();
      });

      if (backdrop) {
        backdrop.addEventListener("click", () => {
          finishTour();
        });
      }

      if (btnReplay) {
        btnReplay.addEventListener("click", () => {
          startTour();
        });
      }

      if (mainContent) {
        mainContent.addEventListener("scroll", () => {
          if (isTourActive) {
            const step = TOUR_STEPS[currentTourStep];
            const target = document.querySelector(step.targetSelector);
            if (target) {
              const rect = target.getBoundingClientRect();
              const padding = 6;
              cutout.style.top = `${Math.round(Math.max(0, rect.top - padding))}px`;
              cutout.style.left = `${Math.round(Math.max(0, rect.left - padding))}px`;
              cutout.style.width = `${Math.round(rect.width + padding * 2)}px`;
              cutout.style.height = `${Math.round(rect.height + padding * 2)}px`;
            }
          }
        }, { passive: true });
      }

      window.addEventListener("keydown", (e) => {
        if (!isTourActive) return;
        if (e.key === "Escape") {
          e.preventDefault();
          finishTour();
        } else if (e.key === "ArrowRight") {
          e.preventDefault();
          if (currentTourStep < TOUR_STEPS.length - 1) showTourStep(currentTourStep + 1);
        } else if (e.key === "ArrowLeft") {
          e.preventDefault();
          if (currentTourStep > 0) showTourStep(currentTourStep - 1);
        }
      });

      window.addEventListener("resize", () => {
        if (isTourActive) updateTourPosition();
      });

      return {
        startTour,
        finishTour,
        checkAutoStart: (settingsSnapshot) => {
          const isCompleted = settingsSnapshot?.has_completed_onboarding === true
            || hasCompletedOnboarding
            || localStorage.getItem("aura_onboarding_completed") === "true";
          if (!isCompleted) {
            startTour();
          }
        }
      };
    }

    // Initialize Settings
    await loadSettings(settings);

    const spotlightTour = initSpotlightTour();
    spotlightTour.checkAutoStart(settings);

    if (settings?.automatic_update_checks) {
      await checkForUpdates(false);
    }

  })();
});
