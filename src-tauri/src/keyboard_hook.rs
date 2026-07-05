use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::OnceLock;
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
static CANCEL_CALLBACK: OnceLock<Box<dyn Fn() + Send + Sync>> = OnceLock::new();
static MODIFIER_PRESSED: AtomicBool = AtomicBool::new(false);
static SHORTCUT_ACTIVE: AtomicBool = AtomicBool::new(false);
static KEY_SUPPRESSED: AtomicBool = AtomicBool::new(false);
// Set by the app while a recording session is active so the hook can intercept Esc
static RECORDING_ACTIVE: AtomicBool = AtomicBool::new(false);

const VK_ESCAPE: u32 = 0x1B;

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

/// A "clean" Alt press-and-release focuses the menu bar of the foreground app
/// (browsers move focus out of the input field). Because we suppress the hotkey
/// letter, the app never sees a key between Alt-down and Alt-up, so it would
/// treat the release as clean. A dummy Ctrl tap while Alt is still held disarms
/// that heuristic without any visible side effect.
fn send_dummy_ctrl_tap() {
    let mut inputs = [
        keyboard_input(VK_CONTROL, 0, 0),
        keyboard_input(VK_CONTROL, 0, KEYEVENTF_KEYUP),
    ];
    unsafe {
        SendInput(inputs.len() as u32, inputs.as_mut_ptr(), std::mem::size_of::<INPUT>() as i32);
    }
}

/// Swallows the real Alt-up and re-injects it preceded by a dummy Ctrl tap so
/// the foreground app never sees a menu-activating "clean" Alt release.
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

/// Parses a hotkey string like "Alt+V" or "F8" into (modifier_vk, key_vk).
/// Returns None if no valid main key could be recognized.
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

    if key == 0 {
        None
    } else {
        Some((modifier, key))
    }
}

/// Dynamically updates the hotkey configuration.
/// Unparseable strings are rejected and the previous hotkey stays active.
pub fn update_hotkey(hotkey_str: &str) {
    let Some((modifier, key)) = parse_hotkey(hotkey_str) else {
        eprintln!("Aura Dev Log ERROR: Could not parse hotkey '{}'; keeping previous hotkey.", hotkey_str);
        return;
    };

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
        
        // Ignore simulated/injected keyboard events from SendInput to prevent self-triggering
        let is_injected = (kbd_struct.flags & 0x10) != 0;
        if is_injected {
            return CallNextHookEx(0, code, wparam, lparam);
        }

        let vk_code = kbd_struct.vkCode;

        let is_down = wparam == WM_KEYDOWN as usize || wparam == WM_SYSKEYDOWN as usize;
        let is_up = wparam == WM_KEYUP as usize || wparam == WM_SYSKEYUP as usize;

        // Esc cancels an active recording session
        if vk_code == VK_ESCAPE && is_down && RECORDING_ACTIVE.load(Ordering::SeqCst) {
            if let Some(cb) = CANCEL_CALLBACK.get() {
                cb();
            }
            return 1; // Suppress Esc so the focused app doesn't react to it
        }

        let (modifier_vk, key_vk) = {
            if let Ok(guard) = HOTKEY_CONFIG.lock() {
                (guard.modifier_vk, guard.key_vk)
            } else {
                (18, 0x56) // Alt+V default
            }
        };

        let is_modifier = is_modifier_key(vk_code, modifier_vk);
        let is_target_key = vk_code == key_vk;

        if modifier_vk != 0 && is_modifier {
            if is_up {
                // If modifier is released, stop shortcut if active
                if SHORTCUT_ACTIVE.swap(false, Ordering::SeqCst) {
                    if let Some(cb) = CALLBACK.get() {
                        cb(false);
                    }
                    // Alt ended the shortcut: replace the real Alt-up with a
                    // disarmed sequence so the focused window keeps its caret.
                    if modifier_vk == 18 {
                        send_disarmed_alt_up(&kbd_struct);
                        return 1;
                    }
                }
            }
        } else if is_target_key {
            // Check real physical state of modifier key using GetAsyncKeyState
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
                    return 1; // Suppress key event
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
                    // The suppressed letter never reached the app, so the
                    // upcoming physical Alt release would look "clean" and
                    // focus the menu bar. Disarm it while Alt is still held.
                    if modifier_vk == 18 && modifier_satisfied {
                        send_dummy_ctrl_tap();
                    }
                }
                if suppressed || modifier_satisfied || was_active {
                    return 1; // Suppress key event
                }
            }
        }
    }

    CallNextHookEx(0, code, wparam, lparam)
}

#[cfg(test)]
mod tests {
    use super::*;

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
