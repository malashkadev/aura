const REPLACEMENT_BATCH_UNITS: usize = 32;
const REPLACEMENT_BATCH_PAUSE_MS: u64 = 2;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FocusTarget {
    pub hwnd: isize,
    pub root_hwnd: isize,
    pub process_id: u32,
}

impl FocusTarget {
    pub fn is_empty(&self) -> bool {
        self.hwnd == 0
    }

    pub fn is_compatible_with(&self, current: &FocusTarget) -> bool {
        if self.is_empty() || current.is_empty() {
            return true;
        }
        if self.hwnd == current.hwnd {
            return true;
        }
        if self.root_hwnd != 0 && self.root_hwnd == current.root_hwnd {
            return true;
        }
        if self.process_id != 0 && self.process_id == current.process_id {
            return true;
        }
        false
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ReplacementDispatchMetrics {
    pub backspaces: usize,
    pub utf16_units: usize,
    pub batches: usize,
}

/// Describes a keyboard replacement that was interrupted or failed before it
/// completed. The committed counters tell the caller exactly how much of the
/// planned change physically landed in the target document, so a `typed_so_far`
/// mirror can be kept accurate and an Esc cancel erases the true visible text.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TextReplacementError {
    pub message: String,
    pub backspaces_committed: usize,
    pub utf16_units_committed: usize,
}

impl std::fmt::Display for TextReplacementError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for TextReplacementError {}

fn replacement_batch_count(units: usize) -> usize {
    units.div_ceil(REPLACEMENT_BATCH_UNITS)
}

fn replacement_dispatch_metrics(backspaces: usize, new_text: &str) -> ReplacementDispatchMetrics {
    let utf16_units = new_text.encode_utf16().count();
    ReplacementDispatchMetrics {
        backspaces,
        utf16_units,
        batches: replacement_batch_count(backspaces)
            .saturating_add(replacement_batch_count(utf16_units)),
    }
}

fn partial_keyup_recovery_index(inserted: u32, expected: u32) -> Option<usize> {
    (inserted < expected && inserted % 2 == 1).then_some(inserted as usize)
}

// ============================================================================
// WINDOWS IMPLEMENTATION
// ============================================================================
#[cfg(target_os = "windows")]
mod windows_impl {
    use super::{
        partial_keyup_recovery_index, replacement_dispatch_metrics, ReplacementDispatchMetrics,
        TextReplacementError, REPLACEMENT_BATCH_PAUSE_MS, REPLACEMENT_BATCH_UNITS,
    };
    use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
        GetAsyncKeyState, SendInput, INPUT, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_KEYUP,
        KEYEVENTF_UNICODE, VK_CONTROL, VK_LWIN, VK_MENU, VK_RWIN, VK_SHIFT,
    };

    fn keyboard_input(virtual_key: u16, scan_code: u16, flags: u32) -> INPUT {
        let mut input = unsafe { std::mem::zeroed::<INPUT>() };
        input.r#type = INPUT_KEYBOARD;
        input.Anonymous.ki = KEYBDINPUT {
            wVk: virtual_key,
            wScan: scan_code,
            dwFlags: flags,
            time: 0,
            dwExtraInfo: 0,
        };
        input
    }

    fn append_key_pair(inputs: &mut Vec<INPUT>, virtual_key: u16, scan_code: u16, flags: u32) {
        inputs.push(keyboard_input(virtual_key, scan_code, flags));
        inputs.push(keyboard_input(
            virtual_key,
            scan_code,
            flags | KEYEVENTF_KEYUP,
        ));
    }

    fn send_input_batch(inputs: &mut [INPUT], label: &str) -> Result<(), (usize, String)> {
        if inputs.is_empty() {
            return Ok(());
        }
        let expected = inputs.len() as u32;
        let inserted = unsafe {
            SendInput(
                expected,
                inputs.as_mut_ptr(),
                std::mem::size_of::<INPUT>() as i32,
            )
        };
        if inserted == expected {
            Ok(())
        } else {
            let recovery =
                if let Some(keyup_index) = partial_keyup_recovery_index(inserted, expected) {
                    let recovered = unsafe {
                        SendInput(
                            1,
                            inputs[keyup_index..].as_mut_ptr(),
                            std::mem::size_of::<INPUT>() as i32,
                        )
                    };
                    if recovered == 1 {
                        "; incomplete key pair was released"
                    } else {
                        "; WARNING: failed to release incomplete key pair"
                    }
                } else {
                    ""
                };
            Err((
                inserted as usize,
                format!(
                    "SendInput accepted {inserted}/{expected} events while dispatching {label}{recovery}"
                ),
            ))
        }
    }

