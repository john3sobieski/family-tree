//! Getting a `.famtree` into the app's private folder (JNI + files): from the launching intent,
//! from the file picker / a file opened while running (URI string), or the last opened one.
use jni::objects::{JObject, JString, JValue};
use jni::{JNIEnv, JavaVM, NativeMethod};
use std::error::Error;
use std::ffi::c_void;
use std::fs::{self, File};
use std::io::{self, Write};
use std::os::fd::FromRawFd;
use std::path::{Path, PathBuf};
use winit::platform::android::activity::AndroidApp;

const LAST_FILE: &str = "last.famtree";
const LAST_NAME: &str = "last_name.txt";
const INCOMING_FILE: &str = "incoming.famtree";
const THEME_FILE: &str = "theme.txt";

pub struct Opened {
    pub path: PathBuf,
    pub name: String,
    /// `true` — a fresh copy from the system; call `commit` once it loads fine.
    pub incoming: bool,
}

/// The file to show: `picked` (URI from the picker / a new intent), else the launching intent's file,
/// else the last opened one.
pub fn find_file(app: &AndroidApp, picked: Option<&str>) -> Result<Option<Opened>, String> {
    let dir = app.internal_data_path().ok_or("no private data folder")?;
    let incoming = dir.join(INCOMING_FILE);
    let name = match picked {
        Some(uri) => Some(copy_picked(app, uri, &incoming)?),
        None => copy_intent_file(app, &incoming)?,
    };
    if let Some(name) = name {
        return Ok(Some(Opened { path: incoming, name, incoming: true }));
    }
    let last = dir.join(LAST_FILE);
    if last.is_file() {
        let name = fs::read_to_string(dir.join(LAST_NAME)).unwrap_or_else(|_| LAST_FILE.to_string());
        return Ok(Some(Opened { path: last, name, incoming: false }));
    }
    Ok(None)
}

/// Makes the incoming file the "last opened" one. Returns the path to load from.
pub fn commit(app: &AndroidApp, f: &Opened) -> Result<PathBuf, String> {
    if !f.incoming {
        return Ok(f.path.clone());
    }
    let dir = app.internal_data_path().ok_or("no private data folder")?;
    let last = dir.join(LAST_FILE);
    fs::rename(&f.path, &last).map_err(|e| e.to_string())?;
    let _ = fs::write(dir.join(LAST_NAME), &f.name);
    Ok(last)
}

/// A broken incoming file must not replace the last good one.
pub fn discard(f: &Opened) {
    if f.incoming {
        let _ = fs::remove_file(&f.path);
    }
}

/// Opens the system file picker (Java `FamActivity.pickFile`).
pub fn pick_file(app: &AndroidApp) -> Result<(), String> {
    with_env(app, |env, activity| {
        env.call_method(activity, "pickFile", "()V", &[])?;
        Ok(())
    })
}

/// Shows the soft keyboard (Java `FamActivity.showKeyboard`); `text` becomes the content of its hidden input.
pub fn show_keyboard(app: &AndroidApp, text: &str) -> Result<(), String> {
    with_env(app, |env, activity| {
        let s = env.new_string(text)?;
        env.call_method(activity, "showKeyboard", "(Ljava/lang/String;)V", &[JValue::Object(&s)])?;
        Ok(())
    })
}

/// Hides the soft keyboard (Java `FamActivity.hideKeyboard`).
pub fn hide_keyboard(app: &AndroidApp) -> Result<(), String> {
    with_env(app, |env, activity| {
        env.call_method(activity, "hideKeyboard", "()V", &[])?;
        Ok(())
    })
}

/// Theme chosen by hand with the sun/moon button: `Some(true)` dark, `Some(false)` light, `None` — follow the system.
pub fn load_theme(app: &AndroidApp) -> Option<bool> {
    let dir = app.internal_data_path()?;
    match fs::read_to_string(dir.join(THEME_FILE)).ok()?.trim() {
        "dark" => Some(true),
        "light" => Some(false),
        _ => None,
    }
}

