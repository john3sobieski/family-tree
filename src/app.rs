#![allow(dead_code)]

use crate::container::{self, MediaStore};
use crate::demo;
use crate::layout::{self, Layout, LayoutConfig};
use crate::model::{PersonId, Tree};
use crate::view::ViewState;
use eframe::egui;
use std::path::PathBuf;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    View,
    Edit,
    List,
}

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
    pub mode: Mode,
    pub view: ViewState,
    /// Человек, по карточке которого кликнули (откроется окно — шаг 8).
    pub selected: Option<PersonId>,
}

impl FamTreeApp {
    /// `path` — файл из аргумента запуска (двойной клик по `.famtree`).
    pub fn new(path: Option<PathBuf>) -> Self {
        let mut app = FamTreeApp {
            tree: Tree::new(),
            store: MediaStore::new(),
            path: None,
            dirty: false,
            layout: Layout::default(),
            layout_cfg: LayoutConfig::default(),
            status: String::new(),
            mode: Mode::View,
            view: ViewState::default(),
            selected: None,
        };
        if let Some(p) = path {
            app.open_file(p);
        }
        app
    }

    pub fn open_file(&mut self, p: PathBuf) {
        match container::load(&p) {
            Ok((tree, store)) => {
                self.set_tree(tree, store, Some(p));
                self.status.clear();
            }
            Err(e) => self.status = format!("Не удалось открыть {}: {e}", p.display()),
        }
    }

    pub fn set_tree(&mut self, tree: Tree, store: MediaStore, path: Option<PathBuf>) {
        self.tree = tree;
        self.store = store;
        self.path = path;
        self.dirty = false;
        self.selected = None;
        self.view.reset();
        self.relayout();
    }

    /// Пересчитать раскладку (после изменения дерева).
    pub fn relayout(&mut self) {
        self.layout = layout::compute(&self.tree, &self.layout_cfg);
    }

    fn selected_name(&self) -> Option<String> {
        let p = self.tree.persons.get(&self.selected?)?;
        Some(format!("{} {}", p.first_name, p.last_name))
    }
}

impl eframe::App for FamTreeApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("Открыть…").clicked() {
                    if let Some(p) = rfd::FileDialog::new()
                        .add_filter("Семейное дерево", &["famtree"])
                        .pick_file()
                    {
                        self.open_file(p);
                    }
                }
                if ui.button("Демо").clicked() {
                    self.set_tree(demo::sample_tree(), MediaStore::new(), None);
                }
                if ui.button("Демо ×1000").clicked() {
                    self.set_tree(demo::big_tree(1000), MediaStore::new(), None);
                }
                ui.separator();
                ui.selectable_value(&mut self.mode, Mode::View, "Просмотр");
                ui.selectable_value(&mut self.mode, Mode::Edit, "Редакция");
                ui.selectable_value(&mut self.mode, Mode::List, "Список");
                ui.separator();
                if self.mode == Mode::View && ui.button("По размеру").clicked() {
                    self.view.fitted = false;
                }
            });
        });

        egui::TopBottomPanel::bottom("status").show(ctx, |ui| {
            ui.horizontal(|ui| {
                match &self.path {
                    Some(p) => ui.label(format!("Файл: {}", p.display())),
                    None => ui.label("Новое дерево"),
                };
                ui.separator();
                ui.label(format!(
                    "Людей: {}, на схеме: {}",
                    self.tree.persons.len(),
                    self.layout.nodes.len()
                ));
                if let Some(name) = self.selected_name() {
                    ui.separator();
                    ui.label(format!("Выбран: {name}"));
                }
                if !self.status.is_empty() {
                    ui.separator();
                    ui.colored_label(egui::Color32::RED, &self.status);
                }
            });
        });

        egui::CentralPanel::default().frame(egui::Frame::none()).show(ctx, |ui| match self.mode {
            Mode::View => {
                let clicked =
                    self.view.show(ui, &self.tree, &self.layout, &self.layout_cfg, &self.store);
                if clicked.is_some() {
                    self.selected = clicked;
                }
            }
            Mode::Edit => {
                ui.centered_and_justified(|ui| ui.label("Режим редакции — скоро"));
            }
            Mode::List => {
                ui.centered_and_justified(|ui| ui.label("Режим списка — скоро"));
            }
        });
    }
}