    fn dispatch_backspace_batches<F>(
        count: usize,
        should_stop: &F,
    ) -> Result<usize, (usize, String)>
    where
        F: Fn() -> bool,
    {
        let mut dispatched = 0usize;
        let mut batches = 0usize;
        while dispatched < count {
            if should_stop() {
                return Err((
                    dispatched,
                    "keyboard replacement interrupted before Backspace batch".to_string(),
                ));
            }
            let units = (count - dispatched).min(REPLACEMENT_BATCH_UNITS);
            for i in 0..units {
                let mut inputs = Vec::with_capacity(2);
                append_key_pair(&mut inputs, 0x08, 0, 0);
                if let Err((inserted, message)) = send_input_batch(&mut inputs, "Backspace batch") {
                    return Err((dispatched + i + inserted / 2, message));
                }
                let current_total = dispatched + i + 1;
                if current_total < count {
                    let delay = if current_total.is_multiple_of(10) {
                        15
                    } else {
                        3
                    };
                    std::thread::sleep(std::time::Duration::from_millis(delay));
                }
            }
            dispatched += units;
            batches += 1;
        }
        Ok(batches)
    }

    fn dispatch_unicode_batches<F>(units: &[u16], should_stop: &F) -> Result<usize, (usize, String)>
    where
        F: Fn() -> bool,
    {
        let mut dispatched = 0usize;
        let mut batches = 0usize;
        while dispatched < units.len() {
            if should_stop() {
                return Err((
                    dispatched,
                    "keyboard replacement interrupted before Unicode batch".to_string(),
                ));
            }
            let end = (dispatched + REPLACEMENT_BATCH_UNITS).min(units.len());
            for (i, &unit) in units[dispatched..end].iter().enumerate() {
                let mut inputs = Vec::with_capacity(2);
                append_key_pair(&mut inputs, 0, unit, KEYEVENTF_UNICODE);
                if let Err((inserted, message)) = send_input_batch(&mut inputs, "Unicode batch") {
                    return Err((dispatched + i + inserted / 2, message));
                }
                let current_total = dispatched + i + 1;
                if current_total < units.len() {
                    let delay = if current_total.is_multiple_of(15) {
                        10
                    } else {
                        1
                    };
                    std::thread::sleep(std::time::Duration::from_millis(delay));
                }
            }
            dispatched = end;
            batches += 1;
        }
        Ok(batches)
    }

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

