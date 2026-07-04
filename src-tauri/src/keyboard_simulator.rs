use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
    SendInput, INPUT, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_KEYUP, KEYEVENTF_UNICODE,
    GetAsyncKeyState, VK_MENU, VK_CONTROL, VK_SHIFT, VK_LWIN, VK_RWIN
};

fn release_modifiers() -> Vec<u16> {
    let mut released = Vec::new();
    unsafe {
        let modifiers = [
            (VK_MENU, "Alt"),
            (VK_CONTROL, "Ctrl"),
            (VK_SHIFT, "Shift"),
            (VK_LWIN, "LWin"),
            (VK_RWIN, "RWin"),
        ];
        
        for &(vk, _) in &modifiers {
            if (GetAsyncKeyState(vk as i32) as u16 & 0x8000) != 0 {
                let mut input = std::mem::zeroed::<INPUT>();
                input.r#type = INPUT_KEYBOARD;
                input.Anonymous.ki = KEYBDINPUT {
                    wVk: vk,
                    wScan: 0,
                    dwFlags: KEYEVENTF_KEYUP,
                    time: 0,
                    dwExtraInfo: 0,
                };
                SendInput(1, &input, std::mem::size_of::<INPUT>() as i32);
                released.push(vk);
            }
        }
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
                dwFlags: 0, // Key down
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
        
        // Press Ctrl
        inputs[0].r#type = INPUT_KEYBOARD;
        inputs[0].Anonymous.ki = KEYBDINPUT {
            wVk: VK_CONTROL,
            wScan: 0,
            dwFlags: 0,
            time: 0,
            dwExtraInfo: 0,
        };

        // Press C
        inputs[1].r#type = INPUT_KEYBOARD;
        inputs[1].Anonymous.ki = KEYBDINPUT {
            wVk: 0x43, // C key
            wScan: 0,
            dwFlags: 0,
            time: 0,
            dwExtraInfo: 0,
        };

        // Release C
        inputs[2].r#type = INPUT_KEYBOARD;
        inputs[2].Anonymous.ki = KEYBDINPUT {
            wVk: 0x43,
            wScan: 0,
            dwFlags: KEYEVENTF_KEYUP,
            time: 0,
            dwExtraInfo: 0,
        };

        // Release Ctrl
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
        
        // Press Ctrl
        inputs[0].r#type = INPUT_KEYBOARD;
        inputs[0].Anonymous.ki = KEYBDINPUT {
            wVk: VK_CONTROL,
            wScan: 0,
            dwFlags: 0,
            time: 0,
            dwExtraInfo: 0,
        };

        // Press V
        inputs[1].r#type = INPUT_KEYBOARD;
        inputs[1].Anonymous.ki = KEYBDINPUT {
            wVk: 0x56, // V key
            wScan: 0,
            dwFlags: 0,
            time: 0,
            dwExtraInfo: 0,
        };

        // Release V
        inputs[2].r#type = INPUT_KEYBOARD;
        inputs[2].Anonymous.ki = KEYBDINPUT {
            wVk: 0x56,
            wScan: 0,
            dwFlags: KEYEVENTF_KEYUP,
            time: 0,
            dwExtraInfo: 0,
        };

        // Release Ctrl
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

/// Returns the handle of the window that currently has keyboard focus.
/// Used to make sure simulated typing goes to the window where dictation started.
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
            0x0419 => "ru".to_string(), // Russian
            0x0409 | 0x0809 | 0x0c09 | 0x1009 | 0x1409 | 0x1809 | 0x1c09 | 0x2009 | 0x2409 | 0x2809 | 0x2c09 | 0x3009 => "en".to_string(), // English
            0x0407 | 0x0807 | 0x0c07 => "de".to_string(), // German
            0x040a | 0x080a | 0x0c0a | 0x100a | 0x140a | 0x180a | 0x1c0a | 0x200a | 0x240a | 0x280a | 0x2c0a | 0x300a | 0x340a | 0x380a | 0x3c0a | 0x400a | 0x440a | 0x480a | 0x4c0a | 0x500a | 0x540a => "es".to_string(), // Spanish
            0x040c | 0x080c | 0x0c0c | 0x100c | 0x140c | 0x180c => "fr".to_string(), // French
            0x0410 | 0x0810 => "it".to_string(), // Italian
            0x0404 | 0x0804 | 0x0c04 | 0x1004 | 0x1404 => "zh".to_string(), // Chinese
            0x0416 | 0x0816 => "pt".to_string(), // Portuguese
            0x041f => "tr".to_string(), // Turkish
            _ => "ru".to_string(), // Default fallback
        }
    }
}

/// Simulated typing of a UTF-8 string using Win32 KEYEVENTF_UNICODE
pub fn type_string(text: &str) {
    let released = release_modifiers();
    unsafe {
        for ch in text.chars() {
            let mut buf = [0; 2];
            let utf16_chars = ch.encode_utf16(&mut buf);
            for &mut val in utf16_chars {
                let mut inputs = [std::mem::zeroed::<INPUT>(); 2];
                
                // Key down
                inputs[0].r#type = INPUT_KEYBOARD;
                inputs[0].Anonymous.ki = KEYBDINPUT {
                    wVk: 0,
                    wScan: val,
                    dwFlags: KEYEVENTF_UNICODE,
                    time: 0,
                    dwExtraInfo: 0,
                };

                // Key up
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

/// Simulated typing of backspaces to delete text
pub fn type_backspaces(count: usize) {
    let released = release_modifiers();
    unsafe {
        for _ in 0..count {
            let mut inputs = [std::mem::zeroed::<INPUT>(); 2];
            
            // Key down BACKSPACE
            inputs[0].r#type = INPUT_KEYBOARD;
            inputs[0].Anonymous.ki = KEYBDINPUT {
                wVk: 0x08, // VK_BACK
                wScan: 0,
                dwFlags: 0,
                time: 0,
                dwExtraInfo: 0,
            };

            // Key up BACKSPACE
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
        // 1. Send backspaces with batch flushes to prevent buffer drops
        for i in 0..backspace_count {
            let mut inputs = [std::mem::zeroed::<INPUT>(); 2];
            inputs[0].r#type = INPUT_KEYBOARD;
            inputs[0].Anonymous.ki = KEYBDINPUT {
                wVk: 0x08, // VK_BACK
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
            
            // Every 10 backspaces, sleep 15ms to let the OS process the queue, otherwise 3ms
            if (i + 1) % 10 == 0 {
                std::thread::sleep(std::time::Duration::from_millis(15));
            } else {
                std::thread::sleep(std::time::Duration::from_millis(3));
            }
        }
        
        // 2. Send characters with batch flushes to prevent buffer drops
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
                
                // Every 15 chars, sleep 10ms to let the OS process the queue, otherwise 1ms
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

