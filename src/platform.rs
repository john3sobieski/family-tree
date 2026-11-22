//! Windows 10/11 differences in one place; the same exe runs on both.

use eframe::CreationContext;

#[cfg(windows)]
mod imp {
    use std::sync::atomic::{AtomicBool, Ordering};

    static ROUNDED: AtomicBool = AtomicBool::new(false);

    #[link(name = "dwmapi")]
    extern "system" {
        fn DwmSetWindowAttribute(hwnd: isize, attr: u32, value: *const core::ffi::c_void, size: u32) -> i32;
    }

    const DWMWA_WINDOW_CORNER_PREFERENCE: u32 = 33;
    const DWMWCP_ROUND: u32 = 2;

    /// Asks DWM for rounded corners. Only Windows 11 accepts it; Windows 10 returns an error.
    pub fn init_window(cc: &eframe::CreationContext<'_>) {
        use raw_window_handle::{HasWindowHandle, RawWindowHandle};
        let Ok(handle) = cc.window_handle() else { return };
        let RawWindowHandle::Win32(h) = handle.as_raw() else { return };
        let pref: u32 = DWMWCP_ROUND;
        let hr = unsafe {
            DwmSetWindowAttribute(
                h.hwnd.get(),
                DWMWA_WINDOW_CORNER_PREFERENCE,
                &pref as *const u32 as *const core::ffi::c_void,
                std::mem::size_of::<u32>() as u32,
            )
        };
        ROUNDED.store(hr == 0, Ordering::Relaxed);
    }

    pub fn corners_rounded() -> bool {
        ROUNDED.load(Ordering::Relaxed)
    }

    /// Windows directory (C: is not guaranteed).
    pub fn system_root() -> std::path::PathBuf {
        std::env::var_os("SystemRoot")
            .or_else(|| std::env::var_os("windir"))
            .map(Into::into)
            .unwrap_or_else(|| r"C:\Windows".into())
    }
}

#[cfg(not(windows))]
mod imp {
    pub fn init_window(_cc: &eframe::CreationContext<'_>) {}

    pub fn corners_rounded() -> bool {
        false
    }

    pub fn system_root() -> std::path::PathBuf {
        std::path::PathBuf::from("/")
    }
}

/// Call once from the eframe creator closure.
pub fn init_window(cc: &CreationContext<'_>) {
    imp::init_window(cc);
}

/// True when DWM accepted rounded corners (Windows 11).
pub fn corners_rounded() -> bool {
    imp::corners_rounded()
}

/// `<Windows>\Fonts\<name>`.
pub fn system_font(name: &str) -> std::path::PathBuf {
    imp::system_root().join("Fonts").join(name)
}
