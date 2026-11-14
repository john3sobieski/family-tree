// Без консольного окна в release-сборке на Windows.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[allow(dead_code)]
mod container;
#[allow(dead_code)]
mod model;

use eframe::egui;

struct FamTreeApp;

impl eframe::App for FamTreeApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Семейное дерево");
        });
    }
}

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_title("Семейное дерево"),
        ..Default::default()
    };
    eframe::run_native(
        "Семейное дерево",
        options,
        Box::new(|_cc| Ok(Box::new(FamTreeApp))),
    )
}
