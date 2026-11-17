use crate::smart_insertion::CaretContext;

/// Default timeout budget for caret/selection context capture via warm UIA.
pub const DEFAULT_CARET_CAPTURE_TIMEOUT_MS: u64 = 35;

#[cfg(target_os = "windows")]
mod fast_win32 {
    use crate::smart_insertion::{CaretContext, CaretContextSource};
    use std::borrow::Cow;
    use windows::Win32::Foundation::{HWND, LPARAM, WPARAM};
    use windows::Win32::UI::WindowsAndMessaging::{
        GetClassNameW, GetForegroundWindow, GetGUIThreadInfo, GetWindowThreadProcessId,
        SendMessageTimeoutW, GUITHREADINFO, SMTO_ABORTIFHUNG, SMTO_BLOCK, WM_GETTEXT,
        WM_GETTEXTLENGTH,
    };

    const EM_GETSEL: u32 = 0x00B0;
    const EM_GETPASSWORDCHAR: u32 = 0x00D2;
    const SCI_GETLENGTH: u32 = 2006;
    const SCI_GETCURRENTPOS: u32 = 2008;
    const SCI_GETANCHOR: u32 = 2009;

    const CONTEXT_CHARS: usize = 100;
    const CONTEXT_MARGIN: usize = 400;
    const MAX_EDIT_TEXT: usize = 32_767;
    const MAX_SCINTILLA_TEXT: usize = 262_144;
    const MSG_TIMEOUT_MS: u32 = 10;

    unsafe fn send_msg(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> Option<usize> {
        let mut result: usize = 0;
        let lr = SendMessageTimeoutW(
            hwnd,
            msg,
            wparam,
            lparam,
            SMTO_ABORTIFHUNG | SMTO_BLOCK,
            MSG_TIMEOUT_MS,
            Some(&mut result as *mut usize),
        );
        (lr.0 != 0).then_some(result)
    }

    fn take_last_chars(s: String, n: usize) -> String {
        let count = s.chars().count();
        if count <= n {
            return s;
        }
        s.chars().skip(count - n).collect()
    }

    fn take_first_chars(s: String, n: usize) -> String {
        s.chars().take(n).collect()
    }

    fn utf16_range_to_string(buf: &[u16], start: usize, end: usize) -> String {
        let mut s = start.min(buf.len());
        let mut e = end.min(buf.len()).max(s);
        if s < e && (0xDC00..=0xDFFF).contains(&buf[s]) {
            s += 1;
        }
        if s < e && (0xD800..=0xDBFF).contains(&buf[e - 1]) {
            e -= 1;
        }
        String::from_utf16_lossy(&buf[s..e])
    }

    fn utf8_range_to_string(buf: &[u8], start: usize, end: usize) -> String {
        let mut s = start.min(buf.len());
        let mut e = end.min(buf.len()).max(s);
        while s < e && (buf[s] & 0xC0) == 0x80 {
            s += 1;
        }
        if e < buf.len() {
            while e > s && (buf[e] & 0xC0) == 0x80 {
                e -= 1;
            }
        }
        match std::str::from_utf8(&buf[s..e]) {
            Ok(valid) => valid.to_string(),
            Err(err) if err.error_len().is_none() => {
                String::from_utf8_lossy(&buf[s..s + err.valid_up_to()]).into_owned()
            }
            Err(_) => String::from_utf8_lossy(&buf[s..e]).into_owned(),
        }
    }

    /// RichEdit 2.0+/4.1+/`RichEditD2DPT` stores line breaks internally as a
    /// single `\r` (1 unit in `EM_GETSEL`), while `WM_GETTEXT` expands each to
    /// `\r\n` (2 `u16` units). Collapsing `\r\n` to `\n` aligns `WM_GETTEXT`
    /// offsets 1:1 with `EM_GETSEL` without cross-process pointers.
    fn normalize_richedit_utf16(wide: &[u16]) -> Cow<'_, [u16]> {
        if !wide.windows(2).any(|pair| pair == [0x000D, 0x000A]) {
            return Cow::Borrowed(wide);
        }
        let mut normalized = Vec::with_capacity(wide.len());
        let mut idx = 0;
        while idx < wide.len() {
            if idx + 1 < wide.len() && wide[idx] == 0x000D && wide[idx + 1] == 0x000A {
                normalized.push(0x000A);
                idx += 2;
            } else {
                normalized.push(wide[idx]);
                idx += 1;
            }
        }
        Cow::Owned(normalized)
    }

