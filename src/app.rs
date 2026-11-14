#![allow(dead_code)]

use crate::container::{self, MediaStore};
use crate::demo;
use crate::edit::{self, EditState, ListAction};
use crate::layout::{self, Layout, LayoutConfig};
use crate::model::{PersonId, Tree};
use crate::person_window::{PersonWindow, WindowEvent};
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
    /// Человек, по карточке которого кликнули (открыто его окно).
    pub selected: Option<PersonId>,
    pub person_win: PersonWindow,
    /// Окно человека открыто (только в режиме «Просмотр»).
    pub open_window: bool,
    pub edit: EditState,
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
            person_win: PersonWindow::default(),
            open_window: false,
            edit: EditState::default(),
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
        self.person_win.reset();
        self.edit.reset();
        self.open_window = false;
        self.view.reset();
        self.relayout();
    }

    /// Пересчитать раскладку (после изменения дерева).
    pub fn relayout(&mut self) {
        self.layout = layout::compute(&self.tree, &self.layout_cfg);
    }

    /// Сохранить в текущий файл (или спросить путь).
    pub fn save(&mut self) {
        match self.path.clone() {
            Some(p) => self.save_to(p),
            None => self.save_as(),
        }
    }

    pub fn save_as(&mut self) {
        if let Some(mut p) = rfd::FileDialog::new()
            .add_filter("Семейное дерево", &["famtree"])
            .set_file_name("семья.famtree")
            .save_file()
        {
            if p.extension().is_none() {
                p.set_extension("famtree");
            }
            self.save_to(p);
        }
    }

    fn save_to(&mut self, p: PathBuf) {
        match container::save(&p, &self.tree, &mut self.store) {
            Ok(()) => {
                self.path = Some(p);
                self.dirty = false;
                self.status.clear();
            }
            Err(e) => self.status = format!("Не удалось сохранить: {e}"),
        }
    }

    /// Выделить человека и показать его на схеме.
    fn select_and_show(&mut self, id: PersonId) {
        self.selected = Some(id);
        self.view.pending_center = Some(id);
    }

    fn selected_name(&self) -> Option<String> {
        let p = self.tree.persons.get(&self.selected?)?;
        Some(format!("{} {}", p.first_name, p.last_name))
    }
}

impl eframe::App for FamTreeApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if ctx.input_mut(|i| i.consume_key(egui::Modifiers::COMMAND, egui::Key::S)) {
            self.save();
        }

        egui::TopBottomPanel::top("top").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("Новое").clicked() {
                    self.set_tree(Tree::new(), MediaStore::new(), None);
                }
                if ui.button("Открыть…").clicked() {
                    if let Some(p) = rfd::FileDialog::new()
                        .add_filter("Семейное дерево", &["famtree"])
                        .pick_file()
                    {
                        self.open_file(p);
                    }
                }
                if ui.button("Сохранить").clicked() {
                    self.save();
                }
                if ui.button("Сохранить как…").clicked() {
                    self.save_as();
                }
                ui.separator();
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

        if self.mode != Mode::View {
            self.open_window = false;
        }

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
                if self.dirty {
                    ui.separator();
                    ui.label("● не сохранено");
                }
                if !self.status.is_empty() {
                    ui.separator();
                    ui.colored_label(egui::Color32::RED, &self.status);
                }
            });
        });

        if self.mode == Mode::Edit {
            let mut list_action = ListAction::None;
            egui::SidePanel::left("people").default_width(240.0).show(ctx, |ui| {
                list_action = edit::people_list(ui, &mut self.edit, &self.tree, self.selected);
            });
            match list_action {
                ListAction::Select(id) => self.select_and_show(id),
                ListAction::New => {
                    let id = edit::new_person(&mut self.tree);
                    self.dirty = true;
                    self.relayout();
                    self.select_and_show(id);
                }
                ListAction::None => {}
            }

            let mut change = edit::Change::default();
            egui::SidePanel::right("editor").default_width(380.0).show(ctx, |ui| {
                let placed = self.selected.map_or(true, |id| self.layout.pos_of(id).is_some());
                change = edit::editor(
                    ui,
                    &mut self.edit,
                    &mut self.tree,
                    &mut self.store,
                    self.selected,
                    placed,
                );
            });
            if change.dirty {
                self.dirty = true;
            }
            if change.relayout {
                self.relayout();
            }
            if change.deselect {
                self.selected = None;
            }
            if let Some(id) = change.select {
                self.select_and_show(id);
            }
            if let Some(e) = change.error {
                self.status = e;
            }
        }

        self.view.highlight = self.selected;
        egui::CentralPanel::default().frame(egui::Frame::none()).show(ctx, |ui| match self.mode {
            Mode::View | Mode::Edit => {
                let clicked =
                    self.view.show(ui, &self.tree, &self.layout, &self.layout_cfg, &self.store);
                if clicked.is_some() {
                    self.selected = clicked;
                    if self.mode == Mode::View {
                        self.open_window = true;
                    }
                }
            }
            Mode::List => {
                ui.centered_and_justified(|ui| ui.label("Режим списка — скоро"));
            }
        });

        // Большое окно человека (по клику на карточку).
        if let (true, Some(id)) = (self.open_window, self.selected) {
            match self.person_win.show(ctx, &self.tree, &self.store, id) {
                WindowEvent::Close => self.open_window = false,
                WindowEvent::Goto(other) => self.selected = Some(other),
                WindowEvent::None => {}
            }
        }
    }
}