            if held
                .iter()
                .any(|&vk| vk == VK_MENU || vk == VK_LWIN || vk == VK_RWIN)
            {
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

            SendInput(
                inputs.len() as u32,
                inputs.as_mut_ptr(),
                std::mem::size_of::<INPUT>() as i32,
            );
        }
        released
    }

    fn restore_modifiers(released: &[u16]) {
        unsafe {
            for &vk in released {
                // Only re-press a modifier that the OS still reports as
                // released: if the user already physically pressed it again
                // during the typing window, synthesizing a key-down with no
                // matching key-up would stick the modifier for everyone
                // (menu bars, shortcuts, selection).
                let state = GetAsyncKeyState(vk as i32);
                if (state as u16 & 0x8000) != 0 {
                    continue;
                }
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
        std::thread::sleep(std::time::Duration::from_millis(30));
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

            let inserted = SendInput(4, inputs.as_mut_ptr(), std::mem::size_of::<INPUT>() as i32);
            if inserted != 4 {
                crate::logger::log(
                    "WARN",
                    "Keyboard",
                    None,
                    &format!("SendInput accepted {inserted}/4 events for copy; target may block injection (UIPI/elevated window)"),
                );
            }
        }
        std::thread::sleep(std::time::Duration::from_millis(30));
        restore_modifiers(&released);
    }

    /// Dispatches Ctrl+V and reports whether the OS accepted every event.
    /// `false` means input never reached the target (typically a UIPI block
    /// because the foreground app runs elevated) — the caller must NOT then
    /// restore the clipboard over the preserved transcript.
    pub fn simulate_paste() -> bool {
        let released = release_modifiers();
        let accepted = unsafe {
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

            let inserted = SendInput(4, inputs.as_mut_ptr(), std::mem::size_of::<INPUT>() as i32);
            let ok = inserted == 4;
            if !ok {
                crate::logger::log(
                    "WARN",
                    "Keyboard",
                    None,
                    &format!("SendInput accepted {inserted}/4 events for paste; target may block injection (UIPI/elevated window)"),
                );
            }
            ok
        };
        restore_modifiers(&released);
        accepted
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

    const GA_ROOT: u32 = 2;

    #[link(name = "user32")]
    extern "system" {
        pub fn GetForegroundWindow() -> isize;
        pub fn GetAncestor(hwnd: isize, gaflags: u32) -> isize;
        pub fn GetWindowThreadProcessId(hwnd: isize, lpdwprocessid: *mut u32) -> u32;
        pub fn GetKeyboardLayout(idthread: u32) -> isize;
    }

    pub fn get_foreground_window() -> isize {
        unsafe { GetForegroundWindow() }
    }

    pub fn get_focus_target() -> super::FocusTarget {
        unsafe {
            let hwnd = GetForegroundWindow();
            if hwnd == 0 {
                return super::FocusTarget::default();
            }
            let mut pid: u32 = 0;
            GetWindowThreadProcessId(hwnd, &mut pid);
            let root = GetAncestor(hwnd, GA_ROOT);
            super::FocusTarget {
                hwnd,
                root_hwnd: if root != 0 { root } else { hwnd },
                process_id: pid,
            }
        }
    }

    pub fn get_active_layout_language() -> String {
        unsafe {
            let hwnd = GetForegroundWindow();
            if hwnd == 0 {
                return "auto".to_string();
            }
            let thread_id = GetWindowThreadProcessId(hwnd, std::ptr::null_mut());
            let layout = GetKeyboardLayout(thread_id);
            let lang_id = (layout as usize) & 0xFFFF;
            match lang_id {
                0x0419 => "ru".to_string(),
                0x0409 | 0x0809 | 0x0c09 | 0x1009 | 0x1409 | 0x1809 | 0x1c09 | 0x2009 | 0x2409
                | 0x2809 | 0x2c09 | 0x3009 => "en".to_string(),
                0x0407 | 0x0807 | 0x0c07 => "de".to_string(),
                0x040a | 0x080a | 0x0c0a | 0x100a | 0x140a | 0x180a | 0x1c0a | 0x200a | 0x240a
                | 0x280a | 0x2c0a | 0x300a | 0x340a | 0x380a | 0x3c0a | 0x400a | 0x440a
                | 0x480a | 0x4c0a | 0x500a | 0x540a => "es".to_string(),
                0x040c | 0x080c | 0x0c0c | 0x100c | 0x140c | 0x180c => "fr".to_string(),
                0x0410 | 0x0810 => "it".to_string(),
                0x0404 | 0x0804 | 0x0c04 | 0x1004 | 0x1404 => "zh".to_string(),
                0x0416 | 0x0816 => "pt".to_string(),
                0x041f => "tr".to_string(),
                0x0413 | 0x0813 => "nl".to_string(),
                // Unknown layout: let the recognizer auto-detect instead of
                // forcing Russian on every other language of the world.
                _ => "auto".to_string(),
            }
        }
    }

    pub fn type_string(text: &str) {
        let released = release_modifiers();
        unsafe {
            for (index, ch) in text.chars().enumerate() {
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
                    let delay = if (index + 1) % 15 == 0 { 10 } else { 1 };
                    std::thread::sleep(std::time::Duration::from_millis(delay));
                }
            }
        }
        restore_modifiers(&released);
    }

    pub fn type_backspaces(count: usize) {
        let released = release_modifiers();
        unsafe {
            for index in 0..count {
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
                let delay = if (index + 1) % 10 == 0 { 15 } else { 3 };
                std::thread::sleep(std::time::Duration::from_millis(delay));
            }
        }
        restore_modifiers(&released);
    }

    pub fn replace_text<F>(
        backspace_count: usize,
        new_text: &str,
        should_stop: F,
    ) -> Result<ReplacementDispatchMetrics, TextReplacementError>
    where
        F: Fn() -> bool,
    {
        let metrics = replacement_dispatch_metrics(backspace_count, new_text);
        if metrics.backspaces == 0 && metrics.utf16_units == 0 {
            return Ok(metrics);
        }

        let released = release_modifiers();
        let utf16_units: Vec<u16> = new_text.encode_utf16().collect();
        let dispatch_result = (|| {
            let backspace_batches = dispatch_backspace_batches(backspace_count, &should_stop)
                .map_err(|(committed, message)| TextReplacementError {
                    message,
                    backspaces_committed: committed,
                    utf16_units_committed: 0,
                })?;
            if backspace_count > 0 && !utf16_units.is_empty() {
                if should_stop() {
                    return Err(TextReplacementError {
                        message: "keyboard replacement interrupted between delete and insert"
                            .to_string(),
                        backspaces_committed: backspace_count,
                        utf16_units_committed: 0,
                    });
                }
                std::thread::sleep(std::time::Duration::from_millis(REPLACEMENT_BATCH_PAUSE_MS));
            }
            let unicode_batches = dispatch_unicode_batches(&utf16_units, &should_stop).map_err(
                |(committed, message)| TextReplacementError {
                    message,
                    backspaces_committed: backspace_count,
                    utf16_units_committed: committed,
                },
            )?;
            debug_assert_eq!(metrics.batches, backspace_batches + unicode_batches);
            Ok(metrics)
        })();
        restore_modifiers(&released);
        dispatch_result
    }
}

