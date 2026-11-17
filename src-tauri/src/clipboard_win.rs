//! Private clipboard writes for transient content.
//!
//! Aura pastes by placing the transcript on the clipboard and sending Ctrl+V.
//! Without the Windows exclusion formats that temporary text also lands in the
//! Win+V history and the cloud clipboard. Restoring the previous clipboard goes
//! through the same writer: that item is already in history from its original
//! copy, and an ordinary restore would add a duplicate on every dictation.
//!
//! If the private write fails the caller falls back to an ordinary write:
//! losing clipboard privacy is better than losing the paste or the user's
//! previous clipboard content.

use std::time::Duration;

use windows_sys::Win32::Foundation::GlobalFree;
use windows_sys::Win32::System::DataExchange::{
    CloseClipboard, EmptyClipboard, OpenClipboard, RegisterClipboardFormatW, SetClipboardData,
};
use windows_sys::Win32::System::Memory::{GlobalAlloc, GlobalLock, GlobalUnlock, GMEM_MOVEABLE};

/// `CF_UNICODETEXT` and `CF_DIB` live in other windows-sys modules; the
/// numeric values are stable Win32 constants.
const CF_UNICODETEXT: u32 = 13;
const CF_DIB: u32 = 8;

/// `CanIncludeInClipboardHistory = 0` keeps the entry out of Win+V.
const EXCLUDE_HISTORY: &str = "CanIncludeInClipboardHistory";
/// `CanUploadToCloudClipboard = 0` keeps the entry off the cloud clipboard.
const EXCLUDE_CLOUD: &str = "CanUploadToCloudClipboard";
/// Presence of this format tells clipboard monitors not to process the write.
const EXCLUDE_MONITOR: &str = "ExcludeClipboardContentFromMonitorProcessing";

const RETRY_ATTEMPTS: usize = 5;
const RETRY_DELAY: Duration = Duration::from_millis(25);

/// Writes `text` as `CF_UNICODETEXT` and marks the entry as transient.
/// Returns `false` when the clipboard could not be opened or written.
pub fn set_text_private(text: &str) -> bool {
    let text_bytes = utf16_bytes(text);
    write_private(|| unsafe { set_bytes(CF_UNICODETEXT, &text_bytes) && mark_transient() })
}

/// Writes an RGBA image as `CF_DIB` and marks the entry as transient.
/// Returns `false` when the dimensions are invalid or the write failed.
pub fn set_image_private(width: usize, height: usize, rgba: &[u8]) -> bool {
    let Some(dib) = dib_from_rgba(width, height, rgba) else {
        return false;
    };
    write_private(|| unsafe { set_bytes(CF_DIB, &dib) && mark_transient() })
}

unsafe fn mark_transient() -> bool {
    set_named_dword(EXCLUDE_HISTORY, 0)
        && set_named_dword(EXCLUDE_CLOUD, 0)
        && set_named_dword(EXCLUDE_MONITOR, 0)
}

fn write_private(mut write: impl FnMut() -> bool) -> bool {
    for _ in 0..RETRY_ATTEMPTS {
        if unsafe { OpenClipboard(0) } == 0 {
            std::thread::sleep(RETRY_DELAY);
            continue;
        }
        let wrote = unsafe {
            EmptyClipboard();
            let ok = write();
            if !ok {
                // Never publish unmarked data if exclusion format registration/setting failed.
                EmptyClipboard();
            }
            ok
        };
        unsafe { CloseClipboard() };
        if wrote {
            return true;
        }
        std::thread::sleep(RETRY_DELAY);
    }
    false
}

