// ============================================================================
// WINDOWS IMPLEMENTATION
// ============================================================================
#[cfg(target_os = "windows")]
mod windows_impl {
    use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
        SendInput, INPUT, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_KEYUP, KEYEVENTF_UNICODE,
        GetAsyncKeyState, VK_MENU, VK_CONTROL, VK_SHIFT, VK_LWIN, VK_RWIN
    };

    fn release_modifiers() -> Vec<u16> {
        let mut released = Vec::new();
        unsafe {
            let modifiers = [VK_MENU, VK_CONTROL, VK_SHIFT, VK_LWIN, VK_RWIN];

            let held: Vec<u16> = modifiers
                .iter()
                .copied()
                .filter(|&vk| (GetAsyncKeyState(vk as i32) as u16 & 0x8000) != 0)
                .collect();
            if held.is_empty() {
                return released;
            }

            let mut inputs: Vec<INPUT> = Vec::with_capacity(held.len() + 2);

            if held.iter().any(|&vk| vk == VK_MENU || vk == VK_LWIN || vk == VK_RWIN) {
                let mut down = std::mem::zeroed::<INPUT>();
                down.r#type = INPUT_KEYBOARD;
                down.Anonymous.ki = KEYBDINPUT {
                    wVk: VK_CONTROL,
                    wScan: 0,
                    dwFlags: 0,
                    time: 0,
                    dwExtraInfo: 0,
                };
                let mut up = down;
                up.Anonymous.ki.dwFlags = KEYEVENTF_KEYUP;
                inputs.push(down);
                inputs.push(up);
            }

            for &vk in &held {
                let mut input = std::mem::zeroed::<INPUT>();
                input.r#type = INPUT_KEYBOARD;
                input.Anonymous.ki = KEYBDINPUT {
                    wVk: vk,
                    wScan: 0,
                    dwFlags: KEYEVENTF_KEYUP,
                    time: 0,
                    dwExtraInfo: 0,
                };
                inputs.push(input);
                released.push(vk);
            }

            SendInput(inputs.len() as u32, inputs.as_mut_ptr(), std::mem::size_of::<INPUT>() as i32);
        }
        released
    }

    fn restore_modifiers(released: &[u16]) {
        unsafe {
            for &vk in released {
                let mut input = std::mem::zeroed::<INPUT>();
                input.r#type = INPUT_KEYBOARD;
                input.Anonymous.ki = KEYBDINPUT {
                    wVk: vk,
                    wScan: 0,
                    dwFlags: 0,
                    time: 0,
                    dwExtraInfo: 0,
                };
                SendInput(1, &input, std::mem::size_of::<INPUT>() as i32);
            }
        }
    }

    pub fn simulate_copy() {
        let released = release_modifiers();
        unsafe {
            let mut inputs = [std::mem::zeroed::<INPUT>(); 4];
            
            inputs[0].r#type = INPUT_KEYBOARD;
            inputs[0].Anonymous.ki = KEYBDINPUT {
                wVk: VK_CONTROL,
                wScan: 0,
                dwFlags: 0,
                time: 0,
                dwExtraInfo: 0,
            };

            inputs[1].r#type = INPUT_KEYBOARD;
            inputs[1].Anonymous.ki = KEYBDINPUT {
                wVk: 0x43, // C key
                wScan: 0,
                dwFlags: 0,
                time: 0,
                dwExtraInfo: 0,
            };

            inputs[2].r#type = INPUT_KEYBOARD;
            inputs[2].Anonymous.ki = KEYBDINPUT {
                wVk: 0x43,
                wScan: 0,
                dwFlags: KEYEVENTF_KEYUP,
                time: 0,
                dwExtraInfo: 0,
            };

            inputs[3].r#type = INPUT_KEYBOARD;
            inputs[3].Anonymous.ki = KEYBDINPUT {
                wVk: VK_CONTROL,
                wScan: 0,
                dwFlags: KEYEVENTF_KEYUP,
                time: 0,
                dwExtraInfo: 0,
            };

            SendInput(4, inputs.as_mut_ptr(), std::mem::size_of::<INPUT>() as i32);
        }
        restore_modifiers(&released);
    }

    pub fn simulate_paste() {
        let released = release_modifiers();
        unsafe {
            let mut inputs = [std::mem::zeroed::<INPUT>(); 4];
            
            inputs[0].r#type = INPUT_KEYBOARD;
            inputs[0].Anonymous.ki = KEYBDINPUT {
                wVk: VK_CONTROL,
                wScan: 0,
                dwFlags: 0,
                time: 0,
                dwExtraInfo: 0,
            };

            inputs[1].r#type = INPUT_KEYBOARD;
            inputs[1].Anonymous.ki = KEYBDINPUT {
                wVk: 0x56, // V key
                wScan: 0,
                dwFlags: 0,
                time: 0,
                dwExtraInfo: 0,
            };

            inputs[2].r#type = INPUT_KEYBOARD;
            inputs[2].Anonymous.ki = KEYBDINPUT {
                wVk: 0x56,
                wScan: 0,
                dwFlags: KEYEVENTF_KEYUP,
                time: 0,
                dwExtraInfo: 0,
            };

            inputs[3].r#type = INPUT_KEYBOARD;
            inputs[3].Anonymous.ki = KEYBDINPUT {
                wVk: VK_CONTROL,
                wScan: 0,
                dwFlags: KEYEVENTF_KEYUP,
                time: 0,
                dwExtraInfo: 0,
            };

            SendInput(4, inputs.as_mut_ptr(), std::mem::size_of::<INPUT>() as i32);
        }
        restore_modifiers(&released);
    }

    pub fn send_dummy_key() {
        unsafe {
            let mut inputs = [std::mem::zeroed::<INPUT>(); 2];
            inputs[0].r#type = INPUT_KEYBOARD;
            inputs[0].Anonymous.ki = KEYBDINPUT {
                wVk: VK_CONTROL,
                wScan: 0,
                dwFlags: 0,
                time: 0,
                dwExtraInfo: 0,
            };
            inputs[1].r#type = INPUT_KEYBOARD;
            inputs[1].Anonymous.ki = KEYBDINPUT {
                wVk: VK_CONTROL,
                wScan: 0,
                dwFlags: KEYEVENTF_KEYUP,
                time: 0,
                dwExtraInfo: 0,
            };
            SendInput(2, inputs.as_mut_ptr(), std::mem::size_of::<INPUT>() as i32);
        }
    }

    #[link(name = "user32")]
    extern "system" {
        pub fn GetForegroundWindow() -> isize;
        pub fn GetWindowThreadProcessId(hwnd: isize, lpdwprocessid: *mut u32) -> u32;
        pub fn GetKeyboardLayout(idthread: u32) -> isize;
    }

    pub fn get_foreground_window() -> isize {
        unsafe { GetForegroundWindow() }
    }

    pub fn get_active_layout_language() -> String {
        unsafe {
            let hwnd = GetForegroundWindow();
            if hwnd == 0 {
                return "ru".to_string();
            }
            let thread_id = GetWindowThreadProcessId(hwnd, std::ptr::null_mut());
            let layout = GetKeyboardLayout(thread_id);
            let lang_id = (layout as usize) & 0xFFFF;
            match lang_id {
                0x0419 => "ru".to_string(),
                0x0409 | 0x0809 | 0x0c09 | 0x1009 | 0x1409 | 0x1809 | 0x1c09 | 0x2009 | 0x2409 | 0x2809 | 0x2c09 | 0x3009 => "en".to_string(),
                0x0407 | 0x0807 | 0x0c07 => "de".to_string(),
                0x040a | 0x080a | 0x0c0a | 0x100a | 0x140a | 0x180a | 0x1c0a | 0x200a | 0x240a | 0x280a | 0x2c0a | 0x300a | 0x340a | 0x380a | 0x3c0a | 0x400a | 0x440a | 0x480a | 0x4c0a | 0x500a | 0x540a => "es".to_string(),
                0x040c | 0x080c | 0x0c0c | 0x100c | 0x140c | 0x180c => "fr".to_string(),
                0x0410 | 0x0810 => "it".to_string(),
                0x0404 | 0x0804 | 0x0c04 | 0x1004 | 0x1404 => "zh".to_string(),
                0x0416 | 0x0816 => "pt".to_string(),
                0x041f => "tr".to_string(),
                _ => "ru".to_string(),
            }
        }
    }

    pub fn type_string(text: &str) {
        let released = release_modifiers();
        unsafe {
            for ch in text.chars() {
                let mut buf = [0; 2];
                let utf16_chars = ch.encode_utf16(&mut buf);
                for &mut val in utf16_chars {
                    let mut inputs = [std::mem::zeroed::<INPUT>(); 2];
                    
                    inputs[0].r#type = INPUT_KEYBOARD;
                    inputs[0].Anonymous.ki = KEYBDINPUT {
                        wVk: 0,
                        wScan: val,
                        dwFlags: KEYEVENTF_UNICODE,
                        time: 0,
                        dwExtraInfo: 0,
                    };

                    inputs[1].r#type = INPUT_KEYBOARD;
                    inputs[1].Anonymous.ki = KEYBDINPUT {
                        wVk: 0,
                        wScan: val,
                        dwFlags: KEYEVENTF_UNICODE | KEYEVENTF_KEYUP,
                        time: 0,
                        dwExtraInfo: 0,
                    };

                    SendInput(2, inputs.as_mut_ptr(), std::mem::size_of::<INPUT>() as i32);
                    std::thread::sleep(std::time::Duration::from_millis(1));
                }
            }
        }
        restore_modifiers(&released);
    }

    pub fn type_backspaces(count: usize) {
        let released = release_modifiers();
        unsafe {
            for _ in 0..count {
                let mut inputs = [std::mem::zeroed::<INPUT>(); 2];
                
                inputs[0].r#type = INPUT_KEYBOARD;
                inputs[0].Anonymous.ki = KEYBDINPUT {
                    wVk: 0x08,
                    wScan: 0,
                    dwFlags: 0,
                    time: 0,
                    dwExtraInfo: 0,
                };

                inputs[1].r#type = INPUT_KEYBOARD;
                inputs[1].Anonymous.ki = KEYBDINPUT {
                    wVk: 0x08,
                    wScan: 0,
                    dwFlags: KEYEVENTF_KEYUP,
                    time: 0,
                    dwExtraInfo: 0,
                };

                SendInput(2, inputs.as_mut_ptr(), std::mem::size_of::<INPUT>() as i32);
                std::thread::sleep(std::time::Duration::from_millis(3));
            }
        }
        restore_modifiers(&released);
    }

    pub fn replace_text(backspace_count: usize, new_text: &str) {
        let released = release_modifiers();
        unsafe {
            for i in 0..backspace_count {
                let mut inputs = [std::mem::zeroed::<INPUT>(); 2];
                inputs[0].r#type = INPUT_KEYBOARD;
                inputs[0].Anonymous.ki = KEYBDINPUT {
                    wVk: 0x08,
                    wScan: 0,
                    dwFlags: 0,
                    time: 0,
                    dwExtraInfo: 0,
                };
                inputs[1].r#type = INPUT_KEYBOARD;
                inputs[1].Anonymous.ki = KEYBDINPUT {
                    wVk: 0x08,
                    wScan: 0,
                    dwFlags: KEYEVENTF_KEYUP,
                    time: 0,
                    dwExtraInfo: 0,
                };
                SendInput(2, inputs.as_mut_ptr(), std::mem::size_of::<INPUT>() as i32);
                
                if (i + 1) % 10 == 0 {
                    std::thread::sleep(std::time::Duration::from_millis(15));
                } else {
                    std::thread::sleep(std::time::Duration::from_millis(3));
                }
            }
            
            let mut char_idx = 0;
            for ch in new_text.chars() {
                let mut buf = [0; 2];
                let utf16_chars = ch.encode_utf16(&mut buf);
                for &mut val in utf16_chars {
                    let mut inputs = [std::mem::zeroed::<INPUT>(); 2];
                    inputs[0].r#type = INPUT_KEYBOARD;
                    inputs[0].Anonymous.ki = KEYBDINPUT {
                        wVk: 0,
                        wScan: val,
                        dwFlags: KEYEVENTF_UNICODE,
                        time: 0,
                        dwExtraInfo: 0,
                    };
                    inputs[1].r#type = INPUT_KEYBOARD;
                    inputs[1].Anonymous.ki = KEYBDINPUT {
                        wVk: 0,
                        wScan: val,
                        dwFlags: KEYEVENTF_UNICODE | KEYEVENTF_KEYUP,
                        time: 0,
                        dwExtraInfo: 0,
                    };
                    SendInput(2, inputs.as_mut_ptr(), std::mem::size_of::<INPUT>() as i32);
                    
                    char_idx += 1;
                    if char_idx % 15 == 0 {
                        std::thread::sleep(std::time::Duration::from_millis(10));
                    } else {
                        std::thread::sleep(std::time::Duration::from_millis(1));
                    }
                }
            }
        }
        restore_modifiers(&released);
    }
}