    fn build_edit_caret_context(
        wide: &[u16],
        start: usize,
        end: usize,
        is_richedit: bool,
    ) -> Option<CaretContext> {
        let aligned = if is_richedit {
            normalize_richedit_utf16(wide)
        } else {
            Cow::Borrowed(wide)
        };
        let slice = aligned.as_ref();
        if start > end || end > slice.len() {
            return None;
        }
        Some(CaretContext {
            text_before: take_last_chars(
                utf16_range_to_string(slice, start.saturating_sub(CONTEXT_MARGIN), start),
                CONTEXT_CHARS,
            ),
            selected_text: utf16_range_to_string(slice, start, end),
            text_after: take_first_chars(
                utf16_range_to_string(slice, end, (end + CONTEXT_MARGIN).min(slice.len())),
                CONTEXT_CHARS,
            ),
            source: CaretContextSource::Win32Edit,
        })
    }

    fn empty_context() -> CaretContext {
        CaretContext {
            source: CaretContextSource::Win32Edit,
            ..Default::default()
        }
    }

    unsafe fn capture_edit(hwnd: HWND, is_richedit: bool) -> Option<CaretContext> {
        if send_msg(hwnd, EM_GETPASSWORDCHAR, WPARAM(0), LPARAM(0))? != 0 {
            return None;
        }
        let length = send_msg(hwnd, WM_GETTEXTLENGTH, WPARAM(0), LPARAM(0))?;
        if length == 0 {
            return Some(empty_context());
        }
        if length > MAX_EDIT_TEXT {
            return None;
        }
        let packed = send_msg(hwnd, EM_GETSEL, WPARAM(0), LPARAM(0))?;
        let start = packed & 0xFFFF;
        let end = (packed >> 16) & 0xFFFF;

        let mut buf = vec![0u16; length + 1];
        let copied = send_msg(
            hwnd,
            WM_GETTEXT,
            WPARAM(length + 1),
            LPARAM(buf.as_mut_ptr() as isize),
        )?;
        if copied == 0 || copied > length {
            return None;
        }
        build_edit_caret_context(&buf[..copied], start, end, is_richedit)
    }

    unsafe fn capture_scintilla(hwnd: HWND) -> Option<CaretContext> {
        let length = send_msg(hwnd, SCI_GETLENGTH, WPARAM(0), LPARAM(0))?;
        if length == 0 {
            return Some(empty_context());
        }
        if length > MAX_SCINTILLA_TEXT {
            return None;
        }
        let current = send_msg(hwnd, SCI_GETCURRENTPOS, WPARAM(0), LPARAM(0))?;
        let anchor = send_msg(hwnd, SCI_GETANCHOR, WPARAM(0), LPARAM(0))?;
        let start = current.min(anchor);
        let end = current.max(anchor);
        if end > length {
            return None;
        }

        let mut buf = vec![0u8; length + 1];
        let copied = send_msg(
            hwnd,
            WM_GETTEXT,
            WPARAM(length + 1),
            LPARAM(buf.as_mut_ptr() as isize),
        )?;
        if copied == 0 || copied > length {
            return None;
        }
        let bytes = &buf[..copied];
        if end > bytes.len() {
            return None;
        }
        Some(CaretContext {
            text_before: take_last_chars(
                utf8_range_to_string(bytes, start.saturating_sub(CONTEXT_MARGIN), start),
                CONTEXT_CHARS,
            ),
            selected_text: utf8_range_to_string(bytes, start, end),
            text_after: take_first_chars(
                utf8_range_to_string(bytes, end, (end + CONTEXT_MARGIN).min(bytes.len())),
                CONTEXT_CHARS,
            ),
            source: CaretContextSource::Win32Edit,
        })
    }

