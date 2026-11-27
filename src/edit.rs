//! Режим «Редакция»: список людей слева, форма человека справа
//! (данные, фото, связи, удаление). Схема посередине — обычный `view`.

use crate::container::{MediaKind, MediaStore};
use crate::crop::{CropDialog, CropOutcome};
use crate::i18n::{tr, trf};
use crate::media;
use crate::model::{Gender, PartialDate, Person, PersonId, Photo, Tree};
use crate::person_window::{format_years, full_name, load_texture};
use crate::widgets::{centered_text, icon_button, icon_text_button, inline_icon, section, Icon};
use eframe::egui::{self, Align, Color32, Layout, RichText, Sense, Stroke, TextureHandle, Vec2};
use std::collections::HashMap;

const THUMB_SMALL: Vec2 = Vec2::new(45.0, 60.0);
const THUMB_MAIN: Vec2 = Vec2::new(75.0, 100.0);
const MAX_THUMBS: usize = 200;
const MAX_REL_RESULTS: usize = 8;
/// Width of the label column in the forms and the gap between columns.
const LABEL_W: f32 = 132.0;
const GRID_GAP: f32 = 10.0;

/// Открытое окно выбора области главного фото.
struct CropSession {
    person: PersonId,
    dlg: CropDialog,
    /// Фото взято из «Фото в заметках» («Сделать главным»): прежнее главное фото уходит в заметки.
    from_gallery: bool,
}

#[derive(Default)]
pub struct EditState {
    pub list_search: String,
    rel_search: String,
    confirm_delete: Option<PersonId>,
    thumbs: HashMap<String, Option<TextureHandle>>,
    crop: Option<CropSession>,
}

impl EditState {
    pub fn reset(&mut self) {
        self.rel_search.clear();
        self.confirm_delete = None;
        self.thumbs.clear();
        self.crop = None;
    }

    fn thumb(&mut self, ctx: &egui::Context, store: &MediaStore, hash: &str) -> Option<egui::TextureId> {
        if !self.thumbs.contains_key(hash) {
            if self.thumbs.len() >= MAX_THUMBS {
                self.thumbs.clear();
            }
            let t = load_texture(ctx, store, MediaKind::Thumb, hash);
            self.thumbs.insert(hash.to_string(), t);
        }
        self.thumbs.get(hash)?.as_ref().map(|t| t.id())
    }
}

/// Результат работы редактора для приложения.
#[derive(Default)]
pub struct Change {
    /// Данные изменились — файл надо пометить несохранённым.
    pub dirty: bool,
    /// Изменилась структура (связи/люди/год рождения) — пересчитать раскладку.
    pub relayout: bool,
    /// Выделить (и показать на схеме) этого человека.
    pub select: Option<PersonId>,
    /// Выбранный человек удалён.
    pub deselect: bool,
    pub error: Option<String>,
}

pub enum ListAction {
    None,
    Select(PersonId),
    New,
}

/// Левая панель: поиск и список всех людей (в том числе не видных на схеме).
pub fn people_list(
    ui: &mut egui::Ui,
    st: &mut EditState,
    tree: &Tree,
    selected: Option<PersonId>,
) -> ListAction {
    let mut action = ListAction::None;
    ui.heading(tr("Люди"));
    ui.add(
        egui::TextEdit::singleline(&mut st.list_search)
            .hint_text(tr("Поиск по имени…"))
            .desired_width(f32::INFINITY),
    );
    let full = ui.available_width();
    if icon_text_button(ui, Icon::Plus, tr("Новый человек"), Some(full)).clicked() {
        action = ListAction::New;
    }
    ui.separator();

    let q = st.list_search.trim().to_lowercase();
    let ids: Vec<PersonId> = tree
        .persons
        .values()
        .filter(|p| q.is_empty() || full_name(p).to_lowercase().contains(&q))
        .map(|p| p.id)
        .collect();
    let row_h = ui.text_style_height(&egui::TextStyle::Body) + ui.spacing().item_spacing.y + 4.0;
    egui::ScrollArea::vertical().auto_shrink([false, false]).show_rows(
        ui,
        row_h,
        ids.len(),
        |ui, range| {
            for i in range {
                let id = ids[i];
                let Some(p) = tree.persons.get(&id) else { continue };
                let years = format_years(p);
                let label = if years.is_empty() {
                    full_name(p)
                } else {
                    format!("{}  ({years})", full_name(p))
                };
                if ui.selectable_label(selected == Some(id), label).clicked() {
                    action = ListAction::Select(id);
                }
            }
        },
    );
    action
}

