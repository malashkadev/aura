use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{Arc, Mutex};

/// `dwExtraInfo` signature stamped on every `SendInput` event Aura emits.
///
/// The low-level hook must be able to tell Aura's own synthetic keystrokes
/// apart from events injected by third-party keyboard utilities (layout
/// switchers such as Caramba/Punto, key remappers, macro tools). Without the
/// tag an injected target key cannot be distinguished from Aura's own paste,
/// so the only safe policy was to pass every injected event through — which
/// let a third-party tool retype the hotkey's own letter into the document.
pub const AURA_INJECT_TAG: usize = 0x4155_5241; // "AURA"
/// Re-assignable callback registries (C14): a failed hook installation must
/// not permanently occupy its slot or recovery becomes impossible. The
/// callback is taken out for the duration of each invocation and put back.
type HotkeyCallback = Box<dyn Fn(bool) + Send + Sync>;
type CancelCallback = Box<dyn Fn() + Send + Sync>;
static CALLBACK: Mutex<Option<HotkeyCallback>> = Mutex::new(None);
static CANCEL_CALLBACK: Mutex<Option<CancelCallback>> = Mutex::new(None);
static USER_INPUT_CALLBACK: Mutex<Option<CancelCallback>> = Mutex::new(None);
static RECORDING_ACTIVE: AtomicBool = AtomicBool::new(false);
static SHORTCUT_ACTIVE: AtomicBool = AtomicBool::new(false);
static KEY_SUPPRESSED: AtomicBool = AtomicBool::new(false);
static HOOK_PAUSED: AtomicBool = AtomicBool::new(false);

/// Marks whether the keyboard hook is paused (e.g. during Gaming Mode).
pub fn set_hook_paused(paused: bool) {
    HOOK_PAUSED.store(paused, Ordering::SeqCst);
    if paused {
        #[cfg(target_os = "windows")]
        windows_impl::clear_phantom_hotkey_state();
        #[cfg(target_os = "macos")]
        macos_impl::clear_phantom_hotkey_state();
    }
}

pub fn is_hook_paused() -> bool {
    HOOK_PAUSED.load(Ordering::SeqCst)
}

/// Marks whether a recording session is active (enables Esc interception).
pub fn set_recording_active(active: bool) {
    RECORDING_ACTIVE.store(active, Ordering::SeqCst);
}

/// Registers the callback fired when the user presses Esc during recording.
pub fn set_cancel_callback<F>(callback: F) -> Result<(), &'static str>
where
    F: Fn() + Send + Sync + 'static,
{
    match CANCEL_CALLBACK.lock() {
        Ok(mut guard) => {
            *guard = Some(Box::new(callback));
            Ok(())
        }
        Err(_) => Err("Cancel callback registry is poisoned"),
    }
}

/// Registers a callback for non-injected physical keyboard input that was not
/// consumed as Aura's own hotkey. The application uses this to stop destructive
/// live-text reconciliation after the user edits or undoes the target document.
pub fn set_user_input_callback<F>(callback: F) -> Result<(), &'static str>
where
    F: Fn() + Send + Sync + 'static,
{
    match USER_INPUT_CALLBACK.lock() {
        Ok(mut guard) => {
            *guard = Some(Box::new(callback));
            Ok(())
        }
        Err(_) => Err("User-input callback registry is poisoned"),
    }
}

fn notify_user_input() {
    invoke_fn_callback(&USER_INPUT_CALLBACK, || {
        crate::logger::log(
            "ERROR",
            "Hotkey",
            None,
            "User-input callback panicked; the panic was contained at the OS hook boundary",
        )
    })
}

/// Takes a registered callback out for the duration of its invocation and
/// puts it back afterwards, so registration stays replaceable and the call
/// itself cannot deadlock on its own registry.
fn invoke_fn_callback(registry: &Mutex<Option<CancelCallback>>, on_panic: impl FnOnce()) {
    let callback = match registry.lock() {
        Ok(mut guard) => guard.take(),
        Err(_) => {
            on_panic();
            return;
        }
    };
    let Some(callback) = callback else { return };
    // Arc lets the callback be invoked by value into catch_unwind while the
    // original stays here to be put back into the registry afterwards.
    let callback = Arc::new(callback);
    let invoke_arc = Arc::clone(&callback);
    let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(move || invoke_arc()));
    if outcome.is_err() {
        on_panic();
    }
    if let Ok(mut guard) = registry.lock() {
        if let Ok(callback) = Arc::try_unwrap(callback) {
            *guard = Some(callback);
        }
    }
}

fn invoke_bool_callback(
    registry: &Mutex<Option<HotkeyCallback>>,
    is_down: bool,
    on_panic: impl FnOnce(),
) {
    let callback = match registry.lock() {
        Ok(mut guard) => guard.take(),
        Err(_) => {
            on_panic();
            return;
        }
    };
    let Some(callback) = callback else { return };
    let callback = Arc::new(callback);
    let invoke_arc = Arc::clone(&callback);
    let outcome =
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(move || invoke_arc(is_down)));
    if outcome.is_err() {
        on_panic();
    }
    if let Ok(mut guard) = registry.lock() {
        if let Ok(callback) = Arc::try_unwrap(callback) {
            *guard = Some(callback);
        }
    }
}