/// Remembers the manual choice (`None` forgets it). Errors are ignored: the theme is a convenience.
pub fn save_theme(app: &AndroidApp, dark: Option<bool>) {
    let Some(dir) = app.internal_data_path() else { return };
    let file = dir.join(THEME_FILE);
    match dark {
        Some(d) => {
            let _ = fs::write(file, if d { "dark" } else { "light" });
        }
        None => {
            let _ = fs::remove_file(file);
        }
    }
}

/// System UI languages in priority order, comma separated (Java `FamActivity.languages`), e.g. "ru,en".
pub fn system_languages(app: &AndroidApp) -> Result<String, String> {
    with_env(app, |env, activity| {
        let s = env.call_method(activity, "languages", "()Ljava/lang/String;", &[])?.l()?;
        let js = JString::from(s);
        let text: String = env.get_string(&js)?.into();
        Ok(text)
    })
}

/// True if the system is in dark mode (Java `FamActivity.isNight`).
pub fn is_night(app: &AndroidApp) -> Result<bool, String> {
    with_env(app, |env, activity| Ok(env.call_method(activity, "isNight", "()Z", &[])?.z()?))
}

/// Asks Java to send the current dark mode and insets through `nativeEnv` (Java `FamActivity.requestEnv`).
pub fn request_env(app: &AndroidApp) -> Result<(), String> {
    with_env(app, |env, activity| {
        env.call_method(activity, "requestEnv", "()V", &[])?;
        Ok(())
    })
}

/// Tells Java which Rust functions implement the `native*` methods of `FamActivity`.
pub fn register_natives(
    app: &AndroidApp,
    uri_fn: *mut c_void,
    back_fn: *mut c_void,
    text_fn: *mut c_void,
    submit_fn: *mut c_void,
    env_fn: *mut c_void,
) -> Result<(), String> {
    with_env(app, |env, activity| {
        let class = env.get_object_class(activity)?;
        env.register_native_methods(
            &class,
            &[
                NativeMethod { name: "nativeUri".into(), sig: "(Ljava/lang/String;)V".into(), fn_ptr: uri_fn },
                NativeMethod { name: "nativeBack".into(), sig: "()Z".into(), fn_ptr: back_fn },
                NativeMethod { name: "nativeText".into(), sig: "(Ljava/lang/String;)V".into(), fn_ptr: text_fn },
                NativeMethod { name: "nativeSubmit".into(), sig: "()V".into(), fn_ptr: submit_fn },
                NativeMethod { name: "nativeEnv".into(), sig: "(ZIIII)V".into(), fn_ptr: env_fn },
            ],
        )?;
        Ok(())
    })
}

/// Runs `f` with an attached JNI environment and the activity object. Clears a pending Java exception on error.
fn with_env<T>(
    app: &AndroidApp,
    f: impl FnOnce(&mut JNIEnv, &JObject) -> Result<T, Box<dyn Error>>,
) -> Result<T, String> {
    let vm = unsafe { JavaVM::from_raw(app.vm_as_ptr().cast()) }.map_err(|e| e.to_string())?;
    let mut env = vm.attach_current_thread().map_err(|e| e.to_string())?;
    let activity = unsafe { JObject::from_raw(app.activity_as_ptr().cast()) };
    let r = f(&mut *env, &activity);
    if r.is_err() {
        // A pending Java exception would break every later JNI call.
        let _ = env.exception_clear();
    }
    r.map_err(|e| e.to_string())
}

fn copy_picked(app: &AndroidApp, uri: &str, dest: &Path) -> Result<String, String> {
    with_env(app, |env, activity| {
        let text = env.new_string(uri)?;
        let uri = env
            .call_static_method(
                "android/net/Uri",
                "parse",
                "(Ljava/lang/String;)Landroid/net/Uri;",
                &[JValue::Object(&text)],
            )?
            .l()?;
        copy_uri(env, activity, &uri, dest)
    })
}

