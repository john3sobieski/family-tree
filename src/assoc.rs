//! Ассоциация .famtree с приложением (Windows, HKCU — без прав администратора).

#[cfg(windows)]
mod imp {
    use std::os::windows::process::CommandExt;
    use std::process::Command;

    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    const PROG_ID: &str = "FamTree.File";
    const EXT_KEY: &str = r"HKCU\Software\Classes\.famtree";
    const PROG_KEY: &str = r"HKCU\Software\Classes\FamTree.File";

    #[link(name = "shell32")]
    extern "system" {
        fn SHChangeNotify(event: i32, flags: u32, a: *const core::ffi::c_void, b: *const core::ffi::c_void);
    }

    fn reg(args: &[&str]) -> Result<String, String> {
        let out = Command::new("reg")
            .args(args)
            .creation_flags(CREATE_NO_WINDOW)
            .output()
            .map_err(|e| e.to_string())?;
        if out.status.success() {
            Ok(String::from_utf8_lossy(&out.stdout).into_owned())
        } else {
            Err(String::from_utf8_lossy(&out.stderr).trim().to_string())
        }
    }

    fn set_default(key: &str, value: &str) -> Result<(), String> {
        reg(&["add", key, "/ve", "/d", value, "/f"]).map(|_| ())
    }

    /// Привязана ли расширение к текущему exe.
    pub fn is_registered() -> bool {
        let Ok(exe) = std::env::current_exe() else { return false };
        let want = format!("\"{}\" \"%1\"", exe.display());
        let cmd = format!(r"{PROG_KEY}\shell\open\command");
        match reg(&["query", &cmd, "/ve"]) {
            Ok(s) => s.contains(&want),
            Err(_) => false,
        }
    }

    pub fn register() -> Result<(), String> {
        let exe = std::env::current_exe().map_err(|e| e.to_string())?;
        let exe = exe.display().to_string();
        set_default(EXT_KEY, PROG_ID)?;
        set_default(PROG_KEY, "Семейное дерево (.famtree)")?;
        set_default(&format!(r"{PROG_KEY}\DefaultIcon"), &format!("\"{exe}\",0"))?;
        set_default(
            &format!(r"{PROG_KEY}\shell\open\command"),
            &format!("\"{exe}\" \"%1\""),
        )?;
        // Сообщить проводнику об изменении ассоциаций (SHCNE_ASSOCCHANGED).
        unsafe { SHChangeNotify(0x0800_0000, 0, std::ptr::null(), std::ptr::null()) };
        Ok(())
    }

    pub fn unregister() -> Result<(), String> {
        let _ = reg(&["delete", EXT_KEY, "/f"]);
        reg(&["delete", PROG_KEY, "/f"])?;
        unsafe { SHChangeNotify(0x0800_0000, 0, std::ptr::null(), std::ptr::null()) };
        Ok(())
    }
}

#[cfg(not(windows))]
mod imp {
    pub fn is_registered() -> bool {
        false
    }
    pub fn register() -> Result<(), String> {
        Err("только Windows".into())
    }
    pub fn unregister() -> Result<(), String> {
        Err("только Windows".into())
    }
}

pub use imp::{is_registered, register, unregister};
