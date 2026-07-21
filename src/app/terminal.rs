/// Detect terminal size. Returns (columns, rows).
pub fn detect_terminal_size() -> (u16, u16) {
    detect_terminal_size_fallback()
}

fn detect_terminal_size_fallback() -> (u16, u16) {
    #[cfg(windows)]
    {
        let r = detect_terminal_size_windows();
        if r != (0, 0) {
            return r;
        }
    }
    // Each grid cell is 10x10 characters. 8 cells wide = 80+ chars for map area
    (200, 80)
}

#[cfg(windows)]
fn detect_terminal_size_windows() -> (u16, u16) {
    type HANDLE = *mut std::ffi::c_void;
    type BOOL = i32;
    const STD_OUTPUT_HANDLE: u32 = 0xFFFFFFF5u32;

    #[repr(C)]
    struct COORD {
        x: i16,
        y: i16,
    }
    #[repr(C)]
    struct SMALL_RECT {
        left: i16,
        top: i16,
        right: i16,
        bottom: i16,
    }
    #[repr(C)]
    struct CONSOLE_SCREEN_BUFFER_INFO {
        dw_size: COORD,
        dw_cursor_position: COORD,
        w_attributes: u16,
        sr_window: SMALL_RECT,
        dw_maximum_window_size: COORD,
    }

    extern "system" {
        fn GetStdHandle(n: u32) -> HANDLE;
        fn GetConsoleScreenBufferInfo(h: HANDLE, lp: *mut CONSOLE_SCREEN_BUFFER_INFO) -> BOOL;
    }

    unsafe {
        let h = GetStdHandle(STD_OUTPUT_HANDLE);
        if h.is_null() || h == std::ptr::null_mut() {
            return (0, 0);
        }
        let mut info: CONSOLE_SCREEN_BUFFER_INFO = std::mem::zeroed();
        if GetConsoleScreenBufferInfo(h, &mut info) == 0 {
            return (0, 0);
        }
        let cols = info.sr_window.right - info.sr_window.left + 1;
        let rows = info.sr_window.bottom - info.sr_window.top + 1;
        if cols > 0 && rows > 0 {
            (cols as u16, rows as u16)
        } else {
            (0, 0)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_terminal_size() {
        let (c, r) = detect_terminal_size();
        assert!(c >= 20 && r >= 10);
    }
}