// ============================================================================
// MACOS IMPLEMENTATION
// ============================================================================
#[cfg(target_os = "macos")]
mod macos_impl {
    use std::ffi::c_void;
    use objc::{msg_send, sel, sel_impl};

    pub type CGEventSourceRef = *mut c_void;
    pub type CGEventRef = *mut c_void;

    #[link(name = "CoreGraphics", kind = "framework")]
    extern "C" {
        pub fn CGEventSourceCreate(state_id: i32) -> CGEventSourceRef;
        pub fn CGEventCreateKeyboardEvent(
            source: CGEventSourceRef,
            keycode: u16,
            keydown: bool,
        ) -> CGEventRef;
        pub fn CGEventSetFlags(event: CGEventRef, flags: u64);
        pub fn CGEventKeyboardSetUnicodeString(
            event: CGEventRef,
            stringLength: usize,
            unicodeString: *const u16,
        );
        pub fn CGEventPost(tap: u32, event: CGEventRef);
    }

    pub const kCGHIDEventTap: u32 = 0;
    pub const kCGEventSourceStateHIDSystemState: i32 = 1;
    
    pub const kCGEventFlagMaskCommand: u64 = 0x00100000;

    mod tis {
        use std::ffi::c_void;
        
        pub type TISInputSourceRef = *mut c_void;
        pub type CFStringRef = *mut c_void;
        pub type CFArrayRef = *mut c_void;

