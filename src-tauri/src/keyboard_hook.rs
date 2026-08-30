use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{Arc, Mutex};
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
    use windows_sys::Win32::Foundation::{LPARAM, LRESULT, WPARAM};
    use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
        GetAsyncKeyState, SendInput, INPUT, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_EXTENDEDKEY,
        KEYEVENTF_KEYUP, VK_CONTROL,
    };
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        CallNextHookEx, DispatchMessageW, GetMessageW, SetWindowsHookExW, TranslateMessage,
        UnhookWindowsHookEx, KBDLLHOOKSTRUCT, MSG, WH_KEYBOARD_LL, WM_KEYDOWN, WM_KEYUP,
        WM_SYSKEYDOWN, WM_SYSKEYUP,
    };

    const MOD_CTRL: u32 = 1 << 0;
    const MOD_ALT: u32 = 1 << 1;
    const MOD_SHIFT: u32 = 1 << 2;
    const MOD_WIN: u32 = 1 << 3;

    /// The configured hotkey packed into one word: (modifier_mask << 16) | key_vk.
    /// A zero key means the hotkey consists only of one or more modifiers.
    /// A single store/load means the hook thread can never observe a torn
    /// pair while settings are being saved (C14).
    static HOTKEY_PAIR: AtomicU32 = AtomicU32::new((MOD_ALT << 16) | 0x56); // Alt+V
    static SUPPRESSED_MODIFIERS: AtomicU32 = AtomicU32::new(0);

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
            dwExtraInfo: 0,
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

    fn modifiers_satisfied(mask: u32, event_vk: u32, event_is_down: bool) -> bool {
        [MOD_CTRL, MOD_ALT, MOD_SHIFT, MOD_WIN]
            .into_iter()
            .filter(|modifier| mask & modifier != 0)
            .all(|modifier| {
                if modifier_bit(event_vk) == Some(modifier) {
                    event_is_down
                } else {
                    modifier_is_down(modifier)
                }
            })
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

        if SHORTCUT_ACTIVE.load(Ordering::SeqCst) {
            // A live session still holds the old key: resetting SHORTCUT_ACTIVE
            // here would orphan it, because its eventual key-up no longer
            // matches the new pair and the session could never end normally.
            // Defer the switch until that release finalizes the session.
            PENDING_HOTKEY.store(packed, Ordering::SeqCst);
            crate::logger::log(
                "INFO",
                "Hotkey",
                None,
                "Recording is active; hotkey change deferred until the current hotkey is released",
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

    fn notify_hotkey(is_down: bool) {
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
                let hook = SetWindowsHookExW(WH_KEYBOARD_LL, Some(low_level_keyboard_proc), 0, 0);

                if hook == 0 {
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
            let kbd_struct = *(lparam as *const KBDLLHOOKSTRUCT);
            let is_injected = (kbd_struct.flags & 0x10) != 0;
            if is_injected {
                return CallNextHookEx(0, code, wparam, lparam);
            }

            let vk_code = kbd_struct.vkCode;
            let is_down = wparam == WM_KEYDOWN as usize || wparam == WM_SYSKEYDOWN as usize;
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
                let event_modifier = event_modifier.expect("required modifiers have a bit");
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
                    let target_still_held = {
                        let state = GetAsyncKeyState(key_vk as i32);
                        (state as u16 & 0x8000) != 0
                    };
                    if target_still_held {
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
                    if modifier_satisfied || SHORTCUT_ACTIVE.load(Ordering::SeqCst) {
                        KEY_SUPPRESSED.store(true, Ordering::SeqCst);
                        if !SHORTCUT_ACTIVE.swap(true, Ordering::SeqCst) {
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
        use super::{is_any_modifier_key, parse_hotkey, MOD_ALT, MOD_CTRL, MOD_SHIFT, MOD_WIN};

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

        if SHORTCUT_ACTIVE.load(Ordering::SeqCst) {
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