/// Builds a bottom-up 32-bit `CF_DIB` payload from RGBA bytes.
fn dib_from_rgba(width: usize, height: usize, rgba: &[u8]) -> Option<Vec<u8>> {
    if width == 0 || height == 0 {
        return None;
    }
    let pixel_bytes = width.checked_mul(height)?.checked_mul(4)?;
    if rgba.len() != pixel_bytes {
        return None;
    }
    let width_i32 = i32::try_from(width).ok()?;
    let height_i32 = i32::try_from(height).ok()?;
    let image_bytes = u32::try_from(pixel_bytes).ok()?;

    let mut dib = Vec::with_capacity(40 + pixel_bytes);
    dib.extend_from_slice(&40u32.to_le_bytes()); // biSize
    dib.extend_from_slice(&width_i32.to_le_bytes()); // biWidth
    dib.extend_from_slice(&height_i32.to_le_bytes()); // biHeight (bottom-up)
    dib.extend_from_slice(&1u16.to_le_bytes()); // biPlanes
    dib.extend_from_slice(&32u16.to_le_bytes()); // biBitCount
    dib.extend_from_slice(&0u32.to_le_bytes()); // BI_RGB
    dib.extend_from_slice(&image_bytes.to_le_bytes()); // biSizeImage
    dib.extend_from_slice(&0i32.to_le_bytes()); // biXPelsPerMeter
    dib.extend_from_slice(&0i32.to_le_bytes()); // biYPelsPerMeter
    dib.extend_from_slice(&0u32.to_le_bytes()); // biClrUsed
    dib.extend_from_slice(&0u32.to_le_bytes()); // biClrImportant

    for row in (0..height).rev() {
        for column in 0..width {
            let index = (row * width + column) * 4;
            dib.push(rgba[index + 2]); // B
            dib.push(rgba[index + 1]); // G
            dib.push(rgba[index]); // R
            dib.push(rgba[index + 3]); // A
        }
    }
    Some(dib)
}

fn utf16_bytes(value: &str) -> Vec<u8> {
    let mut units: Vec<u16> = value.encode_utf16().collect();
    units.push(0);
    let mut bytes = Vec::with_capacity(units.len() * 2);
    for unit in units {
        bytes.extend_from_slice(&unit.to_le_bytes());
    }
    bytes
}

unsafe fn set_named_dword(name: &str, value: u32) -> bool {
    let format = register_format(name);
    if format == 0 {
        return false;
    }
    set_bytes(format, &value.to_le_bytes())
}

unsafe fn register_format(name: &str) -> u32 {
    let wide: Vec<u16> = name.encode_utf16().chain(std::iter::once(0)).collect();
    RegisterClipboardFormatW(wide.as_ptr())
}