    pub fn try_capture() -> Option<CaretContext> {
        unsafe {
            let foreground = GetForegroundWindow();
            if foreground.0 == 0 {
                return None;
            }
            let thread_id = GetWindowThreadProcessId(foreground, None);
            if thread_id == 0 {
                return None;
            }
            let mut info = GUITHREADINFO {
                cbSize: std::mem::size_of::<GUITHREADINFO>() as u32,
                ..Default::default()
            };
            if GetGUIThreadInfo(thread_id, &mut info).is_err() {
                return None;
            }
            let focus = info.hwndFocus;
            if focus.0 == 0 {
                return None;
            }

            let mut class_buf = [0u16; 128];
            let class_len = GetClassNameW(focus, &mut class_buf);
            if class_len <= 0 {
                return None;
            }
            let class = String::from_utf16_lossy(&class_buf[..class_len as usize]);
            let lower = class.to_ascii_lowercase();

            let is_richedit =
                lower.starts_with("richedit") && (lower.ends_with('w') || lower == "richeditd2dpt");
            if lower == "edit" || is_richedit {
                capture_edit(focus, is_richedit)
            } else if lower == "scintilla" {
                capture_scintilla(focus)
            } else {
                None
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn take_last_chars_handles_multibyte() {
            let s = "привет мир".to_string();
            assert_eq!(take_last_chars(s.clone(), 3), "мир");
            assert_eq!(take_last_chars(s, 100), "привет мир");
        }

        #[test]
        fn take_first_chars_handles_surrogates() {
            let s = "𝕋x😀y".to_string();
            assert_eq!(take_first_chars(s.clone(), 2), "𝕋x");
            assert_eq!(take_first_chars(s, 100), "𝕋x😀y");
        }

        #[test]
        fn utf16_range_roundtrip() {
            let s = "a𝕋b";
            let wide: Vec<u16> = s.encode_utf16().collect();
            assert_eq!(utf16_range_to_string(&wide, 0, wide.len()), s);
            assert_eq!(utf16_range_to_string(&wide, 1, wide.len()), "𝕋b");
        }

        #[test]
        fn utf16_range_skips_split_surrogates_without_replacement_char() {
            let s = "😀привет😀";
            let wide: Vec<u16> = s.encode_utf16().collect();
            // Slice starting on the low surrogate of the first emoji and ending on the high surrogate of the last emoji
            let extracted = utf16_range_to_string(&wide, 1, wide.len() - 1);
            assert_eq!(extracted, "привет");
            assert!(!extracted.contains('\u{FFFD}'));
        }

        #[test]
        fn utf8_range_roundtrip() {
            let s = "привет мир";
            let bytes = s.as_bytes();
            assert_eq!(utf8_range_to_string(bytes, 0, bytes.len()), s);
            assert_eq!(utf8_range_to_string(bytes, 13, bytes.len()), "мир");
        }

        #[test]
        fn utf8_range_snaps_to_char_boundaries_without_replacement_char() {
            let s = "😀привет😀";
            let bytes = s.as_bytes();
            // Cut mid-emoji on both left (byte 2 of 4) and right (len - 2)
            let extracted = utf8_range_to_string(bytes, 2, bytes.len() - 2);
            assert_eq!(extracted, "привет");
            assert!(!extracted.contains('\u{FFFD}'));
        }

        #[test]
        fn richedit_crlf_normalizes_to_em_getsel_offsets() {
            // WM_GETTEXT returns CRLF, while RichEdit EM_GETSEL counts each line break as 1 unit.
            let wm_gettext: Vec<u16> = "Первая\r\nВторая\r\nТретья".encode_utf16().collect();
            // Caret right after "Первая\rВторая\r" (6 + 1 + 6 + 1 = 14 in EM_GETSEL)
            let ctx = build_edit_caret_context(&wm_gettext, 14, 20, true)
                .expect("richedit context must align");
            assert_eq!(ctx.text_before, "Первая\nВторая\n");
            assert_eq!(ctx.selected_text, "Третья");
            assert_eq!(ctx.text_after, "");

            // Classic Win32 Edit counts CRLF as 2 units in EM_GETSEL (6 + 2 + 6 + 2 = 16).
            let classic = build_edit_caret_context(&wm_gettext, 16, 22, false)
                .expect("classic edit context must preserve CRLF offsets");
            assert_eq!(classic.text_before, "Первая\r\nВторая\r\n");
            assert_eq!(classic.selected_text, "Третья");
        }

        #[test]
        fn fast_path_smoke() {
            let _ = try_capture();
        }
    }
}

#[cfg(target_os = "windows")]
mod uia_worker {
    use crate::smart_insertion::{CaretContext, CaretContextSource};
    use std::sync::{mpsc, OnceLock};
    use std::time::{Duration, Instant};
    use windows::Win32::System::Com::{
        CoCreateInstance, CoInitializeEx, CoUninitialize, CLSCTX_INPROC_SERVER,
        COINIT_MULTITHREADED,
    };
    use windows::Win32::UI::Accessibility::{
        CUIAutomation, IUIAutomation, IUIAutomationTextPattern, IUIAutomationValuePattern,
        TextPatternRangeEndpoint_End, TextPatternRangeEndpoint_Start, TextUnit_Character,
        UIA_TextPatternId, UIA_ValuePatternId,
    };

    struct UiaCaptureRequest {
        deadline: Instant,
        reply_tx: mpsc::SyncSender<CaretContext>,
    }

    static UIA_WORKER_TX: OnceLock<Option<mpsc::SyncSender<UiaCaptureRequest>>> = OnceLock::new();

    unsafe fn query_uia_context(automation: &IUIAutomation) -> windows::core::Result<CaretContext> {
        let focused = automation.GetFocusedElement()?;

        // 1. Try IUIAutomationTextPattern (Word, Notepad, Browsers, Electron, RichEdit).
        // Only mark source as `UiaTextPattern` when selection retrieval and `GetText` both succeed,
        // so callers can safely treat an empty `selected_text` as authoritative.
        if let Ok(text_pattern) =
            focused.GetCurrentPatternAs::<IUIAutomationTextPattern>(UIA_TextPatternId)
        {
            if let Ok(selection_array) = text_pattern.GetSelection() {
                if let Ok(count) = selection_array.Length() {
                    if count > 0 {
                        if let Ok(range) = selection_array.GetElement(0) {
                            if let Ok(selected_bstr) = range.GetText(-1) {
                                let selected_text = selected_bstr.to_string();
                                let doc_range = text_pattern.DocumentRange().ok();

                                // Extract up to 100 characters before the cursor / selection
                                let mut text_before = String::new();
                                let at_doc_start = doc_range
                                    .as_ref()
                                    .and_then(|doc| {
                                        range
                                            .CompareEndpoints(
                                                TextPatternRangeEndpoint_Start,
                                                doc,
                                                TextPatternRangeEndpoint_Start,
                                            )
                                            .ok()
                                    })
                                    .map(|cmp| cmp <= 0)
                                    .unwrap_or(false);

                                if !at_doc_start {
                                    if let Ok(prefix_range) = range.Clone() {
                                        let _ = prefix_range.MoveEndpointByRange(
                                            TextPatternRangeEndpoint_End,
                                            &range,
                                            TextPatternRangeEndpoint_Start,
                                        );
                                        let _ = prefix_range.MoveEndpointByUnit(
                                            TextPatternRangeEndpoint_Start,
                                            TextUnit_Character,
                                            -100,
                                        );
                                        if let Some(ref doc) = doc_range {
                                            if let Ok(cmp) = prefix_range.CompareEndpoints(
                                                TextPatternRangeEndpoint_Start,
                                                doc,
                                                TextPatternRangeEndpoint_Start,
                                            ) {
                                                if cmp < 0 {
                                                    let _ = prefix_range.MoveEndpointByRange(
                                                        TextPatternRangeEndpoint_Start,
                                                        doc,
                                                        TextPatternRangeEndpoint_Start,
                                                    );
                                                }
                                            }
                                        }
                                        if let Ok(bstr) = prefix_range.GetText(-1) {
                                            let mut s = bstr.to_string();
                                            if !selected_text.is_empty()
                                                && s.ends_with(&selected_text)
                                            {
                                                s.truncate(s.len() - selected_text.len());
                                            }
                                            text_before = s;
                                        }
                                    }
                                }

                                // Extract up to 100 characters after the cursor / selection
                                let mut text_after = String::new();
                                let at_doc_end = doc_range
                                    .as_ref()
                                    .and_then(|doc| {
                                        range
                                            .CompareEndpoints(
                                                TextPatternRangeEndpoint_End,
                                                doc,
                                                TextPatternRangeEndpoint_End,
                                            )
                                            .ok()
                                    })
                                    .map(|cmp| cmp >= 0)
                                    .unwrap_or(false);

                                if !at_doc_end {
                                    if let Ok(suffix_range) = range.Clone() {
                                        let _ = suffix_range.MoveEndpointByRange(
                                            TextPatternRangeEndpoint_Start,
                                            &range,
                                            TextPatternRangeEndpoint_End,
                                        );
                                        let _ = suffix_range.MoveEndpointByUnit(
                                            TextPatternRangeEndpoint_End,
                                            TextUnit_Character,
                                            100,
                                        );
                                        if let Some(ref doc) = doc_range {
                                            if let Ok(cmp) = suffix_range.CompareEndpoints(
                                                TextPatternRangeEndpoint_End,
                                                doc,
                                                TextPatternRangeEndpoint_End,
                                            ) {
                                                if cmp > 0 {
                                                    let _ = suffix_range.MoveEndpointByRange(
                                                        TextPatternRangeEndpoint_End,
                                                        doc,
                                                        TextPatternRangeEndpoint_End,
                                                    );
                                                }
                                            }
                                        }
                                        if let Ok(bstr) = suffix_range.GetText(-1) {
                                            let mut s = bstr.to_string();
                                            if !selected_text.is_empty()
                                                && s.starts_with(&selected_text)
                                            {
                                                s = s[selected_text.len()..].to_string();
                                            }
                                            text_after = s;
                                        }
                                    }
                                }

                                return Ok(CaretContext {
                                    text_before,
                                    selected_text,
                                    text_after,
                                    source: CaretContextSource::UiaTextPattern,
                                });
                            }
                        }
                    }
                }
            }
        }

        // 2. Fallback to IUIAutomationValuePattern (Simple inputs, URL bars, search boxes)
        if let Ok(value_pattern) =
            focused.GetCurrentPatternAs::<IUIAutomationValuePattern>(UIA_ValuePatternId)
        {
            if let Ok(val) = value_pattern.CurrentValue() {
                let val_str = val.to_string();
                if val_str.trim().is_empty() {
                    return Ok(CaretContext {
                        text_before: String::new(),
                        selected_text: String::new(),
                        text_after: String::new(),
                        source: CaretContextSource::UiaValuePattern,
                    });
                }
            }
        }

        Ok(CaretContext::default())
    }

    fn worker_sender() -> Option<&'static mpsc::SyncSender<UiaCaptureRequest>> {
        UIA_WORKER_TX
            .get_or_init(|| {
                let (tx, rx) = mpsc::sync_channel::<UiaCaptureRequest>(4);
                let spawned = std::thread::Builder::new()
                    .name("aura-uia-caret-worker".into())
                    .spawn(move || unsafe {
                        let hr = CoInitializeEx(None, COINIT_MULTITHREADED);
                        let must_uninit = hr.is_ok();
                        let mut automation: Option<IUIAutomation> =
                            CoCreateInstance(&CUIAutomation, None, CLSCTX_INPROC_SERVER).ok();

                        while let Ok(req) = rx.recv() {
                            if Instant::now() >= req.deadline {
                                continue;
                            }
                            if automation.is_none() {
                                automation =
                                    CoCreateInstance(&CUIAutomation, None, CLSCTX_INPROC_SERVER)
                                        .ok();
                            }
                            let ctx = match automation.as_ref() {
                                Some(uia) => query_uia_context(uia).unwrap_or_default(),
                                None => CaretContext::default(),
                            };
                            let _ = req.reply_tx.send(ctx);
                        }

                        drop(automation);
                        if must_uninit {
                            CoUninitialize();
                        }
                    });
                match spawned {
                    Ok(_) => Some(tx),
                    Err(_) => None,
                }
            })
            .as_ref()
    }

    pub fn capture_with_timeout(timeout_ms: u64) -> CaretContext {
        let Some(sender) = worker_sender() else {
            return CaretContext::default();
        };
        let wait = Duration::from_millis(timeout_ms);
        let (reply_tx, reply_rx) = mpsc::sync_channel(1);
        let req = UiaCaptureRequest {
            deadline: Instant::now() + wait,
            reply_tx,
        };
        if sender.try_send(req).is_err() {
            return CaretContext::default();
        }
        reply_rx.recv_timeout(wait).unwrap_or_default()
    }
}

pub fn capture_caret_context(timeout_ms: u64) -> CaretContext {
    #[cfg(target_os = "windows")]
    {
        if timeout_ms == 0 {
            return CaretContext::default();
        }
        if let Some(ctx) = fast_win32::try_capture() {
            return ctx;
        }
        let effective_timeout_ms = timeout_ms.max(DEFAULT_CARET_CAPTURE_TIMEOUT_MS);
        uia_worker::capture_with_timeout(effective_timeout_ms)
    }

    #[cfg(not(target_os = "windows"))]
    {
        let _ = timeout_ms;
        CaretContext::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capture_timeout_safety() {
        let start = std::time::Instant::now();
        let _ctx = capture_caret_context(20);
        let elapsed = start.elapsed();
        // Must complete or abort within safety window (<100ms)
        assert!(elapsed.as_millis() < 100);
    }
}
