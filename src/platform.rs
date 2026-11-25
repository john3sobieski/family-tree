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

    #[link(name = "user32")]
    extern "system" {
        fn GetKeyboardLayout(thread: u32) -> isize;
        fn MapVirtualKeyExW(code: u32, map_type: u32, hkl: isize) -> u32;
        fn ToUnicodeEx(vk: u32, scan: u32, state: *const u8, buf: *mut u16, len: i32, flags: u32, hkl: isize) -> i32;
        fn GetKeyState(vk: i32) -> i16;
    }

    /// Scan code (PS/2 set 1) of a letter/digit key by its physical position.
    fn scancode(k: eframe::egui::Key) -> Option<u32> {
        use eframe::egui::Key::*;
        Some(match k {
            Num1 => 0x02, Num2 => 0x03, Num3 => 0x04, Num4 => 0x05, Num5 => 0x06,
            Num6 => 0x07, Num7 => 0x08, Num8 => 0x09, Num9 => 0x0A, Num0 => 0x0B,
            Q => 0x10, W => 0x11, E => 0x12, R => 0x13, T => 0x14,
            Y => 0x15, U => 0x16, I => 0x17, O => 0x18, P => 0x19,
            A => 0x1E, S => 0x1F, D => 0x20, F => 0x21, G => 0x22,
            H => 0x23, J => 0x24, K => 0x25, L => 0x26,
            Z => 0x2C, X => 0x2D, C => 0x2E, V => 0x2F, B => 0x30, N => 0x31, M => 0x32,
            _ => return None,
        })
    }

    /// Character produced by AltGr + the key in the current keyboard layout.
    fn altgr_text(scan: u32, shift: bool) -> Option<String> {
        unsafe {
            let hkl = GetKeyboardLayout(0);
            let vk = MapVirtualKeyExW(scan, 3, hkl); // MAPVK_VSC_TO_VK_EX
            if vk == 0 {
                return None;
            }
            let mut state = [0u8; 256];
            if shift {
                state[0x10] = 0x80; // VK_SHIFT
            }
            state[0x11] = 0x80; // VK_CONTROL
            state[0x12] = 0x80; // VK_MENU (Ctrl+Alt = AltGr)
            if GetKeyState(0x14) & 1 != 0 {
                state[0x14] = 1; // Caps Lock is on
            }
            let mut buf = [0u16; 8];
            // flag 4: do not change the dead-key state of the layout
            let n = ToUnicodeEx(vk, scan, state.as_ptr(), buf.as_mut_ptr(), buf.len() as i32, 4, hkl);
            if n <= 0 {
                return None;
            }
            let s = String::from_utf16(&buf[..n as usize]).ok()?;
            s.chars().all(|c| !c.is_control()).then_some(s)
        }
    }

    /// AltGr arrives as Ctrl+Alt, and egui drops text typed with Ctrl held (it treats it as a shortcut).
    /// For such a key press, when no text event follows it, add the character the layout gives.
    pub fn fix_altgr(ctx: &eframe::egui::Context) {
        use eframe::egui::Event;
        let extra: Vec<(usize, String)> = ctx.input(|i| {
            let mut out = Vec::new();
            for (n, ev) in i.events.iter().enumerate() {
                let Event::Key { key, physical_key, pressed: true, modifiers, .. } = ev else { continue };
                if !(modifiers.ctrl && modifiers.alt) || matches!(i.events.get(n + 1), Some(Event::Text(_))) {
                    continue;
                }
                let Some(scan) = scancode((*physical_key).unwrap_or(*key)).or_else(|| scancode(*key)) else { continue };
                if let Some(text) = altgr_text(scan, modifiers.shift) {
                    out.push((n, text));
                }
            }
            out
        });
        if !extra.is_empty() {
            ctx.input_mut(|i| {
                for (n, text) in extra.into_iter().rev() {
                    i.events.insert(n + 1, Event::Text(text));
                }
            });
        }
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

    pub fn fix_altgr(_ctx: &eframe::egui::Context) {}

    pub fn system_root() -> std::path::PathBuf {
        std::path::PathBuf::from("/")
    }
}

/// Call once from the eframe creator closure.
pub fn init_window(cc: &CreationContext<'_>) {
    imp::init_window(cc);
}

/// Windows: makes AltGr characters (e.g. Polish ł) typeable. No-op elsewhere.
#[allow(dead_code)]
pub fn fix_altgr(ctx: &eframe::egui::Context) {
    imp::fix_altgr(ctx);
}

/// True when DWM accepted rounded corners (Windows 11).
pub fn corners_rounded() -> bool {
    imp::corners_rounded()
}

/// `<Windows>\Fonts\<name>`.
pub fn system_font(name: &str) -> std::path::PathBuf {
    imp::system_root().join("Fonts").join(name)
}
