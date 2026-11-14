#![allow(dead_code)]

use crate::assoc;
use crate::container::{self, MediaStore};
use crate::demo;
use crate::edit::{self, EditState, ListAction};
use crate::layout::{self, Layout, LayoutConfig};
use crate::list::{self, ListState};
use crate::model::{PersonId, Tree};
use crate::person_window::{PersonWindow, WindowEvent};
use crate::search::{self, SearchState};
use crate::view::ViewState;
use eframe::egui;
use std::path::PathBuf;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    View,
    Edit,
    List,
}

/// Действие, которое выбросит текущее дерево — перед ним спрашиваем про несохранённое.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Guarded {
    Close,
    New,
    Open,
    Demo,
    BigDemo,
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
    assoc_linked: bool,
    pub mode: Mode,
    pub view: ViewState,
    /// Человек, по карточке которого кликнули (открыто его окно).
    pub selected: Option<PersonId>,
    pub person_win: PersonWindow,
    /// Окно человека открыто (только в режиме «Просмотр»).
    pub open_window: bool,
    pub edit: EditState,
    pub list: ListState,
    pub search: SearchState,
    /// Ждёт ответа в окне «Сохранить изменения?».
    pub pending: Option<Guarded>,
    /// Выход подтверждён — не перехватывать закрытие окна.
    allow_close: bool,
    last_title: String,
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
            assoc_linked: assoc::is_registered(),
            mode: Mode::View,
            view: ViewState::default(),
            selected: None,
            person_win: PersonWindow::default(),
            open_window: false,
            edit: EditState::default(),
            list: ListState::default(),
            search: SearchState::default(),
            pending: None,
            allow_close: false,
            last_title: String::new(),
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
        self.list.reset();
        self.search.clear();
        self.open_window = false;
        self.view.reset();
        self.relayout();
    }

    /// Пересчитать раскладку (после изменения дерева).
    pub fn relayout(&mut self) {
        self.layout = layout::compute(&self.tree, &self.layout_cfg);
    }

    /// Выполнить действие, которое заменяет дерево; если есть несохранённые правки —
    /// сначала спросить пользователя.
    pub fn request(&mut self, ctx: &egui::Context, a: Guarded) {
        if self.dirty {
            self.pending = Some(a);
        } else {
            self.perform(ctx, a);
        }
    }

    fn perform(&mut self, ctx: &egui::Context, a: Guarded) {
        self.pending = None;
        match a {
            Guarded::Close => {
                self.allow_close = true;
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
            Guarded::New => self.set_tree(Tree::new(), MediaStore::new(), None),
            Guarded::Open => {
                if let Some(p) = rfd::FileDialog::new()
                    .add_filter("Семейное дерево", &["famtree"])
                    .pick_file()
                {
                    self.open_file(p);
                }
            }
            Guarded::Demo => self.set_tree(demo::sample_tree(), MediaStore::new(), None),
            Guarded::BigDemo => self.set_tree(demo::big_tree(1000), MediaStore::new(), None),
        }
    }

    fn file_label(&self) -> String {
        match &self.path {
            Some(p) => p
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_else(|| p.display().to_string()),
            None => "Новое дерево".to_string(),
        }
    }

    /// Окно «Сохранить изменения?» (поверх всего, пока не ответят).
    fn confirm_dialog(&mut self, ctx: &egui::Context) {
        let Some(action) = self.pending else { return };
        #[derive(PartialEq)]
        enum Choice {
            None,
            Save,
            Discard,
            Cancel,
        }
        let mut choice = Choice::None;
        let name = self.file_label();
        egui::Window::new("Несохранённые изменения")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.label(format!("В «{name}» есть несохранённые изменения. Сохранить их?"));
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    if ui.button("Сохранить").clicked() {
                        choice = Choice::Save;
                    }
                    if ui.button("Не сохранять").clicked() {
                        choice = Choice::Discard;
                    }
                    if ui.button("Отмена").clicked() {
                        choice = Choice::Cancel;
                    }
                });
            });
        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            choice = Choice::Cancel;
        }
        match choice {
            Choice::None => {}
            Choice::Cancel => self.pending = None,
            Choice::Discard => {
                self.dirty = false;
                self.perform(ctx, action);
            }
            Choice::Save => {
                self.save();
                // Если сохранение не удалось или диалог «Сохранить как» закрыли — остаёмся.
                if !self.dirty {
                    self.perform(ctx, action);
                }
            }
        }
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
        self.view.fly_to(id, false);
    }

    fn selected_name(&self) -> Option<String> {
        let p = self.tree.persons.get(&self.selected?)?;
        Some(format!("{} {}", p.first_name, p.last_name))
    }
}