// ============================================================================
// MACOS IMPLEMENTATION
// ============================================================================
#[cfg(target_os = "macos")]
mod macos_impl {
    use super::{replacement_dispatch_metrics, ReplacementDispatchMetrics, TextReplacementError};
    use objc::{msg_send, sel, sel_impl};
    use std::ffi::c_void;

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

    #[link(name = "CoreFoundation", kind = "framework")]
    extern "C" {
        fn CFRelease(value: *const c_void);
    }

    struct OwnedCf(*mut c_void);

    impl OwnedCf {
        fn new(value: *mut c_void) -> Option<Self> {
            (!value.is_null()).then_some(Self(value))
        }

        fn as_ptr(&self) -> *mut c_void {
            self.0
        }
    }

    impl Drop for OwnedCf {
        fn drop(&mut self) {
            unsafe {
                CFRelease(self.0);
            }
        }
    }

    fn create_event_source() -> Option<OwnedCf> {
        OwnedCf::new(unsafe { CGEventSourceCreate(kCGEventSourceStateHIDSystemState) })
    }

    fn post_keyboard_event(
        source: &OwnedCf,
        keycode: u16,
        keydown: bool,
        flags: u64,
        unicode: Option<&[u16]>,
    ) -> bool {
        unsafe {
            let Some(event) = OwnedCf::new(CGEventCreateKeyboardEvent(
                source.as_ptr(),
                keycode,
                keydown,
            )) else {
                return false;
            };
            CGEventSetFlags(event.as_ptr(), flags);
            if let Some(unicode) = unicode {
                CGEventKeyboardSetUnicodeString(event.as_ptr(), unicode.len(), unicode.as_ptr());
            }
            CGEventPost(kCGHIDEventTap, event.as_ptr());
            true
        }
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
        let Some(source) = create_event_source() else {
            crate::logger::log(
                "ERROR",
                "Keyboard",
                None,
                "Failed to create macOS event source",
            );
            return;
        };
        let _ = post_keyboard_event(&source, 8, true, kCGEventFlagMaskCommand, None);
        let _ = post_keyboard_event(&source, 8, false, kCGEventFlagMaskCommand, None);
    }

    pub fn simulate_paste() -> bool {
        let Some(source) = create_event_source() else {
            crate::logger::log(
                "ERROR",
                "Keyboard",
                None,
                "Failed to create macOS event source",
            );
            return false;
        };
        let down = post_keyboard_event(&source, 9, true, kCGEventFlagMaskCommand, None);
        let up = post_keyboard_event(&source, 9, false, kCGEventFlagMaskCommand, None);
        down.is_some() && up.is_some()
    }

    pub fn send_dummy_key() {
        // macOS doesn't have the same "clean Alt release menu focus" issue, so no-op.
    }

