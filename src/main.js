// Retrieve Tauri APIs from window.__TAURI__
const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;

const i18nDict = {
  ru: {
    title_settings: "Настройки",
    tab_general: "Основные",
    tab_speech: "Голос",
    tab_hotkeys: "Управление",
    tab_apikeys: "API-ключи",
    tab_history: "История",
    tab_about: "О программе",
    general_autostart_title: "Автозапуск Aura",
    general_autostart_desc: "Запускать приложение автоматически при входе в операционную систему Windows.",
    general_autostart_checkbox: "Запускать Aura при старте системы",
    engine_title: "Способ распознавания",
    engine_desc: "Выберите между облачной обработкой высокого качества или полностью автономным локальным распознаванием речи.",
    engine_cloud: "Облачный ИИ",
    engine_cloud_meta: "Gemini / OpenAI / Groq (требуется API-ключ)",
    engine_local: "Локальный ИИ",
    engine_local_meta: "Whisper / Parakeet (100% оффлайн)",
    lang_bias_title: "Язык распознавания",
    lang_bias_desc: "Выберите принудительный язык ввода или включите автоопределение.",
    lang_bias_label: "Выберите язык",
    lang_opt_auto: "Автоопределение (по умолчанию)",
    lang_opt_layout: "По раскладке клавиатуры",
    streaming_title: "Режим ввода текста",
    streaming_desc: "Выберите способ отображения наговариваемого текста.",
    streaming_checkbox: "Потоковый ввод в реальном времени (экспериментальный)",
    streaming_subdesc: "Если выключено: текст вставится целиком только после того, как вы отпустите клавиши.",
    punct_title: "Интеллектуальная пунктуация",
    punct_desc: "Преобразовывать голосовые команды (\"запятая\", \"точка с запятой\") в знаки препинания.",
    punct_checkbox: "Включить обработку голосовой пунктуации",
    vocab_title: "Пользовательский словарь",
    vocab_desc: "Внесите термины, имена или брендовые названия через запятую, чтобы улучшить их распознавание.",
    vocab_placeholder: "Например: Аура, коммит, репозиторий...",
    local_model_title: "Локальное распознавание",
    local_model_desc: "Настройте локальный движок распознавания речи для полной приватности.",
    local_model_label: "Размер модели",
    model_meta_tiny: "~75 МБ — сверхбыстрая",
    model_meta_base: "~145 МБ — рекомендуемая",
    model_meta_small: "~465 МБ — точная для русского",
    model_meta_medium: "~1.5 ГБ — продвинутая",
    model_meta_turbo: "~1.6 ГБ — лучшая точность для RU/EN",
    model_meta_turbo_q5: "~550 МБ — почти как Turbo, вдвое легче",
    model_cancel_download: "Отменить загрузку",
    model_download_cancelled: "Загрузка отменена",
    update_available: "Доступно обновление",
    hotkey_title: "Глобальная горячая клавиша",
    hotkey_desc: "Зажмите выбранную комбинацию для начала записи, отпустите для распознавания.",
    hotkey_label: "Комбинация",
    hotkey_toggle_mode: "Режим переключателя (короткое нажатие)",
    hotkey_toggle_mode_desc: "Короткое нажатие начинает запись без удержания клавиши. Повторный клик останавливает запись.",
    sound_title: "Звуковое сопровождение",
    sound_desc: "Звуковые эффекты оверлея при записи.",
    sound_enable: "Включить звуки оверлея",
    sound_volume_label: "Громкость звука",
    sound_theme_label: "Звуковая тема",
    sound_theme_zen: "Дзен (Поющие чаши)",
    sound_theme_rhodes: "Rhodes (Джаз-электропианино)",
    sound_theme_scifi: "Sci-Fi (Космический)",
    sound_theme_classic: "Колокольчик (Классический)",
    api_title: "Авторизация API-ключей",
    api_desc: "Укажите ваши API-ключи для авторизации в облачных сервисах Gemini, OpenAI или Groq.",
    api_provider: "Провайдер API",
    api_key: "API-ключ",
    api_key_placeholder: "Введите ваш API-ключ...",
    api_get_key: "Получить ключ API",
    history_title: "История транскрипций",
    history_clear: "Очистить историю",
    history_desc: "Последние надиктованные фразы хранятся локально.",
    history_empty: "История пуста. Ваши надиктованные тексты будут отображаться здесь.",
    about_app_title: "Голосовой ввод Aura",
    about_version: "v1.0.5",
    about_description: "Инструмент глобального голосового ввода для Windows. Программа переводит речь в текст и вставляет его в любое активное окно с автоматическим форматированием и расстановкой пунктуации.",
    status_ready: "Готово",
    btn_save: "Сохранить настройки",
    confirm_title: "Подтверждение",
    confirm_message: "Вы действительно хотите выполнить это действие?",
    confirm_cancel: "Отмена",
    confirm_ok: "Подтвердить",
    status_loading: "Загрузка настроек...",
    status_modified: "Настройки изменены (не сохранены)",
    status_saving: "Сохранение настроек...",
    status_saved: "Настройки успешно сохранены!",
    status_error: "Ошибка: ",
    model_status_ready: "Установлено",
    model_action_download: "Скачать",
    model_action_delete: "Удалить",
    api_get_key_pattern: "Получить ключ на {name}",
    status_loaded: "Настройки загружены",
    status_load_error: "Ошибка загрузки настроек: ",
    status_save_error: "Ошибка сохранения настроек: ",
    model_downloading_pattern: "Запуск скачивания для модели '{model}'...",
    model_download_error_pattern: "Ошибка скачивания: {err}",
    delete_model_title: "Удаление модели",
    delete_model_confirm_pattern: "Вы действительно хотите удалить локальную модель '{model}'?",
    delete_model_btn: "Удалить",
    model_deleting_pattern: "Удаление модели '{model}'...",
    model_deleted_success: "Модель успешно удалена",
    model_delete_error_pattern: "Ошибка удаления: {err}",
    model_downloaded_success_pattern: "Модель '{model}' скачана!",
    confirm_clear_history_title: "Очистить историю",
    confirm_clear_history_msg: "Вы действительно хотите очистить всю историю транскрипций?",
    general_ui_lang_title: "Язык интерфейса",
    general_ui_lang_desc: "Выберите язык для отображения настроек и уведомлений приложения.",
    hotkey_reset_title: "Сбросить на Alt+V",
    local_engine_label: "Движок распознавания",
    local_engine_whisper: "Whisper.cpp (на базе OpenAI Whisper)",
    local_engine_parakeet: "NVIDIA Parakeet (через sherpa-onnx)",
    parakeet_model_label: "Модель Parakeet",
    model_meta_parakeet: "~670 МБ — оптимизировано NVIDIA"
  },
  en: {
    title_settings: "Settings",
    tab_general: "General",
    tab_speech: "Speech",
    tab_hotkeys: "Hotkeys",
    tab_apikeys: "API Keys",
    tab_history: "History",
    tab_about: "About",
    general_autostart_title: "Aura Autostart",
    general_autostart_desc: "Launch the app automatically when starting Windows.",
    general_autostart_checkbox: "Start Aura at system boot",
    engine_title: "Processing Type",
    engine_desc: "Choose between high-quality cloud transcription or fully local speech recognition.",
    engine_cloud: "Cloud AI",
    engine_cloud_meta: "Gemini / OpenAI / Groq (API key required)",
    engine_local: "Local AI",
    engine_local_meta: "Whisper / Parakeet (100% offline & private)",
    lang_bias_title: "Speech Language",
    lang_bias_desc: "Forcibly set transcription language or use automatic detection.",
    lang_bias_label: "Select Language",
    lang_opt_auto: "Auto-detect (default)",
    lang_opt_layout: "Follow Keyboard Layout",
    streaming_title: "Text Streaming",
    streaming_desc: "Choose how transcribed text is displayed.",
    streaming_checkbox: "Real-time streaming typing (experimental)",
    streaming_subdesc: "If disabled: text is typed as a whole only when you release hotkeys.",
    punct_title: "Smart Punctuation",
    punct_desc: "Convert spoken punctuation commands (like \"comma\", \"period\") into punctuation.",
    punct_checkbox: "Enable spoken punctuation processing",
    vocab_title: "Custom Vocabulary",
    vocab_desc: "Add specific terms, names, or jargon separated by commas to improve recognition.",
    vocab_placeholder: "e.g. Aura, commit, repository...",
    local_model_title: "Local Recognition",
    local_model_desc: "Configure a local speech-to-text engine for absolute privacy.",
    local_model_label: "Model Size",
    model_meta_tiny: "~75 MB — superfast",
    model_meta_base: "~145 MB — recommended",
    model_meta_small: "~465 MB — accurate",
    model_meta_medium: "~1.5 GB — advanced",
    model_meta_turbo: "~1.6 GB — best accuracy for RU/EN",
    model_meta_turbo_q5: "~550 MB — near-Turbo, half the size",
    model_cancel_download: "Cancel download",
    model_download_cancelled: "Download cancelled",
    update_available: "Update available",
    hotkey_title: "Global Hotkey",
    hotkey_desc: "Hold down the selected hotkey to record, release to transcribe.",
    hotkey_label: "Combination",
    hotkey_toggle_mode: "Toggle mode (short tap)",
    hotkey_toggle_mode_desc: "Short tap starts/stops recording without holding key down.",
    sound_title: "Overlay Audio Feedback",
    sound_desc: "Audio sound effects when recording states change.",
    sound_enable: "Enable overlay sounds",
    sound_volume_label: "Sound Volume",
    sound_theme_label: "Sound Theme",
    sound_theme_zen: "Zen (Singing Bowls)",
    sound_theme_rhodes: "Rhodes (Jazz Electric Piano)",
    sound_theme_scifi: "Sci-Fi (Space/Synth)",
    sound_theme_classic: "Bell (Classic)",
    api_title: "API Keys Authorization",
    api_desc: "Provide API keys for Gemini, OpenAI, or Groq cloud services.",
    api_provider: "API Provider",
    api_key: "API Key",
    api_key_placeholder: "Enter your API key...",
    api_get_key: "Get API Key",
    history_title: "Transcription History",
    history_clear: "Clear History",
    history_desc: "Your latest transcribed phrases are cached locally.",
    history_empty: "History is empty. Dictated text fragments will appear here.",
    about_app_title: "Aura Voice Input",
    about_version: "v1.0.5",
    about_description: "Global voice input tool for Windows. The program transcribes speech to text and inserts it into any active window with automatic formatting and punctuation.",
    status_ready: "Ready",
    btn_save: "Save Settings",
    confirm_title: "Confirmation",
    confirm_message: "Are you sure you want to perform this action?",
    confirm_cancel: "Cancel",
    confirm_ok: "Confirm",
    status_loading: "Loading settings...",
    status_modified: "Settings changed (unsaved)",
    status_saving: "Saving settings...",
    status_saved: "Settings saved successfully!",
    status_error: "Error: ",
    model_status_ready: "Installed",
    model_action_download: "Download",
    model_action_delete: "Delete",
    api_get_key_pattern: "Get key on {name}",
    status_loaded: "Settings loaded",
    status_load_error: "Failed to load settings: ",
    status_save_error: "Failed to save settings: ",
    model_downloading_pattern: "Starting download for model '{model}'...",
    model_download_error_pattern: "Download error: {err}",
    delete_model_title: "Delete model",
    delete_model_confirm_pattern: "Are you sure you want to delete the local model '{model}'?",
    delete_model_btn: "Delete",
    model_deleting_pattern: "Deleting model '{model}'...",
    model_deleted_success: "Model deleted successfully",
    model_delete_error_pattern: "Delete error: {err}",
    model_downloaded_success_pattern: "Model '{model}' downloaded!",
    confirm_clear_history_title: "Clear history",
    confirm_clear_history_msg: "Are you sure you want to clear all transcription history?",
    general_ui_lang_title: "Interface Language",
    general_ui_lang_desc: "Select the language for settings and application notifications.",
    hotkey_reset_title: "Reset to Alt+V",
    local_engine_label: "ASR Engine",
    local_engine_whisper: "Whisper.cpp (OpenAI Whisper)",
    local_engine_parakeet: "NVIDIA Parakeet (via sherpa-onnx)",
    parakeet_model_label: "Parakeet Model",
    model_meta_parakeet: "~670 MB — optimized by NVIDIA"
  },
  de: {
    title_settings: "Einstellungen",
    tab_general: "Allgemein",
    tab_speech: "Diktat",
    tab_hotkeys: "Tastenkombinationen",
    tab_apikeys: "API-Schlüssel",
    tab_history: "Verlauf",
    tab_about: "Über Aura",
    general_autostart_title: "Aura Autostart",
    general_autostart_desc: "Startet die App automatisch beim Anmelden in Windows.",
    general_autostart_checkbox: "Aura beim Systemstart starten",
    engine_title: "Verarbeitungstyp",
    engine_desc: "Wählen Sie zwischen Cloud-Transkription oder vollständig lokaler Spracherkennung.",
    engine_cloud: "Cloud-KI",
    engine_cloud_meta: "Gemini / OpenAI / Groq (API-Schlüssel erforderlich)",
    engine_local: "Lokale KI",
    engine_local_meta: "Whisper.cpp (100% offline & privat)",
    lang_bias_title: "Sprache",
    lang_bias_desc: "Wählen Sie eine feste Sprache für das Diktat oder aktivieren Sie die Auto-Erkennung.",
    lang_bias_label: "Sprache auswählen",
    lang_opt_auto: "Auto-Erkennung (Standard)",
    lang_opt_layout: "Tastaturlayout folgen",
    streaming_title: "Text-Streaming",
    streaming_desc: "Wählen Sie, wie die Transkription eingegeben wird.",
    streaming_checkbox: "Echtzeit-Streaming-Eingabe (experimentell)",
    streaming_subdesc: "Wenn deaktiviert: Text wird als Ganzes eingefügt, wenn die Taste losgelassen wird.",
    punct_title: "Intelligente Interpunktion",
    punct_desc: "Gesprochene Satzzeichen (z. B. \"Komma\", \"Punkt\") in Interpunktion umwandeln.",
    punct_checkbox: "Verarbeitung gesprochener Satzzeichen aktivieren",
    vocab_title: "Eigenes Wörterbuch",
    vocab_desc: "Tragen Sie Begriffe, Namen oder Fachbegriffe durch Komma getrennt ein, um die Erkennung zu verbessern.",
    vocab_placeholder: "z.B. Aura, Commit, Repository...",
    local_model_title: "Lokales Whisper-Modell",
    local_model_desc: "Laden Sie ein Modell für den Offline-Betrieb herunter. Größere Modelle sind genauer, benötigen jedoch mehr Speicher.",
    local_model_label: "Modellgröße",
    model_meta_tiny: "~75 MB — superschnell",
    model_meta_base: "~145 MB — empfohlen",
    model_meta_small: "~465 MB — präzise",
    model_meta_medium: "~1.5 GB — fortgeschritten",
    model_meta_turbo: "~1.6 GB — beste Genauigkeit für RU/EN",
    model_meta_turbo_q5: "~550 MB — fast wie Turbo, halb so groß",
    hotkey_title: "Globale Taste",
    hotkey_desc: "Tastenkombination gedrückt halten, um aufzunehmen, loslassen zur Transkription.",
    hotkey_label: "Kombination",
    hotkey_toggle_mode: "Umschaltmodus (kurzes Tippen)",
    hotkey_toggle_mode_desc: "Kurzes Antippen startet/stoppt Aufnahme ohne Halten.",
    sound_title: "Audio-Rückmeldung",
    sound_desc: "Soundeffekte des Overlays während der Aufnahme.",
    sound_enable: "Overlay-Sounds aktivieren",
    sound_volume_label: "Tonlautstärke",
    sound_theme_label: "Sound-Theme",
    sound_theme_zen: "Zen (Klangschalen)",
    sound_theme_rhodes: "Rhodes (Jazz Electric Piano)",
    sound_theme_scifi: "Sci-Fi (Weltraum)",
    sound_theme_classic: "Glocke (Klassisch)",
    api_title: "API-Schlüssel Autorisierung",
    api_desc: "Geben Sie Ihre API-Schlüssel für Gemini, OpenAI oder Groq Cloud-Dienste ein.",
    api_provider: "API-Provider",
    api_key: "API-Schlüssel",
    api_key_placeholder: "Geben Sie Ihren API-Schlüssel ein...",
    api_get_key: "API-Schlüssel erhalten",
    history_title: "Diktatverlauf",
    history_clear: "Verlauf löschen",
    history_desc: "Die letzten aufgezeichneten Sätze werden lokal gespeichert.",
    history_empty: "Der Verlauf ist leer. Transkribierte Texte werden hier angezeigt.",
    about_app_title: "Aura Spracheingabe",
    about_version: "v1.0.5",
    about_description: "Globales Spracheingabe-Tool für Windows. Die Anwendung überträgt Sprache in Text und fügt ihn mit automatischer Formatierung und Zeichensetzung in jedes aktive Fenster ein.",
    status_ready: "Bereit",
    btn_save: "Einstellungen speichern",
    confirm_title: "Bestätigung",
    confirm_message: "Sind Sie sicher, dass Sie diese Aktion ausführen möchten?",
    confirm_cancel: "Abbrechen",
    confirm_ok: "Bestätigen",
    status_loading: "Einstellungen werden geladen...",
    status_modified: "Einstellungen geändert (ungespeichert)",
    status_saving: "Einstellungen werden gespeichert...",
    status_saved: "Einstellungen erfolgreich gespeichert!",
    status_error: "Fehler: ",
    model_status_ready: "Installiert",
    model_action_download: "Herunterladen",
    model_action_delete: "Löschen",
    api_get_key_pattern: "Schlüssel erhalten auf {name}",
    status_loaded: "Einstellungen geladen",
    status_load_error: "Fehler beim Laden der Einstellungen: ",
    status_save_error: "Fehler beim Speichern der Einstellungen: ",
    model_downloading_pattern: "Download für Modell '{model}' wird gestartet...",
    model_download_error_pattern: "Download-Fehler: {err}",
    delete_model_title: "Modell löschen",
    delete_model_confirm_pattern: "Möchten Sie das lokale Modell '{model}' wirklich löschen?",
    delete_model_btn: "Löschen",
    model_deleting_pattern: "Modell '{model}' wird gelöscht...",
    model_deleted_success: "Modell erfolgreich gelöscht",
    model_delete_error_pattern: "Fehler beim Löschen: {err}",
    model_downloaded_success_pattern: "Modell '{model}' heruntergeladen!",
    confirm_clear_history_title: "Verlauf löschen",
    confirm_clear_history_msg: "Möchten Sie den gesamten Transkriptionsverlauf wirklich löschen?",
    general_ui_lang_title: "Sprache der Benutzeroberfläche",
    general_ui_lang_desc: "Wählen Sie die Sprache für Einstellungen und Benachrichtigungen.",
    hotkey_reset_title: "Auf Alt+V zurücksetzen"
  },
  es: {
    title_settings: "Ajustes",
    tab_general: "General",
    tab_speech: "Voz",
    tab_hotkeys: "Accesos rápidos",
    tab_apikeys: "Claves API",
    tab_history: "Historial",
    tab_about: "Acerca de",
    general_autostart_title: "Inicio automático",
    general_autostart_desc: "Iniciar la aplicación de forma automática al arrancar Windows.",
    general_autostart_checkbox: "Iniciar Aura con el sistema",
    engine_title: "Tipo de procesamiento",
    engine_desc: "Seleccione entre el procesamiento en la nube de alta calidad o el reconocimiento local totalmente autónomo.",
    engine_cloud: "IA en la nube",
    engine_cloud_meta: "Gemini / OpenAI / Groq (requiere clave API)",
    engine_local: "IA local",
    engine_local_meta: "Whisper.cpp (100% offline y privado)",
    lang_bias_title: "Idioma de dictado",
    lang_bias_desc: "Forzar un idioma específico para la transcripción o usar detección automática.",
    lang_bias_label: "Seleccionar idioma",
    lang_opt_auto: "Autodetectar (por defecto)",
    lang_opt_layout: "Según teclado activo",
    streaming_title: "Escritura fluida",
    streaming_desc: "Seleccione el método para mostrar el texto transcrito.",
    streaming_checkbox: "Escritura en tiempo real (experimental)",
    streaming_subdesc: "Si está desactivado: el texto se inserta completo tras soltar el atajo.",
    punct_title: "Puntuación inteligente",
    punct_desc: "Convertir comandos de voz (ej. \"coma\", \"punto\") en signos gráficos.",
    punct_checkbox: "Activar procesamiento de puntuación por voz",
    vocab_title: "Vocabulario personalizado",
    vocab_desc: "Añada términos específicos, nombres o siglas separados por comas para mejorar el dictado.",
    vocab_placeholder: "ej. Aura, commit, repositorio...",
    local_model_title: "Modelo Whisper local",
    local_model_desc: "Descargue un modelo para usar sin conexión. Los modelos grandes son más precisos pero usan más memoria.",
    local_model_label: "Tamaño del modelo",
    model_meta_tiny: "~75 MB — superrápido",
    model_meta_base: "~145 MB — recomendado",
    model_meta_small: "~465 MB — preciso",
    model_meta_medium: "~1.5 GB — avanzado",
    model_meta_turbo: "~1.6 GB — mejor precisión para RU/EN",
    model_meta_turbo_q5: "~550 MB — casi como Turbo, mitad de tamaño",
    hotkey_title: "Acceso rápido global",
    hotkey_desc: "Mantenga presionadas las teclas seleccionadas para grabar, suéltelas para transcribir.",
    hotkey_label: "Combinación",
    hotkey_toggle_mode: "Modo alternar (pulsación corta)",
    hotkey_toggle_mode_desc: "Una pulsación corta inicia/detiene la grabación sin mantener la tecla.",
    sound_title: "Efectos de audio",
    sound_desc: "Efectos sonoros del overlay al grabar.",
    sound_enable: "Activar sonidos del overlay",
    sound_volume_label: "Volumen del sonido",
    sound_theme_label: "Tema sonoro",
    sound_theme_zen: "Zen (Cuencos tibetanos)",
    sound_theme_rhodes: "Rhodes (Piano eléctrico)",
    sound_theme_scifi: "Sci-Fi (Futurista)",
    sound_theme_classic: "Campana (Clásico)",
    api_title: "Autorización de claves API",
    api_desc: "Introduzca sus claves API para los servicios en la nube de Gemini, OpenAI o Groq.",
    api_provider: "Proveedor de API",
    api_key: "Clave API",
    api_key_placeholder: "Introduzca su clave API...",
    api_get_key: "Obtener clave API",
    history_title: "Historial de transcripción",
    history_clear: "Limpiar historial",
    history_desc: "Las últimas frases dictadas se guardan de forma local.",
    history_empty: "El historial está vacío. Los textos dictados se mostrarán aquí.",
    about_app_title: "Dictado por voz Aura",
    about_version: "v1.0.5",
    about_description: "Herramienta de entrada de voz global para Windows. El programa transcribe el habla en texto y lo inserta en cualquier ventana activa con formato y puntuación automáticos.",
    status_ready: "Listo",
    btn_save: "Guardar ajustes",
    confirm_title: "Confirmación",
    confirm_message: "¿Está seguro de realizar esta acción?",
    confirm_cancel: "Cancelar",
    confirm_ok: "Confirmar",
    status_loading: "Cargando ajustes...",
    status_modified: "Ajustes modificados (sin guardar)",
    status_saving: "Guardando ajustes...",
    status_saved: "¡Ajustes guardados correctamente!",
    status_error: "Error: ",
    model_status_ready: "Instalado",
    model_action_download: "Descargar",
    model_action_delete: "Eliminar",
    api_get_key_pattern: "Obtener clave en {name}",
    status_loaded: "Ajustes cargados",
    status_load_error: "Error al cargar los ajustes: ",
    status_save_error: "Error al guardar los ajustes: ",
    model_downloading_pattern: "Iniciando descarga para el modelo '{model}'...",
    model_download_error_pattern: "Error de descarga: {err}",
    delete_model_title: "Eliminar modelo",
    delete_model_confirm_pattern: "¿Está seguro de que desea eliminar el modelo local '{model}'?",
    delete_model_btn: "Eliminar",
    model_deleting_pattern: "Eliminando modelo '{model}'...",
    model_deleted_success: "Modelo eliminado correctamente",
    model_delete_error_pattern: "Error al eliminar: {err}",
    model_downloaded_success_pattern: "¡Modelo '{model}' descargado!",
    confirm_clear_history_title: "Limpiar historial",
    confirm_clear_history_msg: "¿Está seguro de que desea limpiar todo el historial de transcripciones?",
    general_ui_lang_title: "Idioma de la interfaz",
    general_ui_lang_desc: "Seleccione el idioma para los ajustes y las notificaciones.",
    hotkey_reset_title: "Restablecer a Alt+V"
  },
  fr: {
    title_settings: "Paramètres",
    tab_general: "Général",
    tab_speech: "Dictée",
    tab_hotkeys: "Raccourcis",
    tab_apikeys: "Clés API",
    tab_history: "Historique",
    tab_about: "À propos",
    general_autostart_title: "Lancement automatique",
    general_autostart_desc: "Lancer l'application automatiquement au démarrage de Windows.",
    general_autostart_checkbox: "Démarrer Aura avec Windows",
    engine_title: "Type de traitement",
    engine_desc: "Choisissez entre un traitement cloud de haute qualité ou une reconnaissance locale 100% hors ligne.",
    engine_cloud: "IA Cloud",
    engine_cloud_meta: "Gemini / OpenAI / Groq (clé API requise)",
    engine_local: "IA Locale",
    engine_local_meta: "Whisper.cpp (100% hors ligne et privé)",
    lang_bias_title: "Langue de dictée",
    lang_bias_desc: "Forcer une langue spécifique pour la dictée ou utiliser la détection automatique.",
    lang_bias_label: "Sélectionner la langue",
    lang_opt_auto: "Détection automatique",
    lang_opt_layout: "Selon le clavier actif",
    streaming_title: "Saisie en continu",
    streaming_desc: "Sélectionnez le mode d'affichage du texte transcrit.",
    streaming_checkbox: "Affichage du texte en temps réel (expérimental)",
    streaming_subdesc: "Si désactivé: le texte est inséré en une fois lorsque vous relâchez le raccourci.",
    punct_title: "Ponctuation intelligente",
    punct_desc: "Convertir les commandes vocales (ex. \"virgule\", \"point\") en signes de ponctuation.",
    punct_checkbox: "Activer le traitement de la ponctuation dictée",
    vocab_title: "Vocabulaire personnalisé",
    vocab_desc: "Ajoutez des termes spécifiques, noms propres ou sigles séparés par des virgules pour améliorer la dictée.",
    vocab_placeholder: "ex. Aura, commit, dépôt...",
    local_model_title: "Modèle Whisper local",
    local_model_desc: "Téléchargez un modèle pour une utilisation hors ligne. Les modèles volumineux sont plus précis mais consomment plus de mémoire.",
    local_model_label: "Taille du modèle",
    model_meta_tiny: "~75 Mo — super rapide",
    model_meta_base: "~145 Mo — recommandé",
    model_meta_small: "~465 Mo — précis",
    model_meta_medium: "~1.5 Go — avancé",
    model_meta_turbo: "~1.6 Go — meilleure précision RU/EN",
    model_meta_turbo_q5: "~550 Mo — proche de Turbo, deux fois plus léger",
    hotkey_title: "Raccourci global",
    hotkey_desc: "Maintenez le raccourci pour enregistrer, relâchez pour transcrire.",
    hotkey_label: "Combinaison",
    hotkey_toggle_mode: "Mode alterné (appui court)",
    hotkey_toggle_mode_desc: "Un appui court démarre/arrête l'enregistrement sans maintenir la touche.",
    sound_title: "Retours audio",
    sound_desc: "Effets sonores de l'overlay lors de l'enregistrement.",
    sound_enable: "Activer les sons de l'overlay",
    sound_volume_label: "Volume du son",
    sound_theme_label: "Thème sonore",
    sound_theme_zen: "Zen (Bols chantants)",
    sound_theme_rhodes: "Rhodes (Piano électrique)",
    sound_theme_scifi: "Sci-Fi (Spatiale)",
    sound_theme_classic: "Cloche (Classique)",
    api_title: "Clés d'API",
    api_desc: "Saisissez vos clés d'API pour les services Gemini, OpenAI ou Groq.",
    api_provider: "Fournisseur d'API",
    api_key: "Clé d'API",
    api_key_placeholder: "Saisissez votre clé d'API...",
    api_get_key: "Obtenir une clé d'API",
    history_title: "Historique de dictée",
    history_clear: "Effacer l'historique",
    history_desc: "Les dernières phrases dictées sont enregistrées localement.",
    history_empty: "Historique vide. Vos textes transcrits s'afficheront ici.",
    about_app_title: "Dictée vocale Aura",
    about_version: "v1.0.5",
    about_description: "Outil de saisie vocale globale pour Windows. Le programme transcrit la parole en texte et l'insère dans n'importe quelle fenêtre active avec un formatage et une ponctuation automatiques.",
    status_ready: "Prêt",
    btn_save: "Enregistrer",
    confirm_title: "Confirmation",
    confirm_message: "Voulez-vous vraiment effectuer cette action?",
    confirm_cancel: "Annuler",
    confirm_ok: "Confirmer",
    status_loading: "Chargement...",
    status_modified: "Modifications non enregistrées",
    status_saving: "Enregistrement...",
    status_saved: "Paramètres enregistrés !",
    status_error: "Erreur: ",
    model_status_ready: "Installé",
    model_action_download: "Télécharger",
    model_action_delete: "Supprimer",
    api_get_key_pattern: "Obtenir la clé sur {name}",
    status_loaded: "Paramètres chargés",
    status_load_error: "Échec du chargement des paramètres : ",
    status_save_error: "Échec de l'enregistrement des paramètres : ",
    model_downloading_pattern: "Démarrage du téléchargement pour le modèle '{model}'...",
    model_download_error_pattern: "Erreur de téléchargement: {err}",
    delete_model_title: "Supprimer le modèle",
    delete_model_confirm_pattern: "Voulez-vous vraiment supprimer le modèle local '{model}' ?",
    delete_model_btn: "Supprimer",
    model_deleting_pattern: "Suppression du modèle '{model}'...",
    model_deleted_success: "Modèle supprimé avec succès",
    model_delete_error_pattern: "Erreur de suppression: {err}",
    model_downloaded_success_pattern: "Modèle '{model}' téléchargé !",
    confirm_clear_history_title: "Effacer l'historique",
    confirm_clear_history_msg: "Voulez-vous vraiment effacer tout l'historique des transcriptions ?",
    general_ui_lang_title: "Langue de l'interface",
    general_ui_lang_desc: "Sélectionnez la langue pour les paramètres et les notifications de l'application.",
    hotkey_reset_title: "Réinitialiser à Alt+V"
  },
  it: {
    title_settings: "Impostazioni",
    tab_general: "Generale",
    tab_speech: "Dettatura",
    tab_hotkeys: "Scorciatoie",
    tab_apikeys: "Chiavi API",
    tab_history: "Cronologia",
    tab_about: "Informazioni",
    general_autostart_title: "Avvio automatico",
    general_autostart_desc: "Avvia l'app automaticamente all'accesso di Windows.",
    general_autostart_checkbox: "Avvia Aura con il sistema",
    engine_title: "Tipo di elaborazione",
    engine_desc: "Scegli tra l'elaborazione cloud di alta qualità o il riconoscimento locale offline.",
    engine_cloud: "IA Cloud",
    engine_cloud_meta: "Gemini / OpenAI / Groq (chiave API richiesta)",
    engine_local: "IA Locale",
    engine_local_meta: "Whisper.cpp (100% offline e privato)",
    lang_bias_title: "Lingua dettatura",
    lang_bias_desc: "Imposta una lingua fissa per la transrizione o usa il rilevamento automatico.",
    lang_bias_label: "Seleziona lingua",
    lang_opt_auto: "Rilevamento automatico",
    lang_opt_layout: "In base alla tastiera",
    streaming_title: "Dattilografia a scorrimento",
    streaming_desc: "Seleziona come visualizzare il testo trascritto.",
    streaming_checkbox: "Inserimento del testo in tempo reale (sperimentale)",
    streaming_subdesc: "Se disattivato: il testo viene inserito interamente solo quando rilasci la scorciatoia.",
    punct_title: "Punteggiatura intelligente",
    punct_desc: "Converte i comandi vocali (es. \"virgola\", \"punto\") in simboli grafici.",
    punct_checkbox: "Attiva elaborazione della punteggiatura vocale",
    vocab_title: "Vocabolario personalizzato",
    vocab_desc: "Aggiungi parole specifiche, nomi o acronimi separati da virgole per migliorare la precisione.",
    vocab_placeholder: "es. Aura, commit, repository...",
    local_model_title: "Modello Whisper locale",
    local_model_desc: "Scarica un modello per l'uso offline. I modelli più grandi sono più precisi ma richiedono più memoria.",
    local_model_label: "Dimensione modello",
    model_meta_tiny: "~75 MB — superveloce",
    model_meta_base: "~145 MB — consigliato",
    model_meta_small: "~465 MB — preciso",
    model_meta_medium: "~1.5 GB — avanzato",
    model_meta_turbo: "~1.6 GB — massima precisione RU/EN",
    model_meta_turbo_q5: "~550 MB — quasi come Turbo, metà del peso",
    hotkey_title: "Tasto di scelta rapida",
    hotkey_desc: "Tieni premuto il tasto per registrare, rilascelo per trascrivere.",
    hotkey_label: "Scorciatoia",
    hotkey_toggle_mode: "Modalità alternata (tocco breve)",
    hotkey_toggle_mode_desc: "Un tocco breve avvia/ferma la registrazione senza tenere premuto.",
    sound_title: "Feedback sonori",
    sound_desc: "Effetti acustici dell'overlay durante la registrazione.",
    sound_enable: "Attiva i suoni dell'overlay",
    sound_volume_label: "Volume del suono",
    sound_theme_label: "Tema sonoro",
    sound_theme_zen: "Zen (Campane tibetane)",
    sound_theme_rhodes: "Rhodes (Piano elettrico)",
    sound_theme_scifi: "Sci-Fi (Spaziale)",
    sound_theme_classic: "Campanella (Classico)",
    api_title: "Autorizzazione chiavi API",
    api_desc: "Inserisci le tue chiavi API per Gemini, OpenAI o Groq.",
    api_provider: "Provider API",
    api_key: "Chiave API",
    api_key_placeholder: "Inserisci la tua chiave API...",
    api_get_key: "Ottieni chiave API",
    history_title: "Cronologia dettati",
    history_clear: "Cancella cronologia",
    history_desc: "Le ultime frasi dettate vengono salvate in locale.",
    history_empty: "La cronologia è vuota. I testi dettati appariranno qui.",
    about_app_title: "Dettatura vocale Aura",
    about_version: "v1.0.5",
    about_description: "Strumento di inserimento vocale globale per Windows. Il programma trascrive la voce in testo e la inserisce in qualsiasi finestra attiva con formattazione e punteggiatura automatiche.",
    status_ready: "Pronto",
    btn_save: "Salva impostazioni",
    confirm_title: "Conferma",
    confirm_message: "Sei sicuro di voler procedere?",
    confirm_cancel: "Annulla",
    confirm_ok: "Conferma",
    status_loading: "Caricamento...",
    status_modified: "Impostazioni modificate (non salvate)",
    status_saving: "Salvataggio...",
    status_saved: "Impostazioni salvate con successo!",
    status_error: "Errore: ",
    model_status_ready: "Installato",
    model_action_download: "Scarica",
    model_action_delete: "Elimina",
    api_get_key_pattern: "Ottieni la chiave su {name}",
    status_loaded: "Impostazioni caricate",
    status_load_error: "Impossibile caricare le impostazioni: ",
    status_save_error: "Impossibile salvare le impostazioni: ",
    model_downloading_pattern: "Avvio del download per il modello '{model}'...",
    model_download_error_pattern: "Errore di download: {err}",
    delete_model_title: "Elimina modello",
    delete_model_confirm_pattern: "Sei sicuro di voler eliminare il modello locale '{model}'?",
    delete_model_btn: "Elimina",
    model_deleting_pattern: "Eliminazione del modello '{model}'...",
    model_deleted_success: "Modello eliminato con successo",
    model_delete_error_pattern: "Errore di eliminazione: {err}",
    model_downloaded_success_pattern: "Modello '{model}' scaricato!",
    confirm_clear_history_title: "Cancella cronologia",
    confirm_clear_history_msg: "Sei sicuro di voler cancellare tutta la cronologia delle trascrizioni?",
    general_ui_lang_title: "Lingua dell'interfaccia",
    general_ui_lang_desc: "Seleziona la lingua per le impostazioni e le notifiche dell'applicazione.",
    hotkey_reset_title: "Ripristina ad Alt+V"
  },
  zh: {
    title_settings: "设置",
    tab_general: "常规",
    tab_speech: "语音",
    tab_hotkeys: "快捷键",
    tab_apikeys: "API密钥",
    tab_history: "历史记录",
    tab_about: "关于我们",
    general_autostart_title: "自启动设置",
    general_autostart_desc: "在Windows启动时自动运行此应用程序。",
    general_autostart_checkbox: "系统启动时运行 Aura",
    engine_title: "处理类型",
    engine_desc: "选择高品质云端识别，或完全离线的本地语音识别。",
    engine_cloud: "云端智能 AI",
    engine_cloud_meta: "Gemini / OpenAI / Groq (需要 API 密钥)",
    engine_local: "本地 AI (离线)",
    engine_local_meta: "Whisper.cpp (100% 离线和私密)",
    lang_bias_title: "识别语言",
    lang_bias_desc: "强制设定特定的听写语言，或使用自动检测。",
    lang_bias_label: "选择语言",
    lang_opt_auto: "自动检测 (默认)",
    lang_opt_layout: "遵循当前键盘布局",
    streaming_title: "输入模式",
    streaming_desc: "选择转换后文本的录入方式。",
    streaming_checkbox: "实时流式文本录入 (实验性)",
    streaming_subdesc: "如果关闭: 只有松开按键后，文字才会一次性录入。",
    punct_title: "智能标点符号",
    punct_desc: "将语音指令(如“逗号”、“句号”)转换为对应的标点符号。",
    punct_checkbox: "开启语音标点转换处理",
    vocab_title: "自定义词典",
    vocab_desc: "以逗号分隔输入专用术语、人名或品牌，以便提高识别精度。",
    vocab_placeholder: "例如：Aura, commit, 仓库...",
    local_model_title: "本地 Whisper 模型",
    local_model_desc: "下载模型以支持离线识别。越大的模型越精准，但会占用更多内存。",
    local_model_label: "模型大小",
    model_meta_tiny: "~75 MB — 超快速",
    model_meta_base: "~145 MB — 推荐",
    model_meta_small: "~465 MB — 精准",
    model_meta_medium: "~1.5 GB — 高级",
    model_meta_turbo: "~1.6 GB — RU/EN 最佳精度",
    model_meta_turbo_q5: "~550 MB — 接近 Turbo，体积减半",
    hotkey_title: "全局快捷键",
    hotkey_desc: "按住选择的组合键开始录音，松开即可完成转文字并录入。",
    hotkey_label: "组合按键",
    hotkey_toggle_mode: "触发模式 (短按切换)",
    hotkey_toggle_mode_desc: "短按启动/停止录音，无需一直按住按键。",
    sound_title: "声音反馈",
    sound_desc: "录音状态切换时播放提示音。",
    sound_enable: "启用悬浮条声音反馈",
    sound_volume_label: "音量",
    sound_theme_label: "声音主题",
    sound_theme_zen: "禅宗 (颂钵音)",
    sound_theme_rhodes: "Rhodes (爵士电钢琴)",
    sound_theme_scifi: "科幻 (太空合成器)",
    sound_theme_classic: "铃声 (经典八音盒)",
    api_title: "API 密钥授权",
    api_desc: "输入您在 Gemini、OpenAI 或 Groq 云端服务的 API 密钥。",
    api_provider: "API 供应商",
    api_key: "API 密钥",
    api_key_placeholder: "在此输入您的 API 密钥...",
    api_get_key: "获取 API 密钥",
    history_title: "听写历史记录",
    history_clear: "清空历史",
    history_desc: "您最近转换出的文字将缓存在本地。",
    history_empty: "历史记录为空。您听写的文字会显示在这里。",
    about_app_title: "Aura 智能语音输入",
    about_version: "v1.0.5",
    about_description: "适用于 Windows 的全局语音输入工具。本程序可以将语音转录为文本，并以自动格式和标点符号插入到任何活动窗口中。",
    status_ready: "就绪",
    btn_save: "保存设置",
    confirm_title: "确认",
    confirm_message: "您确定要执行此操作吗？",
    confirm_cancel: "取消",
    confirm_ok: "确认",
    status_loading: "正在加载设置...",
    status_modified: "设置已更改 (未保存)",
    status_saving: "正在保存设置...",
    status_saved: "设置保存成功！",
    status_error: "发生错误: ",
    model_status_ready: "已安装",
    model_action_download: "下载",
    model_action_delete: "删除",
    api_get_key_pattern: "在 {name} 获取密钥",
    status_loaded: "设置已加载",
    status_load_error: "加载设置失败: ",
    status_save_error: "保存设置失败: ",
    model_downloading_pattern: "正在启动模型 '{model}' 的下载...",
    model_download_error_pattern: "下载错误: {err}",
    delete_model_title: "删除模型",
    delete_model_confirm_pattern: "您确定要删除本地模型 '{model}' 吗？",
    delete_model_btn: "删除",
    model_deleting_pattern: "正在删除模型 '{model}'...",
    model_deleted_success: "模型删除成功",
    model_delete_error_pattern: "删除错误: {err}",
    model_downloaded_success_pattern: "模型 '{model}' 已下载！",
    confirm_clear_history_title: "清空历史",
    confirm_clear_history_msg: "您确定要清空所有听写历史记录吗？",
    general_ui_lang_title: "界面语言",
    general_ui_lang_desc: "选择设置和应用程序通知的语言。",
    hotkey_reset_title: "重置为 Alt+V"
  },
  pt: {
    title_settings: "Configurações",
    tab_general: "Geral",
    tab_speech: "Voz",
    tab_hotkeys: "Teclas de atalho",
    tab_apikeys: "Chaves API",
    tab_history: "Histórico",
    tab_about: "Sobre",
    general_autostart_title: "Inicialização",
    general_autostart_desc: "Iniciar o aplicativo automaticamente com o Windows.",
    general_autostart_checkbox: "Iniciar o Aura com o Windows",
    engine_title: "Tipo de processamento",
    engine_desc: "Escolha entre processamento na nuvem de alta qualidade ou reconhecimento de voz local 100% offline.",
    engine_cloud: "IA na Nuvem",
    engine_cloud_meta: "Gemini / OpenAI / Groq (chave API necessária)",
    engine_local: "IA Local",
    engine_local_meta: "Whisper.cpp (100% offline e privado)",
    lang_bias_title: "Idioma do Diktat",
    lang_bias_desc: "Forçar um idioma específico para a transcrição ou usar detecção automática.",
    lang_bias_label: "Selecionar idioma",
    lang_opt_auto: "Auto-detectar (padrão)",
    lang_opt_layout: "Seguir o teclado ativo",
    streaming_title: "Fluxo de texto",
    streaming_desc: "Escolha o método para exibir o texto transcrito.",
    streaming_checkbox: "Escrita em tempo real (experimental)",
    streaming_subdesc: "Se desativado: o texto é colado inteiro apenas ao soltar o atalho.",
    punct_title: "Pontuação inteligente",
    punct_desc: "Converter comandos de voz (ex. \"vírgula\", \"ponto\") em pontuação correspondente.",
    punct_checkbox: "Habilitar processamento de pontuação por voz",
    vocab_title: "Dicionário personalizado",
    vocab_desc: "Adicione termos específicos, nomes ou siglas separados por vírgula para melhorar o reconhecimento.",
    vocab_placeholder: "ex. Aura, commit, repositório...",
    local_model_title: "Modelo Whisper local",
    local_model_desc: "Baixe um modelo para uso offline. Modelos maiores são mais precisos mas usam mais memória.",
    local_model_label: "Tamanho do modelo",
    model_meta_tiny: "~75 MB — super-rápido",
    model_meta_base: "~145 MB — recomendado",
    model_meta_small: "~465 MB — preciso",
    model_meta_medium: "~1.5 GB — avançado",
    model_meta_turbo: "~1.6 GB — melhor precisão RU/EN",
    model_meta_turbo_q5: "~550 MB — quase como Turbo, metade do tamanho",
    hotkey_title: "Teclas globais",
    hotkey_desc: "Segure o atalho para gravar, solte para transcrever.",
    hotkey_label: "Combinação",
    hotkey_toggle_mode: "Modo alternar (toque rápido)",
    hotkey_toggle_mode_desc: "Um toque rápido inicia/para a gravação sem segurar o botão.",
    sound_title: "Feedback sonoro",
    sound_desc: "Efeitos sonoros do overlay ao gravar.",
    sound_enable: "Habilitar sons do overlay",
    sound_volume_label: "Volume do som",
    sound_theme_label: "Tema sonoro",
    sound_theme_zen: "Zen (Tigelas cantantes)",
    sound_theme_rhodes: "Rhodes (Piano elétrico)",
    sound_theme_scifi: "Sci-Fi (Sintetizador espacial)",
    sound_theme_classic: "Sino (Clássico)",
    api_title: "Autorização de chaves API",
    api_desc: "Insira suas chaves API para os serviços Gemini, OpenAI ou Groq.",
    api_provider: "Provedor de API",
    api_key: "Chave API",
    api_key_placeholder: "Insira sua chave API...",
    api_get_key: "Obter chave API",
    history_title: "Histórico de transcrição",
    history_clear: "Limpar histórico",
    history_desc: "As últimas frases ditadas são armazenadas localmente.",
    history_empty: "O histórico está vazio. Seus textos ditados aparecerão aqui.",
    about_app_title: "Ditado de voz Aura",
    about_version: "v1.0.5",
    about_description: "Ferramenta de entrada de voz global para Windows. O programa transcreve a fala em texto e a insere em qualquer janela ativa com formatação e pontuação automáticas.",
    status_ready: "Pronto",
    btn_save: "Salvar configurações",
    confirm_title: "Confirmação",
    confirm_message: "Tem certeza de que deseja executar esta ação?",
    confirm_cancel: "Cancelar",
    confirm_ok: "Confirmar",
    status_loading: "Carregando...",
    status_modified: "Configurações alteradas (não salvas)",
    status_saving: "Salvando...",
    status_saved: "Configurações salvas com sucesso!",
    status_error: "Erro: ",
    model_status_ready: "Instalado",
    model_action_download: "Baixar",
    model_action_delete: "Excluir",
    api_get_key_pattern: "Obter chave em {name}",
    status_loaded: "Configurações carregadas",
    status_load_error: "Falha ao carregar configurações: ",
    status_save_error: "Falha ao salvar configurações: ",
    model_downloading_pattern: "Iniciando download para o modelo '{model}'...",
    model_download_error_pattern: "Erro de download: {err}",
    delete_model_title: "Excluir modelo",
    delete_model_confirm_pattern: "Tem certeza de que deseja excluir o modelo local '{model}'?",
    delete_model_btn: "Excluir",
    model_deleting_pattern: "Excluindo modelo '{model}'...",
    model_deleted_success: "Modelo excluído com sucesso",
    model_delete_error_pattern: "Erro ao excluir: {err}",
    model_downloaded_success_pattern: "Modelo '{model}' baixado!",
    confirm_clear_history_title: "Limpar histórico",
    confirm_clear_history_msg: "Tem certeza de que deseja limpar todo o histórico de transcrições?",
    general_ui_lang_title: "Idioma da interface",
    general_ui_lang_desc: "Selecione o idioma para as configurações e notificações do aplicativo.",
    hotkey_reset_title: "Redefinir para Alt+V"
  },
  tr: {
    title_settings: "Ayarlar",
    tab_general: "Genel",
    tab_speech: "Ses",
    tab_hotkeys: "Kısayollar",
    tab_apikeys: "API Anahtarları",
    tab_history: "Geçmiş",
    tab_about: "Hakkında",
    general_autostart_title: "Başlangıçta Çalıştır",
    general_autostart_desc: "Windows açıldığında uygulamayı otomatik olarak başlat.",
    general_autostart_checkbox: "Aura'yı sistem açılışında başlat",
    engine_title: "İşlem Türü",
    engine_desc: "Yüksek kaliteli bulut işleme veya tamamen çevrimdışı yerel konuşma tanıma arasında seçim yapın.",
    engine_cloud: "Bulut Yapay Zekası",
    engine_cloud_meta: "Gemini / OpenAI / Groq (API anahtarı gerekli)",
    engine_local: "Yerel Yapay Zeka",
    engine_local_meta: "Whisper.cpp (100% çevrimdışı ve gizli)",
    lang_bias_title: "Yazım Dili",
    lang_bias_desc: "Transkripsiyon için belirli bir dili zorlayın veya otomatik algılamayı kullanın.",
    lang_bias_label: "Dil Seçin",
    lang_opt_auto: "Otomatik Algıla (varsayılan)",
    lang_opt_layout: "Klavye Düzenine Göre",
    streaming_title: "Yazım Modu",
    streaming_desc: "Transkripsiyonu ekleme yöntemini seçin.",
    streaming_checkbox: "Gerçek zamanlı akışlı metin girişi (deneysel)",
    streaming_subdesc: "Kapatılırsa: metin sadece tuşu bıraktığınızda bir bütün olarak eklenir.",
    punct_title: "Akıllı Noktalama",
    punct_desc: "Konuşulan noktalama komutlarını (\"virgül\", \"nokta\") simgelere dönüştür.",
    punct_checkbox: "Sesli noktalama işaretlerini işlemeyi etkinleştir",
    vocab_title: "Özel Sözlük",
    vocab_desc: "Algılama kalitesini artırmak için özel terimleri, isimleri virgülle ayırarak girin.",
    vocab_placeholder: "örn. Aura, commit, depo...",
    local_model_title: "Yerel Whisper Modülü",
    local_model_desc: "Çevrimdışı kullanım için bir model indirin. Daha büyük modeller daha doğrudur ancak daha fazla bellek kullanır.",
    local_model_label: "Model Boyutu",
    model_meta_tiny: "~75 MB — süper hızlı",
    model_meta_base: "~145 MB — önerilen",
    model_meta_small: "~465 MB — hassas",
    model_meta_medium: "~1.5 GB — gelişmiş",
    model_meta_turbo: "~1.6 GB — RU/EN için en iyi doğruluk",
    model_meta_turbo_q5: "~550 MB — Turbo'ya yakın, yarı boyut",
    hotkey_title: "Global Kısayol",
    hotkey_desc: "Kayda başlamak için seçilen kombinasyonu basılı tutun, transkripsiyon için bırakın.",
    hotkey_label: "Kombinasyon",
    hotkey_toggle_mode: "Geçiş modu (kısa basma)",
    hotkey_toggle_mode_desc: "Kısa bir basış, basılı tutmadan kaydı başlatır veya durdurur.",
    sound_title: "Ses Geri Bildirimi",
    sound_desc: "Kayıt durumları değiştiğinde çalınacak ses efektleri.",
    sound_enable: "Overlay seslerini etkinleştir",
    sound_volume_label: "Ses Seviyesi",
    sound_theme_label: "Ses Teması",
    sound_theme_zen: "Zen (Nepal Çanakları)",
    sound_theme_rhodes: "Rhodes (Caz Elektro Piyano)",
    sound_theme_scifi: "Sci-Fi (Uzay Sentezleyici)",
    sound_theme_classic: "Zil (Klasik)",
    api_title: "API Anahtarları Yetkilendirme",
    api_desc: "Gemini, OpenAI veya Groq bulut hizmetleri için API anahtarlarınızı girin.",
    api_provider: "API Sağlayıcısı",
    api_key: "API Anahtarı",
    api_key_placeholder: "API anahtarınızı buraya girin...",
    api_get_key: "API Anahtarı Al",
    history_title: "Yazım Geçmişi",
    history_clear: "Geçmişi Temizle",
    history_desc: "Son sesli yazımlarınız yerel olarak saklanır.",
    history_empty: "Geçmiş boş. Yazdığınız metinler burada görünecektir.",
    about_app_title: "Aura Sesli Giriş",
    about_version: "v1.0.5",
    about_description: "Windows için genel sesli giriş aracı. Program, konuşmayı metne dönüştürür ve otomatik biçimlendirme ve noktalama işaretleriyle herhangi bir aktif pencereye ekler.",
    status_ready: "Hazır",
    btn_save: "Ayarları Kaydet",
    confirm_title: "Onay",
    confirm_message: "Bu işlemi gerçekleştirmek istediğinizden emin misiniz?",
    confirm_cancel: "İptal",
    confirm_ok: "Onayla",
    status_loading: "Ayarlar yükleniyor...",
    status_modified: "Ayarlar değiştirildi (kaydedilmedi)",
    status_saving: "Ayarlar kaydediliyor...",
    status_saved: "Ayarlar başarıyla kaydedildi!",
    status_error: "Hata: ",
    model_status_ready: "Yüklendi",
    model_action_download: "İndir",
    model_action_delete: "Sil",
    api_get_key_pattern: "{name} üzerinden anahtar al",
    status_loaded: "Ayarlar yüklendi",
    status_load_error: "Ayarlar yüklenemedi: ",
    status_save_error: "Ayarlar kaydedilemedi: ",
    model_downloading_pattern: "'{model}' modeli için indirme başlatılıyor...",
    model_download_error_pattern: "İndirme hatası: {err}",
    delete_model_title: "Modeli sil",
    delete_model_confirm_pattern: "Yerel '{model}' modelini silmek istediğinizden emin misiniz?",
    delete_model_btn: "Sil",
    model_deleting_pattern: "'{model}' modeli siliniyor...",
    model_deleted_success: "Model başarıyla silindi",
    model_delete_error_pattern: "Silme hatası: {err}",
    model_downloaded_success_pattern: "'{model}' modeli indirildi!",
    confirm_clear_history_title: "Geçmişi Temizle",
    confirm_clear_history_msg: "Tüm transkripsiyon geçmişini temizlemek istediğinizden emin misiniz?",
    general_ui_lang_title: "Arayüz Dili",
    general_ui_lang_desc: "Ayarlar ve uygulama bildirimleri için dili seçin.",
    hotkey_reset_title: "Alt+V'ye Sıfırla"
  }
};