/// Copies `bytes` into moveable global memory and hands it to the clipboard.
/// The system owns the block after a successful `SetClipboardData`, so it must
/// not be freed here; a failed call leaves it ours and frees it.
unsafe fn set_bytes(format: u32, bytes: &[u8]) -> bool {
    let handle = GlobalAlloc(GMEM_MOVEABLE, bytes.len());
    if handle.is_null() {
        return false;
    }
    let pointer = GlobalLock(handle);
    if pointer.is_null() {
        GlobalFree(handle);
        return false;
    }
    std::ptr::copy_nonoverlapping(bytes.as_ptr(), pointer.cast::<u8>(), bytes.len());
    let _ = GlobalUnlock(handle);
    if SetClipboardData(format, handle as isize) == 0 {
        GlobalFree(handle);
        return false;
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    /// The Windows clipboard is process-global; two tests writing it at the
    /// same time race on the open/empty/set sequence.
    static CLIPBOARD_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn utf16_bytes_are_nul_terminated() {
        let bytes = utf16_bytes("Aura");
        assert_eq!(bytes.len(), 10);
        assert_eq!(&bytes[8..], &[0, 0]);
    }

    #[test]
    fn utf16_bytes_preserve_non_ascii() {
        let bytes = utf16_bytes("Аура");
        assert_eq!(bytes.len(), 10);
        assert_eq!(&bytes[..2], &[0x10, 0x04]);
    }

    #[test]
    fn exclusion_formats_register() {
        assert_ne!(unsafe { register_format(EXCLUDE_HISTORY) }, 0);
        assert_ne!(unsafe { register_format(EXCLUDE_CLOUD) }, 0);
        assert_ne!(unsafe { register_format(EXCLUDE_MONITOR) }, 0);
    }

    #[test]
    fn dib_describes_a_bottom_up_32_bit_bitmap() {
        let rgba = [
            255, 0, 0, 255, 0, 255, 0, 255, // top row: red, green
            0, 0, 255, 255, 255, 255, 255, 255, // bottom row: blue, white
        ];
        let dib = dib_from_rgba(2, 2, &rgba).expect("valid image");
        assert_eq!(dib.len(), 40 + 16);
        assert_eq!(u32::from_le_bytes(dib[0..4].try_into().unwrap()), 40);
        assert_eq!(i32::from_le_bytes(dib[4..8].try_into().unwrap()), 2);
        assert_eq!(i32::from_le_bytes(dib[8..12].try_into().unwrap()), 2);
        // Bottom row first, BGR order: blue, white, then top row: red, green.
        assert_eq!(&dib[40..44], &[255, 0, 0, 255]);
        assert_eq!(&dib[44..48], &[255, 255, 255, 255]);
        assert_eq!(&dib[48..52], &[0, 0, 255, 255]);
        assert_eq!(&dib[52..56], &[0, 255, 0, 255]);
    }

    #[test]
    fn dib_rejects_invalid_dimensions() {
        assert!(dib_from_rgba(0, 2, &[]).is_none());
        assert!(dib_from_rgba(2, 2, &[0; 15]).is_none());
    }

    /// Touches the real clipboard, so it stays ignored in CI; run it locally
    /// with `cargo test -- --ignored` to prove the transcript survives a
    /// round-trip through the private writer unchanged.
    #[test]
    #[ignore = "writes to the real Windows clipboard"]
    fn private_text_round_trips() {
        let _guard = CLIPBOARD_LOCK
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let previous_text = arboard::Clipboard::new()
            .ok()
            .and_then(|mut clipboard| clipboard.get_text().ok());
        let previous_image = if previous_text.is_none() {
            arboard::Clipboard::new()
                .ok()
                .and_then(|mut clipboard| clipboard.get_image().ok())
        } else {
            None
        };

        assert!(set_text_private("Привет, Aura!\nстрока\tи эмодзи 🙂"));

        for name in [EXCLUDE_HISTORY, EXCLUDE_CLOUD, EXCLUDE_MONITOR] {
            let format = unsafe { register_format(name) };
            assert_ne!(format, 0);
            assert_ne!(
                unsafe {
                    windows_sys::Win32::System::DataExchange::IsClipboardFormatAvailable(format)
                },
                0,
                "{name} must be present on the clipboard"
            );
        }

        let read_back = arboard::Clipboard::new()
            .ok()
            .and_then(|mut clipboard| clipboard.get_text().ok());
        assert_eq!(
            read_back.as_deref(),
            Some("Привет, Aura!\nстрока\tи эмодзи 🙂")
        );

        if let Ok(mut clipboard) = arboard::Clipboard::new() {
            if let Some(text) = previous_text {
                let _ = clipboard.set_text(text);
            } else if let Some(image) = previous_image {
                let _ = clipboard.set_image(image);
            }
        }
    }

    #[test]
    #[ignore = "writes to the real Windows clipboard"]
    fn private_image_round_trips() {
        let _guard = CLIPBOARD_LOCK
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let previous_text = arboard::Clipboard::new()
            .ok()
            .and_then(|mut clipboard| clipboard.get_text().ok());
        let previous_image = if previous_text.is_none() {
            arboard::Clipboard::new()
                .ok()
                .and_then(|mut clipboard| clipboard.get_image().ok())
        } else {
            None
        };

        let rgba = vec![
            255, 0, 0, 255, 0, 255, 0, 255, // red, green
            0, 0, 255, 255, 255, 255, 255, 255, // blue, white
        ];
        assert!(set_image_private(2, 2, &rgba));

        let image = arboard::Clipboard::new()
            .ok()
            .and_then(|mut clipboard| clipboard.get_image().ok())
            .expect("image must survive the private write");
        assert_eq!((image.width, image.height), (2, 2));
        assert_eq!(&image.bytes[0..3], &[255, 0, 0]);
        assert_eq!(&image.bytes[4..7], &[0, 255, 0]);
        assert_eq!(&image.bytes[8..11], &[0, 0, 255]);
        assert_eq!(&image.bytes[12..15], &[255, 255, 255]);

        if let Ok(mut clipboard) = arboard::Clipboard::new() {
            if let Some(text) = previous_text {
                let _ = clipboard.set_text(text);
            } else if let Some(image) = previous_image {
                let _ = clipboard.set_image(image);
            }
        }
    }
}