        #[link(name = "Carbon", kind = "framework")]
        extern "C" {
            pub fn TISCopyCurrentKeyboardInputSource() -> TISInputSourceRef;
            pub fn TISGetInputSourceProperty(
                inputSource: TISInputSourceRef,
                propertyKey: CFStringRef,
            ) -> *mut c_void;
            pub static kTISPropertyInputSourceLanguages: CFStringRef;
        }

        #[link(name = "CoreFoundation", kind = "framework")]
        extern "C" {
            pub fn CFArrayGetCount(theArray: CFArrayRef) -> isize;
            pub fn CFArrayGetValueAtIndex(theArray: CFArrayRef, idx: isize) -> *const c_void;
            pub fn CFStringGetCString(
                theString: CFStringRef,
                buffer: *mut u8,
                bufferSize: isize,
                encoding: u32,
            ) -> bool;
        }
        
        pub const kCFStringEncodingUTF8: u32 = 0x08000100;
    }

    pub fn simulate_copy() {
        unsafe {
            let source = CGEventSourceCreate(kCGEventSourceStateHIDSystemState);
            
            // Cmd + C Down
            let event_down = CGEventCreateKeyboardEvent(source, 8, true);
            CGEventSetFlags(event_down, kCGEventFlagMaskCommand);
            CGEventPost(kCGHIDEventTap, event_down);
            
            // Cmd + C Up
            let event_up = CGEventCreateKeyboardEvent(source, 8, false);
            CGEventSetFlags(event_up, kCGEventFlagMaskCommand);
            CGEventPost(kCGHIDEventTap, event_up);
        }
    }

