// FamTree Android viewer. Step A7: tree (same layout and cards as on desktop) + person page + search,
// system language and dark mode, CJK font, safe-area insets.
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
#[allow(dead_code)]
#[path = "../../src/person_window.rs"]
mod person_window;
#[allow(dead_code)]
#[path = "../../src/search.rs"]
mod search;
mod fonts;
mod person_screen;
mod search_screen;
mod sysenv;
#[cfg(target_os = "android")]
mod source;

#[cfg(target_os = "android")]
mod app {
    use crate::container::{self, MediaStore};
    use crate::i18n::{tr, trf};
    use crate::layout::{self, Layout, LayoutConfig};
    use crate::model::{PersonId, Tree};
    use crate::person_screen::{Event, PersonScreen};
    use crate::search_screen::Event as SearchEvent;
    use crate::source;
    use crate::view::ViewState;
    use eframe::egui;
    use jni::objects::{JObject, JString};
    use jni::JNIEnv;
    use std::sync::mpsc::{channel, Receiver, TryRecvError};
    use std::sync::atomic::{AtomicBool, Ordering};
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

    /// Text typed on the soft keyboard (hidden Java EditText -> Rust).
    static SEARCH_TEXT: Mutex<Option<String>> = Mutex::new(None);
    /// The keyboard's "Search" key was pressed.
    static SEARCH_SUBMIT: AtomicBool = AtomicBool::new(false);

    /// Registered as `FamActivity.nativeText(String)`; runs on the Java UI thread, so it only stores the string.
    extern "system" fn native_text(mut env: JNIEnv, _this: JObject, text: JString) {
        if let Ok(s) = env.get_string(&text) {
            let s: String = s.into();
            if let Ok(mut g) = SEARCH_TEXT.lock() {
                *g = Some(s);
            }
            if let Some(ctx) = CTX.get() {
                ctx.request_repaint();
            }
        }
    }

    /// Registered as `FamActivity.nativeSubmit()`: the keyboard's "Search" key.
    extern "system" fn native_submit(_env: JNIEnv, _this: JObject) {
        SEARCH_SUBMIT.store(true, Ordering::SeqCst);
        if let Some(ctx) = CTX.get() {
            ctx.request_repaint();
        }
    }

    /// True while the person page or the search is open (the system "Back" then closes it instead of leaving the app).
    static PAGE_OPEN: AtomicBool = AtomicBool::new(false);
    /// Set by `native_back`, consumed by the next UI frame.
    static BACK_REQ: AtomicBool = AtomicBool::new(false);

    /// Registered as `FamActivity.nativeBack()`: true — Rust handles "Back" itself, false — Java moves the app to the background.
    extern "system" fn native_back(_env: JNIEnv, _this: JObject) -> jni::sys::jboolean {
        if !PAGE_OPEN.load(Ordering::SeqCst) {
            return 0;
        }
        BACK_REQ.store(true, Ordering::SeqCst);
        if let Some(ctx) = CTX.get() {
            ctx.request_repaint();
        }
        1
    }

    /// Registered as `FamActivity.nativeEnv(boolean, int, int, int, int)`: system dark mode and insets (px).
    extern "system" fn native_env(
        _env: JNIEnv,
        _this: JObject,
        dark: jni::sys::jboolean,
        left: jni::sys::jint,
        top: jni::sys::jint,
        right: jni::sys::jint,
        bottom: jni::sys::jint,
    ) {
        crate::sysenv::set(dark != 0, left, top, right, bottom);
        if let Some(ctx) = CTX.get() {
            ctx.request_repaint();
        }
    }

    pub fn register(android: &AndroidApp) {
        let uri = native_uri as extern "system" fn(JNIEnv, JObject, JString) as *mut std::ffi::c_void;
        let back = native_back as extern "system" fn(JNIEnv, JObject) -> jni::sys::jboolean as *mut std::ffi::c_void;
        let text = native_text as extern "system" fn(JNIEnv, JObject, JString) as *mut std::ffi::c_void;
        let submit = native_submit as extern "system" fn(JNIEnv, JObject) as *mut std::ffi::c_void;
        let env = native_env
            as extern "system" fn(
                JNIEnv,
                JObject,
                jni::sys::jboolean,
                jni::sys::jint,
                jni::sys::jint,
                jni::sys::jint,
                jni::sys::jint,
            ) as *mut std::ffi::c_void;
        if let Err(e) = source::register_natives(android, uri, back, text, submit, env) {
            log::error!("register_natives failed: {e}");
        }
        // The insets are only known once the window exists: Java sends them (and the dark mode) right away.
        if let Err(e) = source::request_env(android) {
            log::error!("request_env failed: {e}");
        }
    }

