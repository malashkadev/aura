use crate::smart_insertion::{CaretContext, CaretContextSource};

#[cfg(target_os = "windows")]
fn capture_caret_context_internal() -> CaretContext {
    use windows::Win32::System::Com::{
        CoCreateInstance, CoInitializeEx, CoUninitialize, CLSCTX_INPROC_SERVER,
        COINIT_MULTITHREADED,
    };
    use windows::Win32::UI::Accessibility::{
        CUIAutomation, IUIAutomation, IUIAutomationTextPattern, IUIAutomationValuePattern,
        TextPatternRangeEndpoint_End, TextPatternRangeEndpoint_Start, TextUnit_Character,
        UIA_TextPatternId, UIA_ValuePatternId,
    };

    unsafe {
        let hr = CoInitializeEx(None, COINIT_MULTITHREADED);
        let must_uninit = hr.is_ok();

        let result = (|| -> windows::core::Result<CaretContext> {
            let automation: IUIAutomation =
                CoCreateInstance(&CUIAutomation, None, CLSCTX_INPROC_SERVER)?;
            let focused = automation.GetFocusedElement()?;

            // 1. Try IUIAutomationTextPattern (Word, Notepad, Browsers, RichEdit)
            if let Ok(text_pattern) =
                focused.GetCurrentPatternAs::<IUIAutomationTextPattern>(UIA_TextPatternId)
            {
                if let Ok(selection_array) = text_pattern.GetSelection() {
                    if let Ok(count) = selection_array.Length() {
                        if count > 0 {
                            if let Ok(range) = selection_array.GetElement(0) {
                                let selected_text = range
                                    .GetText(-1)
                                    .map(|b| b.to_string())
                                    .unwrap_or_default();

                                // Extract up to 100 characters before the cursor / selection
                                let mut text_before = String::new();
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
                                    if let Ok(bstr) = prefix_range.GetText(-1) {
                                        let mut s = bstr.to_string();
                                        if !selected_text.is_empty() && s.ends_with(&selected_text)
                                        {
                                            s.truncate(s.len() - selected_text.len());
                                        }
                                        text_before = s;
                                    }
                                }

                                // Extract up to 100 characters after the cursor / selection
                                let mut text_after = String::new();
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
        })();

        if must_uninit {
            CoUninitialize();
        }

        result.unwrap_or_default()
    }
}

pub fn capture_caret_context(timeout_ms: u64) -> CaretContext {
    #[cfg(target_os = "windows")]
    {
        let (tx, rx) = std::sync::mpsc::channel();
        let spawn_result = std::thread::Builder::new()
            .name("aura-caret-capture".into())
            .spawn(move || {
                let ctx = capture_caret_context_internal();
                let _ = tx.send(ctx);
            });

        if spawn_result.is_err() {
            return CaretContext::default();
        }

        rx.recv_timeout(std::time::Duration::from_millis(timeout_ms))
            .unwrap_or_default()
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