// ============================================================================
// WINDOWS IMPLEMENTATION
// ============================================================================
#[cfg(target_os = "windows")]
mod windows_impl {
    use super::*;
    use std::sync::atomic::AtomicU64;
    use std::time::{SystemTime, UNIX_EPOCH};
    use windows_sys::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
    use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
    use windows_sys::Win32::System::RemoteDesktop::{
        WTSRegisterSessionNotification, WTSUnRegisterSessionNotification, NOTIFY_FOR_THIS_SESSION,
    };
    use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
        GetAsyncKeyState, SendInput, INPUT, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_EXTENDEDKEY,
        KEYEVENTF_KEYUP, KEYEVENTF_UNICODE, VK_CONTROL,
    };
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        CallNextHookEx, CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW,
        GetMessageW, RegisterClassW, SetWindowsHookExW, TranslateMessage, UnhookWindowsHookEx,
        HWND_MESSAGE, KBDLLHOOKSTRUCT, MSG, WH_KEYBOARD_LL, WM_KEYDOWN, WM_KEYUP, WM_SYSKEYDOWN,
        WM_SYSKEYUP, WM_WTSSESSION_CHANGE, WNDCLASSW, WTS_SESSION_LOCK, WTS_SESSION_UNLOCK,
    };

    const MOD_CTRL: u32 = 1 << 0;
    const MOD_ALT: u32 = 1 << 1;
    const MOD_SHIFT: u32 = 1 << 2;
    const MOD_WIN: u32 = 1 << 3;

    /// `KEYEVENTF_UNICODE` is a `KEYBDINPUT` flag, not a
    /// `KBDLLHOOKSTRUCT.flags` bit: Windows reports such an injection to a
    /// low-level hook as `vkCode = VK_PACKET` with the UTF-16 code unit in
    /// `scanCode`. Checking `flags & 0x04` alone never matches.
    const VK_PACKET: u32 = 0xE7;

    /// The configured hotkey packed into one word: (modifier_mask << 16) | key_vk.
    /// A zero key means the hotkey consists only of one or more modifiers.
    /// A single store/load means the hook thread can never observe a torn
    /// pair while settings are being saved (C14).
    static HOTKEY_PAIR: AtomicU32 = AtomicU32::new((MOD_ALT << 16) | 0x56); // Alt+V
    static SUPPRESSED_MODIFIERS: AtomicU32 = AtomicU32::new(0);

    /// Rate-limits the injected-target-key warning to one message per session.
    static INJECTED_TARGET_WARNED: AtomicBool = AtomicBool::new(false);

    /// Wall-clock deadline (epoch ms) until which injected target-key events
    /// are attributed to the hotkey aftermath. Layout switchers commit and
    /// retype a word a short moment after the chord, sometimes after the key
    /// itself was released, so the guard outlives `SHORTCUT_ACTIVE`.
    static INJECTED_GUARD_UNTIL_MS: AtomicU64 = AtomicU64::new(0);
    const INJECTED_GUARD_WINDOW_MS: u64 = 1_500;

    fn now_ms() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0)
    }

    /// Decides whether an injected key event must be swallowed.
    ///
    /// Aura's own injections are tagged and always pass. A third-party tool
    /// re-sending the hotkey's own letter shortly after the chord is a layout
    /// switcher / remapper replaying the buffered hotkey key, and letting it
    /// through types stray characters into the focused field. A modified
    /// `Ctrl+V` (a real paste) is never suppressed.
    fn should_suppress_foreign_injected_target(
        is_own_injection: bool,
        is_target_key: bool,
        guard_active: bool,
        ctrl_physically_down: bool,
    ) -> bool {
        !is_own_injection && is_target_key && guard_active && !ctrl_physically_down
    }

    /// Detects a `SendInput` Unicode injection.
    ///
    /// Windows delivers `KEYEVENTF_UNICODE` input to a low-level hook as
    /// `vkCode = VK_PACKET` with the UTF-16 code unit in `scanCode`; the
    /// `KBDLLHOOKSTRUCT.flags` field has no `KEYEVENTF_UNICODE` bit. The flag
    /// is still checked as a fallback for tools that set it anyway.
    fn is_unicode_injection(vk_code: u32, flags: u32) -> bool {
        vk_code == VK_PACKET || (flags & KEYEVENTF_UNICODE) != 0
    }

    /// US-layout rows and their ЙЦУКЕН counterparts. Layout switchers replay
    /// buffered text either as virtual keys (VK) or as Unicode code units, so
    /// the hook has to recognize the hotkey's letter in both layouts.
    const EN_ROW_TOP: &str = "qwertyuiop[]";
    const RU_ROW_TOP: &str = "йцукенгшщзхъ";
    const EN_ROW_HOME: &str = "asdfghjkl;'";
    const RU_ROW_HOME: &str = "фывапролджэ";
    const EN_ROW_BOTTOM: &str = "zxcvbnm,./";
    const RU_ROW_BOTTOM: &str = "ячсмитьбю.";

    fn russian_counterpart(ascii: char) -> Option<char> {
        let lower = ascii.to_ascii_lowercase();
        for (en, ru) in [
            (EN_ROW_TOP, RU_ROW_TOP),
            (EN_ROW_HOME, RU_ROW_HOME),
            (EN_ROW_BOTTOM, RU_ROW_BOTTOM),
        ] {
            if let Some(index) = en.find(lower) {
                return ru.chars().nth(index);
            }
        }
        (lower == '`').then_some('ё')
    }

    /// Maps a virtual-key code to the ASCII character it produces on a US
    /// layout. Only the keys `parse_hotkey` can bind are covered.
    fn ascii_for_vk(vk: u32) -> Option<char> {
        match vk {
            0x30..=0x39 => char::from_u32(vk),
            0x41..=0x5A => char::from_u32(vk).map(|c| c.to_ascii_lowercase()),
            _ => None,
        }
    }

    /// True when an injected Unicode code unit spells the hotkey's own letter
    /// in either the English or the Russian layout.
    fn is_hotkey_character(code_unit: u32, key_vk: u32) -> bool {
        let Some(ch) = char::from_u32(code_unit) else {
            return false;
        };
        let Some(ascii) = ascii_for_vk(key_vk) else {
            return false;
        };
        if ch.eq_ignore_ascii_case(&ascii) {
            return true;
        }
        russian_counterpart(ascii).is_some_and(|ru| ch.to_lowercase().eq(ru.to_lowercase()))
    }

    /// Packed (modifier_mask << 16) | key_vk waiting to be applied once an
    /// in-flight recording releases; u32::MAX means "nothing pending".
    static PENDING_HOTKEY: AtomicU32 = AtomicU32::new(u32::MAX);

    const VK_ESCAPE: u32 = 0x1B;

    fn keyboard_input(vk: u16, scan: u16, flags: u32) -> INPUT {
        let mut input: INPUT = unsafe { std::mem::zeroed() };
        input.r#type = INPUT_KEYBOARD;
        input.Anonymous.ki = KEYBDINPUT {
            wVk: vk,
            wScan: scan,
            dwFlags: flags,
            time: 0,
            dwExtraInfo: AURA_INJECT_TAG,
        };
        input
    }

    fn send_dummy_ctrl_tap() {
        let mut inputs = [
            keyboard_input(VK_CONTROL, 0, 0),
            keyboard_input(VK_CONTROL, 0, KEYEVENTF_KEYUP),
        ];
        unsafe {
            SendInput(
                inputs.len() as u32,
                inputs.as_mut_ptr(),
                std::mem::size_of::<INPUT>() as i32,
            );
        }
    }

    fn send_disarmed_modifier_up(kbd: &KBDLLHOOKSTRUCT) {
        let ext = if (kbd.flags & 0x01) != 0 {
            KEYEVENTF_EXTENDEDKEY
        } else {
            0
        };
        let mut inputs = [
            keyboard_input(VK_CONTROL, 0, 0),
            keyboard_input(VK_CONTROL, 0, KEYEVENTF_KEYUP),
            keyboard_input(
                kbd.vkCode as u16,
                kbd.scanCode as u16,
                KEYEVENTF_KEYUP | ext,
            ),
        ];
        unsafe {
            SendInput(
                inputs.len() as u32,
                inputs.as_mut_ptr(),
                std::mem::size_of::<INPUT>() as i32,
            );
        }
    }

    fn modifier_bit(vk_code: u32) -> Option<u32> {
        match vk_code {
            17 | 162 | 163 => Some(MOD_CTRL),  // Ctrl / LCtrl / RCtrl
            18 | 164 | 165 => Some(MOD_ALT),   // Alt / LAlt / RAlt
            16 | 160 | 161 => Some(MOD_SHIFT), // Shift / LShift / RShift
            91 | 92 => Some(MOD_WIN),          // LWin / RWin
            _ => None,
        }
    }

    fn modifier_is_down(modifier: u32) -> bool {
        let key_is_down = |vk: i32| {
            let state = unsafe { GetAsyncKeyState(vk) };
            (state as u16 & 0x8000) != 0
        };
        match modifier {
            MOD_CTRL => key_is_down(17),
            MOD_ALT => key_is_down(18),
            MOD_SHIFT => key_is_down(16),
            MOD_WIN => key_is_down(91) || key_is_down(92),
            _ => false,
        }
    }

    fn modifiers_satisfied_state<F>(
        mask: u32,
        event_vk: u32,
        event_is_down: bool,
        mut is_mod_down: F,
    ) -> bool
    where
        F: FnMut(u32) -> bool,
    {
        [MOD_CTRL, MOD_ALT, MOD_SHIFT, MOD_WIN]
            .into_iter()
            .all(|modifier| {
                let is_down = if modifier_bit(event_vk) == Some(modifier) {
                    event_is_down
                } else {
                    is_mod_down(modifier)
                };
                if mask & modifier != 0 {
                    is_down
                } else {
                    !is_down
                }
            })
    }

    fn modifiers_satisfied(mask: u32, event_vk: u32, event_is_down: bool) -> bool {
        modifiers_satisfied_state(mask, event_vk, event_is_down, modifier_is_down)
    }

    /// Decides whether the target key is still physically held when a modifier
    /// is released. A key-down swallowed by this hook (`return 1`) never lands
    /// in the Win32 asynchronous key state, so `GetAsyncKeyState` reads "up"
    /// while the user still holds the key; once the target key-down has been
    /// intercepted, the `KEY_SUPPRESSED` latch is therefore authoritative.
    fn target_key_still_held(key_suppressed: bool, async_state_down: bool) -> bool {
        key_suppressed || async_state_down
    }

    /// Decides whether a target-key key-down must be swallowed. Only an
    /// explicit hotkey chord or a hold already in progress qualifies.
    /// `RECORDING_ACTIVE` must never be added here: in toggle mode the
    /// recording outlives the chord, and plain typing of the target key would
    /// be swallowed and would stop the session on its key-up.
    fn should_suppress_target_down(modifier_satisfied: bool, shortcut_active: bool) -> bool {
        modifier_satisfied || shortcut_active
    }

    fn is_any_modifier_key(vk_code: u32) -> bool {
        matches!(
            vk_code,
            16 | 17 | 18 | 160 | 161 | 162 | 163 | 164 | 165 | 91 | 92
        )
    }

    fn parse_hotkey(hotkey_str: &str) -> Option<(u32, u32)> {
        let mut modifier_mask = 0;
        let mut key = None;

        for part in hotkey_str.split('+') {
            let clean = part.trim().to_lowercase();
            match clean.as_str() {
                "alt" => {
                    if modifier_mask & MOD_ALT != 0 {
                        return None;
                    }
                    modifier_mask |= MOD_ALT;
                }
                "ctrl" | "control" => {
                    if modifier_mask & MOD_CTRL != 0 {
                        return None;
                    }
                    modifier_mask |= MOD_CTRL;
                }
                "shift" => {
                    if modifier_mask & MOD_SHIFT != 0 {
                        return None;
                    }
                    modifier_mask |= MOD_SHIFT;
                }
                "win" | "windows" | "super" | "meta" => {
                    if modifier_mask & MOD_WIN != 0 {
                        return None;
                    }
                    modifier_mask |= MOD_WIN;
                }
                other => {
                    let parsed = if let [byte] = other.as_bytes() {
                        byte.is_ascii_alphanumeric()
                            .then(|| byte.to_ascii_uppercase() as u32)
                    } else {
                        match other {
                            "space" | "пробел" => Some(0x20),
                            "capslock" | "caps lock" => Some(0x14),
                            "tab" => Some(0x09),
                            "f1" => Some(0x70),
                            "f2" => Some(0x71),
                            "f3" => Some(0x72),
                            "f4" => Some(0x73),
                            "f5" => Some(0x74),
                            "f6" => Some(0x75),
                            "f7" => Some(0x76),
                            "f8" => Some(0x77),
                            "f9" => Some(0x78),
                            "f10" => Some(0x79),
                            "f11" => Some(0x7A),
                            "f12" => Some(0x7B),
                            _ => None,
                        }
                    }?;
                    if key.replace(parsed).is_some() {
                        return None;
                    }
                }
            }
        }

        if key.is_none() && modifier_mask.count_ones() < 2 {
            None
        } else {
            Some((modifier_mask, key.unwrap_or(0)))
        }
    }

    pub fn validate_hotkey(hotkey_str: &str) -> Result<(), String> {
        parse_hotkey(hotkey_str)
            .map(|_| ())
            .ok_or_else(|| format!("Unsupported hotkey: {hotkey_str}"))
    }

    pub fn update_hotkey(hotkey_str: &str) -> Result<(), String> {
        let (modifier_mask, key) =
            parse_hotkey(hotkey_str).ok_or_else(|| format!("Unsupported hotkey: {hotkey_str}"))?;
        let packed = (modifier_mask << 16) | key;

        if SHORTCUT_ACTIVE.load(Ordering::SeqCst) || KEY_SUPPRESSED.load(Ordering::SeqCst) {
            // A live session still holds the old key: resetting SHORTCUT_ACTIVE
            // here would orphan it, because its eventual key-up no longer
            // matches the new pair and the session could never end normally.
            // Defer the switch until that release finalizes the session.
            PENDING_HOTKEY.store(packed, Ordering::SeqCst);
            crate::logger::log(
                "INFO",
                "Hotkey",
                None,
                "Recording or key-hold is active; hotkey change deferred until the current hotkey is released",
            );
            return Ok(());
        }

        HOTKEY_PAIR.store(packed, Ordering::SeqCst);
        PENDING_HOTKEY.store(u32::MAX, Ordering::SeqCst);
        SUPPRESSED_MODIFIERS.store(0, Ordering::SeqCst);
        SHORTCUT_ACTIVE.store(false, Ordering::SeqCst);
        KEY_SUPPRESSED.store(false, Ordering::SeqCst);
        Ok(())
    }

    /// Applies a hotkey change that was deferred because a recording session
    /// was mid-flight when settings were saved.
    fn apply_pending_hotkey() {
        let packed = PENDING_HOTKEY.swap(u32::MAX, Ordering::SeqCst);
        if packed != u32::MAX {
            HOTKEY_PAIR.store(packed, Ordering::SeqCst);
            SUPPRESSED_MODIFIERS.store(0, Ordering::SeqCst);
            KEY_SUPPRESSED.store(false, Ordering::SeqCst);
            crate::logger::log(
                "INFO",
                "Hotkey",
                None,
                "Deferred hotkey change applied after the recording ended",
            );
        }
    }

    pub(super) fn clear_phantom_hotkey_state() {
        SUPPRESSED_MODIFIERS.store(0, Ordering::SeqCst);
        SHORTCUT_ACTIVE.store(false, Ordering::SeqCst);
        KEY_SUPPRESSED.store(false, Ordering::SeqCst);
        apply_pending_hotkey();
        crate::logger::log(
            "INFO",
            "Hotkey",
            None,
            "Session lock/unlock or hook pause detected; stale hotkey state cleared",
        );
    }

    unsafe extern "system" fn session_wnd_proc(
        hwnd: HWND,
        msg: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        if msg == WM_WTSSESSION_CHANGE
            && (wparam == WTS_SESSION_LOCK as WPARAM || wparam == WTS_SESSION_UNLOCK as WPARAM)
        {
            clear_phantom_hotkey_state();
        }
        DefWindowProcW(hwnd, msg, wparam, lparam)
    }

    unsafe fn create_session_notification_window() -> Option<HWND> {
        let class_name: Vec<u16> = "AuraSessionNotification"
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect();
        let module = GetModuleHandleW(std::ptr::null());
        let mut window_class: WNDCLASSW = std::mem::zeroed();
        window_class.lpfnWndProc = Some(session_wnd_proc);
        window_class.hInstance = module;
        window_class.lpszClassName = class_name.as_ptr();
        RegisterClassW(&window_class);
        let hwnd = CreateWindowExW(
            0,
            class_name.as_ptr(),
            class_name.as_ptr(),
            0,
            0,
            0,
            0,
            0,
            HWND_MESSAGE,
            0,
            module,
            std::ptr::null(),
        );
        if hwnd == 0 {
            crate::logger::log(
                "ERROR",
                "Hotkey",
                None,
                "Failed to create session-notification window; lock/unlock recovery is disabled",
            );
            return None;
        }
        if WTSRegisterSessionNotification(hwnd, NOTIFY_FOR_THIS_SESSION) == 0 {
            crate::logger::log(
                "ERROR",
                "Hotkey",
                None,
                "WTSRegisterSessionNotification failed; lock/unlock recovery is disabled",
            );
        }
        Some(hwnd)
    }

    fn notify_hotkey(is_down: bool) {
        // Any hotkey edge re-arms the window in which third-party injected
        // target-key events are treated as replay of the hotkey itself.
        INJECTED_GUARD_UNTIL_MS.store(
            now_ms().saturating_add(INJECTED_GUARD_WINDOW_MS),
            Ordering::Relaxed,
        );
        invoke_bool_callback(&CALLBACK, is_down, || {
            crate::logger::log(
                "ERROR",
                "Hotkey",
                None,
                "Hotkey callback panicked; the panic was contained at the WinAPI boundary",
            )
        });
    }

    fn notify_cancel() {
        invoke_fn_callback(&CANCEL_CALLBACK, || {
            crate::logger::log(
                "ERROR",
                "Hotkey",
                None,
                "Cancel callback panicked; the panic was contained at the WinAPI boundary",
            )
        });
    }

    pub fn start_hook<F>(callback: F) -> Result<(), String>
    where
        F: Fn(bool) + Send + Sync + 'static,
    {
        {
            let mut guard = CALLBACK
                .lock()
                .map_err(|_| "Hook callback registry is poisoned".to_string())?;
            *guard = Some(Box::new(callback));
        }

        let (ready_tx, ready_rx) = std::sync::mpsc::sync_channel(1);
        std::thread::Builder::new()
            .name("aura-keyboard-hook".to_string())
            .spawn(move || unsafe {
                let session_window = create_session_notification_window();

                let hook = SetWindowsHookExW(WH_KEYBOARD_LL, Some(low_level_keyboard_proc), 0, 0);

                if hook == 0 {
                    if let Some(hwnd) = session_window {
                        WTSUnRegisterSessionNotification(hwnd);
                        DestroyWindow(hwnd);
                    }
                    let _ = ready_tx.send(Err(format!(
                        "Failed to install global keyboard hook: {}",
                        std::io::Error::last_os_error()
                    )));
                    return;
                }
                if ready_tx.send(Ok(())).is_err() {
                    UnhookWindowsHookEx(hook);
                    return;
                }

                let mut msg: MSG = std::mem::zeroed();
                loop {
                    let result = GetMessageW(&mut msg, 0, 0, 0);
                    if result == -1 {
                        crate::logger::log(
                            "ERROR",
                            "Hotkey",
                            None,
                            &format!(
                                "Keyboard message loop failed: {}",
                                std::io::Error::last_os_error()
                            ),
                        );
                        break;
                    }
                    if result == 0 {
                        break;
                    }
                    TranslateMessage(&msg);
                    DispatchMessageW(&msg);
                }

                UnhookWindowsHookEx(hook);

                if let Some(hwnd) = session_window {
                    WTSUnRegisterSessionNotification(hwnd);
                    DestroyWindow(hwnd);
                }
            })
            .map_err(|error| format!("Failed to spawn keyboard hook thread: {error}"))?;

        ready_rx
            .recv_timeout(std::time::Duration::from_secs(5))
            .map_err(|error| format!("Keyboard hook initialization timed out: {error}"))?
    }

    unsafe extern "system" fn low_level_keyboard_proc(
        code: i32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        if code >= 0 && lparam != 0 {
            if HOOK_PAUSED.load(Ordering::Relaxed) {
                return CallNextHookEx(0, code, wparam, lparam);
            }

            let kbd_struct = *(lparam as *const KBDLLHOOKSTRUCT);
            let vk_code = kbd_struct.vkCode;
            let is_down = wparam == WM_KEYDOWN as usize || wparam == WM_SYSKEYDOWN as usize;
            let is_injected = (kbd_struct.flags & 0x10) != 0;
            if is_injected {
                let packed_hotkey = HOTKEY_PAIR.load(Ordering::Relaxed);
                let key_vk = packed_hotkey & 0xFFFF;
                let is_own_injection = kbd_struct.dwExtraInfo == AURA_INJECT_TAG;
                let guard_active = SHORTCUT_ACTIVE.load(Ordering::Relaxed)
                    || now_ms() < INJECTED_GUARD_UNTIL_MS.load(Ordering::Relaxed);
                let is_target_event = key_vk != 0
                    && (vk_code == key_vk
                        || (is_unicode_injection(vk_code, kbd_struct.flags)
                            && is_hotkey_character(kbd_struct.scanCode, key_vk)));

                if should_suppress_foreign_injected_target(
                    is_own_injection,
                    is_target_event,
                    guard_active,
                    modifier_is_down(MOD_CTRL),
                ) {
                    if !INJECTED_TARGET_WARNED.swap(true, Ordering::Relaxed) {
                        crate::logger::log(
                            "WARN",
                            "Hotkey",
                            None,
                            &format!(
                                "Suppressed injected target-key {} from a third-party input tool \
                                 (vk=0x{vk_code:02X}, flags=0x{:02X}); a layout switcher or key \
                                 remapper replayed the hotkey key into the focused field",
                                if is_down { "key-down" } else { "key-up" },
                                kbd_struct.flags,
                            ),
                        );
                    }
                    return 1;
                }

                if !is_own_injection
                    && is_target_event
                    && SHORTCUT_ACTIVE.load(Ordering::Relaxed)
                    && !INJECTED_TARGET_WARNED.swap(true, Ordering::Relaxed)
                {
                    crate::logger::log(
                        "WARN",
                        "Hotkey",
                        None,
                        "Injected target key event detected while hotkey session active (possible third-party keyboard utility conflict)",
                    );
                }
                return CallNextHookEx(0, code, wparam, lparam);
            }

            let is_up = wparam == WM_KEYUP as usize || wparam == WM_SYSKEYUP as usize;

            if vk_code == VK_ESCAPE && is_down && RECORDING_ACTIVE.load(Ordering::SeqCst) {
                notify_cancel();
                return 1; // Suppress Esc
            }

            let packed_hotkey = HOTKEY_PAIR.load(Ordering::SeqCst);
            let modifier_mask = packed_hotkey >> 16;
            let key_vk = packed_hotkey & 0xFFFF;

            let event_modifier = modifier_bit(vk_code);
            let is_required_modifier = event_modifier
                .map(|modifier| modifier_mask & modifier != 0)
                .unwrap_or(false);
            let is_target_key = key_vk != 0 && vk_code == key_vk;

            if key_vk == 0 && is_required_modifier {
                let Some(event_modifier) = event_modifier else {
                    return CallNextHookEx(0, code, wparam, lparam);
                };
                if is_down {
                    let already_active = SHORTCUT_ACTIVE.load(Ordering::SeqCst);
                    if !already_active && modifiers_satisfied(modifier_mask, vk_code, true) {
                        SUPPRESSED_MODIFIERS.fetch_or(event_modifier, Ordering::SeqCst);
                        SHORTCUT_ACTIVE.store(true, Ordering::SeqCst);
                        notify_hotkey(true);
                    }
                    if SUPPRESSED_MODIFIERS.load(Ordering::SeqCst) & event_modifier != 0 {
                        return 1;
                    }
                } else if is_up {
                    let suppressed_modifiers = SUPPRESSED_MODIFIERS.load(Ordering::SeqCst);
                    let was_suppressed = SUPPRESSED_MODIFIERS
                        .fetch_and(!event_modifier, Ordering::SeqCst)
                        & event_modifier
                        != 0;
                    let was_active = SHORTCUT_ACTIVE.swap(false, Ordering::SeqCst);
                    let mut suppress_event = was_suppressed;
                    if was_active {
                        // Prevent a visible Alt or Win prefix from becoming a
                        // standalone OS action when its partner was suppressed.
                        let ctrl_was_visible =
                            modifier_mask & MOD_CTRL != 0 && suppressed_modifiers & MOD_CTRL == 0;
                        let visible_os_modifiers =
                            modifier_mask & (MOD_ALT | MOD_WIN) & !suppressed_modifiers;
                        if !ctrl_was_visible && visible_os_modifiers != 0 {
                            if visible_os_modifiers & event_modifier != 0 {
                                send_disarmed_modifier_up(&kbd_struct);
                                suppress_event = true;
                            } else {
                                send_dummy_ctrl_tap();
                            }
                        }
                        notify_hotkey(false);
                        apply_pending_hotkey();
                    }
                    if suppress_event {
                        return 1;
                    }
                }
            } else if is_required_modifier {
                // Releasing the *modifier* while the target key is still
                // physically held must not finalize the session: the release
                // order "Alt up, V still down" is a natural way to end a
                // press, and treating it as the session end would cut the
                // dictation short and let V-repeat leak through as stray
                // keystrokes. Defer the release until the target key's own
                // key-up clears the shortcut state; disarming an Alt/Win OS
                // action still needs to happen while that modifier is released.
                if is_up {
                    let target_still_held =
                        target_key_still_held(KEY_SUPPRESSED.load(Ordering::SeqCst), {
                            let state = GetAsyncKeyState(key_vk as i32);
                            (state as u16 & 0x8000) != 0
                        });
                    if target_still_held {
                        crate::logger::log(
                            "INFO",
                            "Hotkey",
                            None,
                            "Modifier released while target key still held; deferring session finalization until target key release",
                        );
                        if matches!(event_modifier, Some(MOD_ALT) | Some(MOD_WIN))
                            && modifier_mask & MOD_CTRL == 0
                        {
                            send_disarmed_modifier_up(&kbd_struct);
                            return 1; // Suppress the physical modifier up we replaced
                        }
                    } else if SHORTCUT_ACTIVE.swap(false, Ordering::SeqCst) {
                        notify_hotkey(false);
                        apply_pending_hotkey();
                        if matches!(event_modifier, Some(MOD_ALT) | Some(MOD_WIN))
                            && modifier_mask & MOD_CTRL == 0
                        {
                            send_disarmed_modifier_up(&kbd_struct);
                            return 1;
                        }
                    }
                }
            } else if is_target_key {
                let modifier_satisfied = modifiers_satisfied(modifier_mask, vk_code, is_down);

                if is_down {
                    if should_suppress_target_down(
                        modifier_satisfied,
                        SHORTCUT_ACTIVE.load(Ordering::SeqCst),
                    ) {
                        KEY_SUPPRESSED.store(true, Ordering::SeqCst);
                        if !SHORTCUT_ACTIVE.swap(true, Ordering::SeqCst) {
                            INJECTED_TARGET_WARNED.store(false, Ordering::Relaxed);
                            notify_hotkey(true);
                        }
                        return 1;
                    } else {
                        KEY_SUPPRESSED.store(false, Ordering::SeqCst);
                    }
                } else if is_up {
                    let suppressed = KEY_SUPPRESSED.swap(false, Ordering::SeqCst);
                    let was_active = SHORTCUT_ACTIVE.swap(false, Ordering::SeqCst);
                    if was_active {
                        notify_hotkey(false);
                        apply_pending_hotkey();
                        if modifier_mask & (MOD_ALT | MOD_WIN) != 0
                            && modifier_mask & MOD_CTRL == 0
                            && modifier_satisfied
                        {
                            send_dummy_ctrl_tap();
                        }
                    }
                    if suppressed || modifier_satisfied || was_active {
                        return 1;
                    }
                }
            }

            // Actual Aura hotkey events returned above. Any remaining physical
            // non-modifier key can change the target editor (typing, Backspace,
            // Ctrl+Z, navigation, etc.). Injected SendInput events returned at
            // the top of this hook and therefore never desynchronize Aura.
            if is_down && !is_any_modifier_key(vk_code) {
                notify_user_input();
            }
        }
        CallNextHookEx(0, code, wparam, lparam)
    }

    #[cfg(test)]
    mod tests {
        use super::{
            clear_phantom_hotkey_state, is_any_modifier_key, parse_hotkey, KEY_SUPPRESSED, MOD_ALT,
            MOD_CTRL, MOD_SHIFT, MOD_WIN, SHORTCUT_ACTIVE, SUPPRESSED_MODIFIERS,
        };

        #[test]
        fn test_parse_hotkey_combinations() {
            assert_eq!(parse_hotkey("Alt+V"), Some((MOD_ALT, 0x56)));
            assert_eq!(parse_hotkey("Ctrl+Space"), Some((MOD_CTRL, 0x20)));
            assert_eq!(parse_hotkey("F8"), Some((0, 0x77)));
            assert_eq!(parse_hotkey("F12"), Some((0, 0x7B)));
            assert_eq!(parse_hotkey("Caps Lock"), Some((0, 0x14)));
            assert_eq!(parse_hotkey("Shift+Tab"), Some((MOD_SHIFT, 0x09)));
            assert_eq!(parse_hotkey("Win+V"), Some((MOD_WIN, 0x56)));
            assert_eq!(parse_hotkey("Ctrl+Win"), Some((MOD_CTRL | MOD_WIN, 0)));
            assert_eq!(parse_hotkey("Alt+Super"), Some((MOD_ALT | MOD_WIN, 0)));
            assert_eq!(
                parse_hotkey("Ctrl+Alt+Shift+Meta+V"),
                Some((MOD_CTRL | MOD_ALT | MOD_SHIFT | MOD_WIN, 0x56))
            );
        }

        #[test]
        fn test_parse_hotkey_invalid() {
            assert_eq!(parse_hotkey(""), None);
            assert_eq!(parse_hotkey("Alt"), None);
            assert_eq!(parse_hotkey("Win"), None);
            assert_eq!(parse_hotkey("Alt+Unknown"), None);
            assert_eq!(parse_hotkey("Alt+V+Unknown"), None);
            assert_eq!(parse_hotkey("Ctrl+Control"), None);
            assert_eq!(parse_hotkey("Win+Super"), None);
        }

        #[test]
        fn user_edit_detection_ignores_modifier_only_events() {
            assert!(is_any_modifier_key(0x10)); // Shift
            assert!(is_any_modifier_key(0x11)); // Ctrl
            assert!(is_any_modifier_key(0x12)); // Alt
            assert!(is_any_modifier_key(0x5B)); // Left Win
            assert!(!is_any_modifier_key(0x5A)); // Z in Ctrl+Z
            assert!(!is_any_modifier_key(0x08)); // Backspace
        }

        #[test]
        fn unicode_injection_is_detected_by_vk_packet() {
            use super::{is_unicode_injection, KEYEVENTF_UNICODE, VK_PACKET};

            // Windows reports `KEYEVENTF_UNICODE` as VK_PACKET, flags carry
            // only LLKHF_* bits (injected = 0x10).
            assert!(is_unicode_injection(VK_PACKET, 0x10));
            assert!(is_unicode_injection(VK_PACKET, 0));
            // Defensive fallback for third-party tools that set 0x04 anyway.
            assert!(is_unicode_injection(0x56, KEYEVENTF_UNICODE));
            // A plain injected V or Cyrillic M key is not VK_PACKET.
            assert!(!is_unicode_injection(0x56, 0x10));
            assert!(!is_unicode_injection(0x4D, 0x10));
        }

        #[test]
        fn test_strict_modifier_mask_satisfaction() {
            use super::modifiers_satisfied_state;

            // Target hotkey: Alt+V (mask = MOD_ALT, event_vk = 0x56)
            let alt_mask = MOD_ALT;
            let v_key = 0x56;

            // Scenario 1: Only Alt is down -> satisfies
            assert!(modifiers_satisfied_state(alt_mask, v_key, true, |m| m == MOD_ALT));

            // Scenario 2: Alt is down, BUT Ctrl is also down (e.g. user pressed Ctrl+V while Alt stuck, or Ctrl+Alt+V) -> REJECTED!
            assert!(!modifiers_satisfied_state(alt_mask, v_key, true, |m| m
                == MOD_ALT
                || m == MOD_CTRL));

            // Scenario 3: Alt is down, but Shift is also down -> REJECTED!
            assert!(!modifiers_satisfied_state(alt_mask, v_key, true, |m| m
                == MOD_ALT
                || m == MOD_SHIFT));

            // Scenario 4: Alt is down, but Win is also down -> REJECTED!
            assert!(!modifiers_satisfied_state(alt_mask, v_key, true, |m| m
                == MOD_ALT
                || m == MOD_WIN));

            // Scenario 5: Alt is not down -> REJECTED!
            assert!(!modifiers_satisfied_state(alt_mask, v_key, true, |_| false));

            // Multi-modifier hotkey: Ctrl+Alt+V (mask = MOD_CTRL | MOD_ALT)
            let ctrl_alt_mask = MOD_CTRL | MOD_ALT;
            assert!(modifiers_satisfied_state(ctrl_alt_mask, v_key, true, |m| m
                == MOD_CTRL
                || m == MOD_ALT));
            // Fails if Shift also pressed
            assert!(!modifiers_satisfied_state(
                ctrl_alt_mask,
                v_key,
                true,
                |m| m == MOD_CTRL || m == MOD_ALT || m == MOD_SHIFT
            ));
        }

        #[test]
        fn session_change_clears_phantom_hotkey_state() {
            use std::sync::atomic::Ordering;

            SUPPRESSED_MODIFIERS.store(MOD_ALT | MOD_SHIFT, Ordering::SeqCst);
            SHORTCUT_ACTIVE.store(true, Ordering::SeqCst);
            KEY_SUPPRESSED.store(true, Ordering::SeqCst);

            clear_phantom_hotkey_state();

            assert_eq!(SUPPRESSED_MODIFIERS.load(Ordering::SeqCst), 0);
            assert!(!SHORTCUT_ACTIVE.load(Ordering::SeqCst));
            assert!(!KEY_SUPPRESSED.load(Ordering::SeqCst));
        }

        #[test]
        fn test_alt_released_first_defers_finalization_via_latch() {
            use super::target_key_still_held;

            // Regression: a key-down swallowed by the hook never reaches the
            // Win32 async key state, so GetAsyncKeyState reads "up" while the
            // user still holds the key. The latch must win in that case.
            assert!(
                target_key_still_held(true, false),
                "The KEY_SUPPRESSED latch must defer finalization while the key is held"
            );
            assert!(
                target_key_still_held(false, true),
                "The async state is authoritative when the latch is not armed"
            );
            assert!(
                !target_key_still_held(false, false),
                "Neither latch nor async state means the key is up"
            );
        }

        #[test]
        fn foreign_injected_target_is_suppressed_only_in_hotkey_aftermath() {
            use super::should_suppress_foreign_injected_target;

            // Regression (Caramba Switcher): a layout switcher whose hook runs
            // before Aura's buffers the held hotkey letter, then replays it as
            // injected keystrokes. Aura must swallow those replays, otherwise
            // the document receives a burst of the hotkey letter.
            assert!(should_suppress_foreign_injected_target(
                false, true, true, false
            ));
            // Aura's own tagged paste must always pass.
            assert!(!should_suppress_foreign_injected_target(
                true, true, true, false
            ));
            // A real Ctrl+V injection (another tool pasting) must pass.
            assert!(!should_suppress_foreign_injected_target(
                false, true, true, true
            ));
            // Outside the hotkey window nothing is touched.
            assert!(!should_suppress_foreign_injected_target(
                false, true, false, false
            ));
            // Non-target keys are never filtered.
            assert!(!should_suppress_foreign_injected_target(
                false, false, true, false
            ));
        }

        #[test]
        fn hotkey_character_matches_both_layouts() {
            use super::is_hotkey_character;

            // A layout switcher may replay the buffered hotkey letter either as
            // VK_V or as a Unicode code unit. The latter cannot be identified by
            // virtual key, so the character must be recognized in EN and RU.
            for ch in ['v', 'V', 'м', 'М'] {
                assert!(
                    is_hotkey_character(ch as u32, 0x56),
                    "{ch} must be recognized as the V hotkey letter"
                );
            }
            assert!(!is_hotkey_character('d' as u32, 0x56));
            assert!(!is_hotkey_character('ы' as u32, 0x56));
            assert!(is_hotkey_character('1' as u32, 0x31));
            assert!(!is_hotkey_character('2' as u32, 0x31));
            // Function keys have no character and must never match a replay.
            assert!(!is_hotkey_character('v' as u32, 0x77));
        }

        #[test]
        fn test_toggle_mode_typing_v_is_not_suppressed() {
            use super::should_suppress_target_down;

            // Contract: only an explicit chord or an in-progress hold suppresses
            // the target key. A toggle-latched session (RECORDING_ACTIVE) and
            // plain typing must pass through, otherwise the letter is swallowed
            // and the session stops on its key-up.
            assert!(!should_suppress_target_down(false, false));
            assert!(should_suppress_target_down(true, false));
            assert!(should_suppress_target_down(false, true));
        }
    }
}