fn copy_intent_file(app: &AndroidApp, dest: &Path) -> Result<Option<String>, String> {
    with_env(app, |env, activity| {
        let intent = env.call_method(activity, "getIntent", "()Landroid/content/Intent;", &[])?.l()?;
        if intent.is_null() {
            return Ok(None);
        }
        let uri = env.call_method(&intent, "getData", "()Landroid/net/Uri;", &[])?.l()?;
        if uri.is_null() {
            return Ok(None);
        }
        Ok(Some(copy_uri(env, activity, &uri, dest)?))
    })
}

/// Copies the content behind `uri` to `dest` (via a temporary file). Returns the file's display name.
fn copy_uri(env: &mut JNIEnv, activity: &JObject, uri: &JObject, dest: &Path) -> Result<String, Box<dyn Error>> {
    let resolver = env
        .call_method(activity, "getContentResolver", "()Landroid/content/ContentResolver;", &[])?
        .l()?;
    let name = display_name(env, &resolver, uri).unwrap_or_else(|| "FamTree".to_string());

    let mode = env.new_string("r")?;
    let pfd = env
        .call_method(
            &resolver,
            "openFileDescriptor",
            "(Landroid/net/Uri;Ljava/lang/String;)Landroid/os/ParcelFileDescriptor;",
            &[JValue::Object(uri), JValue::Object(&mode)],
        )?
        .l()?;
    if pfd.is_null() {
        return Err("the system could not open the file".into());
    }
    // detachFd hands the descriptor over to us; File closes it.
    let fd = env.call_method(&pfd, "detachFd", "()I", &[])?.i()?;
    let mut src = unsafe { File::from_raw_fd(fd) };

    let tmp = dest.with_extension("tmp");
    {
        let mut out = File::create(&tmp)?;
        io::copy(&mut src, &mut out)?;
        out.flush()?;
    }
    fs::rename(&tmp, dest)?;
    Ok(name)
}

/// File name shown to the user (`OpenableColumns.DISPLAY_NAME`); `None` if the provider does not tell.
fn display_name(env: &mut JNIEnv, resolver: &JObject, uri: &JObject) -> Option<String> {
    match display_name_inner(env, resolver, uri) {
        Ok(v) => v,
        Err(_) => {
            let _ = env.exception_clear();
            None
        }
    }
}

fn display_name_inner(env: &mut JNIEnv, resolver: &JObject, uri: &JObject) -> Result<Option<String>, jni::errors::Error> {
    let null = JObject::null();
    let cursor = env
        .call_method(
            resolver,
            "query",
            "(Landroid/net/Uri;[Ljava/lang/String;Ljava/lang/String;[Ljava/lang/String;Ljava/lang/String;)Landroid/database/Cursor;",
            &[
                JValue::Object(uri),
                JValue::Object(&null),
                JValue::Object(&null),
                JValue::Object(&null),
                JValue::Object(&null),
            ],
        )?
        .l()?;
    if cursor.is_null() {
        return Ok(None);
    }
    let mut result = None;
    if env.call_method(&cursor, "moveToFirst", "()Z", &[])?.z()? {
        let column = env.new_string("_display_name")?;
        let idx = env
            .call_method(&cursor, "getColumnIndex", "(Ljava/lang/String;)I", &[JValue::Object(&column)])?
            .i()?;
        if idx >= 0 {
            let s = env.call_method(&cursor, "getString", "(I)Ljava/lang/String;", &[JValue::Int(idx)])?.l()?;
            if !s.is_null() {
                let js = JString::from(s);
                let text: String = env.get_string(&js)?.into();
                result = Some(text);
            }
        }
    }
    env.call_method(&cursor, "close", "()V", &[])?;
    Ok(result)
}