/// Placeholder name for a new person: the translated phrase split into first and last name.
fn placeholder_name(key: &'static str) -> (String, String) {
    let s = tr(key);
    match s.split_once(' ') {
        Some((a, b)) => (a.to_string(), b.to_string()),
        None => (s.to_string(), String::new()),
    }
}

/// Создаёт нового человека. Первый человек в пустом дереве становится «я».
pub fn new_person(tree: &mut Tree) -> PersonId {
    let (first, last) = placeholder_name("Новый человек");
    let id = tree.add_person(Person::new(&first, &last));
    if tree.root.is_none() {
        tree.root = Some(id);
    }
    id
}

/// Делает `parent` родителем `child`. Максимум два родителя; второй родитель
/// автоматически становится партнёром первого (мать и отец рядом на схеме).
pub(crate) fn link_parent(tree: &mut Tree, parent: PersonId, child: PersonId) -> Result<(), String> {
    let existing = tree.parents_of(child);
    if existing.len() >= 2 {
        return Err(tr("У человека уже есть два родителя").to_string());
    }
    tree.add_parent_child(parent, child).map_err(|e| e.to_string())?;
    if existing.len() == 1 {
        let _ = tree.add_partners(existing[0], parent); // уже партнёры — не страшно
    }
    Ok(())
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Rel {
    Parent,
    Partner,
    Child,
    /// Только для кнопки «Брат/сестра» (в списках связей не используется).
    Sibling,
}

enum Action {
    Select(PersonId),
    Remove(Rel, PersonId),
    Link(Rel, PersonId),
    New(Rel),
    SetRoot,
    Delete,
}

/// Число в текстовом поле с пустым значением = «неизвестно». Пока поле в фокусе,
/// набираемый текст хранится отдельно, чтобы можно было стирать и печатать.
fn num_field(ui: &mut egui::Ui, key: egui::Id, val: &mut Option<i64>, lo: i64, hi: i64, hint: &str, width: f32) {
    let mut buf: String = ui
        .data(|d| d.get_temp::<String>(key))
        .unwrap_or_else(|| val.map(|v| v.to_string()).unwrap_or_default());
    // Explicit id: an auto id would change whenever a widget above appears/disappears,
    // and the field would lose focus after the first typed digit.
    let resp = ui.add(egui::TextEdit::singleline(&mut buf).id(key.with("edit")).hint_text(hint).desired_width(width));
    if resp.changed() {
        let t = buf.trim();
        if t.is_empty() {
            *val = None;
        } else if let Ok(n) = t.parse::<i64>() {
            if n >= lo && n <= hi {
                *val = Some(n);
            }
        }
    }
    if resp.has_focus() {
        ui.data_mut(|d| d.insert_temp(key, buf));
    } else {
        ui.data_mut(|d| d.remove_temp::<String>(key));
    }
}

fn date_row(ui: &mut egui::Ui, id: PersonId, tag: &str, d: &mut PartialDate, width: f32) {
    ui.horizontal(|ui| {
        // The three fields together are exactly as wide as the text fields below/above.
        let free = (width - 2.0 * ui.spacing().item_spacing.x).max(90.0);
        let k = egui::Id::new(("date-field", id, tag, "day"));
        let mut day = d.day.map(i64::from);
        num_field(ui, k, &mut day, 1, 31, tr("день"), free * 0.26);
        d.day = day.map(|v| v as u8);

        let k = egui::Id::new(("date-field", id, tag, "month"));
        let mut month = d.month.map(i64::from);
        num_field(ui, k, &mut month, 1, 12, tr("месяц"), free * 0.34);
        d.month = month.map(|v| v as u8);

        let k = egui::Id::new(("date-field", id, tag, "year"));
        let mut year = d.year.map(i64::from);
        num_field(ui, k, &mut year, 1, 2999, tr("год"), free * 0.40);
        d.year = year.map(|v| v as i32);
    });
}

fn thumb_image(ui: &mut egui::Ui, st: &mut EditState, store: &MediaStore, hash: &str, size: Vec2) {
    match st.thumb(ui.ctx(), store, hash) {
        Some(tid) => {
            ui.image((tid, size));
        }
        None => {
            let (rect, _) = ui.allocate_exact_size(size, Sense::hover());
            ui.painter().rect_filled(rect, 0.0, ui.visuals().faint_bg_color);
        }
    }
}

/// Диалог выбора фото + импорт. Возвращает hash успешно импортированных.
fn import_dialog(store: &mut MediaStore, multiple: bool, errors: &mut Vec<String>) -> Vec<String> {
    let dlg = rfd::FileDialog::new().add_filter(tr("Изображения"), &["jpg", "jpeg", "png", "webp", "bmp"]);
    let files: Vec<std::path::PathBuf> = if multiple {
        dlg.pick_files().unwrap_or_default()
    } else {
        dlg.pick_file().into_iter().collect()
    };
    let mut hashes = Vec::new();
    for f in files {
        match media::import_photo_file(store, &f) {
            Ok(h) => hashes.push(h),
            Err(e) => errors.push(format!("{}: {e}", f.display())),
        }
    }
    hashes
}

/// Диалог выбора файла и открытие окна выбора области (в хранилище пока ничего не попадает).
fn open_crop_from_file(ctx: &egui::Context, st: &mut EditState, person: PersonId, errors: &mut Vec<String>) {
    let dlg = rfd::FileDialog::new().add_filter(tr("Изображения"), &["jpg", "jpeg", "png", "webp", "bmp"]);
    let Some(f) = dlg.pick_file() else { return };
    let img = std::fs::read(&f)
        .map_err(media::MediaError::from)
        .and_then(|bytes| media::decode(&bytes));
    match img {
        Ok(img) => st.crop = Some(CropSession { person, dlg: CropDialog::new(ctx, img), from_gallery: false }),
        Err(e) => errors.push(format!("{}: {e}", f.display())),
    }
}

/// Name of a field in the label column of a form: muted, one line, cut with an ellipsis if too long.
fn field_label(ui: &mut egui::Ui, text: &str) {
    ui.allocate_ui_with_layout(
        egui::vec2(LABEL_W, ui.spacing().interact_size.y),
        Layout::left_to_right(Align::Center),
        |ui| {
            ui.add(egui::Label::new(RichText::new(text).color(crate::theme::p().muted)).truncate())
                .on_hover_text(text);
        },
    );
}

/// Выпадающий список стран с поиском. Свой popup вместо `ComboBox`: у `ComboBox` любой клик внутри
/// списка (в том числе по полю поиска) закрывает его. Закрывается: выбор страны, клик мимо, Esc.
fn country_picker(ui: &mut egui::Ui, list: &[String], width: f32) -> Option<&'static str> {
    let open_id = egui::Id::new("nationality-open");
    let filter_id = egui::Id::new("nationality-filter");
    let edit_id = egui::Id::new("nationality-filter-edit");
    let mut open: bool = ui.data(|d| d.get_temp(open_id)).unwrap_or(false);
    let mut picked: Option<&'static str> = None;

    let btn = ui.add(egui::Button::new(tr("Добавить страну…")).min_size(egui::vec2(width, ui.spacing().interact_size.y)));
    if btn.clicked() {
        open = !open;
        if open {
            ui.data_mut(|d| d.insert_temp(filter_id, String::new()));
            ui.ctx().memory_mut(|m| m.request_focus(edit_id));
        }
    }

    if open {
        let area = egui::Area::new(egui::Id::new("nationality-popup"))
            .order(egui::Order::Foreground)
            .fixed_pos(btn.rect.left_bottom() + egui::vec2(0.0, 2.0))
            .show(ui.ctx(), |ui| {
                egui::Frame::popup(ui.style()).show(ui, |ui| {
                    // Фиксированная ширина: Area не ограничена по ширине, а `f32::INFINITY` в поле
                    // поиска растягивал попап на весь экран.
                    let inner_w = (width - 16.0).max(180.0);
                    ui.set_width(inner_w);
                    // Search box: the list has ~200 countries.
                    let mut filter: String = ui.data_mut(|d| d.get_temp(filter_id)).unwrap_or_default();
                    ui.add(
                        egui::TextEdit::singleline(&mut filter)
                            .id(edit_id)
                            .hint_text(tr("Поиск…"))
                            .desired_width(inner_w),
                    );
                    ui.data_mut(|d| d.insert_temp(filter_id, filter.clone()));
                    let query = filter.trim().to_lowercase();
                    egui::ScrollArea::vertical().max_height(260.0).auto_shrink([true, true]).show(ui, |ui| {
                        for (code, name) in crate::flags::sorted(crate::i18n::current()) {
                            if list.iter().any(|c| c.as_str() == code) {
                                continue;
                            }
                            // Match the name in the current language or the English one.
                            let english = crate::flags::name_in(crate::i18n::Lang::En, code).unwrap_or("");
                            if !query.is_empty()
                                && !name.to_lowercase().contains(&query)
                                && !english.to_lowercase().contains(&query)
                            {
                                continue;
                            }
                            ui.horizontal(|ui| {
                                crate::flags::icon(ui, code, 14.0);
                                if ui.selectable_label(false, name).clicked() {
                                    picked = Some(code);
                                }
                            });
                        }
                    });
                });
            });
        let (pressed_at, esc) = ui.ctx().input(|i| {
            (
                if i.pointer.any_pressed() { i.pointer.interact_pos() } else { None },
                i.key_pressed(egui::Key::Escape),
            )
        });
        let outside = pressed_at.map_or(false, |pos| !area.response.rect.contains(pos) && !btn.rect.contains(pos));
        if picked.is_some() || outside || esc {
            open = false;
        }
    }
    ui.data_mut(|d| d.insert_temp(open_id, open));
    picked
}

