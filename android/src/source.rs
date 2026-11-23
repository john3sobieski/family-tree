//! Getting a `.famtree` into the app's private folder: from the launching intent (JNI) or the last opened one.
use jni::objects::{JObject, JString, JValue};
use jni::{JNIEnv, JavaVM};
use std::error::Error;
use std::fs::{self, File};
use std::io::{self, Write};
use std::os::fd::FromRawFd;
use std::path::{Path, PathBuf};
use winit::platform::android::activity::AndroidApp;

const LAST_FILE: &str = "last.famtree";
const LAST_NAME: &str = "last_name.txt";
const INCOMING_FILE: &str = "incoming.famtree";

pub struct Opened {
    pub path: PathBuf,
    pub name: String,
    /// `true` — a fresh copy from an intent; call `commit` once it loads fine.
    pub incoming: bool,
}

/// The file to show: one handed over by the system ("Open with"), else the last opened one.
pub fn find_file(app: &AndroidApp) -> Result<Option<Opened>, String> {
    let dir = app.internal_data_path().ok_or("no private data folder")?;
    let incoming = dir.join(INCOMING_FILE);
    if let Some(name) = copy_intent_file(app, &incoming)? {
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

fn copy_intent_file(app: &AndroidApp, dest: &Path) -> Result<Option<String>, String> {
    let vm = unsafe { JavaVM::from_raw(app.vm_as_ptr().cast()) }.map_err(|e| e.to_string())?;
    let mut env = vm.attach_current_thread().map_err(|e| e.to_string())?;
    let activity = unsafe { JObject::from_raw(app.activity_as_ptr().cast()) };
    let r = copy_inner(&mut env, &activity, dest);
    if r.is_err() {
        // A pending Java exception would break every later JNI call.
        let _ = env.exception_clear();
    }
    r.map_err(|e| e.to_string())
}

fn copy_inner(env: &mut JNIEnv, activity: &JObject, dest: &Path) -> Result<Option<String>, Box<dyn Error>> {
    let intent = env.call_method(activity, "getIntent", "()Landroid/content/Intent;", &[])?.l()?;
    if intent.is_null() {
        return Ok(None);
    }
    let uri = env.call_method(&intent, "getData", "()Landroid/net/Uri;", &[])?.l()?;
    if uri.is_null() {
        return Ok(None);
    }
    let resolver = env
        .call_method(activity, "getContentResolver", "()Landroid/content/ContentResolver;", &[])?
        .l()?;
    let name = display_name(env, &resolver, &uri).unwrap_or_else(|| "FamTree".to_string());

    let mode = env.new_string("r")?;
    let pfd = env
        .call_method(
            &resolver,
            "openFileDescriptor",
            "(Landroid/net/Uri;Ljava/lang/String;)Landroid/os/ParcelFileDescriptor;",
            &[JValue::Object(&uri), JValue::Object(&mode)],
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
    Ok(Some(name))
}

/// File name shown to the user (`OpenableColumns.DISPLAY_NAME`); `None` if the provider does not tell.
fn display_name(env: &mut JNIEnv, resolver: &JObject, uri: &JObject) -> Option<String> {
    let r = display_name_inner(env, resolver, uri);
    match r {
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
