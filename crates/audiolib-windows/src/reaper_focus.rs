// crates/audiolib-windows/src/reaper_focus.rs
//! Opt-in focus of the REAPER window that acknowledged an explicit user send.

fn window_address(value: &str) -> Result<usize, String> {
    let value = value
        .strip_prefix("userdata:")
        .ok_or("REAPER returned an unsupported window handle")?
        .trim();
    let value = value.strip_prefix("0x").unwrap_or(value);
    if value.is_empty() || !value.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err("Invalid REAPER window handle".into());
    }
    let address = usize::from_str_radix(value, 16).map_err(|_| "Invalid REAPER window handle")?;
    if address == 0 {
        return Err("REAPER window is unavailable".into());
    }
    Ok(address)
}

/// Activate only an acknowledged REAPER main window, rechecking identity first.
/// Windows may deny foreground activation; the audio insertion remains successful.
pub fn focus_reaper(window: &str) -> Result<bool, String> {
    let address = window_address(window)?;
    #[cfg(windows)]
    {
        use std::ffi::c_void;
        #[link(name = "user32")]
        unsafe extern "system" {
            fn IsWindow(window: *mut c_void) -> i32;
            fn IsIconic(window: *mut c_void) -> i32;
            fn GetClassNameW(window: *mut c_void, class: *mut u16, capacity: i32) -> i32;
            fn ShowWindow(window: *mut c_void, command: i32) -> i32;
            fn SetForegroundWindow(window: *mut c_void) -> i32;
            fn GetForegroundWindow() -> *mut c_void;
        }
        let window = address as *mut c_void;
        let mut class = [0_u16; 128];
        // SAFETY: The handle is treated as untrusted. Win32 validates it; the
        // writable class buffer has the advertised capacity and outlives the call.
        let length = unsafe {
            if IsWindow(window) == 0 {
                return Ok(false);
            }
            GetClassNameW(window, class.as_mut_ptr(), class.len() as i32)
        };
        if length <= 0 || String::from_utf16_lossy(&class[..length as usize]) != "REAPERwnd" {
            return Err("The acknowledged window is no longer a REAPER main window".into());
        }
        // SAFETY: Rechecked REAPER HWND, no borrowed pointers. These normal Win32
        // activation calls run only after an explicit user send with focus enabled.
        Ok(unsafe {
            if IsIconic(window) != 0 {
                ShowWindow(window, 9);
            }
            SetForegroundWindow(window);
            GetForegroundWindow() == window
        })
    }
    #[cfg(not(windows))]
    {
        let _ = address;
        Err("REAPER Auto-Focus currently requires Windows".into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parses_lua_pointer_without_calling_any_window_api() {
        assert_eq!(
            window_address("userdata: 000000000123abcd").unwrap(),
            0x123abcd
        );
        assert_eq!(window_address("userdata: 0x123abcd").unwrap(), 0x123abcd);
        for invalid in [
            "0x123abcd",
            "userdata: 0",
            "userdata: -1",
            "userdata: 123xyz",
            "userdata: ",
        ] {
            assert!(window_address(invalid).is_err());
        }
    }
}
