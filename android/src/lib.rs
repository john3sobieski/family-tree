// FamTree Android viewer. Step A3: open a .famtree handed over by the system and show basic info.
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

use i18n::{tr, trf};

#[cfg(target_os = "android")]
mod app {
    use super::*;
    use crate::container::{self, MediaStore};
    use crate::model::Tree;
    use eframe::egui;
    use std::sync::mpsc::{channel, Receiver};
    use winit::platform::android::activity::AndroidApp;

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
        state: State,
        rx: Receiver<LoadResult>,
    }

    impl App {
        pub fn new(android: AndroidApp, ctx: egui::Context) -> Self {
            let (tx, rx) = channel();
            std::thread::spawn(move || {
                let _ = tx.send(load(&android));
                ctx.request_repaint();
            });
            App { state: State::Loading, rx }
        }
    }

    fn file_error(e: String) -> String {
        trf("Ошибка файла: {err}", &[("err", e)])
    }

    fn load(android: &AndroidApp) -> LoadResult {
        let Some(found) = source::find_file(android).map_err(file_error)? else {
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
            if let Ok(r) = self.rx.try_recv() {
                self.state = match r {
                    Ok(Some(l)) => State::Ready(l),
                    Ok(None) => State::Empty,
                    Err(e) => State::Failed(e),
                };
            }
            egui::CentralPanel::default().show(ctx, |ui| {
                // Temporary top margin (status bar); proper insets come in step A7.
                ui.add_space(48.0);
                ui.heading("FamTree");
                ui.add_space(12.0);
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

// Entry point called by the Android runtime (NativeActivity).
#[cfg(target_os = "android")]
#[no_mangle]
fn android_main(android: winit::platform::android::activity::AndroidApp) {
    // eframe 0.29 takes the Android app through the event loop builder hook.
    use winit::platform::android::EventLoopBuilderExtAndroid;
    android_logger::init_once(
        android_logger::Config::default().with_max_level(log::LevelFilter::Info),
    );
    i18n::init();
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
