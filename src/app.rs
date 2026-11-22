#![allow(dead_code)]

use crate::assoc;
use crate::chrome;
use crate::theme;
use crate::container::{self, MediaStore};
use crate::demo;
use crate::edit::{self, EditState, ListAction};
use crate::i18n::{self, tr, trf, Lang};
use crate::layout::{self, Layout, LayoutConfig};
use crate::list::{self, ListState};
use crate::model::{PersonId, Tree};
use crate::person_window::{PersonWindow, WindowEvent};
use crate::search::{self, SearchState};
use crate::view::ViewState;
use eframe::egui;
use std::path::PathBuf;
use std::time::{Duration, Instant};

/// Через сколько после последней правки сохраняем файл сами.
const AUTOSAVE_DELAY: Duration = Duration::from_millis(1500);
/// Через сколько пробуем снова, если автосохранение не удалось.
const AUTOSAVE_RETRY: Duration = Duration::from_secs(5);

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
    /// Когда сработает автосохранение (после правки сдвигается на `AUTOSAVE_DELAY`).
    autosave_at: Option<Instant>,
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
            autosave_at: None,
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
            Err(e) => {
                self.status = trf(
                    "Не удалось открыть {path}: {err}",
                    &[("path", p.display().to_string()), ("err", e.to_string())],
                )
            }
        }
    }

    pub fn set_tree(&mut self, tree: Tree, store: MediaStore, path: Option<PathBuf>) {
        self.tree = tree;
        self.store = store;
        self.path = path;
        self.dirty = false;
        self.autosave_at = None;
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
        let before = self.layout.nodes.len();
        self.layout = layout::compute(&self.tree, &self.layout_cfg);
        // Появились/исчезли люди — показать всю схему целиком, чтобы новое не уезжало за экран.
        if self.layout.nodes.len() != before {
            self.view.fitted = false;
        }
    }

    /// Выполнить действие, которое заменяет дерево; если есть несохранённые правки —
    /// сначала спросить пользователя.
    pub fn request(&mut self, ctx: &egui::Context, a: Guarded) {
        // Файл уже есть — сохраняем молча, вопрос нужен только если не получилось.
        if self.dirty && self.path.is_some() {
            self.save();
        }
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
                    .add_filter(tr("Семейное дерево"), &["famtree"])
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
            None => tr("Новое дерево").to_string(),
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
        egui::Window::new(tr("Несохранённые изменения"))
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.label(trf("В «{name}» есть несохранённые изменения. Сохранить их?", &[("name", name.to_string())]));
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    if ui.button(tr("Сохранить")).clicked() {
                        choice = Choice::Save;
                    }
                    if ui.button(tr("Не сохранять")).clicked() {
                        choice = Choice::Discard;
                    }
                    if ui.button(tr("Отмена")).clicked() {
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
            .add_filter(tr("Семейное дерево"), &["famtree"])
            .set_file_name(format!("{}.famtree", tr("семья")))
            .save_file()
        {
            if p.extension().is_none() {
                p.set_extension("famtree");
            }
            self.save_to(p);
        }
    }

    /// Отметить правку: дерево изменено, автосохранение запланировано.
    fn mark_dirty(&mut self) {
        self.dirty = true;
        self.autosave_at = Some(Instant::now() + AUTOSAVE_DELAY);
    }

    /// Автосохранение: через `AUTOSAVE_DELAY` после последней правки пишем в текущий файл.
    /// У нового (ещё не сохранённого) дерева файла нет — там сначала нужно «Сохранить как…».
    fn autosave_tick(&mut self, ctx: &egui::Context) {
        let Some(at) = self.autosave_at else { return };
        if !self.dirty || self.path.is_none() {
            self.autosave_at = None;
            return;
        }
        // Пока открыт диалог «Сохранить изменения?» — не вмешиваемся.
        if self.pending.is_some() {
            return;
        }
        let now = Instant::now();
        if now < at {
            ctx.request_repaint_after(at - now);
            return;
        }
        self.save();
        if self.dirty {
            // Не получилось (ошибка в статусе) — повторим чуть позже.
            self.autosave_at = Some(Instant::now() + AUTOSAVE_RETRY);
            ctx.request_repaint_after(AUTOSAVE_RETRY);
        } else {
            self.autosave_at = None;
        }
    }

    fn save_to(&mut self, p: PathBuf) {
        match container::save(&p, &self.tree, &mut self.store) {
            Ok(()) => {
                self.path = Some(p);
                self.dirty = false;
                self.status.clear();
            }
            Err(e) => self.status = trf("Не удалось сохранить: {err}", &[("err", e.to_string())]),
        }
    }

    /// Выделить человека и показать его на схеме.
    /// Switches the interface language, remembers it and reloads the fonts (Chinese needs a CJK font).
    fn set_language(&mut self, ctx: &egui::Context, lang: Lang) {
        i18n::set(lang);
        i18n::save(lang);
        theme::set_fonts(ctx, lang);
        self.status.clear();
    }

    fn select_and_show(&mut self, id: PersonId) {
        self.selected = Some(id);
        self.view.fly_to(id, false);
    }

    fn selected_name(&self) -> Option<String> {
        let p = self.tree.persons.get(&self.selected?)?;
        Some(crate::person_window::full_name(p))
    }
}