let currentLanguage = "ru";

function getTranslation(key, params = {}) {
  const dict = i18nDict[currentLanguage] || i18nDict.ru;
  let template = dict[key] || i18nDict.ru[key] || key;
  for (const [k, v] of Object.entries(params)) {
    template = template.replaceAll(`{${k}}`, v);
  }
  return template;
}

document.addEventListener("DOMContentLoaded", () => {
  // Navigation Tabs
  const tabs = document.querySelectorAll(".nav-tab");
  const panels = document.querySelectorAll(".tab-panel");
  
  tabs.forEach(tab => {
    tab.addEventListener("click", () => {
      tabs.forEach(t => {
        t.classList.remove("active");
        t.setAttribute("aria-selected", "false");
      });
      panels.forEach(p => p.style.display = "none");

      tab.classList.add("active");
      tab.setAttribute("aria-selected", "true");
      const targetPanel = document.getElementById(`panel-${tab.dataset.tab}`);
      if (targetPanel) {
        targetPanel.style.display = "flex";
      }

      // If clicking history tab, reload history entries
      if (tab.dataset.tab === "history") {
        loadHistoryList();
      }
    });
  });

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
  const localModelCard = document.getElementById("card-local-model");
  const selectLocalEngine = document.getElementById("select-local-engine");
  const groupWhisperModels = document.getElementById("group-whisper-models");
  const groupParakeetModels = document.getElementById("group-parakeet-models");

  function updateEngineUI() {
    if (radioLocal.checked) {
      localModelCard.style.display = "flex";
      updateLocalEngineUI();
    } else {
      localModelCard.style.display = "none";
    }
  }

  function updateLocalEngineUI() {
    if (!selectLocalEngine || !groupWhisperModels || !groupParakeetModels) return;
    if (selectLocalEngine.value === "parakeet") {
      groupWhisperModels.style.display = "none";
      groupParakeetModels.style.display = "block";
      selectModelCard("parakeet-v3");
    } else {
      groupWhisperModels.style.display = "block";
      groupParakeetModels.style.display = "none";
      if (selectedModelName === "parakeet-v3") {
        selectModelCard("base");
      }
    }
  }

  if (selectLocalEngine) {
    selectLocalEngine.addEventListener("change", () => {
      updateLocalEngineUI();
      markSettingsModified();
    });
  }

  radioCloud.addEventListener("change", updateEngineUI);
  radioLocal.addEventListener("change", updateEngineUI);

  // Dynamic API Key Links
  const linkGetKey = document.getElementById("link-get-key");
  const providerLinks = {
    gemini: { url: "https://aistudio.google.com/", name: "Google AI Studio" },
    openai: { url: "https://platform.openai.com/api-keys", name: "OpenAI Platform" },
    groq: { url: "https://console.groq.com/keys", name: "Groq Console" }
  };
  function updateApiKeyLink() {
    const prov = selectProvider ? selectProvider.value : "gemini";
    const info = providerLinks[prov] || providerLinks.gemini;
    if (linkGetKey) {
      linkGetKey.href = info.url;
      linkGetKey.textContent = getTranslation("api_get_key_pattern", { name: info.name });
    }
  }

  // Settings elements
  const selectProvider = document.getElementById("select-provider");
  const selectHotkey = document.getElementById("input-hotkey");
  const selectLanguage = document.getElementById("select-language");
  const textareaDictionary = document.getElementById("textarea-dictionary");
  const checkboxToggle = document.getElementById("checkbox-toggle");
  const checkboxPunctuation = document.getElementById("checkbox-punctuation");
  const checkboxCloudFallback = document.getElementById("checkbox-cloud-fallback");
  const checkboxAutostart = document.getElementById("checkbox-autostart");
  const btnSaveSettings = document.getElementById("btn-save-settings");
  
  const checkboxSounds = document.getElementById("checkbox-sounds");
  const selectSoundTheme = document.getElementById("select-sound-theme");
  const rangeVolume = document.getElementById("range-sound-volume");
  const volumeLabel = document.getElementById("volume-value-label");

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

  const allowedSpecialKeys = {
    "Space": "Space",
    " ": "Space",
    "CapsLock": "Caps Lock",
    "Tab": "Tab",
    "F1": "F1", "F2": "F2", "F3": "F3", "F4": "F4", "F5": "F5", "F6": "F6",
    "F7": "F7", "F8": "F8", "F9": "F9", "F10": "F10", "F11": "F11", "F12": "F12"
  };

  if (selectHotkey) {
    selectHotkey.addEventListener("focus", () => {
      isRecordingHotkey = true;
      hasRecordedThisSession = false;
      selectHotkey.value = "Нажмите клавиши...";
      selectHotkey.classList.add("recording");
    });

    selectHotkey.addEventListener("blur", () => {
      isRecordingHotkey = false;
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

      // Ignore modifiers themselves
      if (key === "Control" || key === "Alt" || key === "Shift" || key === "Meta" ||
          code === "ControlLeft" || code === "ControlRight" ||
          code === "AltLeft" || code === "AltRight" ||
          code === "ShiftLeft" || code === "ShiftRight") {
        return;
      }

      let modifier = "";
      if (e.ctrlKey) modifier = "Ctrl";
      else if (e.altKey) modifier = "Alt";
      else if (e.shiftKey) modifier = "Shift";

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

      const hotkeyStr = modifier ? `${modifier}+${keyName}` : keyName;
      hasRecordedThisSession = true; // Mark as successfully recorded
      selectHotkey.value = hotkeyStr;
      isRecordingHotkey = false;
      selectHotkey.classList.remove("recording");
      selectHotkey.blur();

      // Trigger modified state
      selectHotkey.dispatchEvent(new Event("change", { bubbles: true }));
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
    groq: ""
  };
  let previousSelProvider = selectProvider.value;
  
  const footerStatusText = document.getElementById("footer-status-text");

  let selectedModelName = "base";

  let settingsModified = false;
  let isSettingsLoaded = false;

  function markSettingsModified() {
    if (!isSettingsLoaded) return;
    if (!settingsModified) {
      settingsModified = true;
      showStatus("Настройки изменены (не сохранены)", false, true);
    }
  }

  function bindSettingsChangeListeners() {
    const checkboxStreaming = document.getElementById("checkbox-streaming");
    const inputs = [
      radioCloud, radioLocal, selectProvider, apiKeyInput, selectHotkey,
      selectLanguage, textareaDictionary, checkboxToggle, checkboxPunctuation, checkboxCloudFallback,
      checkboxAutostart, checkboxStreaming, checkboxSounds, selectSoundTheme,
      rangeVolume, selectLocalEngine
    ];
    inputs.forEach(input => {
      if (input) {
        input.addEventListener("change", markSettingsModified);
        input.addEventListener("input", markSettingsModified);
      }
    });
  }
  const modelCards = document.querySelectorAll(".model-card");

  modelCards.forEach(card => {
    card.addEventListener("click", (e) => {
      // Prevent selection trigger when clicking delete/download buttons inside the card
      if (e.target.closest(".btn-delete-card-model") || e.target.closest(".btn-download-card-model") || e.target.closest(".btn-cancel-download")) {
        return;
      }
      selectModelCard(card.dataset.model);
    });

    card.addEventListener("keydown", (e) => {
      if (e.key === "Enter" || e.key === " ") {
        e.preventDefault();
        selectModelCard(card.dataset.model);
      }
    });
  });

  function selectModelCard(model) {
    if (selectedModelName !== model) {
      selectedModelName = model;
      markSettingsModified();
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
        
        selectModelCard(settings.model_name || "base");
        apiKeys.gemini = settings.api_key_gemini || "";
        apiKeys.openai = settings.api_key_openai || "";
        apiKeys.groq = settings.api_key_groq || "";
        
        selectProvider.value = settings.api_provider || "gemini";
        previousSelProvider = selectProvider.value;
        apiKeyInput.value = apiKeys[selectProvider.value] || "";
        updateApiKeyLink();
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
        if (checkboxPunctuation) {
          checkboxPunctuation.checked = !!settings.voice_punctuation;
        }
        if (checkboxCloudFallback) {
          checkboxCloudFallback.checked = settings.cloud_fallback_enabled !== false;
        }
        if (checkboxAutostart) {
          checkboxAutostart.checked = !!settings.autostart;
        }
 
        const checkboxStreaming = document.getElementById("checkbox-streaming");
        if (checkboxStreaming) {
          checkboxStreaming.checked = !!settings.streaming_enabled;
        }
 
        if (checkboxSounds) {
          checkboxSounds.checked = settings.overlay_sounds !== false;
        }
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
        await refreshDownloadedModels();
        
        isSettingsLoaded = true;
        settingsModified = false;
        
        // Sync custom dropdown values
        syncCustomSelects();
        
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
    } catch (err) {
      console.error("Failed to check downloaded models", err);
    }
  }

  // Save Settings to Backend
  async function saveSettings() {
    try {
      const dict = i18nDict[currentLanguage] || i18nDict.ru;
      showStatus(dict.status_saving || "Сохранение настроек...");
      
      const checkboxStreaming = document.getElementById("checkbox-streaming");
      
      // Update apiKeys cache from active input first:
      apiKeys[selectProvider.value] = apiKeyInput.value;

      const soundVolFloat = rangeVolume ? parseFloat(rangeVolume.value) / 100 : 0.8;

      const settings = {
        transcription_mode: radioLocal.checked ? "local" : "cloud",
        api_provider: selectProvider.value,
        api_key: apiKeyInput.value,
        api_key_gemini: apiKeys.gemini,
        api_key_openai: apiKeys.openai,
        api_key_groq: apiKeys.groq,
        model_name: selectedModelName,
        hotkey: selectHotkey ? selectHotkey.value : "Alt+V",
        streaming_enabled: checkboxStreaming ? checkboxStreaming.checked : false,
        toggle_enabled: checkboxToggle ? checkboxToggle.checked : false,
        language: selectLanguage ? selectLanguage.value : "auto",
        dictionary: textareaDictionary ? textareaDictionary.value : "",
        voice_punctuation: checkboxPunctuation ? checkboxPunctuation.checked : false,
        cloud_fallback_enabled: checkboxCloudFallback ? checkboxCloudFallback.checked : true,
        autostart: checkboxAutostart ? checkboxAutostart.checked : false,
        local_engine: selectLocalEngine ? selectLocalEngine.value : "whisper",
        overlay_sounds: checkboxSounds ? checkboxSounds.checked : true,
        overlay_sound_theme: selectSoundTheme ? selectSoundTheme.value : "zen",
        overlay_sound_volume: soundVolFloat
      };

      await invoke("set_settings", { settings });
      settingsModified = false;
      showStatus(dict.status_saved || "Настройки успешно сохранены!");
      
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
    }
  }

  async function downloadModelCard(model) {
    try {
      showStatus(getTranslation("model_downloading_pattern", { model }));
      const actionEl = document.getElementById(`action-${model}`);
      const progressEl = document.getElementById(`progress-${model}`);
      const fillEl = document.getElementById(`fill-${model}`);
      const pctEl = document.getElementById(`pct-${model}`);

      // Hide actions, show progress
      actionEl.style.display = "none";
      progressEl.style.display = "flex";
      fillEl.style.width = "0%";
      pctEl.textContent = "0%";

      // Add a fresh cancel (×) button while downloading
      if (progressEl) {
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
          invoke("cancel_model_download", { modelName: model }).catch(e2 => console.error(e2));
        });
        progressEl.appendChild(cancelBtn);
      }

      await invoke("download_model_command", { modelName: model });
    } catch (err) {
      console.error(err);
      const errStr = String(err).toLowerCase();
      if (errStr.includes("cancel")) {
        showStatus(getTranslation("model_download_cancelled") || "Загрузка отменена");
      } else {
        showStatus(getTranslation("model_download_error_pattern", { err }), true);
      }
      refreshDownloadedModels();
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
      await refreshDownloadedModels();
    } catch (err) {
      console.error(err);
      showStatus(getTranslation("model_delete_error_pattern", { err }), true);
    }
  }

  // Listen to model-download-progress events from Rust
  listen("model-download-progress", (event) => {
    const payload = event.payload;
    if (!payload) return;

    const model = payload.model;
    const percent = Math.round(payload.percentage);
    
    const fillEl = document.getElementById(`fill-${model}`);
    const pctEl = document.getElementById(`pct-${model}`);
    const progressEl = document.getElementById(`progress-${model}`);
    const actionEl = document.getElementById(`action-${model}`);
    
    if (fillEl && pctEl) {
      fillEl.style.width = `${percent}%`;
      pctEl.textContent = `${percent}%`;
    }

    if (payload.done) {
      showStatus(getTranslation("model_downloaded_success_pattern", { model }));
      if (progressEl) {
        const cancelBtn = progressEl.querySelector(".btn-cancel-download");
        if (cancelBtn) cancelBtn.remove();
        progressEl.style.display = "none";
      }
      if (actionEl) actionEl.style.display = "flex";
      refreshDownloadedModels();
    }
  });

  // --- Asynchronous Custom Confirm Dialog ---
  function showConfirm(title, message, confirmText = "ОК", cancelText = "Отмена") {
    return new Promise((resolve) => {
      const modal = document.getElementById("custom-confirm-modal");
      const titleEl = document.getElementById("confirm-modal-title");
      const msgEl = document.getElementById("confirm-modal-message");
      const btnOk = document.getElementById("btn-confirm-ok");
      const btnCancel = document.getElementById("btn-confirm-cancel");

      if (!modal) {
        resolve(false);
        return;
      }

      titleEl.textContent = title;
      msgEl.textContent = message;
      btnOk.textContent = confirmText;
      btnCancel.textContent = cancelText;

      modal.style.display = "flex";
      modal.offsetHeight; // force reflow
      modal.classList.add("active");

      function cleanUp(result) {
        modal.classList.remove("active");
        btnOk.removeEventListener("click", onOk);
        btnCancel.removeEventListener("click", onCancel);
        setTimeout(() => {
          modal.style.display = "none";
          resolve(result);
        }, 200);
      }

      function onOk() { cleanUp(true); }
      function onCancel() { cleanUp(false); }

      btnOk.addEventListener("click", onOk);
      btnCancel.addEventListener("click", onCancel);
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
  btnSaveSettings.addEventListener("click", saveSettings);
  selectProvider.addEventListener("change", () => {
    // 1. Save input to current provider
    apiKeys[previousSelProvider] = apiKeyInput.value;
    // 2. Switch provider
    previousSelProvider = selectProvider.value;
    // 3. Load input from new provider
    apiKeyInput.value = apiKeys[selectProvider.value] || "";
    updateApiKeyLink();
  });

  // Window controls via Tauri IPC commands
  const btnWindowMinimize = document.getElementById("btn-window-minimize");
  const btnWindowClose = document.getElementById("btn-window-close");
  
  if (btnWindowMinimize) {
    btnWindowMinimize.addEventListener("click", () => invoke("minimize_window"));
  }
  if (btnWindowClose) {
    btnWindowClose.addEventListener("click", () => invoke("close_window"));
  }

  // Window dragging via mousedown on header (bypasses click-through/drag bugs in Webview2)
  const appHeader = document.querySelector(".app-header");
  if (appHeader) {
    appHeader.addEventListener("mousedown", (e) => {
      // Only trigger drag on left click and avoid dragging when clicking on control buttons or select elements
      if (e.button === 0 && !e.target.closest(".window-control-btn") && !e.target.closest("button") && !e.target.closest("select")) {
        invoke("start_dragging_command");
      }
    });
  }

  // Custom Select Dropdowns generator
  function initCustomSelects() {
    const selects = document.querySelectorAll("select.custom-select");
    selects.forEach(select => {
      // If already initialized, skip
      if (select.nextElementSibling && select.nextElementSibling.classList.contains("custom-dropdown")) {
        return;
      }
      
      select.style.display = "none";
      
      const dropdown = document.createElement("div");
      dropdown.className = "custom-dropdown";
      dropdown.dataset.selectId = select.id;
      
      const trigger = document.createElement("div");
      trigger.className = "custom-dropdown-trigger";
      trigger.tabIndex = 0;
      
      const valueSpan = document.createElement("span");
      valueSpan.className = "custom-dropdown-value";
      
      const arrow = document.createElement("div");
      arrow.className = "custom-dropdown-arrow";
      
      trigger.appendChild(valueSpan);
      trigger.appendChild(arrow);
      dropdown.appendChild(trigger);
      
      const optionsContainer = document.createElement("div");
      optionsContainer.className = "custom-dropdown-options";
      
      const options = select.querySelectorAll("option");
      options.forEach(opt => {
        const optionDiv = document.createElement("div");
        optionDiv.className = "custom-dropdown-option";
        optionDiv.dataset.value = opt.value;
        optionDiv.textContent = opt.textContent;
        
        // Replicate custom list translations
        const i18n = opt.getAttribute("data-i18n");
        if (i18n) {
          optionDiv.setAttribute("data-i18n", i18n);
        }
        
        if (opt.selected) {
          optionDiv.classList.add("selected");
          valueSpan.textContent = opt.textContent;
          if (i18n) {
            valueSpan.setAttribute("data-i18n", i18n);
          }
        }
        
        optionDiv.addEventListener("click", (e) => {
          e.stopPropagation();
          select.value = opt.value;
          
          dropdown.querySelectorAll(".custom-dropdown-option").forEach(item => {
            item.classList.remove("selected");
          });
          optionDiv.classList.add("selected");
          
          valueSpan.textContent = opt.textContent;
          const optI18n = opt.getAttribute("data-i18n");
          if (optI18n) {
            valueSpan.setAttribute("data-i18n", optI18n);
          } else {
            valueSpan.removeAttribute("data-i18n");
          }
          
          dropdown.classList.remove("open");
          
          // Dispatch change and input events
          select.dispatchEvent(new Event("change", { bubbles: true }));
          select.dispatchEvent(new Event("input", { bubbles: true }));
        });
        
        optionsContainer.appendChild(optionDiv);
      });
      
      dropdown.appendChild(optionsContainer);
      
      trigger.addEventListener("click", (e) => {
        e.stopPropagation();
        
        document.querySelectorAll(".custom-dropdown").forEach(d => {
          if (d !== dropdown) {
            d.classList.remove("open");
          }
        });
        
        dropdown.classList.toggle("open");
      });
      
      trigger.addEventListener("keydown", (e) => {
        if (e.key === "Enter" || e.key === " ") {
          e.preventDefault();
          trigger.click();
        }
      });
      
      select.parentNode.insertBefore(dropdown, select.nextSibling);
    });
    
    // Close dropdowns on outside click
    document.addEventListener("click", () => {
      document.querySelectorAll(".custom-dropdown").forEach(d => {
        d.classList.remove("open");
      });
    });
  }

  function syncCustomSelects() {
    const dropdowns = document.querySelectorAll(".custom-dropdown");
    dropdowns.forEach(dropdown => {
      const selectId = dropdown.dataset.selectId;
      const select = document.getElementById(selectId);
      if (!select) return;
      
      const valueSpan = dropdown.querySelector(".custom-dropdown-value");
      const selectedOption = select.options[select.selectedIndex];
      if (selectedOption && valueSpan) {
        valueSpan.textContent = selectedOption.textContent;
        const i18n = selectedOption.getAttribute("data-i18n");
        if (i18n) {
          valueSpan.setAttribute("data-i18n", i18n);
        } else {
          valueSpan.removeAttribute("data-i18n");
        }
      }
      
      const optionDivs = dropdown.querySelectorAll(".custom-dropdown-option");
      optionDivs.forEach(optDiv => {
        const isSelected = optDiv.dataset.value === select.value;
        optDiv.classList.toggle("selected", isSelected);
      });
    });
  }

  // Translations Helper
  function applyLanguage(lang) {
    currentLanguage = lang;
    const dict = i18nDict[lang] || i18nDict.ru;
    
    // Update data-i18n elements
    const elements = document.querySelectorAll("[data-i18n]");
    elements.forEach(el => {
      const key = el.getAttribute("data-i18n");
      if (dict[key]) {
        el.textContent = dict[key];
      }
    });

    // Update custom dropdown labels
    const dropdowns = document.querySelectorAll(".custom-dropdown");
    dropdowns.forEach(dropdown => {
      const selectId = dropdown.dataset.selectId;
      const select = document.getElementById(selectId);
      if (!select) return;
      
      const valueSpan = dropdown.querySelector(".custom-dropdown-value");
      const selectedOption = select.options[select.selectedIndex];
      if (selectedOption && valueSpan) {
        valueSpan.textContent = selectedOption.textContent;
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
    
    // Update dynamic link text
    updateApiKeyLink();
    
    // Refresh model cards status/actions
    refreshDownloadedModels();
    
    // If settings modified status is showing, update it
    if (settingsModified) {
      showStatus(dict.status_modified, false, true);
    }
    
    // Reload history list if the active panel is panel-history
    const historyTab = document.getElementById("tab-btn-history");
    if (historyTab && historyTab.classList.contains("active")) {
      loadHistoryList();
    }
  }

  // --- History List & Clear Interactions ---
  const historyContainer = document.getElementById("history-items-container");
  const btnClearHistory = document.getElementById("btn-clear-history");

  async function loadHistoryList() {
    if (!historyContainer) return;
    try {
      const history = await invoke("get_history");
      const dict = i18nDict[currentLanguage] || i18nDict.ru;
      
      if (!history || history.length === 0) {
        historyContainer.innerHTML = `<div class="history-empty-state" id="history-empty-text" data-i18n="history_empty">${dict.history_empty}</div>`;
        return;
      }

      historyContainer.innerHTML = "";
      const fragment = document.createDocumentFragment();
      history.forEach(entry => {
        const date = new Date(entry.timestamp_ms);
        const timeStr = date.toLocaleTimeString(currentLanguage, { hour: '2-digit', minute: '2-digit', second: '2-digit' });
        const dateStr = date.toLocaleDateString(currentLanguage, { month: 'short', day: 'numeric' });
        const displayTime = `${dateStr}, ${timeStr}`;

        const itemEl = document.createElement("div");
        itemEl.className = "history-item";
        
        itemEl.innerHTML = `
          <div class="history-item-body">
            <div class="history-item-meta">
              <span class="history-item-time">${displayTime}</span>
              <span class="history-item-badge">${entry.mode === 'local' ? 'Local' : 'Cloud'}</span>
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
    } catch (err) {
      console.error("Failed to load history", err);
    }
  }

  function escapeHtml(text) {
    return (text || "")
      .replace(/&/g, "&amp;")
      .replace(/</g, "&lt;")
      .replace(/>/g, "&gt;")
      .replace(/"/g, "&quot;")
      .replace(/'/g, "&#039;");
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

  // Initialize custom selects
  initCustomSelects();

  // Initialize UI language and Settings
  (async () => {
    let settings = null;
    try {
      settings = await invoke("get_settings");
    } catch (err) {
      console.error(err);
    }

    let savedUiLang = localStorage.getItem("aura_ui_lang");
    if (savedUiLang === null) {
      savedUiLang = localStorage.getItem("ui-language");
    }

    const supportedLangs = ["ru", "en", "de", "es", "fr", "it", "zh", "pt", "tr"];
    if (savedUiLang === null || !supportedLangs.includes(savedUiLang)) {
      if (settings && settings.language && supportedLangs.includes(settings.language)) {
        savedUiLang = settings.language;
      } else {
        savedUiLang = "ru";
      }
    }

    // UI Language Selector Setup
    const selectUiLang = document.getElementById("select-ui-lang");
    if (selectUiLang) {
      selectUiLang.value = savedUiLang;
      
      selectUiLang.addEventListener("change", (e) => {
        const selectedLang = e.target.value;
        localStorage.setItem("aura_ui_lang", selectedLang);
        localStorage.setItem("ui-language", selectedLang);
        applyLanguage(selectedLang);
      });
    }
    
    // Apply initial language choice outside the if block so translations initialize even if #select-ui-lang is missing
    applyLanguage(savedUiLang);

    // Initialize Settings
    await loadSettings(settings);

    // Check GitHub for a newer release; show the update badge in About if found
    try {
      const badge = document.getElementById("update-badge");
      const badgeText = document.getElementById("update-badge-text");
      const navDot = document.getElementById("update-dot");
      if (badge) {
        const info = await invoke("check_for_update");
        if (info && info.available) {
          const label = getTranslation("update_available") || "Доступно обновление";
          badgeText.textContent = `${label} (v${info.latest})`;
          badge.style.display = "inline-flex";
          badge.addEventListener("click", async () => {
            try {
              showStatus("Скачивание и установка обновления...");
              const { check } = window.__TAURI__.plugins.updater;
              const update = await check();
              if (update) {
                await update.downloadAndInstall();
                showStatus("Обновление установлено. Перезапуск...");
                await invoke("relaunch_app");
              } else {
                showStatus("Обновление не найдено.");
              }
            } catch (e) {
              console.error("Failed to install update via Tauri updater", e);
              showStatus("Открытие страницы релиза в браузере...");
              invoke("open_url", { url: info.url }).catch(err => console.error(err));
            }
          });
          // Visible without opening the About tab: a dot on the nav item itself
          if (navDot) navDot.style.display = "inline-block";
        }
      }
    } catch (err) {
      console.error("Update check failed", err);
    }
  })();
});