    /// Language of the interface: the first system language we have a translation for, else English.
    pub fn system_lang(android: &AndroidApp) -> crate::i18n::Lang {
        let list = source::system_languages(android).unwrap_or_default();
        list.split(',').find_map(crate::i18n::Lang::from_code).unwrap_or(crate::i18n::Lang::En)
    }

    struct Loaded {
        tree: Tree,
        store: MediaStore,
        /// Computed in the loading thread with the desktop defaults, so the picture matches the PC.
        layout: Layout,
        cfg: LayoutConfig,
        /// The tree has Chinese/Japanese/Korean text: the CJK font is needed whatever the UI language is.
        cjk: bool,
    }

    enum State {
        Loading,
        Empty,
        Failed(String),
        Ready(Loaded),
    }

    type LoadResult = Result<Option<Loaded>, String>;

    /// How many suggestions the search page shows.
    const SEARCH_LIMIT: usize = 30;

    pub struct App {
        android: AndroidApp,
        ctx: egui::Context,
        state: State,
        rx: Option<Receiver<LoadResult>>,
        view: ViewState,
        selected: Option<PersonId>,
        /// Person page currently open (None — the tree is shown).
        person: Option<PersonId>,
        /// Persons visited before the current one (relatives are opened one from another).
        history: Vec<PersonId>,
        person_screen: PersonScreen,
        /// Last system dark mode we followed (a manual switch lasts until the system changes).
        sys_dark: bool,
        /// Search page open (it replaces the tree and the person page).
        searching: bool,
        search_text: String,
        search_hits: Vec<PersonId>,
        /// The text changed: recompute `search_hits` on the next frame.
        search_dirty: bool,
    }

