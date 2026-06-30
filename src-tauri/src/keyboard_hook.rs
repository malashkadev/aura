use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::OnceLock;
use windows_sys::Win32::Foundation::{LPARAM, LRESULT, WPARAM};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, DispatchMessageW, GetMessageW, SetWindowsHookExW,
    TranslateMessage, UnhookWindowsHookEx, KBDLLHOOKSTRUCT, MSG,
    WH_KEYBOARD_LL, WM_KEYDOWN, WM_KEYUP, WM_SYSKEYDOWN, WM_SYSKEYUP,
};

use std::sync::Mutex;

struct HotkeyConfig {
    modifier_vk: u32,
    key_vk: u32,
}

static HOTKEY_CONFIG: Mutex<HotkeyConfig> = Mutex::new(HotkeyConfig {
    modifier_vk: 18, // VK_MENU (Alt)
    key_vk: 0x56,    // VK_V (V)
});

static CALLBACK: OnceLock<Box<dyn Fn(bool) + Send + Sync>> = OnceLock::new();
static MODIFIER_PRESSED: AtomicBool = AtomicBool::new(false);
static SHORTCUT_ACTIVE: AtomicBool = AtomicBool::new(false);
static KEY_SUPPRESSED: AtomicBool = AtomicBool::new(false);

fn is_modifier_key(vk_code: u32, target_modifier_vk: u32) -> bool {
    match target_modifier_vk {
        18 => vk_code == 18 || vk_code == 164 || vk_code == 165, // Alt / LAlt / RAlt
        17 => vk_code == 17 || vk_code == 162 || vk_code == 163, // Ctrl / LCtrl / RCtrl
        16 => vk_code == 16 || vk_code == 160 || vk_code == 161, // Shift / LShift / RShift
        _ => false,
    }
}

/// Dynamically updates the hotkey configuration.
pub fn update_hotkey(hotkey_str: &str) {
    let mut modifier = 0;
    let mut key = 0;
    
    let parts: Vec<&str> = hotkey_str.split('+').collect();
    for part in parts {
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
                        "space" => key = 0x20,
                        "capslock" | "caps lock" => key = 0x14,
                        "f8" => key = 0x77,
                        "f9" => key = 0x78,
                        "f10" => key = 0x79,
                        _ => {}
                    }
                }
            }
        }
    }
    
    if let Ok(mut guard) = HOTKEY_CONFIG.lock() {
        guard.modifier_vk = modifier;
        guard.key_vk = key;
        
        // Reset state flags
        MODIFIER_PRESSED.store(false, Ordering::SeqCst);
        SHORTCUT_ACTIVE.store(false, Ordering::SeqCst);
        KEY_SUPPRESSED.store(false, Ordering::SeqCst);
    }
}

/// Starts the global low-level keyboard hook on a background thread.
/// The `callback` is called with `true` when the configured hotkey is pressed/held,
/// and `false` when it is released.
pub fn start_hook<F>(callback: F) -> Result<(), &'static str>
where
    F: Fn(bool) + Send + Sync + 'static,
{
    if CALLBACK.set(Box::new(callback)).is_err() {
        return Err("Hook callback is already initialized");
    }

    std::thread::spawn(|| unsafe {
        // Install the low-level keyboard hook
        let hook = SetWindowsHookExW(
            WH_KEYBOARD_LL,
            Some(low_level_keyboard_proc),
            0, // HMODULE is 0 for the current process
            0, // Thread ID is 0 for global hooks
        );

        if hook == 0 {
            eprintln!("Error: Failed to install global keyboard hook.");
            return;
        }

        // Run the message loop (required for the hook to receive events)
        let mut msg: MSG = std::mem::zeroed();
        while GetMessageW(&mut msg, 0, 0, 0) > 0 {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }

        // Clean up the hook when exiting the message loop
        UnhookWindowsHookEx(hook);
    });

    Ok(())
}

/// The hook callback function called by Windows.
unsafe extern "system" fn low_level_keyboard_proc(
    code: i32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    // If code is less than 0, the hook procedure must pass the message to CallNextHookEx
    if code >= 0 {
        let kbd_struct = *(lparam as *const KBDLLHOOKSTRUCT);
        let vk_code = kbd_struct.vkCode;

        let is_down = wparam == WM_KEYDOWN as usize || wparam == WM_SYSKEYDOWN as usize;
        let is_up = wparam == WM_KEYUP as usize || wparam == WM_SYSKEYUP as usize;

        let (modifier_vk, key_vk) = {
            if let Ok(guard) = HOTKEY_CONFIG.lock() {
                (guard.modifier_vk, guard.key_vk)
            } else {
                (18, 0x4E) // Alt+N default
            }
        };

        let is_modifier = is_modifier_key(vk_code, modifier_vk);
        let is_target_key = vk_code == key_vk;

        if modifier_vk != 0 && is_modifier {
            if is_down {
                MODIFIER_PRESSED.store(true, Ordering::SeqCst);
            } else if is_up {
                MODIFIER_PRESSED.store(false, Ordering::SeqCst);
                if SHORTCUT_ACTIVE.swap(false, Ordering::SeqCst) {
                    if let Some(cb) = CALLBACK.get() {
                        cb(false);
                    }
                }
            }
        } else if is_target_key {
            let modifier_satisfied = modifier_vk == 0 || MODIFIER_PRESSED.load(Ordering::SeqCst);

            if is_down {
                if modifier_satisfied {
                    KEY_SUPPRESSED.store(true, Ordering::SeqCst);
                    if !SHORTCUT_ACTIVE.swap(true, Ordering::SeqCst) {
                        if let Some(cb) = CALLBACK.get() {
                            cb(true);
                        }
                    }
                    return 1; // Suppress key event
                } else {
                    KEY_SUPPRESSED.store(false, Ordering::SeqCst);
                }
            } else if is_up && (KEY_SUPPRESSED.swap(false, Ordering::SeqCst) || modifier_satisfied) {
                if SHORTCUT_ACTIVE.swap(false, Ordering::SeqCst) {
                    if let Some(cb) = CALLBACK.get() {
                        cb(false);
                    }
                }
                if modifier_satisfied {
                    return 1; // Suppress key event
                }
            }
        }
    }

    CallNextHookEx(0, code, wparam, lparam)
}