    pub fn simulate_paste() {
        unsafe {
            let source = CGEventSourceCreate(kCGEventSourceStateHIDSystemState);
            
            // Cmd + V Down
            let event_down = CGEventCreateKeyboardEvent(source, 9, true);
            CGEventSetFlags(event_down, kCGEventFlagMaskCommand);
            CGEventPost(kCGHIDEventTap, event_down);
            
            // Cmd + V Up
            let event_up = CGEventCreateKeyboardEvent(source, 9, false);
            CGEventSetFlags(event_up, kCGEventFlagMaskCommand);
            CGEventPost(kCGHIDEventTap, event_up);
        }
    }

    pub fn send_dummy_key() {
        // macOS doesn't have the same "clean Alt release menu focus" issue, so no-op.
    }

    pub fn get_foreground_window() -> isize {
        unsafe {
            let ns_workspace: objc::runtime::Class = *objc::runtime::Class::get("NSWorkspace").unwrap();
            let workspace: *mut objc::runtime::Object = msg_send![ns_workspace, sharedWorkspace];
            if workspace.is_null() {
                return 0;
            }
            let app: *mut objc::runtime::Object = msg_send![workspace, frontmostApplication];
            if app.is_null() {
                return 0;
            }
            let pid: i32 = msg_send![app, processIdentifier];
            pid as isize
        }
    }