// ============================================================================
// MACOS IMPLEMENTATION
// ============================================================================
#[cfg(target_os = "macos")]
mod macos_impl {
    use super::*;
    use std::ffi::c_void;
    use std::sync::atomic::AtomicPtr;

    pub type CGEventTapProxy = *mut c_void;
    pub type CGEventRef = *mut c_void;
    pub type CGEventTapCallBack = unsafe extern "C" fn(
        proxy: CGEventTapProxy,
        type_: u32,
        event: CGEventRef,
        refcon: *mut c_void,
    ) -> CGEventRef;

    pub type CFRunLoopSourceRef = *mut c_void;
    pub type CFRunLoopSourceRef = *mut c_void;
    pub type CFRunLoopRef = *mut c_void;
    pub type CFStringRef = *mut c_void;

    #[link(name = "CoreGraphics", kind = "framework")]
    extern "C" {
        pub fn CGEventTapCreate(
            tap: u32,
            place: u32,
            options: u32,
            eventsOfInterest: u64,
            callback: CGEventTapCallBack,
            refcon: *mut c_void,
        ) -> CFMachPortRef;
        pub fn CGEventTapEnable(tap: CFMachPortRef, enable: bool);
        pub fn CGEventGetFlags(event: CGEventRef) -> u64;
        pub fn CGEventGetIntegerValueField(event: CGEventRef, field: u32) -> i64;
    }

