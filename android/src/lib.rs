// FamTree Android viewer. Step A3: open a .famtree (file manager, picker button) and show basic info.
// Shared code from the desktop crate (same files, no copies).
#[allow(dead_code)]
#[path = "../../src/i18n.rs"]
mod i18n;
#[allow(dead_code)]
#[path = "../../src/model.rs"]
mod model;
#[allow(dead_code)]
#[path = "../../src/container.rs"]
mod container;
#[cfg(target_os = "android")]
mod source;

#[cfg(target_os = "android")]
mod app {
    use crate::container::{self, MediaStore};
    use crate::i18n::{tr, trf};
    use crate::model::Tree;
    use crate::source;
    use eframe::egui;
    use jni::objects::{JObject, JString};
    use jni::JNIEnv;
    use std::sync::mpsc::{channel, Receiver};
    use std::sync::{Mutex, OnceLock};
    use winit::platform::android::activity::AndroidApp;

    /// URI string sent by Java (picker result or a file opened while the app runs).
    static PICKED: Mutex<Option<String>> = Mutex::new(None);
    /// Lets the Java callback wake the UI.
    static CTX: OnceLock<egui::Context> = OnceLock::new();

    /// Registered as `FamActivity.nativeUri(String)`; runs on the Java UI thread, so it only stores the string.
    extern "system" fn native_uri(mut env: JNIEnv, _this: JObject, uri: JString) {
        if let Ok(s) = env.get_string(&uri) {
            let s: String = s.into();
            if let Ok(mut g) = PICKED.lock() {
                *g = Some(s);
            }
            if let Some(ctx) = CTX.get() {
                ctx.request_repaint();
            }
        }
    }

    pub fn register(android: &AndroidApp) {
        let f = native_uri as extern "system" fn(JNIEnv, JObject, JString) as *mut std::ffi::c_void;
        if let Err(e) = source::register_natives(android, f) {
            log::error!("register_natives failed: {e}");
        }
    }

    struct Loaded {
        tree: Tree,
        _store: MediaStore,
        name: String,
    }

    enum State {
        Loading,
        Empty,
        Failed(String),
        Ready(Loaded),
    }

    type LoadResult = Result<Option<Loaded>, String>;

    pub struct App {
        android: AndroidApp,
        ctx: egui::Context,
        state: State,
        rx: Option<Receiver<LoadResult>>,
    }

    impl App {
        pub fn new(android: AndroidApp, ctx: egui::Context) -> Self {
            let _ = CTX.set(ctx.clone());
            let mut app = App { android, ctx, state: State::Loading, rx: None };
            app.start(None);
            app
        }

        /// Loads in a background thread: the picked URI, else the launching intent / last opened file.
        fn start(&mut self, picked: Option<String>) {
            let (tx, rx) = channel();
            let (android, ctx) = (self.android.clone(), self.ctx.clone());
            std::thread::spawn(move || {
                let _ = tx.send(load(&android, picked.as_deref()));
                ctx.request_repaint();
            });
            self.rx = Some(rx);
            self.state = State::Loading;
        }
    }

    fn file_error(e: String) -> String {
        trf("Ошибка файла: {err}", &[("err", e)])
    }

    fn load(android: &AndroidApp, picked: Option<&str>) -> LoadResult {
        let Some(found) = source::find_file(android, picked).map_err(file_error)? else {
            return Ok(None);
        };
        // Check the file first; only a good one becomes the "last opened".
        if let Err(e) = container::load(&found.path) {
            source::discard(&found);
            return Err(e.to_string());
        }
        let path = source::commit(android, &found).map_err(file_error)?;
        let (tree, store) = container::load(&path).map_err(|e| e.to_string())?;
        Ok(Some(Loaded { tree, _store: store, name: found.name }))
    }

    impl eframe::App for App {
        fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
            let got = self.rx.as_ref().and_then(|rx| rx.try_recv().ok());
            if let Some(r) = got {
                self.rx = None;
                self.state = match r {
                    Ok(Some(l)) => State::Ready(l),
                    Ok(None) => State::Empty,
                    Err(e) => State::Failed(e),
                };
            }
            // One load at a time: both threads would write the same temporary file.
            if self.rx.is_none() {
                let picked = PICKED.lock().ok().and_then(|mut g| g.take());
                if picked.is_some() {
                    self.start(picked);
                }
            }

            egui::CentralPanel::default().show(ctx, |ui| {
                // Temporary top margin (status bar); proper insets come in step A7.
                ui.add_space(48.0);
                ui.heading("FamTree");
                ui.add_space(12.0);
                if !matches!(self.state, State::Loading) {
                    let open = egui::Button::new(tr("Открыть")).min_size(egui::vec2(160.0, 48.0));
                    if ui.add(open).clicked() {
                        if let Err(e) = source::pick_file(&self.android) {
                            self.state = State::Failed(file_error(e));
                        }
                    }
                    ui.add_space(12.0);
                }
                match &self.state {
                    State::Loading => {
                        ui.horizontal(|ui| {
                            ui.spinner();
                            ui.label(tr("Загрузка…"));
                        });
                    }
                    State::Empty => {
                        ui.label(tr("Откройте файл .famtree из файлового менеджера (Открыть с помощью → FamTree)."));
                    }
                    State::Failed(e) => {
                        ui.colored_label(egui::Color32::LIGHT_RED, tr("Не удалось открыть файл"));
                        ui.label(e);
                    }
                    State::Ready(l) => {
                        ui.label(trf("Файл: {name}", &[("name", l.name.clone())]));
                        ui.label(trf("Людей в дереве: {n}", &[("n", l.tree.persons.len().to_string())]));
                    }
                }
            });
        }
    }
}

// Entry point called by the Android runtime (our FamActivity extends NativeActivity).
#[cfg(target_os = "android")]
#[no_mangle]
fn android_main(android: winit::platform::android::activity::AndroidApp) {
    // eframe 0.29 takes the Android app through the event loop builder hook.
    use winit::platform::android::EventLoopBuilderExtAndroid;
    android_logger::init_once(
        android_logger::Config::default().with_max_level(log::LevelFilter::Info),
    );
    i18n::init();
    app::register(&android);
    let for_loop = android.clone();
    let options = eframe::NativeOptions {
        event_loop_builder: Some(Box::new(move |builder| {
            builder.with_android_app(for_loop);
        })),
        ..Default::default()
    };
    if let Err(e) = eframe::run_native(
        "FamTree",
        options,
        Box::new(move |cc| Ok(Box::new(app::App::new(android, cc.egui_ctx.clone())))),
    ) {
        log::error!("eframe failed: {e}");
    }
}