/// Выбор национальностей (до двух, все страны мира).
fn nationality_row(ui: &mut egui::Ui, list: &mut Vec<String>, width: f32) {
    ui.vertical(|ui| {
        let mut remove: Option<usize> = None;
        for (i, code) in list.iter().enumerate() {
            ui.allocate_ui_with_layout(egui::vec2(width, 24.0), Layout::left_to_right(Align::Center), |ui| {
                crate::flags::icon(ui, code, 16.0);
                ui.label(crate::flags::name(code).unwrap_or(code.as_str()));
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if icon_button(ui, Icon::Cross, tr("Убрать"), true).clicked() {
                        remove = Some(i);
                    }
                });
            });
        }
        if let Some(i) = remove {
            list.remove(i);
        }
        if list.len() < crate::flags::MAX_PER_PERSON {
            if let Some(code) = country_picker(ui, list, width) {
                list.push(code.to_string());
            }
        }
    });
}

/// Правая панель: форма выбранного человека.
pub fn editor(
    ui: &mut egui::Ui,
    st: &mut EditState,
    tree: &mut Tree,
    store: &mut MediaStore,
    id: Option<PersonId>,
    placed: bool,
) -> Change {
    let mut ch = Change::default();
    let Some(id) = id.filter(|i| tree.persons.contains_key(i)) else {
        ui.add_space(8.0);
        ui.label(tr("Выберите человека в списке или кликните по карточке на схеме."));
        return ch;
    };
    if st.confirm_delete.is_some() && st.confirm_delete != Some(id) {
        st.confirm_delete = None;
    }

    let orig = tree.persons[&id].clone();
    let mut p = orig.clone();
    let mut actions: Vec<Action> = Vec::new();
    let mut errors: Vec<String> = Vec::new();
    let is_root = tree.root == Some(id);
    let partners = tree.partners_of(id);

    let muted = crate::theme::p().muted;
    egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
        ui.add_space(2.0);
        ui.heading(full_name(&orig));
        // Always drawn (a space when empty): otherwise the form jumps down when the first digit
        // of the birth year appears here.
        let years = format_years(&orig);
        ui.label(RichText::new(if years.is_empty() { " ".to_string() } else { years }).color(muted));
        if !placed {
            ui.add_space(4.0);
            ui.colored_label(
                Color32::from_rgb(220, 160, 40),
                "Этот человек пока не связан с деревом «я» и не виден на схеме. \
                 Добавьте связь ниже.",
            );
        }
        ui.add_space(10.0);

        // --- Основное ---
        section(ui, Some(tr("Основное")), |ui| {
            let w = (ui.available_width() - LABEL_W - GRID_GAP).max(100.0);
            egui::Grid::new("edit-main").num_columns(2).spacing([GRID_GAP, 8.0]).min_col_width(LABEL_W).show(ui, |ui| {
                field_label(ui, tr("Имя"));
                ui.add(egui::TextEdit::singleline(&mut p.first_name).desired_width(w));
                ui.end_row();
                let female = p.gender == Gender::Female;
                field_label(ui, if female { tr("Фамилия (сейчас)") } else { tr("Фамилия") });
                ui.add(egui::TextEdit::singleline(&mut p.last_name).desired_width(w));
                ui.end_row();
                if female {
                    field_label(ui, tr("Фамилия при рождении"));
                    ui.add(
                        egui::TextEdit::singleline(&mut p.maiden_name)
                            .hint_text(tr("если сменила после свадьбы"))
                            .desired_width(w),
                    );
                    ui.end_row();
                }
                field_label(ui, tr("Отчество"));
                ui.add(egui::TextEdit::singleline(&mut p.patronymic).desired_width(w));
                ui.end_row();
                field_label(ui, tr("Пол"));
                let gender_text = match p.gender {
                    Gender::Male => tr("Мужской"),
                    Gender::Female => tr("Женский"),
                    Gender::Unknown => tr("Не указан"),
                };
                egui::ComboBox::from_id_salt("gender").width(w).selected_text(gender_text).show_ui(ui, |ui| {
                    ui.selectable_value(&mut p.gender, Gender::Unknown, tr("Не указан"));
                    ui.selectable_value(&mut p.gender, Gender::Male, tr("Мужской"));
                    ui.selectable_value(&mut p.gender, Gender::Female, tr("Женский"));
                });
                ui.end_row();
            });
            // Outside the grid: a grid centers a cell vertically, and this block is taller than a line.
            ui.add_space(2.0);
            ui.horizontal_top(|ui| {
                ui.spacing_mut().item_spacing.x = GRID_GAP;
                field_label(ui, tr("Национальность"));
                nationality_row(ui, &mut p.nationalities, w);
            });
        });

        // --- Даты и места ---
        section(ui, Some(tr("Даты и места")), |ui| {
            let w = (ui.available_width() - LABEL_W - GRID_GAP).max(100.0);
            egui::Grid::new("edit-dates").num_columns(2).spacing([GRID_GAP, 8.0]).min_col_width(LABEL_W).show(ui, |ui| {
                field_label(ui, tr("Родился(ась)"));
                date_row(ui, id, "birth", &mut p.birth, w);
                ui.end_row();
                field_label(ui, tr("Место рождения"));
                ui.add(egui::TextEdit::singleline(&mut p.birth_place).desired_width(w));
                ui.end_row();
                field_label(ui, tr("Умер(ла)"));
                date_row(ui, id, "death", &mut p.death, w);
                ui.end_row();
                field_label(ui, tr("Место смерти"));
                ui.add(egui::TextEdit::singleline(&mut p.death_place).desired_width(w));
                ui.end_row();
                field_label(ui, tr("Специальность"));
                ui.add(egui::TextEdit::singleline(&mut p.profession).desired_width(w));
                ui.end_row();
            });
        });

        // --- Заметки ---
        section(ui, Some(tr("Заметки")), |ui| {
            ui.add(
                egui::TextEdit::multiline(&mut p.notes)
                    .desired_rows(5)
                    .desired_width(f32::INFINITY),
            );
        });

        // --- Фото ---
        section(ui, Some(tr("Главное фото (на карточке)")), |ui| {
            ui.horizontal(|ui| {
                match p.main_photo.clone() {
                    Some(h) => thumb_image(ui, st, store, &h, THUMB_MAIN),
                    None => {
                        let (rect, _) = ui.allocate_exact_size(THUMB_MAIN, Sense::hover());
                        let vis = ui.visuals();
                        ui.painter().rect(
                            rect,
                            6.0,
                            vis.faint_bg_color,
                            Stroke::new(1.0_f32, vis.widgets.noninteractive.bg_stroke.color),
                        );
                        centered_text(ui, rect, tr("нет"));
                    }
                }
                ui.vertical(|ui| {
                    if ui.button(tr("Выбрать…")).clicked() {
                        let ctx = ui.ctx().clone();
                        open_crop_from_file(&ctx, st, id, &mut errors);
                    }
                    if p.main_photo.is_some() && icon_text_button(ui, Icon::Cross, tr("Убрать"), None).clicked() {
                        p.main_photo = None;
                    }
                });
            });

            ui.add_space(8.0);
            ui.separator();
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.label(RichText::new(trf("Фото в заметках: {n}", &[("n", p.photos.len().to_string())])).strong());
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if icon_text_button(ui, Icon::Plus, tr("Добавить…"), None).clicked() {
                        for h in import_dialog(store, true, &mut errors) {
                            if !p.photos.iter().any(|ph| ph.hash == h) {
                                p.photos.push(Photo { hash: h, caption: String::new() });
                            }
                        }
                    }
                });
            });
            let mut remove_photo: Option<usize> = None;
            let mut make_main: Option<usize> = None;
            for (i, ph) in p.photos.iter_mut().enumerate() {
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    thumb_image(ui, st, store, &ph.hash, THUMB_SMALL);
                    ui.vertical(|ui| {
                        ui.add(
                            egui::TextEdit::singleline(&mut ph.caption)
                                .hint_text(tr("Подпись…"))
                                .desired_width(ui.available_width().max(100.0)),
                        );
                        ui.horizontal(|ui| {
                            if ui.small_button(tr("Сделать главным")).clicked() {
                                make_main = Some(i);
                            }
                            if icon_button(ui, Icon::Cross, tr("Убрать"), true).clicked() {
                                remove_photo = Some(i);
                            }
                        });
                    });
                });
            }
            if let Some(i) = make_main {
                // Фото из заметок тоже сначала показываем в окне выбора области.
                let hash = p.photos[i].hash.clone();
                let img = store
                    .get(MediaKind::Full, &hash)
                    .map_err(|e| e.to_string())
                    .and_then(|bytes| media::decode(&bytes).map_err(|e| e.to_string()));
                match img {
                    Ok(img) => {
                        let ctx = ui.ctx().clone();
                        st.crop = Some(CropSession { person: id, dlg: CropDialog::new(&ctx, img), from_gallery: true });
                    }
                    Err(e) => errors.push(e),
                }
            }
            if let Some(i) = remove_photo {
                p.photos.remove(i);
            }
        });

        // --- Связи ---
        section(ui, Some(tr("Связи")), |ui| {
            let groups = [
                (tr("Родители"), Rel::Parent, tree.parents_of(id)),
                (tr("Партнёры"), Rel::Partner, partners.clone()),
                (tr("Дети"), Rel::Child, tree.children_of(id)),
            ];
            for (n, (title, rel, ids)) in groups.into_iter().enumerate() {
                if n > 0 {
                    ui.add_space(6.0);
                }
                ui.label(RichText::new(title).color(muted));
                if ids.is_empty() {
                    ui.label(RichText::new("—").color(muted));
                }
                for rid in ids {
                    let Some(r) = tree.persons.get(&rid) else { continue };
                    ui.horizontal(|ui| {
                        if ui.link(full_name(r)).clicked() {
                            actions.push(Action::Select(rid));
                        }
                        let years = format_years(r);
                        if !years.is_empty() {
                            ui.label(RichText::new(years).color(muted));
                        }
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            if icon_button(ui, Icon::Cross, tr("Убрать связь"), true).clicked() {
                                actions.push(Action::Remove(rel, rid));
                            }
                        });
                    });
                }
            }

            ui.add_space(10.0);
            ui.separator();
            ui.add_space(6.0);
            ui.label(RichText::new(tr("Добавить нового человека:")).color(muted));
            let gap = 8.0;
            let bw = ((ui.available_width() - gap) / 2.0).floor();
            egui::Grid::new("edit-add-rel").num_columns(2).spacing([gap, 6.0]).show(ui, |ui| {
                if icon_text_button(ui, Icon::Plus, tr("Родитель"), Some(bw)).clicked() {
                    actions.push(Action::New(Rel::Parent));
                }
                if icon_text_button(ui, Icon::Plus, tr("Партнёр"), Some(bw)).clicked() {
                    actions.push(Action::New(Rel::Partner));
                }
                ui.end_row();
                if icon_text_button(ui, Icon::Plus, tr("Ребёнок"), Some(bw)).clicked() {
                    actions.push(Action::New(Rel::Child));
                }
                if icon_text_button(ui, Icon::Plus, tr("Брат/сестра"), Some(bw)).clicked() {
                    actions.push(Action::New(Rel::Sibling));
                }
                ui.end_row();
            });

            ui.add_space(10.0);
            ui.label(RichText::new(tr("Связать с существующим человеком:")).color(muted));
            ui.add(
                egui::TextEdit::singleline(&mut st.rel_search)
                    .hint_text(tr("Начните вводить имя…"))
                    .desired_width(f32::INFINITY),
            );
            let q = st.rel_search.trim().to_lowercase();
            if !q.is_empty() {
                let found: Vec<&Person> = tree
                    .persons
                    .values()
                    .filter(|o| o.id != id && full_name(o).to_lowercase().contains(&q))
                    .take(MAX_REL_RESULTS)
                    .collect();
                if found.is_empty() {
                    ui.label(RichText::new(tr("Никого не найдено")).color(muted));
                }
                for o in found {
                    ui.add_space(2.0);
                    ui.horizontal_wrapped(|ui| {
                        let years = format_years(o);
                        let name = if years.is_empty() {
                            full_name(o)
                        } else {
                            format!("{} ({years})", full_name(o))
                        };
                        ui.label(name);
                        if ui.small_button(tr("родитель")).clicked() {
                            actions.push(Action::Link(Rel::Parent, o.id));
                        }
                        if ui.small_button(tr("партнёр")).clicked() {
                            actions.push(Action::Link(Rel::Partner, o.id));
                        }
                        if ui.small_button(tr("ребёнок")).clicked() {
                            actions.push(Action::Link(Rel::Child, o.id));
                        }
                    });
                }
            }
        });

        // --- Корень и удаление ---
        section(ui, None, |ui| {
            if is_root {
                ui.horizontal(|ui| {
                    let color = ui.visuals().text_color();
                    inline_icon(ui, Icon::Check, color);
                    ui.label(tr("Это «я» — схема строится от предков этого человека."));
                });
            } else if ui
                .add_sized([ui.available_width(), 28.0], egui::Button::new(tr("Это «я» (сделать корнем дерева)")))
                .clicked()
            {
                actions.push(Action::SetRoot);
            }
            ui.add_space(8.0);
            let danger = crate::theme::p().danger;
            if st.confirm_delete == Some(id) {
                ui.colored_label(danger, tr("Удалить человека вместе со всеми его связями?"));
                ui.horizontal(|ui| {
                    if ui.add(egui::Button::new(RichText::new(tr("Да, удалить")).color(danger))).clicked() {
                        actions.push(Action::Delete);
                    }
                    if ui.button(tr("Отмена")).clicked() {
                        st.confirm_delete = None;
                    }
                });
            } else if ui
                .add_sized(
                    [ui.available_width(), 28.0],
                    egui::Button::new(RichText::new(tr("Удалить человека…")).color(danger)),
                )
                .clicked()
            {
                st.confirm_delete = Some(id);
            }
        });
        ui.add_space(12.0);
    });

    // Окно выбора области главного фото. Если выбрали другого человека — окно закрывается.
    if st.crop.as_ref().is_some_and(|c| c.person != id) {
        st.crop = None;
    }
    if let Some(mut session) = st.crop.take() {
        match session.dlg.show(ui.ctx()) {
            CropOutcome::Open => st.crop = Some(session),
            CropOutcome::Cancel => {}
            CropOutcome::Done(rect) => match media::import_cropped(store, session.dlg.image(), rect) {
                Ok(h) => {
                    if session.from_gallery {
                        if let Some(old) = p.main_photo.take() {
                            if old != h && !p.photos.iter().any(|ph| ph.hash == old) {
                                p.photos.push(Photo { hash: old, caption: String::new() });
                            }
                        }
                    }
                    p.main_photo = Some(h);
                }
                Err(e) => errors.push(e.to_string()),
            },
        }
    }

    // Правки полей.
    if p != orig {
        if p.birth.year != orig.birth.year {
            ch.relayout = true;
        }
        tree.persons.insert(id, p);
        ch.dirty = true;
    }

    // Действия со связями.
    for a in actions {
        match a {
            Action::Select(o) => ch.select = Some(o),
            Action::Remove(rel, o) => {
                match rel {
                    Rel::Parent => tree.remove_parent_child(o, id),
                    Rel::Partner => tree.remove_partners(id, o),
                    Rel::Child => tree.remove_parent_child(id, o),
                    Rel::Sibling => false,
                };
                ch.dirty = true;
                ch.relayout = true;
            }
            Action::Link(rel, o) => {
                let r = match rel {
                    Rel::Parent => link_parent(tree, o, id),
                    Rel::Partner => tree.add_partners(id, o).map_err(|e| e.to_string()),
                    Rel::Child => link_parent(tree, id, o),
                    Rel::Sibling => Err(tr("Нельзя связать так").to_string()),
                };
                match r {
                    Ok(()) => {
                        ch.dirty = true;
                        ch.relayout = true;
                        st.rel_search.clear();
                    }
                    Err(e) => errors.push(e),
                }
            }
            Action::New(rel) => {
                let (first, last) = placeholder_name(match rel {
                    Rel::Parent => "Новый родитель",
                    Rel::Partner => "Новый партнёр",
                    Rel::Child => "Новый ребёнок",
                    Rel::Sibling => "Новый брат/сестра",
                });
                let my_parents = tree.parents_of(id);
                if rel == Rel::Sibling && my_parents.is_empty() {
                    errors.push(tr("Чтобы добавить брата/сестру, сначала добавьте родителя").to_string());
                    continue;
                }
                let nid = tree.add_person(Person::new(&first, &last));
                let r = match rel {
                    Rel::Parent => link_parent(tree, nid, id),
                    Rel::Partner => tree.add_partners(id, nid).map_err(|e| e.to_string()),
                    Rel::Child => {
                        let r = link_parent(tree, id, nid);
                        // Если у человека один партнёр — он тоже родитель ребёнка.
                        if r.is_ok() && partners.len() == 1 {
                            let _ = link_parent(tree, partners[0], nid);
                        }
                        r
                    }
                    Rel::Sibling => {
                        // Те же родители, что и у выбранного (второй станет партнёром первого).
                        let mut r = Ok(());
                        for p in &my_parents {
                            if let Err(e) = link_parent(tree, *p, nid) {
                                r = Err(e);
                                break;
                            }
                        }
                        r
                    }
                };
                match r {
                    Ok(()) => {
                        ch.dirty = true;
                        ch.relayout = true;
                        // Выбор НЕ меняется: можно подряд добавить мать и отца к одному человеку.
                    }
                    Err(e) => {
                        tree.remove_person(nid);
                        errors.push(e);
                    }
                }
            }
            Action::SetRoot => {
                tree.root = Some(id);
                ch.dirty = true;
                ch.relayout = true;
            }
            Action::Delete => {
                tree.remove_person(id);
                st.confirm_delete = None;
                ch.dirty = true;
                ch.relayout = true;
                ch.deselect = true;
            }
        }
    }
    if !errors.is_empty() {
        ch.error = Some(errors.join("; "));
    }
    ch
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::{compute, LayoutConfig};

    #[test]
    fn two_parents_are_partners_above_child() {
        let mut t = Tree::new();
        let me = t.add_person(Person::new("Я", "Я"));
        t.root = Some(me);
        let mom = t.add_person(Person::new("Мать", "М"));
        let dad = t.add_person(Person::new("Отец", "О"));
        link_parent(&mut t, mom, me).unwrap();
        link_parent(&mut t, dad, me).unwrap();
        assert_eq!(t.parents_of(me).len(), 2);
        assert!(t.children_of(me).is_empty());
        assert!(t.partners_of(mom).contains(&dad));
        let l = compute(&t, &LayoutConfig::default());
        let (pm, pd, pme) = (l.pos_of(mom).unwrap(), l.pos_of(dad).unwrap(), l.pos_of(me).unwrap());
        assert!(pm.y < pme.y && pd.y < pme.y); // родители выше
        assert!((pm.y - pd.y).abs() < 0.01); // в одном ряду
        assert!(pm.x != pd.x);
    }

    #[test]
    fn third_parent_is_rejected() {
        let mut t = Tree::new();
        let me = t.add_person(Person::new("Я", "Я"));
        for n in ["А", "Б"] {
            let p = t.add_person(Person::new(n, n));
            link_parent(&mut t, p, me).unwrap();
        }
        let x = t.add_person(Person::new("В", "В"));
        assert!(link_parent(&mut t, x, me).is_err());
        assert_eq!(t.parents_of(me).len(), 2);
    }

    #[test]
    fn child_of_person_goes_below() {
        let mut t = Tree::new();
        let me = t.add_person(Person::new("Я", "Я"));
        t.root = Some(me);
        let kid = t.add_person(Person::new("Р", "Р"));
        link_parent(&mut t, me, kid).unwrap();
        let l = compute(&t, &LayoutConfig::default());
        assert!(l.pos_of(kid).unwrap().y > l.pos_of(me).unwrap().y);
    }
}