impl eframe::App for FamTreeApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Закрытие окна: если файл есть, последние правки (ещё не дождавшиеся автосохранения)
        // сохраняем молча.
        if ctx.input(|i| i.viewport().close_requested()) && self.dirty && self.path.is_some() {
            self.save();
        }
        // Закрытие окна с несохранённым: отменяем и спрашиваем.
        if ctx.input(|i| i.viewport().close_requested()) && self.dirty && !self.allow_close {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            self.pending = Some(Guarded::Close);
        }
        // Заголовок окна: имя файла и «*» при несохранённых правках.
        let title = format!("{}{} — {}", self.file_label(), if self.dirty { " *" } else { "" }, tr("Семейное дерево"));
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

        chrome::title_bar(ctx, &self.file_label(), self.dirty);
        chrome::window_border(ctx);

        egui::TopBottomPanel::top("toolbar").exact_height(48.0).frame(theme::bar_frame(14.0, 6.0)).show(ctx, |ui| {
            ui.horizontal_centered(|ui| {
                if ui.button(tr("Открыть")).clicked() {
                    self.request(ctx, Guarded::Open);
                }
                if theme::primary_button(ui, tr("Сохранить")).clicked() {
                    self.save();
                }
                ui.menu_button(tr("Файл"), |ui| {
                    if ui.button(tr("Новое")).clicked() {
                        self.request(ctx, Guarded::New);
                        ui.close_menu();
                    }
                    if ui.button(tr("Сохранить как…")).clicked() {
                        self.save_as();
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button(tr("Демо")).clicked() {
                        self.request(ctx, Guarded::Demo);
                        ui.close_menu();
                    }
                    if ui.button(tr("Демо ×1000")).clicked() {
                        self.request(ctx, Guarded::BigDemo);
                        ui.close_menu();
                    }
                    ui.separator();
                    let linked = self.assoc_linked;
                    let label = if linked {
                        tr("Отвязать .famtree от приложения")
                    } else {
                        tr("Открывать .famtree двойным кликом")
                    };
                    if ui.button(label).clicked() {
                        let r = if linked { assoc::unregister() } else { assoc::register() };
                        match r {
                            Ok(()) => self.status.clear(),
                            Err(e) => self.status = trf("Ассоциация .famtree: {err}", &[("err", e.to_string())]),
                        }
                        self.assoc_linked = assoc::is_registered();
                        ui.close_menu();
                    }
                });
                ui.menu_button(tr("Язык"), |ui| {
                    for lang in Lang::ALL {
                        if ui.selectable_label(i18n::current() == lang, lang.native_name()).clicked() {
                            self.set_language(ctx, lang);
                            ui.close_menu();
                        }
                    }
                });

                ui.add_space(12.0);
                ui.selectable_value(&mut self.mode, Mode::View, tr("Просмотр"));
                ui.selectable_value(&mut self.mode, Mode::Edit, tr("Редакция"));
                ui.selectable_value(&mut self.mode, Mode::List, tr("Список"));

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if self.mode != Mode::List {
                        if let Some(id) = search::show(ui, &mut self.search, &self.tree, &self.layout) {
                            self.selected = Some(id);
                            self.view.fly_to(id, true);
                        }
                    }
                    if self.mode == Mode::View && ui.button(tr("По размеру")).clicked() {
                        self.view.fitted = false;
                    }
                });
            });
        });

        if self.mode == Mode::Edit {
            self.open_window = false;
        }
        if self.mode != Mode::List {
            // В списке данные пересчитаются заново после правок в других режимах.
            self.list.invalidate();
        }

        egui::TopBottomPanel::bottom("status").exact_height(26.0).frame(theme::bar_frame(14.0, 3.0)).show(ctx, |ui| {
            ui.horizontal_centered(|ui| {
                let dim = |t: String| egui::RichText::new(t).small().color(theme::MUTED);
                match &self.path {
                    Some(p) => ui.label(dim(p.display().to_string())),
                    None => ui.label(dim(tr("Новое дерево").into())),
                };
                ui.label(dim(trf(
                    "·  Людей: {total}  ·  на схеме: {shown}",
                    &[
                        ("total", self.tree.persons.len().to_string()),
                        ("shown", self.layout.nodes.len().to_string()),
                    ],
                )));
                if let Some(name) = self.selected_name() {
                    ui.label(dim(trf("·  Выбран: {name}", &[("name", name.to_string())])));
                }
                if self.dirty {
                    ui.label(egui::RichText::new(tr("●  не сохранено")).small().color(theme::TEXT));
                    if self.path.is_none() {
                        ui.label(dim(tr("(автосохранение заработает после «Сохранить как…»)").into()));
                    }
                }
                if !self.status.is_empty() {
                    ui.label(egui::RichText::new(&self.status).small().color(theme::DANGER));
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
                    self.mark_dirty();
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
                self.mark_dirty();
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
        self.autosave_tick(ctx);
    }
}
