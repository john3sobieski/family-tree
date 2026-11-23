// Без консольного окна в release-сборке на Windows.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod assoc;
mod chrome;
#[allow(dead_code)]
mod container;
mod countries;
mod demo;
mod edit;
mod export;
mod flags;
mod i18n;
#[allow(dead_code)]
mod layout;
mod list;
#[allow(dead_code)]
mod media;
#[allow(dead_code)]
mod model;
mod person_window;
mod platform;
mod search;
mod theme;
mod view;

use eframe::egui;
use std::path::PathBuf;

fn main() -> eframe::Result {
    // Двойной клик по .famtree передаёт путь первым аргументом.
    let path: Option<PathBuf> = std::env::args_os().nth(1).map(PathBuf::from);
    i18n::init();
    theme::init();

    let mut viewport = egui::ViewportBuilder::default()
        .with_inner_size([1200.0, 800.0])
        .with_title(i18n::tr("Семейное дерево"))
        .with_decorations(false)
        .with_min_inner_size([900.0, 560.0])
        .with_maximized(true);
    if let Ok(img) = image::load_from_memory(include_bytes!("../assets/icon.png")) {
        let img = img.to_rgba8();
        let (width, height) = img.dimensions();
        viewport = viewport.with_icon(egui::IconData { rgba: img.into_raw(), width, height });
    }
    let options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };
    eframe::run_native(
        "Семейное дерево",
        options,
        Box::new(move |cc| {
            platform::init_window(cc);
            theme::apply(&cc.egui_ctx);
            Ok(Box::new(app::FamTreeApp::new(path)))
        }),
    )
}
