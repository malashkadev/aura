use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::OnceLock;
use windows_sys::Win32::Foundation::{LPARAM, LRESULT, WPARAM};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, DispatchMessageW, GetMessageW, SetWindowsHookExW,
    TranslateMessage, UnhookWindowsHookEx, KBDLLHOOKSTRUCT, MSG,
    WH_KEYBOARD_LL, WM_KEYDOWN, WM_KEYUP, WM_SYSKEYDOWN, WM_SYSKEYUP,
};
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
    VK_LMENU, VK_MENU, VK_RMENU,
};

static CALLBACK: OnceLock<Box<dyn Fn(bool) + Send + Sync>> = OnceLock::new();
static ALT_PRESSED: AtomicBool = AtomicBool::new(false);
static SHORTCUT_ACTIVE: AtomicBool = AtomicBool::new(false);
static N_SUPPRESSED: AtomicBool = AtomicBool::new(false);

/// Starts the global low-level keyboard hook on a background thread.
/// The `callback` is called with `true` when Alt+N is pressed/held,
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

        let is_alt = vk_code == VK_LMENU as u32 || vk_code == VK_RMENU as u32 || vk_code == VK_MENU as u32;
        let is_n = vk_code == 0x4E; // VK_N

        if is_alt {
            if is_down {
                ALT_PRESSED.store(true, Ordering::SeqCst);
            } else if is_up {
                ALT_PRESSED.store(false, Ordering::SeqCst);
                if SHORTCUT_ACTIVE.swap(false, Ordering::SeqCst) {
                    if let Some(cb) = CALLBACK.get() {
                        cb(false);
                    }
                }
            }
        } else if is_n {
            if is_down {
                if ALT_PRESSED.load(Ordering::SeqCst) {
                    N_SUPPRESSED.store(true, Ordering::SeqCst);
                    if !SHORTCUT_ACTIVE.swap(true, Ordering::SeqCst) {
                        if let Some(cb) = CALLBACK.get() {
                            cb(true);
                        }
                    }
                    return 1; // Suppress the N key event
                } else {
                    N_SUPPRESSED.store(false, Ordering::SeqCst);
                }
            } else if is_up {
                if N_SUPPRESSED.swap(false, Ordering::SeqCst) {
                    if SHORTCUT_ACTIVE.swap(false, Ordering::SeqCst) {
                        if let Some(cb) = CALLBACK.get() {
                            cb(false);
                        }
                    }
                    return 1; // Suppress the N key event
                }
            }
        }
    }

    CallNextHookEx(0, code, wparam, lparam)
}