impl eframe::App for FamTreeApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Закрытие окна с несохранённым: отменяем и спрашиваем.
        if ctx.input(|i| i.viewport().close_requested()) && self.dirty && !self.allow_close {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            self.pending = Some(Guarded::Close);
        }
        // Заголовок окна: имя файла и «*» при несохранённых правках.
        let title = format!("{}{} — Семейное дерево", self.file_label(), if self.dirty { " *" } else { "" });
        if title != self.last_title {
            ctx.send_viewport_cmd(egui::ViewportCommand::Title(title.clone()));
            self.last_title = title;
        }
        if ctx.input_mut(|i| i.consume_key(egui::Modifiers::COMMAND, egui::Key::S)) {
            self.save();
        }

        if self.mode != Mode::List && ctx.input_mut(|i| i.consume_key(egui::Modifiers::COMMAND, egui::Key::F)) {
            ctx.memory_mut(|m| m.request_focus(egui::Id::new(search::FIELD_ID)));
        }

        egui::TopBottomPanel::top("top").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("Новое").clicked() {
                    self.request(ctx, Guarded::New);
                }
                if ui.button("Открыть…").clicked() {
                    self.request(ctx, Guarded::Open);
                }
                if ui.button("Сохранить").clicked() {
                    self.save();
                }
                if ui.button("Сохранить как…").clicked() {
                    self.save_as();
                }
                ui.separator();
                if ui.button("Демо").clicked() {
                    self.request(ctx, Guarded::Demo);
                }
                if ui.button("Демо ×1000").clicked() {
                    self.request(ctx, Guarded::BigDemo);
                }
                let linked = self.assoc_linked;
                let label = if linked { "✔ .famtree" } else { "Связать .famtree" };
                let hint = if linked {
                    "Файлы .famtree открываются этим приложением. Клик — отвязать."
                } else {
                    "Открывать файлы .famtree двойным кликом (без прав администратора)"
                };
                if ui.button(label).on_hover_text(hint).clicked() {
                    let r = if linked { assoc::unregister() } else { assoc::register() };
                    match r {
                        Ok(()) => self.status.clear(),
                        Err(e) => self.status = format!("Ассоциация .famtree: {e}"),
                    }
                    self.assoc_linked = assoc::is_registered();
                }
                ui.separator();
                ui.selectable_value(&mut self.mode, Mode::View, "Просмотр");
                ui.selectable_value(&mut self.mode, Mode::Edit, "Редакция");
                ui.selectable_value(&mut self.mode, Mode::List, "Список");
                ui.separator();
                if self.mode == Mode::View && ui.button("По размеру").clicked() {
                    self.view.fitted = false;
                }
                if self.mode != Mode::List {
                    ui.separator();
                    if let Some(id) = search::show(ui, &mut self.search, &self.tree, &self.layout) {
                        self.selected = Some(id);
                        self.view.fly_to(id, true);
                    }
                }
            });
        });

        if self.mode == Mode::Edit {
            self.open_window = false;
        }
        if self.mode != Mode::List {
            // В списке данные пересчитаются заново после правок в других режимах.
            self.list.invalidate();
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
                let clicked = list::show(ui, &mut self.list, &self.tree, &self.store, self.selected);
                if clicked.is_some() {
                    self.selected = clicked;
                    self.open_window = true;
                }
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

        self.confirm_dialog(ctx);
    }
}