    pub fn get_active_layout_language() -> String {
        unsafe {
            let source = tis::TISCopyCurrentKeyboardInputSource();
            if source.is_null() {
                return "ru".to_string();
            }
            let langs_array = tis::TISGetInputSourceProperty(source, tis::kTISPropertyInputSourceLanguages) as tis::CFArrayRef;
            if langs_array.is_null() {
                return "ru".to_string();
            }
            let count = tis::CFArrayGetCount(langs_array);
            if count <= 0 {
                return "ru".to_string();
            }
            let lang_string = tis::CFArrayGetValueAtIndex(langs_array, 0) as tis::CFStringRef;
            if lang_string.is_null() {
                return "ru".to_string();
            }
            
            let mut buf = [0u8; 16];
            if tis::CFStringGetCString(lang_string, buf.as_mut_ptr(), buf.len() as isize, tis::kCFStringEncodingUTF8) {
                let s = std::ffi::CStr::from_ptr(buf.as_ptr() as *const i8)
                    .to_string_lossy()
                    .into_owned();
                if s.starts_with("ru") {
                    return "ru".to_string();
                } else if s.starts_with("en") {
                    return "en".to_string();
                } else if s.starts_with("de") {
                    return "de".to_string();
                } else if s.starts_with("es") {
                    return "es".to_string();
                } else if s.starts_with("fr") {
                    return "fr".to_string();
                } else if s.starts_with("it") {
                    return "it".to_string();
                } else if s.starts_with("zh") {
                    return "zh".to_string();
                } else if s.starts_with("pt") {
                    return "pt".to_string();
                } else if s.starts_with("tr") {
                    return "tr".to_string();
                }
            }
            "ru".to_string()
        }
    }

    pub fn type_string(text: &str) {
        unsafe {
            let source = CGEventSourceCreate(kCGEventSourceStateHIDSystemState);
            for ch in text.chars() {
                let mut buf = [0; 2];
                let utf16_chars = ch.encode_utf16(&mut buf);
                
                let event_down = CGEventCreateKeyboardEvent(source, 0, true);
                CGEventKeyboardSetUnicodeString(event_down, utf16_chars.len(), utf16_chars.as_ptr());
                CGEventPost(kCGHIDEventTap, event_down);
                
                let event_up = CGEventCreateKeyboardEvent(source, 0, false);
                CGEventKeyboardSetUnicodeString(event_up, utf16_chars.len(), utf16_chars.as_ptr());
                CGEventPost(kCGHIDEventTap, event_up);
                
                std::thread::sleep(std::time::Duration::from_millis(1));
            }
        }
    }

    pub fn type_backspaces(count: usize) {
        unsafe {
            let source = CGEventSourceCreate(kCGEventSourceStateHIDSystemState);
            for _ in 0..count {
                let event_down = CGEventCreateKeyboardEvent(source, 51, true); // 51 is Backspace keycode on macOS
                CGEventPost(kCGHIDEventTap, event_down);
                
                let event_up = CGEventCreateKeyboardEvent(source, 51, false);
                CGEventPost(kCGHIDEventTap, event_up);
                
                std::thread::sleep(std::time::Duration::from_millis(3));
            }
        }
    }

    pub fn replace_text(backspace_count: usize, new_text: &str) {
        unsafe {
            let source = CGEventSourceCreate(kCGEventSourceStateHIDSystemState);
            
            // 1. Backspaces
            for i in 0..backspace_count {
                let event_down = CGEventCreateKeyboardEvent(source, 51, true);
                CGEventPost(kCGHIDEventTap, event_down);
                
                let event_up = CGEventCreateKeyboardEvent(source, 51, false);
                CGEventPost(kCGHIDEventTap, event_up);
                
                if (i + 1) % 10 == 0 {
                    std::thread::sleep(std::time::Duration::from_millis(15));
                } else {
                    std::thread::sleep(std::time::Duration::from_millis(3));
                }
            }
            
            // 2. Typing characters
            let mut char_idx = 0;
            for ch in new_text.chars() {
                let mut buf = [0; 2];
                let utf16_chars = ch.encode_utf16(&mut buf);
                
                let event_down = CGEventCreateKeyboardEvent(source, 0, true);
                CGEventKeyboardSetUnicodeString(event_down, utf16_chars.len(), utf16_chars.as_ptr());
                CGEventPost(kCGHIDEventTap, event_down);
                
                let event_up = CGEventCreateKeyboardEvent(source, 0, false);
                CGEventKeyboardSetUnicodeString(event_up, utf16_chars.len(), utf16_chars.as_ptr());
                CGEventPost(kCGHIDEventTap, event_up);
                
                char_idx += 1;
                if char_idx % 15 == 0 {
                    std::thread::sleep(std::time::Duration::from_millis(10));
                } else {
                    std::thread::sleep(std::time::Duration::from_millis(1));
                }
            }
        }
    }
}

// ============================================================================
// EXPOSED PUBLIC API GATES
// ============================================================================
#[cfg(target_os = "windows")]
pub use windows_impl::{
    simulate_copy, simulate_paste, send_dummy_key, get_foreground_window,
    get_active_layout_language, type_string, type_backspaces, replace_text,
};

#[cfg(target_os = "macos")]
pub use macos_impl::{
    simulate_copy, simulate_paste, send_dummy_key, get_foreground_window,
    get_active_layout_language, type_string, type_backspaces, replace_text,
};
