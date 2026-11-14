// Без консольного окна в release-сборке на Windows.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
#[allow(dead_code)]
mod container;
mod demo;
mod edit;
#[allow(dead_code)]
mod layout;
mod list;
#[allow(dead_code)]
mod media;
#[allow(dead_code)]
mod model;
mod person_window;
mod search;
mod view;

use eframe::egui;
use std::path::PathBuf;

fn main() -> eframe::Result {
    // Двойной клик по .famtree передаёт путь первым аргументом.
    let path: Option<PathBuf> = std::env::args_os().nth(1).map(PathBuf::from);

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_title("Семейное дерево"),
        ..Default::default()
    };
    eframe::run_native(
        "Семейное дерево",
        options,
        Box::new(move |_cc| Ok(Box::new(app::FamTreeApp::new(path)))),
    )
}
