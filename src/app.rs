#![allow(dead_code)]

use crate::container::{self, MediaStore};
use crate::layout::{self, Layout, LayoutConfig};
use crate::model::Tree;
use eframe::egui;
use std::path::PathBuf;

pub struct FamTreeApp {
    pub tree: Tree,
    pub store: MediaStore,
    /// Текущий файл `.famtree` (None — новое дерево).
    pub path: Option<PathBuf>,
    /// Есть несохранённые изменения.
    pub dirty: bool,
    pub layout: Layout,
    pub layout_cfg: LayoutConfig,
    pub status: String,
}

impl FamTreeApp {
    /// `path` — файл из аргумента запуска (двойной клик по `.famtree`).
    pub fn new(path: Option<PathBuf>) -> Self {
        let layout_cfg = LayoutConfig::default();
        let mut app = FamTreeApp {
            tree: Tree::new(),
            store: MediaStore::new(),
            path: None,
            dirty: false,
            layout: Layout::default(),
            layout_cfg,
            status: String::new(),
        };
        if let Some(p) = path {
            match container::load(&p) {
                Ok((tree, store)) => {
                    app.tree = tree;
                    app.store = store;
                    app.path = Some(p);
                }
                Err(e) => {
                    app.status = format!("Не удалось открыть {}: {e}", p.display());
                }
            }
        }
        app.relayout();
        app
    }

    /// Пересчитать раскладку (после изменения дерева).
    pub fn relayout(&mut self) {
        self.layout = layout::compute(&self.tree, &self.layout_cfg);
    }
}

impl eframe::App for FamTreeApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Семейное дерево");
            match &self.path {
                Some(p) => ui.label(format!("Файл: {}", p.display())),
                None => ui.label("Новое дерево"),
            };
            ui.label(format!(
                "Людей: {}, на схеме: {}",
                self.tree.persons.len(),
                self.layout.nodes.len()
            ));
            if !self.status.is_empty() {
                ui.colored_label(egui::Color32::RED, &self.status);
            }
        });
    }
}
