use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::OnceLock;
use std::sync::Mutex;

static CALLBACK: OnceLock<Box<dyn Fn(bool) + Send + Sync>> = OnceLock::new();
static CANCEL_CALLBACK: OnceLock<Box<dyn Fn() + Send + Sync>> = OnceLock::new();
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
    CANCEL_CALLBACK
        .set(Box::new(callback))
        .map_err(|_| "Cancel callback is already initialized")
}

// ============================================================================
// WINDOWS IMPLEMENTATION
// ============================================================================
#[cfg(target_os = "windows")]
mod windows_impl {
    use super::*;
    use windows_sys::Win32::Foundation::{LPARAM, LRESULT, WPARAM};
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        CallNextHookEx, DispatchMessageW, GetMessageW, SetWindowsHookExW,
        TranslateMessage, UnhookWindowsHookEx, KBDLLHOOKSTRUCT, MSG,
        WH_KEYBOARD_LL, WM_KEYDOWN, WM_KEYUP, WM_SYSKEYDOWN, WM_SYSKEYUP,
    };
    use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
        GetAsyncKeyState, SendInput, INPUT, INPUT_KEYBOARD, KEYBDINPUT,
        KEYEVENTF_EXTENDEDKEY, KEYEVENTF_KEYUP, VK_CONTROL,
    };

    struct HotkeyConfig {
        modifier_vk: u32,
        key_vk: u32,
    }

    static HOTKEY_CONFIG: Mutex<HotkeyConfig> = Mutex::new(HotkeyConfig {
        modifier_vk: 18, // VK_MENU (Alt)
        key_vk: 0x56,    // VK_V (V)
    });

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
            SendInput(inputs.len() as u32, inputs.as_mut_ptr(), std::mem::size_of::<INPUT>() as i32);
        }
    }

    fn send_disarmed_alt_up(kbd: &KBDLLHOOKSTRUCT) {
        let ext = if (kbd.flags & 0x01) != 0 { KEYEVENTF_EXTENDEDKEY } else { 0 };
        let mut inputs = [
            keyboard_input(VK_CONTROL, 0, 0),
            keyboard_input(VK_CONTROL, 0, KEYEVENTF_KEYUP),
            keyboard_input(kbd.vkCode as u16, kbd.scanCode as u16, KEYEVENTF_KEYUP | ext),
        ];
        unsafe {
            SendInput(inputs.len() as u32, inputs.as_mut_ptr(), std::mem::size_of::<INPUT>() as i32);
        }
    }

    fn is_modifier_key(vk_code: u32, target_modifier_vk: u32) -> bool {
        match target_modifier_vk {
            18 => vk_code == 18 || vk_code == 164 || vk_code == 165, // Alt / LAlt / RAlt
            17 => vk_code == 17 || vk_code == 162 || vk_code == 163, // Ctrl / LCtrl / RCtrl
            16 => vk_code == 16 || vk_code == 160 || vk_code == 161, // Shift / LShift / RShift
            _ => false,
        }
    }

    fn parse_hotkey(hotkey_str: &str) -> Option<(u32, u32)> {
        let mut modifier = 0;
        let mut key = 0;

        for part in hotkey_str.split('+') {
            let clean = part.trim().to_lowercase();
            match clean.as_str() {
                "alt" => modifier = 18,
                "ctrl" | "control" => modifier = 17,
                "shift" => modifier = 16,
                other => {
                    if other.len() == 1 {
                        key = other.chars().next().unwrap().to_ascii_uppercase() as u32;
                    } else {
                        match other {
                            "space" | "пробел" => key = 0x20,
                            "capslock" | "caps lock" => key = 0x14,
                            "tab" => key = 0x09,
                            "f1" => key = 0x70,
                            "f2" => key = 0x71,
                            "f3" => key = 0x72,
                            "f4" => key = 0x73,
                            "f5" => key = 0x74,
                            "f6" => key = 0x75,
                            "f7" => key = 0x76,
                            "f8" => key = 0x77,
                            "f9" => key = 0x78,
                            "f10" => key = 0x79,
                            "f11" => key = 0x7A,
                            "f12" => key = 0x7B,
                            _ => {}
                        }
                    }
                }
            }
        }

        if key == 0 { None } else { Some((modifier, key)) }
    }

    pub fn update_hotkey(hotkey_str: &str) {
        let Some((modifier, key)) = parse_hotkey(hotkey_str) else {
            eprintln!("Aura Dev Log ERROR: Could not parse hotkey '{}'; keeping previous.", hotkey_str);
            return;
        };

        if let Ok(mut guard) = HOTKEY_CONFIG.lock() {
            guard.modifier_vk = modifier;
            guard.key_vk = key;

            SHORTCUT_ACTIVE.store(false, Ordering::SeqCst);
            KEY_SUPPRESSED.store(false, Ordering::SeqCst);
        }
    }

    pub fn start_hook<F>(callback: F) -> Result<(), &'static str>
    where
        F: Fn(bool) + Send + Sync + 'static,
    {
        if CALLBACK.set(Box::new(callback)).is_err() {
            return Err("Hook callback is already initialized");
        }

        std::thread::spawn(|| unsafe {
            let hook = SetWindowsHookExW(
                WH_KEYBOARD_LL,
                Some(low_level_keyboard_proc),
                0,
                0,
            );

            if hook == 0 {
                eprintln!("Error: Failed to install global keyboard hook.");
                return;
            }

            let mut msg: MSG = std::mem::zeroed();
            while GetMessageW(&mut msg, 0, 0, 0) > 0 {
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }

            UnhookWindowsHookEx(hook);
        });

        Ok(())
    }

    unsafe extern "system" fn low_level_keyboard_proc(
        code: i32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        if code >= 0 {
            let kbd_struct = *(lparam as *const KBDLLHOOKSTRUCT);
            let is_injected = (kbd_struct.flags & 0x10) != 0;
            if is_injected {
                return CallNextHookEx(0, code, wparam, lparam);
            }

            let vk_code = kbd_struct.vkCode;
            let is_down = wparam == WM_KEYDOWN as usize || wparam == WM_SYSKEYDOWN as usize;
            let is_up = wparam == WM_KEYUP as usize || wparam == WM_SYSKEYUP as usize;

            if vk_code == VK_ESCAPE && is_down && RECORDING_ACTIVE.load(Ordering::SeqCst) {
                if let Some(cb) = CANCEL_CALLBACK.get() {
                    cb();
                }
                return 1; // Suppress Esc
            }

            let (modifier_vk, key_vk) = {
                if let Ok(guard) = HOTKEY_CONFIG.lock() {
                    (guard.modifier_vk, guard.key_vk)
                } else {
                    (18, 0x56)
                }
            };

            let is_modifier = is_modifier_key(vk_code, modifier_vk);
            let is_target_key = vk_code == key_vk;

            if modifier_vk != 0 && is_modifier {
                if is_up {
                    if SHORTCUT_ACTIVE.swap(false, Ordering::SeqCst) {
                        if let Some(cb) = CALLBACK.get() {
                            cb(false);
                        }
                        if modifier_vk == 18 {
                            send_disarmed_alt_up(&kbd_struct);
                            return 1;
                        }
                    }
                }
            } else if is_target_key {
                let modifier_satisfied = modifier_vk == 0 || {
                    let state = GetAsyncKeyState(modifier_vk as i32);
                    (state as u16 & 0x8000) != 0
                };

                if is_down {
                    if modifier_satisfied || SHORTCUT_ACTIVE.load(Ordering::SeqCst) {
                        KEY_SUPPRESSED.store(true, Ordering::SeqCst);
                        if !SHORTCUT_ACTIVE.swap(true, Ordering::SeqCst) {
                            if let Some(cb) = CALLBACK.get() {
                                cb(true);
                            }
                        }
                        return 1;
                    } else {
                        KEY_SUPPRESSED.store(false, Ordering::SeqCst);
                    }
                } else if is_up {
                    let suppressed = KEY_SUPPRESSED.swap(false, Ordering::SeqCst);
                    let was_active = SHORTCUT_ACTIVE.swap(false, Ordering::SeqCst);
                    if was_active {
                        if let Some(cb) = CALLBACK.get() {
                            cb(false);
                        }
                        if modifier_vk == 18 && modifier_satisfied {
                            send_dummy_ctrl_tap();
                        }
                    }
                    if suppressed || modifier_satisfied || was_active {
                        return 1;
                    }
                }
            }
        }
        CallNextHookEx(0, code, wparam, lparam)
    }

    #[cfg(test)]
    mod tests {
        use super::parse_hotkey;

        #[test]
        fn test_parse_hotkey_combinations() {
            assert_eq!(parse_hotkey("Alt+V"), Some((18, 0x56)));
            assert_eq!(parse_hotkey("Ctrl+Space"), Some((17, 0x20)));
            assert_eq!(parse_hotkey("F8"), Some((0, 0x77)));
            assert_eq!(parse_hotkey("F12"), Some((0, 0x7B)));
            assert_eq!(parse_hotkey("Caps Lock"), Some((0, 0x14)));
            assert_eq!(parse_hotkey("Shift+Tab"), Some((16, 0x09)));
        }

        #[test]
        fn test_parse_hotkey_invalid() {
            assert_eq!(parse_hotkey(""), None);
            assert_eq!(parse_hotkey("Alt"), None);
            assert_eq!(parse_hotkey("Alt+Unknown"), None);
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

    pub type CGEventTapProxy = *mut c_void;
    pub type CGEventRef = *mut c_void;
    pub type CGEventTapCallBack = unsafe extern "C" fn(
        proxy: CGEventTapProxy,
        type_: u32,
        event: CGEventRef,
        refcon: *mut c_void,
    ) -> CGEventRef;

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
        ) -> *mut c_void;
        pub fn CFRunLoopSourceCreate(
            allocator: *mut c_void,
            order: isize,
            context: *mut c_void,
        ) -> CFRunLoopSourceRef;
        pub fn CFRunLoopGetCurrent() -> CFRunLoopRef;
        pub fn CFRunLoopAddSource(
            rl: CFRunLoopRef,
            source: CFRunLoopSourceRef,
            mode: CFStringRef,
        );
        pub fn CFRunLoopRun();
        pub fn CGEventGetFlags(event: CGEventRef) -> u64;
        pub fn CGEventGetIntegerValueField(event: CGEventRef, field: u32) -> i64;
        
        pub static kCFRunLoopCommonModes: CFStringRef;
    }

    pub const kCGSessionEventTap: u32 = 1;
    pub const kCGHeadInsertEventTap: u32 = 0;
    pub const kCGEventTapOptionDefault: u32 = 0;

    pub const kCGEventKeyDown: u32 = 10;
    pub const kCGEventKeyUp: u32 = 11;
    pub const kCGEventFlagsChanged: u32 = 12;

    pub const kCGKeyboardEventKeycode: u32 = 9;

    pub const kCGEventFlagMaskAlternate: u64 = 0x00080000;
    pub const kCGEventFlagMaskControl: u64 = 0x00040000;
    pub const kCGEventFlagMaskShift: u64 = 0x00020000;
    pub const kCGEventFlagMaskCommand: u64 = 0x00100000;

    struct HotkeyConfig {
        modifier_mask: u64,
        key_code: u32,
    }

    static HOTKEY_CONFIG: Mutex<HotkeyConfig> = Mutex::new(HotkeyConfig {
        modifier_mask: kCGEventFlagMaskAlternate, // Option (Alt)
        key_code: 9,                              // V key
    });

    const VK_ESCAPE: u32 = 53;

    fn parse_hotkey(hotkey_str: &str) -> Option<(u64, u32)> {
        let mut modifier_mask = 0;
        let mut key = 0;

        for part in hotkey_str.split('+') {
            let clean = part.trim().to_lowercase();
            match clean.as_str() {
                "alt" | "option" => modifier_mask |= kCGEventFlagMaskAlternate,
                "ctrl" | "control" => modifier_mask |= kCGEventFlagMaskControl,
                "shift" => modifier_mask |= kCGEventFlagMaskShift,
                "cmd" | "command" | "win" => modifier_mask |= kCGEventFlagMaskCommand,
                other => {
                    if other.len() == 1 {
                        let ch = other.chars().next().unwrap().to_ascii_uppercase();
                        key = match ch {
                            'A' => 0, 'B' => 11, 'C' => 8, 'D' => 2, 'E' => 14, 'F' => 3, 'G' => 5,
                            'H' => 4, 'I' => 34, 'J' => 38, 'K' => 40, 'L' => 37, 'M' => 46, 'N' => 45,
                            'O' => 31, 'P' => 35, 'Q' => 12, 'R' => 15, 'S' => 1, 'T' => 17, 'U' => 32,
                            'V' => 9, 'W' => 13, 'X' => 7, 'Y' => 16, 'Z' => 6,
                            _ => 0,
                        };
                    } else {
                        match other {
                            "space" => key = 49,
                            "capslock" => key = 57,
                            "tab" => key = 48,
                            "f1" => key = 122, "f2" => key = 120, "f3" => key = 99, "f4" => key = 118,
                            "f5" => key = 96, "f6" => key = 97, "f7" => key = 98, "f8" => key = 100,
                            "f9" => key = 101, "f10" => key = 109, "f11" => key = 103, "f12" => key = 111,
                            _ => {}
                        }
                    }
                }
            }
        }

        if key == 0 { None } else { Some((modifier_mask, key)) }
    }

    pub fn update_hotkey(hotkey_str: &str) {
        let Some((modifier, key)) = parse_hotkey(hotkey_str) else {
            eprintln!("Aura Dev Log ERROR: Could not parse hotkey '{}'; keeping previous.", hotkey_str);
            return;
        };

        if let Ok(mut guard) = HOTKEY_CONFIG.lock() {
            guard.modifier_mask = modifier;
            guard.key_code = key;

            SHORTCUT_ACTIVE.store(false, Ordering::SeqCst);
            KEY_SUPPRESSED.store(false, Ordering::SeqCst);
        }
    }

    pub fn start_hook<F>(callback: F) -> Result<(), &'static str>
    where
        F: Fn(bool) + Send + Sync + 'static,
    {
        if CALLBACK.set(Box::new(callback)).is_err() {
            return Err("Hook callback is already initialized");
        }

        std::thread::spawn(|| unsafe {
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
                eprintln!("Error: Failed to create CGEventTap. Ensure Accessibility permissions are granted.");
                return;
            }

            let run_loop_source = CFRunLoopSourceCreate(std::ptr::null_mut(), 0, tap);
            if run_loop_source.is_null() {
                eprintln!("Error: Failed to create CFRunLoopSource.");
                return;
            }

            let run_loop = CFRunLoopGetCurrent();
            CFRunLoopAddSource(run_loop, run_loop_source, kCFRunLoopCommonModes);
            CFRunLoopRun();
        });

        Ok(())
    }

    unsafe extern "system" fn macos_event_tap_callback(
        _proxy: CGEventTapProxy,
        type_: u32,
        event: CGEventRef,
        _refcon: *mut c_void,
    ) -> CGEventRef {
        let keycode = CGEventGetIntegerValueField(event, kCGKeyboardEventKeycode) as u32;
        let flags = CGEventGetFlags(event);

        if keycode == VK_ESCAPE && type_ == kCGEventKeyDown && RECORDING_ACTIVE.load(Ordering::SeqCst) {
            if let Some(cb) = CANCEL_CALLBACK.get() {
                cb();
            }
            return std::ptr::null_mut(); // Suppress Escape key
        }

        let (modifier_mask, target_keycode) = {
            if let Ok(guard) = HOTKEY_CONFIG.lock() {
                (guard.modifier_mask, guard.key_code)
            } else {
                (kCGEventFlagMaskAlternate, 9)
            }
        };

        let modifier_satisfied = modifier_mask == 0 || (flags & modifier_mask) != 0;

        if keycode == target_keycode {
            if type_ == kCGEventKeyDown {
                if modifier_satisfied || SHORTCUT_ACTIVE.load(Ordering::SeqCst) {
                    KEY_SUPPRESSED.store(true, Ordering::SeqCst);
                    if !SHORTCUT_ACTIVE.swap(true, Ordering::SeqCst) {
                        if let Some(cb) = CALLBACK.get() {
                            cb(true);
                        }
                    }
                    return std::ptr::null_mut(); // Suppress target key
                } else {
                    KEY_SUPPRESSED.store(false, Ordering::SeqCst);
                }
            } else if type_ == kCGEventKeyUp {
                let suppressed = KEY_SUPPRESSED.swap(false, Ordering::SeqCst);
                let was_active = SHORTCUT_ACTIVE.swap(false, Ordering::SeqCst);
                if was_active {
                    if let Some(cb) = CALLBACK.get() {
                        cb(false);
                    }
                }
                if suppressed || modifier_satisfied || was_active {
                    return std::ptr::null_mut(); // Suppress target key release
                }
            }
        } else if type_ == kCGEventFlagsChanged && modifier_mask != 0 {
            // If the modifier mask is active and was released logical-wise
            if !modifier_satisfied {
                if SHORTCUT_ACTIVE.swap(false, Ordering::SeqCst) {
                    if let Some(cb) = CALLBACK.get() {
                        cb(false);
                    }
                }
            }
        }

        event
    }
}

// ============================================================================
// EXPOSED PUBLIC API GATES
// ============================================================================
#[cfg(target_os = "windows")]
pub use windows_impl::{start_hook, update_hotkey};

#[cfg(target_os = "macos")]
pub use macos_impl::{start_hook, update_hotkey};
