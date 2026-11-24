// FamTree Android viewer. Step A4: open a .famtree and draw the tree (same layout and cards as on desktop).
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
#[allow(dead_code)]
#[path = "../../src/countries.rs"]
mod countries;
#[allow(dead_code)]
#[path = "../../src/flags.rs"]
mod flags;
#[allow(dead_code)]
#[path = "../../src/layout.rs"]
mod layout;
#[allow(dead_code)]
#[path = "../../src/platform.rs"]
mod platform;
#[allow(dead_code)]
#[path = "../../src/theme.rs"]
mod theme;
#[allow(dead_code)]
#[path = "../../src/view.rs"]
mod view;
#[cfg(target_os = "android")]
mod source;

#[cfg(target_os = "android")]
mod app {
    use crate::container::{self, MediaStore};
    use crate::i18n::{tr, trf};
    use crate::layout::{self, Layout, LayoutConfig};
    use crate::model::{PersonId, Tree};
    use crate::source;
    use crate::view::ViewState;
    use eframe::egui;
    use jni::objects::{JObject, JString};
    use jni::JNIEnv;
    use std::sync::mpsc::{channel, Receiver, TryRecvError};
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
        store: MediaStore,
        name: String,
        /// Computed in the loading thread with the desktop defaults, so the picture matches the PC.
        layout: Layout,
        cfg: LayoutConfig,
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
        view: ViewState,
        selected: Option<PersonId>,
    }

    impl App {
        pub fn new(android: AndroidApp, ctx: egui::Context) -> Self {
            let _ = CTX.set(ctx.clone());
            crate::theme::apply(&ctx);
            let mut app = App {
                android,
                ctx,
                state: State::Loading,
                rx: None,
                view: ViewState::default(),
                selected: None,
            };
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
        let cfg = LayoutConfig::default();
        let layout = layout::compute(&tree, &cfg);
        Ok(Some(Loaded { tree, store, name: found.name, layout, cfg }))
    }

    impl eframe::App for App {
        fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
            let got = self.rx.as_ref().map(|rx| rx.try_recv());
            match got {
                Some(Ok(r)) => {
                    self.rx = None;
                    self.state = match r {
                        Ok(Some(l)) => {
                            self.view.reset();
                            self.selected = None;
                            State::Ready(l)
                        }
                        Ok(None) => State::Empty,
                        Err(e) => State::Failed(e),
                    };
                }
                // The loading thread died (panic): do not stay on the spinner forever.
                Some(Err(TryRecvError::Disconnected)) => {
                    self.rx = None;
                    self.state = State::Failed(tr("Не удалось открыть файл").to_string());
                }
                _ => {}
            }
            // One load at a time: both threads would write the same temporary file.
            if self.rx.is_none() {
                let picked = PICKED.lock().ok().and_then(|mut g| g.take());
                if picked.is_some() {
                    self.start(picked);
                }
            }

            let mut open_clicked = false;
            if let State::Ready(l) = &self.state {
                let mut fit_clicked = false;
                let who = self
                    .selected
                    .and_then(|id| l.tree.persons.get(&id))
                    .map(|p| format!("{} {}", p.first_name.trim(), p.surname_display()).trim().to_string())
                    .unwrap_or_default();
                egui::TopBottomPanel::top("bar").frame(crate::theme::bar_frame(10.0, 6.0)).show(ctx, |ui| {
                    // Temporary top margin (status bar); proper insets come in step A7.
                    ui.add_space(42.0);
                    ui.horizontal(|ui| {
                        let big = |t: &str| egui::Button::new(t.to_string()).min_size(egui::vec2(0.0, 44.0));
                        open_clicked = ui.add(big(tr("Открыть"))).clicked();
                        fit_clicked = ui.add(big(tr("По размеру"))).clicked();
                        ui.label(who);
                    });
                });
                egui::CentralPanel::default().frame(egui::Frame::none()).show(ctx, |ui| {
                    self.view.highlight = self.selected;
                    if let Some(id) = self.view.show(ui, &l.tree, &l.layout, &l.cfg, &l.store) {
                        self.selected = Some(id);
                    }
                });
                if fit_clicked {
                    self.view.fitted = false;
                }
            } else {
                egui::CentralPanel::default().show(ctx, |ui| {
                    // Temporary top margin (status bar); proper insets come in step A7.
                    ui.add_space(48.0);
                    ui.heading("FamTree");
                    ui.add_space(12.0);
                    if !matches!(self.state, State::Loading) {
                        let open = egui::Button::new(tr("Открыть")).min_size(egui::vec2(160.0, 48.0));
                        open_clicked = ui.add(open).clicked();
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
                        State::Ready(_) => {}
                    }
                });
            }
            if open_clicked {
                if let Err(e) = source::pick_file(&self.android) {
                    self.state = State::Failed(file_error(e));
                }
            }
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
    theme::init();
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