    impl App {
        pub fn new(android: AndroidApp, ctx: egui::Context) -> Self {
            let _ = CTX.set(ctx.clone());
            // System theme (dark if Java cannot tell); `update` follows later changes.
            let sys = source::is_night(&android).unwrap_or(true);
            crate::sysenv::init_dark(sys);
            // A theme chosen by hand with the button wins until the system theme changes.
            let dark = source::load_theme(&android).unwrap_or(sys);
            crate::theme::set_dark(&ctx, dark);
            crate::theme::apply(&ctx);
            if crate::i18n::current() == crate::i18n::Lang::Zh {
                crate::fonts::ensure(&ctx);
            }
            let mut app = App {
                android,
                ctx,
                state: State::Loading,
                rx: None,
                view: ViewState::default(),
                selected: None,
                person: None,
                history: Vec::new(),
                person_screen: PersonScreen::default(),
                sys_dark: sys,
                searching: false,
                search_text: String::new(),
                search_hits: Vec::new(),
                search_dirty: false,
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

        fn open_search(&mut self) {
            self.searching = true;
            self.search_text.clear();
            self.search_hits.clear();
            self.search_dirty = false;
            if let Err(e) = source::show_keyboard(&self.android, "") {
                log::error!("show_keyboard failed: {e}");
            }
        }

        fn close_search(&mut self) {
            if self.searching {
                self.searching = false;
                let _ = source::hide_keyboard(&self.android);
            }
        }

        /// Closes the search, selects the person and flies the camera to them (with the pulsing highlight).
        fn pick_found(&mut self, id: PersonId) {
            self.close_search();
            self.selected = Some(id);
            self.view.fly_to(id, true);
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
        let cjk = crate::fonts::tree_has_cjk(&tree);
        if cjk {
            // Read the big font file here, not on the UI thread.
            crate::fonts::prefetch();
        }
        Ok(Some(Loaded { tree, store, layout, cfg, cjk }))
    }

    impl eframe::App for App {
        fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
            // Follow the system dark mode, then reserve the status bar / cutout / navigation bar areas.
            let sys = crate::sysenv::is_dark();
            if sys != self.sys_dark {
                self.sys_dark = sys;
                crate::theme::set_dark(ctx, sys);
                // The system decides again: forget the manual choice.
                source::save_theme(&self.android, None);
            }
            crate::sysenv::show_insets(ctx);

            let got = self.rx.as_ref().map(|rx| rx.try_recv());
            match got {
                Some(Ok(r)) => {
                    self.rx = None;
                    self.state = match r {
                        Ok(Some(l)) => {
                            if l.cjk {
                                crate::fonts::ensure(ctx);
                            }
                            self.close_search();
                            self.view.reset();
                            self.selected = None;
                            self.person = None;
                            self.history.clear();
                            self.person_screen.reset();
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

            // Typed text and the "Search" key from the soft keyboard (stale text after the search closed is dropped).
            let typed = SEARCH_TEXT.lock().ok().and_then(|mut g| g.take());
            if let Some(t) = typed {
                if self.searching && t != self.search_text {
                    self.search_text = t;
                    self.search_dirty = true;
                }
            }
            let submit = SEARCH_SUBMIT.swap(false, Ordering::SeqCst);

            let mut open_clicked = false;
            let mut theme_clicked = false;
            let mut search_clicked = false;
            let mut tapped: Option<PersonId> = None;
            let mut page_event = Event::None;
            let mut search_event = SearchEvent::None;
            let back_key = BACK_REQ.swap(false, Ordering::SeqCst);
            if let State::Ready(l) = &self.state {
                if self.searching {
                    if self.search_dirty {
                        self.search_hits = crate::search::find(&l.tree, &l.layout, &self.search_text, SEARCH_LIMIT);
                        self.search_dirty = false;
                    }
                    search_event = crate::search_screen::show(ctx, &l.tree, &self.search_text, &self.search_hits);
                } else if let Some(pid) = self.person {
                    page_event = self.person_screen.show(ctx, &l.tree, &l.store, pid);
                } else {
                    let mut fit_clicked = false;
                    let who = self
                        .selected
                        .and_then(|id| l.tree.persons.get(&id))
                        .map(|p| format!("{} {}", p.first_name.trim(), p.surname_display()).trim().to_string())
                        .unwrap_or_default();
                    egui::TopBottomPanel::top("bar").frame(crate::theme::bar_frame(10.0, 6.0)).show(ctx, |ui| {
                        ui.horizontal(|ui| {
                            let big = |t: &str| egui::Button::new(t.to_string()).min_size(egui::vec2(0.0, 44.0));
                            search_clicked = ui.add(big("🔍")).clicked();
                            open_clicked = ui.add(big(tr("Открыть"))).clicked();
                            fit_clicked = ui.add(big(tr("По размеру"))).clicked();
                            theme_clicked = crate::sysenv::theme_button(ui);
                            ui.label(who);
                        });
                    });
                    egui::CentralPanel::default().frame(egui::Frame::none()).show(ctx, |ui| {
                        self.view.highlight = self.selected;
                        tapped = self.view.show(ui, &l.tree, &l.layout, &l.cfg, &l.store);
                    });
                    if fit_clicked {
                        self.view.fitted = false;
                    }
                }
            } else {
                egui::CentralPanel::default().show(ctx, |ui| {
                    ui.add_space(8.0);
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
                            // The bundled fonts have no arrow glyph (it shows as a box).
                            ui.label(tr("Откройте файл .famtree из файлового менеджера (Открыть с помощью → FamTree).").replace('→', ">"));
                        }
                        State::Failed(e) => {
                            ui.colored_label(egui::Color32::LIGHT_RED, tr("Не удалось открыть файл"));
                            ui.label(e);
                        }
                        State::Ready(_) => {}
                    }
                });
            }
            if back_key && self.searching {
                search_event = SearchEvent::Back;
            } else if back_key && self.person.is_some() {
                page_event = Event::Back;
            }
            match search_event {
                SearchEvent::Back => self.close_search(),
                SearchEvent::Clear => {
                    self.search_text.clear();
                    self.search_hits.clear();
                    self.search_dirty = false;
                    // Also empties the Java EditText and keeps the keyboard up.
                    let _ = source::show_keyboard(&self.android, "");
                }
                SearchEvent::Focus => {
                    let _ = source::show_keyboard(&self.android, &self.search_text);
                }
                SearchEvent::Pick(id) => self.pick_found(id),
                SearchEvent::None => {}
            }
            if submit && self.searching {
                match self.search_hits.first().copied() {
                    Some(id) => self.pick_found(id),
                    None => {
                        let _ = source::hide_keyboard(&self.android);
                    }
                }
            }
            if search_clicked {
                self.open_search();
            }
            if theme_clicked {
                let dark = !crate::theme::is_dark();
                crate::theme::set_dark(ctx, dark);
                source::save_theme(&self.android, Some(dark));
            }
            // Navigation: tap on a card opens the page; "Back" returns to the previous person, then to the tree.
            if let Some(id) = tapped {
                self.selected = Some(id);
                self.person = Some(id);
                self.history.clear();
            }
            match page_event {
                Event::Back => match self.history.pop() {
                    Some(prev) => {
                        self.person = Some(prev);
                        self.selected = Some(prev);
                    }
                    None => self.person = None,
                },
                Event::Goto(rid) => {
                    if let Some(cur) = self.person {
                        self.history.push(cur);
                    }
                    self.person = Some(rid);
                    self.selected = Some(rid);
                }
                Event::None => {}
            }
            PAGE_OPEN.store(self.person.is_some() || self.searching, Ordering::SeqCst);
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
    // There is no saved language on Android: follow the system one.
    i18n::set(app::system_lang(&android));
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