    pub fn get_foreground_window() -> isize {
        unsafe {
            let Some(ns_workspace) = objc::runtime::Class::get("NSWorkspace") else {
                return 0;
            };
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

    pub fn get_focus_target() -> super::FocusTarget {
        let hwnd = get_foreground_window();
        super::FocusTarget {
            hwnd,
            root_hwnd: hwnd,
            process_id: hwnd as u32,
        }
    }

    pub fn get_active_layout_language() -> String {
        unsafe {
            let source = tis::TISCopyCurrentKeyboardInputSource();
            let Some(_source_guard) = OwnedCf::new(source) else {
                return "auto".to_string();
            };
            let langs_array =
                tis::TISGetInputSourceProperty(source, tis::kTISPropertyInputSourceLanguages)
                    as tis::CFArrayRef;
            if langs_array.is_null() {
                return "auto".to_string();
            }
            let count = tis::CFArrayGetCount(langs_array);
            if count <= 0 {
                return "auto".to_string();
            }
            let lang_string = tis::CFArrayGetValueAtIndex(langs_array, 0) as tis::CFStringRef;
            if lang_string.is_null() {
                return "auto".to_string();
            }

            let mut buf = [0u8; 16];
            if tis::CFStringGetCString(
                lang_string,
                buf.as_mut_ptr(),
                buf.len() as isize,
                tis::kCFStringEncodingUTF8,
            ) {
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
                } else if s.starts_with("nl") {
                    return "nl".to_string();
                }
            }
            // Unknown input source: let the recognizer auto-detect instead of
            // forcing Russian on every other keyboard in the world.
            "auto".to_string()
        }
    }

    pub fn type_string(text: &str) {
        let Some(source) = create_event_source() else {
            crate::logger::log(
                "ERROR",
                "Keyboard",
                None,
                "Failed to create macOS event source",
            );
            return;
        };
        for ch in text.chars() {
            let mut buffer = [0; 2];
            let utf16 = ch.encode_utf16(&mut buffer);
            let _ = post_keyboard_event(&source, 0, true, 0, Some(utf16));
            let _ = post_keyboard_event(&source, 0, false, 0, Some(utf16));
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
    }

    pub fn type_backspaces(count: usize) {
        let Some(source) = create_event_source() else {
            crate::logger::log(
                "ERROR",
                "Keyboard",
                None,
                "Failed to create macOS event source",
            );
            return;
        };
        for _ in 0..count {
            let _ = post_keyboard_event(&source, 51, true, 0, None);
            let _ = post_keyboard_event(&source, 51, false, 0, None);
            std::thread::sleep(std::time::Duration::from_millis(3));
        }
    }

    pub fn replace_text<F>(
        backspace_count: usize,
        new_text: &str,
        should_stop: F,
    ) -> Result<ReplacementDispatchMetrics, TextReplacementError>
    where
        F: Fn() -> bool,
    {
        let Some(source) = create_event_source() else {
            return Err(TextReplacementError {
                message: "Failed to create macOS event source".to_string(),
                backspaces_committed: 0,
                utf16_units_committed: 0,
            });
        };
        let mut metrics = replacement_dispatch_metrics(backspace_count, new_text);
        metrics.batches = backspace_count.saturating_add(new_text.chars().count());

        for index in 0..backspace_count {
            if should_stop() {
                return Err(TextReplacementError {
                    message: "keyboard replacement interrupted before Backspace event".to_string(),
                    backspaces_committed: index,
                    utf16_units_committed: 0,
                });
            }
            if !post_keyboard_event(&source, 51, true, 0, None)
                || !post_keyboard_event(&source, 51, false, 0, None)
            {
                return Err(TextReplacementError {
                    message: "Failed to create macOS Backspace event".to_string(),
                    backspaces_committed: index,
                    utf16_units_committed: 0,
                });
            }
            let delay = if (index + 1) % 10 == 0 { 15 } else { 3 };
            std::thread::sleep(std::time::Duration::from_millis(delay));
        }

        for (index, ch) in new_text.chars().enumerate() {
            if should_stop() {
                let utf16_units_committed =
                    new_text.chars().take(index).map(|c| c.len_utf16()).sum();
                return Err(TextReplacementError {
                    message: "keyboard replacement interrupted before Unicode event".to_string(),
                    backspaces_committed: backspace_count,
                    utf16_units_committed,
                });
            }
            let mut buffer = [0; 2];
            let utf16 = ch.encode_utf16(&mut buffer);
            if !post_keyboard_event(&source, 0, true, 0, Some(utf16))
                || !post_keyboard_event(&source, 0, false, 0, Some(utf16))
            {
                let utf16_units_committed =
                    new_text.chars().take(index).map(|c| c.len_utf16()).sum();
                return Err(TextReplacementError {
                    message: "Failed to create macOS Unicode event".to_string(),
                    backspaces_committed: backspace_count,
                    utf16_units_committed,
                });
            }
            let delay = if (index + 1) % 15 == 0 { 10 } else { 1 };
            std::thread::sleep(std::time::Duration::from_millis(delay));
        }
        Ok(metrics)
    }
}

// ============================================================================
// EXPOSED PUBLIC API GATES
// ============================================================================
#[cfg(target_os = "windows")]
pub use windows_impl::{
    get_active_layout_language, get_focus_target, get_foreground_window, replace_text,
    send_dummy_key, simulate_copy, simulate_paste, type_backspaces, type_string,
};

#[cfg(target_os = "macos")]
pub use macos_impl::{
    get_active_layout_language, get_focus_target, get_foreground_window, replace_text,
    send_dummy_key, simulate_copy, simulate_paste, type_backspaces, type_string,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn replacement_dispatch_metrics_handle_empty_and_batch_boundaries() {
        assert_eq!(
            replacement_dispatch_metrics(0, ""),
            ReplacementDispatchMetrics::default()
        );
        assert_eq!(
            replacement_dispatch_metrics(REPLACEMENT_BATCH_UNITS, "a"),
            ReplacementDispatchMetrics {
                backspaces: REPLACEMENT_BATCH_UNITS,
                utf16_units: 1,
                batches: 2,
            }
        );
        assert_eq!(
            replacement_dispatch_metrics(REPLACEMENT_BATCH_UNITS + 1, ""),
            ReplacementDispatchMetrics {
                backspaces: REPLACEMENT_BATCH_UNITS + 1,
                utf16_units: 0,
                batches: 2,
            }
        );
    }

    #[test]
    fn replacement_dispatch_metrics_count_utf16_surrogate_pairs() {
        let text = format!("{}😀{}", "a".repeat(REPLACEMENT_BATCH_UNITS - 1), "b");
        assert_eq!(text.chars().count(), REPLACEMENT_BATCH_UNITS + 1);

        assert_eq!(
            replacement_dispatch_metrics(0, &text),
            ReplacementDispatchMetrics {
                backspaces: 0,
                utf16_units: REPLACEMENT_BATCH_UNITS + 2,
                batches: 2,
            }
        );
    }

    #[test]
    fn replacement_dispatch_metrics_stay_bounded_for_large_replacements() {
        let metrics = replacement_dispatch_metrics(10_000, &"я".repeat(10_000));

        assert_eq!(metrics.backspaces, 10_000);
        assert_eq!(metrics.utf16_units, 10_000);
        assert_eq!(metrics.batches, 626);
    }

    #[test]
    fn partial_dispatch_recovers_only_an_unpaired_keydown() {
        assert_eq!(partial_keyup_recovery_index(0, 64), None);
        assert_eq!(partial_keyup_recovery_index(2, 64), None);
        assert_eq!(partial_keyup_recovery_index(1, 64), Some(1));
        assert_eq!(partial_keyup_recovery_index(63, 64), Some(63));
        assert_eq!(partial_keyup_recovery_index(64, 64), None);
    }

    #[test]
    fn test_focus_target_compatibility() {
        let empty = FocusTarget::default();
        let target_a = FocusTarget {
            hwnd: 100,
            root_hwnd: 1000,
            process_id: 42,
        };
        let target_b = FocusTarget {
            hwnd: 200,
            root_hwnd: 1000, // Same root window (e.g. split editor)
            process_id: 42,
        };
        let target_c = FocusTarget {
            hwnd: 300,
            root_hwnd: 3000,
            process_id: 42, // Same process (e.g. child popup dialog)
        };
        let target_other = FocusTarget {
            hwnd: 500,
            root_hwnd: 5000,
            process_id: 99, // Completely different window & process
        };

        assert!(empty.is_compatible_with(&target_a));
        assert!(target_a.is_compatible_with(&empty));
        assert!(target_a.is_compatible_with(&target_a));
        assert!(target_a.is_compatible_with(&target_b));
        assert!(target_a.is_compatible_with(&target_c));
        assert!(!target_a.is_compatible_with(&target_other));
    }
}