    #[link(name = "CoreFoundation", kind = "framework")]
    extern "C" {
        pub fn CFMachPortCreateRunLoopSource(
            allocator: *mut c_void,
            port: CFMachPortRef,
            order: isize,
        ) -> CFRunLoopSourceRef;
        pub fn CFRunLoopGetCurrent() -> CFRunLoopRef;
        pub fn CFRunLoopAddSource(rl: CFRunLoopRef, source: CFRunLoopSourceRef, mode: CFStringRef);
        pub fn CFRunLoopRun();
        pub fn CFRelease(value: *const c_void);
        pub static kCFRunLoopCommonModes: CFStringRef;
    }

    pub const kCGSessionEventTap: u32 = 1;
    pub const kCGHeadInsertEventTap: u32 = 0;
    pub const kCGEventTapOptionDefault: u32 = 0;

    pub const kCGEventKeyDown: u32 = 10;
    pub const kCGEventKeyUp: u32 = 11;
    pub const kCGEventFlagsChanged: u32 = 12;
    pub const kCGEventTapDisabledByTimeout: u32 = u32::MAX - 1;
    pub const kCGEventTapDisabledByUserInput: u32 = u32::MAX;

    static EVENT_TAP: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());
    pub const kCGKeyboardEventKeycode: u32 = 9;

    pub const kCGEventFlagMaskAlternate: u64 = 0x00080000;
    pub const kCGEventFlagMaskControl: u64 = 0x00040000;
    pub const kCGEventFlagMaskShift: u64 = 0x00020000;
    pub const kCGEventFlagMaskCommand: u64 = 0x00100000;

    struct HotkeyConfig {
        modifier_mask: u64,
        key_code: Option<u32>,
    }

    static HOTKEY_CONFIG: Mutex<HotkeyConfig> = Mutex::new(HotkeyConfig {
        modifier_mask: kCGEventFlagMaskAlternate, // Option (Alt)
        key_code: Some(9),                        // V key
    });

    /// A hotkey change deferred because a recording was mid-flight when
    /// settings were saved; applied on the session's release.
    static PENDING_HOTKEY: Mutex<Option<(u64, u32)>> = Mutex::new(None);

    const VK_ESCAPE: u32 = 53;

    fn parse_hotkey(hotkey_str: &str) -> Option<(u64, Option<u32>)> {
        let mut modifier_mask = 0;
        let mut key = None;

        for part in hotkey_str.split('+') {
            let clean = part.trim().to_lowercase();
            match clean.as_str() {
                "alt" | "option" => modifier_mask |= kCGEventFlagMaskAlternate,
                "ctrl" | "control" => modifier_mask |= kCGEventFlagMaskControl,
                "shift" => modifier_mask |= kCGEventFlagMaskShift,
                "cmd" | "command" | "win" | "windows" | "super" | "meta" => {
                    modifier_mask |= kCGEventFlagMaskCommand
                }
                other => {
                    let parsed = if let [byte] = other.as_bytes() {
                        match byte.to_ascii_uppercase() {
                            b'A' => Some(0),
                            b'B' => Some(11),
                            b'C' => Some(8),
                            b'D' => Some(2),
                            b'E' => Some(14),
                            b'F' => Some(3),
                            b'G' => Some(5),
                            b'H' => Some(4),
                            b'I' => Some(34),
                            b'J' => Some(38),
                            b'K' => Some(40),
                            b'L' => Some(37),
                            b'M' => Some(46),
                            b'N' => Some(45),
                            b'O' => Some(31),
                            b'P' => Some(35),
                            b'Q' => Some(12),
                            b'R' => Some(15),
                            b'S' => Some(1),
                            b'T' => Some(17),
                            b'U' => Some(32),
                            b'V' => Some(9),
                            b'W' => Some(13),
                            b'X' => Some(7),
                            b'Y' => Some(16),
                            b'Z' => Some(6),
                            _ => None,
                        }
                    } else {
                        match other {
                            "space" => Some(49),
                            "capslock" => Some(57),
                            "tab" => Some(48),
                            "f1" => Some(122),
                            "f2" => Some(120),
                            "f3" => Some(99),
                            "f4" => Some(118),
                            "f5" => Some(96),
                            "f6" => Some(97),
                            "f7" => Some(98),
                            "f8" => Some(100),
                            "f9" => Some(101),
                            "f10" => Some(109),
                            "f11" => Some(103),
                            "f12" => Some(111),
                            _ => None,
                        }
                    }?;
                    if key.replace(parsed).is_some() {
                        return None;
                    }
                }
            }
        }

        if key.is_none() && modifier_mask.count_ones() < 2 {
            None
        } else {
            Some((modifier_mask, key))
        }
    }

    pub fn validate_hotkey(hotkey_str: &str) -> Result<(), String> {
        parse_hotkey(hotkey_str)
            .map(|_| ())
            .ok_or_else(|| format!("Unsupported hotkey: {hotkey_str}"))
    }

    pub fn update_hotkey(hotkey_str: &str) -> Result<(), String> {
        let (modifier, key) =
            parse_hotkey(hotkey_str).ok_or_else(|| format!("Unsupported hotkey: {hotkey_str}"))?;

        if SHORTCUT_ACTIVE.load(Ordering::SeqCst) || KEY_SUPPRESSED.load(Ordering::SeqCst) {
            // A live session still holds the old key; defer to avoid orphaning
            // it (its key-up would no longer match the new configuration).
            match PENDING_HOTKEY.lock() {
                Ok(mut guard) => *guard = Some((modifier, key)),
                Err(poisoned) => {
                    crate::logger::log(
                        "ERROR",
                        "Hotkey",
                        None,
                        "Recovering poisoned macOS pending-hotkey mutex",
                    );
                    *poisoned.into_inner() = Some((modifier, key));
                }
            }
            crate::logger::log(
                "INFO",
                "Hotkey",
                None,
                "Recording is active; hotkey change deferred until the current hotkey is released",
            );
            return Ok(());
        }

        let mut guard = match HOTKEY_CONFIG.lock() {
            Ok(guard) => guard,
            Err(poisoned) => {
                crate::logger::log(
                    "ERROR",
                    "Hotkey",
                    None,
                    "Recovering poisoned macOS hotkey mutex",
                );
                poisoned.into_inner()
            }
        };
        guard.modifier_mask = modifier;
        guard.key_code = key;
        drop(guard);
        if let Ok(mut pending) = PENDING_HOTKEY.lock() {
            *pending = None;
        }
        KEY_SUPPRESSED.store(false, Ordering::SeqCst);
        Ok(())
    }

    fn apply_pending_hotkey() {
        let pending = match PENDING_HOTKEY.lock() {
            Ok(mut guard) => guard.take(),
            Err(poisoned) => poisoned.into_inner().take(),
        };
        if let Some((modifier, key)) = pending {
            match HOTKEY_CONFIG.lock() {
                Ok(mut guard) => {
                    guard.modifier_mask = modifier;
                    guard.key_code = key;
                }
                Err(poisoned) => {
                    crate::logger::log(
                        "ERROR",
                        "Hotkey",
                        None,
                        "Recovering poisoned macOS hotkey mutex",
                    );
                    let mut guard = poisoned.into_inner();
                    guard.modifier_mask = modifier;
                    guard.key_code = key;
                }
            }
            KEY_SUPPRESSED.store(false, Ordering::SeqCst);
            crate::logger::log(
                "INFO",
                "Hotkey",
                None,
                "Deferred hotkey change applied after the recording ended",
            );
        }
    }

    pub(super) fn clear_phantom_hotkey_state() {
        SHORTCUT_ACTIVE.store(false, Ordering::SeqCst);
        KEY_SUPPRESSED.store(false, Ordering::SeqCst);
        apply_pending_hotkey();
        crate::logger::log(
            "INFO",
            "Hotkey",
            None,
            "Session lock/unlock or hook pause detected; stale macOS hotkey state cleared",
        );
    }

    pub fn start_hook<F>(callback: F) -> Result<(), String>
    where
        F: Fn(bool) + Send + Sync + 'static,
    {
        {
            let mut guard = CALLBACK
                .lock()
                .map_err(|_| "Hook callback registry is poisoned".to_string())?;
            *guard = Some(Box::new(callback));
        }

        let (ready_tx, ready_rx) = std::sync::mpsc::sync_channel(1);
        std::thread::Builder::new()
            .name("aura-macos-keyboard-hook".to_string())
            .spawn(move || unsafe {
                let event_mask = (1u64 << kCGEventKeyDown)
                    | (1u64 << kCGEventKeyUp)
                    | (1u64 << kCGEventFlagsChanged);

                let tap = CGEventTapCreate(
                    kCGSessionEventTap,
                    kCGHeadInsertEventTap,
                    kCGEventTapOptionDefault,
                    event_mask,
                    macos_event_tap_callback,
                    std::ptr::null_mut(),
                );
                if tap.is_null() {
                    let _ = ready_tx.send(Err(
                        "Failed to create CGEventTap; grant Aura Accessibility permission"
                            .to_string(),
                    ));
                    return;
                }

                let run_loop_source = CFMachPortCreateRunLoopSource(std::ptr::null_mut(), tap, 0);
                if run_loop_source.is_null() {
                    CFRelease(tap);
                    let _ = ready_tx.send(Err(
                        "Failed to create the macOS keyboard-hook run-loop source".to_string(),
                    ));
                    return;
                }

                let run_loop = CFRunLoopGetCurrent();
                if run_loop.is_null() {
                    CFRelease(run_loop_source);
                    CFRelease(tap);
                    let _ = ready_tx.send(Err(
                        "Failed to access the macOS keyboard-hook run loop".to_string()
                    ));
                    return;
                }

                EVENT_TAP.store(tap, Ordering::SeqCst);
                CGEventTapEnable(tap, true);
                CFRunLoopAddSource(run_loop, run_loop_source, kCFRunLoopCommonModes);
                if ready_tx.send(Ok(())).is_err() {
                    EVENT_TAP.store(std::ptr::null_mut(), Ordering::SeqCst);
                    CFRelease(run_loop_source);
                    CFRelease(tap);
                    return;
                }

                CFRunLoopRun();

                EVENT_TAP.store(std::ptr::null_mut(), Ordering::SeqCst);
                CFRelease(run_loop_source);
                CFRelease(tap);
            })
            .map_err(|error| format!("Failed to spawn macOS keyboard-hook thread: {error}"))?;

        ready_rx
            .recv_timeout(std::time::Duration::from_secs(5))
            .map_err(|error| format!("macOS keyboard-hook initialization timed out: {error}"))?
    }

    fn notify_hotkey(is_down: bool) {
        invoke_bool_callback(&CALLBACK, is_down, || {
            crate::logger::log(
                "ERROR",
                "Hotkey",
                None,
                "Hotkey callback panicked; the panic was contained at the macOS FFI boundary",
            )
        });
    }

    fn notify_cancel() {
        invoke_fn_callback(&CANCEL_CALLBACK, || {
            crate::logger::log(
                "ERROR",
                "Hotkey",
                None,
                "Cancel callback panicked; the panic was contained at the macOS FFI boundary",
            )
        });
    }

    unsafe extern "C" fn macos_event_tap_callback(
        _proxy: CGEventTapProxy,
        type_: u32,
        event: CGEventRef,
        _refcon: *mut c_void,
    ) -> CGEventRef {
        if type_ == kCGEventTapDisabledByTimeout || type_ == kCGEventTapDisabledByUserInput {
            let tap = EVENT_TAP.load(Ordering::SeqCst);
            if !tap.is_null() {
                CGEventTapEnable(tap, true);
            }
            return event;
        }
        if event.is_null() {
            return event;
        }
        if HOOK_PAUSED.load(Ordering::Relaxed) {
            return event;
        }

        let keycode = CGEventGetIntegerValueField(event, kCGKeyboardEventKeycode) as u32;
        let flags = CGEventGetFlags(event);

        if keycode == VK_ESCAPE
            && type_ == kCGEventKeyDown
            && RECORDING_ACTIVE.load(Ordering::SeqCst)
        {
            notify_cancel();
            return std::ptr::null_mut(); // Suppress Escape key
        }

        let (modifier_mask, target_keycode) = {
            let guard = match HOTKEY_CONFIG.lock() {
                Ok(guard) => guard,
                Err(poisoned) => {
                    crate::logger::log(
                        "ERROR",
                        "Hotkey",
                        None,
                        "Recovering poisoned macOS hotkey mutex",
                    );
                    poisoned.into_inner()
                }
            };
            (guard.modifier_mask, guard.key_code)
        };

        let modifier_satisfied = modifier_mask == 0 || (flags & modifier_mask) == modifier_mask;

        if target_keycode == Some(keycode) {
            if type_ == kCGEventKeyDown {
                if modifier_satisfied || SHORTCUT_ACTIVE.load(Ordering::SeqCst) {
                    KEY_SUPPRESSED.store(true, Ordering::SeqCst);
                    if !SHORTCUT_ACTIVE.swap(true, Ordering::SeqCst) {
                        notify_hotkey(true);
                    }
                    return std::ptr::null_mut(); // Suppress target key
                } else {
                    KEY_SUPPRESSED.store(false, Ordering::SeqCst);
                }
            } else if type_ == kCGEventKeyUp {
                let suppressed = KEY_SUPPRESSED.swap(false, Ordering::SeqCst);
                let was_active = SHORTCUT_ACTIVE.swap(false, Ordering::SeqCst);
                if was_active {
                    notify_hotkey(false);
                    apply_pending_hotkey();
                }
                if suppressed || modifier_satisfied || was_active {
                    return std::ptr::null_mut(); // Suppress target key release
                }
            }
        } else if type_ == kCGEventFlagsChanged && modifier_mask != 0 {
            if target_keycode.is_none() && modifier_satisfied {
                if !SHORTCUT_ACTIVE.swap(true, Ordering::SeqCst) {
                    notify_hotkey(true);
                }
            } else if !modifier_satisfied
                && target_keycode.is_some()
                && KEY_SUPPRESSED.load(Ordering::SeqCst)
            {
                // Modifier released while the target key is still held: defer
                // finalization to the target key-up (mirrors the Windows latch
                // behavior) so remaining auto-repeats stay suppressed.
                crate::logger::log(
                    "INFO",
                    "Hotkey",
                    None,
                    "Modifier released while target key still held; deferring session finalization until target key release",
                );
            } else if !modifier_satisfied && SHORTCUT_ACTIVE.swap(false, Ordering::SeqCst) {
                notify_hotkey(false);
                apply_pending_hotkey();
            }
        }

        event
    }
}

// ============================================================================
// EXPOSED PUBLIC API GATES
// ============================================================================
#[cfg(target_os = "windows")]
pub use windows_impl::{start_hook, update_hotkey, validate_hotkey};

#[cfg(target_os = "macos")]
pub use macos_impl::{start_hook, update_hotkey, validate_hotkey};
