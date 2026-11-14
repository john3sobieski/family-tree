//! Режим «Редакция»: список людей слева, форма человека справа
//! (данные, фото, связи, удаление). Схема посередине — обычный `view`.

use crate::container::{MediaKind, MediaStore};
use crate::media;
use crate::model::{Gender, PartialDate, Person, PersonId, Photo, Tree};
use crate::person_window::{format_years, full_name, load_texture};
use eframe::egui::{self, Color32, RichText, Sense, TextureHandle, Vec2};
use std::collections::HashMap;

const THUMB_SMALL: Vec2 = Vec2::new(45.0, 60.0);
const THUMB_MAIN: Vec2 = Vec2::new(75.0, 100.0);
const MAX_THUMBS: usize = 200;
const MAX_REL_RESULTS: usize = 8;

#[derive(Default)]
pub struct EditState {
    pub list_search: String,
    rel_search: String,
    confirm_delete: Option<PersonId>,
    thumbs: HashMap<String, Option<TextureHandle>>,
}

impl EditState {
    pub fn reset(&mut self) {
        self.rel_search.clear();
        self.confirm_delete = None;
        self.thumbs.clear();
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
    ui.heading("Люди");
    ui.add(
        egui::TextEdit::singleline(&mut st.list_search)
            .hint_text("Поиск по имени…")
            .desired_width(f32::INFINITY),
    );
    if ui.button("＋ Новый человек").clicked() {
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

/// Создаёт нового человека. Первый человек в пустом дереве становится «я».
pub fn new_person(tree: &mut Tree) -> PersonId {
    let id = tree.add_person(Person::new("Новый", "человек"));
    if tree.root.is_none() {
        tree.root = Some(id);
    }
    id
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Rel {
    Parent,
    Partner,
    Child,
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
    let resp = ui.add(egui::TextEdit::singleline(&mut buf).hint_text(hint).desired_width(width));
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

fn date_row(ui: &mut egui::Ui, id: PersonId, tag: &str, d: &mut PartialDate) {
    ui.horizontal(|ui| {
        let k = ui.id().with((id, tag, "day"));
        let mut day = d.day.map(i64::from);
        num_field(ui, k, &mut day, 1, 31, "день", 42.0);
        d.day = day.map(|v| v as u8);

        let k = ui.id().with((id, tag, "month"));
        let mut month = d.month.map(i64::from);
        num_field(ui, k, &mut month, 1, 12, "месяц", 52.0);
        d.month = month.map(|v| v as u8);

        let k = ui.id().with((id, tag, "year"));
        let mut year = d.year.map(i64::from);
        num_field(ui, k, &mut year, 1, 2999, "год", 56.0);
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
    let dlg = rfd::FileDialog::new().add_filter("Изображения", &["jpg", "jpeg", "png", "webp", "bmp"]);
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
        ui.label("Выберите человека в списке или кликните по карточке на схеме.");
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

    egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
        ui.heading(full_name(&orig));
        if !placed {
            ui.colored_label(
                Color32::from_rgb(220, 160, 40),
                "Этот человек пока не связан с деревом «я» и не виден на схеме. \
                 Добавьте связь ниже.",
            );
        }
        ui.add_space(6.0);

        egui::Grid::new("edit-main").num_columns(2).spacing([10.0, 6.0]).show(ui, |ui| {
            let w = (ui.available_width() - 4.0).max(120.0);
            ui.label("Имя");
            ui.add(egui::TextEdit::singleline(&mut p.first_name).desired_width(w));
            ui.end_row();
            ui.label("Фамилия");
            ui.add(egui::TextEdit::singleline(&mut p.last_name).desired_width(w));
            ui.end_row();
            ui.label("Пол");
            let gender_text = match p.gender {
                Gender::Male => "Мужской",
                Gender::Female => "Женский",
                Gender::Unknown => "Не указан",
            };
            egui::ComboBox::from_id_salt("gender").selected_text(gender_text).show_ui(ui, |ui| {
                ui.selectable_value(&mut p.gender, Gender::Unknown, "Не указан");
                ui.selectable_value(&mut p.gender, Gender::Male, "Мужской");
                ui.selectable_value(&mut p.gender, Gender::Female, "Женский");
            });
            ui.end_row();
            ui.label("Родился(ась)");
            date_row(ui, id, "birth", &mut p.birth);
            ui.end_row();
            ui.label("Умер(ла)");
            date_row(ui, id, "death", &mut p.death);
            ui.end_row();
            ui.label("Специальность");
            ui.add(egui::TextEdit::singleline(&mut p.profession).desired_width(w));
            ui.end_row();
        });

        ui.add_space(6.0);
        ui.label("Заметки");
        ui.add(
            egui::TextEdit::multiline(&mut p.notes)
                .desired_rows(5)
                .desired_width(f32::INFINITY),
        );

        // --- Фото ---
        ui.separator();
        ui.label(RichText::new("Главное фото (на карточке)").strong());
        ui.horizontal(|ui| {
            match p.main_photo.clone() {
                Some(h) => thumb_image(ui, st, store, &h, THUMB_MAIN),
                None => {
                    ui.label("нет");
                }
            }
            ui.vertical(|ui| {
                if ui.button("Выбрать…").clicked() {
                    if let Some(h) = import_dialog(store, false, &mut errors).into_iter().next() {
                        p.main_photo = Some(h);
                    }
                }
                if p.main_photo.is_some() && ui.button("Убрать").clicked() {
                    p.main_photo = None;
                }
            });
        });

        ui.add_space(6.0);
        ui.horizontal(|ui| {
            ui.label(RichText::new(format!("Фото в заметках: {}", p.photos.len())).strong());
            if ui.button("Добавить…").clicked() {
                for h in import_dialog(store, true, &mut errors) {
                    if !p.photos.iter().any(|ph| ph.hash == h) {
                        p.photos.push(Photo { hash: h, caption: String::new() });
                    }
                }
            }
        });
        let mut remove_photo: Option<usize> = None;
        let mut make_main: Option<usize> = None;
        for (i, ph) in p.photos.iter_mut().enumerate() {
            ui.horizontal(|ui| {
                thumb_image(ui, st, store, &ph.hash, THUMB_SMALL);
                ui.vertical(|ui| {
                    ui.add(
                        egui::TextEdit::singleline(&mut ph.caption)
                            .hint_text("Подпись…")
                            .desired_width(ui.available_width().max(100.0)),
                    );
                    ui.horizontal(|ui| {
                        if ui.small_button("Сделать главным").clicked() {
                            make_main = Some(i);
                        }
                        if ui.small_button("✕ Убрать").clicked() {
                            remove_photo = Some(i);
                        }
                    });
                });
            });
        }
        if let Some(i) = make_main {
            let hash = p.photos[i].hash.clone();
            if let Some(old) = p.main_photo.replace(hash) {
                if !p.photos.iter().any(|ph| ph.hash == old) {
                    p.photos.push(Photo { hash: old, caption: String::new() });
                }
            }
        }
        if let Some(i) = remove_photo {
            p.photos.remove(i);
        }

        // --- Связи ---
        ui.separator();
        ui.label(RichText::new("Связи").strong());
        let groups = [
            ("Родители", Rel::Parent, tree.parents_of(id)),
            ("Партнёры", Rel::Partner, partners.clone()),
            ("Дети", Rel::Child, tree.children_of(id)),
        ];
        for (title, rel, ids) in groups {
            ui.label(RichText::new(title).weak());
            if ids.is_empty() {
                ui.label("—");
            }
            for rid in ids {
                let Some(r) = tree.persons.get(&rid) else { continue };
                ui.horizontal(|ui| {
                    if ui.link(full_name(r)).clicked() {
                        actions.push(Action::Select(rid));
                    }
                    let years = format_years(r);
                    if !years.is_empty() {
                        ui.label(RichText::new(years).weak());
                    }
                    if ui.small_button("✕").on_hover_text("Убрать связь").clicked() {
                        actions.push(Action::Remove(rel, rid));
                    }
                });
            }
        }

        ui.add_space(6.0);
        ui.label("Добавить нового человека:");
        ui.horizontal_wrapped(|ui| {
            if ui.button("＋ Родитель").clicked() {
                actions.push(Action::New(Rel::Parent));
            }
            if ui.button("＋ Партнёр").clicked() {
                actions.push(Action::New(Rel::Partner));
            }
            if ui.button("＋ Ребёнок").clicked() {
                actions.push(Action::New(Rel::Child));
            }
        });

        ui.add_space(6.0);
        ui.label("Связать с существующим человеком:");
        ui.add(
            egui::TextEdit::singleline(&mut st.rel_search)
                .hint_text("Начните вводить имя…")
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
                ui.label(RichText::new("Никого не найдено").weak());
            }
            for o in found {
                ui.horizontal_wrapped(|ui| {
                    let years = format_years(o);
                    let name = if years.is_empty() {
                        full_name(o)
                    } else {
                        format!("{} ({years})", full_name(o))
                    };
                    ui.label(name);
                    if ui.small_button("родитель").clicked() {
                        actions.push(Action::Link(Rel::Parent, o.id));
                    }
                    if ui.small_button("партнёр").clicked() {
                        actions.push(Action::Link(Rel::Partner, o.id));
                    }
                    if ui.small_button("ребёнок").clicked() {
                        actions.push(Action::Link(Rel::Child, o.id));
                    }
                });
            }
        }

        // --- Корень и удаление ---
        ui.separator();
        if is_root {
            ui.label("✔ Это «я» — схема строится от предков этого человека.");
        } else if ui.button("Это «я» (сделать корнем дерева)").clicked() {
            actions.push(Action::SetRoot);
        }
        ui.add_space(8.0);
        if st.confirm_delete == Some(id) {
            ui.colored_label(Color32::from_rgb(220, 70, 70), "Удалить человека вместе со всеми его связями?");
            ui.horizontal(|ui| {
                if ui.button("Да, удалить").clicked() {
                    actions.push(Action::Delete);
                }
                if ui.button("Отмена").clicked() {
                    st.confirm_delete = None;
                }
            });
        } else if ui.button("Удалить человека…").clicked() {
            st.confirm_delete = Some(id);
        }
        ui.add_space(20.0);
    });

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
                };
                ch.dirty = true;
                ch.relayout = true;
            }
            Action::Link(rel, o) => {
                let r = match rel {
                    Rel::Parent => tree.add_parent_child(o, id),
                    Rel::Partner => tree.add_partners(id, o),
                    Rel::Child => tree.add_parent_child(id, o),
                };
                match r {
                    Ok(()) => {
                        ch.dirty = true;
                        ch.relayout = true;
                        st.rel_search.clear();
                    }
                    Err(e) => errors.push(e.to_string()),
                }
            }
            Action::New(rel) => {
                let nid = tree.add_person(Person::new("Новый", "человек"));
                let r = match rel {
                    Rel::Parent => tree.add_parent_child(nid, id),
                    Rel::Partner => tree.add_partners(id, nid),
                    Rel::Child => {
                        let r = tree.add_parent_child(id, nid);
                        // Если у человека один партнёр — он тоже родитель ребёнка.
                        if partners.len() == 1 {
                            let _ = tree.add_parent_child(partners[0], nid);
                        }
                        r
                    }
                };
                if let Err(e) = r {
                    errors.push(e.to_string());
                }
                ch.dirty = true;
                ch.relayout = true;
                ch.select = Some(nid);
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
