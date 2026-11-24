// Full-screen person page for the phone: photo, gallery, data, relatives (tap to jump), notes.
// Data helpers (names, dates, age) are shared with the desktop person window.
use crate::container::{MediaKind, MediaStore};
use crate::i18n::{tr, trf};
use crate::model::{Gender, Person, PersonId, Tree};
use crate::person_window::{
    age_text, format_date, format_years, full_name, load_texture, nationalities_text, non_empty,
    photo_list, today,
};
use crate::view::draw_placeholder;
use eframe::egui::{self, Align2, FontId, RichText, Sense, TextureHandle, Vec2};
use std::collections::HashMap;

/// Temporary top margin (status bar); proper insets come in step A7.
const TOP_INSET: f32 = 42.0;
/// Gallery thumbnail size (same 3:4 as the card photo).
const GAL_W: f32 = 72.0;
const GAL_H: f32 = 96.0;
const MAX_FULL: usize = 4;
const MAX_THUMBS: usize = 200;

pub enum Event {
    None,
    /// "Back" button: previous person, or the tree.
    Back,
    /// A relative was tapped.
    Goto(PersonId),
}

#[derive(Default)]
pub struct PersonScreen {
    for_id: Option<PersonId>,
    /// Index of the big photo in the person's photo list.
    current: usize,
    full: HashMap<String, Option<TextureHandle>>,
    thumbs: HashMap<String, Option<TextureHandle>>,
}

impl PersonScreen {
    /// Drops cached textures (another file was loaded).
    pub fn reset(&mut self) {
        self.for_id = None;
        self.current = 0;
        self.full.clear();
        self.thumbs.clear();
    }

    pub fn show(&mut self, ctx: &egui::Context, tree: &Tree, store: &MediaStore, id: PersonId) -> Event {
        let Some(person) = tree.persons.get(&id) else {
            return Event::Back;
        };
        let photos = photo_list(person);
        if self.for_id != Some(id) {
            self.for_id = Some(id);
            self.current = person
                .main_photo
                .as_ref()
                .and_then(|m| photos.iter().position(|(h, _)| h == m))
                .unwrap_or(0);
        }
        if self.current >= photos.len() {
            self.current = 0;
        }

        let mut event = Event::None;
        egui::TopBottomPanel::top("person-bar").frame(crate::theme::bar_frame(10.0, 6.0)).show(ctx, |ui| {
            ui.add_space(TOP_INSET);
            let back = egui::Button::new(tr("Назад")).min_size(Vec2::new(0.0, 44.0));
            if ui.add(back).clicked() {
                event = Event::Back;
            }
        });
        egui::CentralPanel::default().show(ctx, |ui| {
            // Own id per person: the scroll position starts from the top for each one.
            ui.push_id(id, |ui| {
                egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
                    self.big_photo(ui, store, &photos);
                    self.gallery(ui, store, &photos);
                    if let Some(e) = info(ui, tree, person) {
                        event = e;
                    }
                    ui.add_space(24.0);
                });
            });
        });
        event
    }

    fn big_photo(&mut self, ui: &mut egui::Ui, store: &MediaStore, photos: &[(String, String)]) {
        let w = ui.available_width();
        let max_h = (ui.ctx().screen_rect().height() * 0.6).max(200.0);
        let tex = photos
            .get(self.current)
            .and_then(|(h, _)| self.full_texture(ui.ctx(), store, h));
        match tex {
            Some((tid, size)) => {
                let k = (w / size.x).min(max_h / size.y).max(0.01);
                ui.vertical_centered(|ui| {
                    ui.image((tid, size * k));
                });
            }
            None => {
                let h = (w * 4.0 / 3.0).min(max_h);
                let (rect, _) = ui.allocate_exact_size(Vec2::new(w, h), Sense::hover());
                let v = ui.visuals();
                draw_placeholder(ui.painter(), rect, v.widgets.noninteractive.bg_stroke.color, v.faint_bg_color);
                if !photos.is_empty() {
                    ui.painter().text(
                        rect.center(),
                        Align2::CENTER_CENTER,
                        tr("не удалось загрузить фото"),
                        FontId::proportional(14.0),
                        v.weak_text_color(),
                    );
                }
            }
        }
        if let Some((_, cap)) = photos.get(self.current) {
            if !cap.trim().is_empty() {
                ui.label(RichText::new(cap).italics());
            }
        }
    }

    fn gallery(&mut self, ui: &mut egui::Ui, store: &MediaStore, photos: &[(String, String)]) {
        if photos.len() < 2 {
            return;
        }
        // Missing thumbnails are read in one go: the archive is opened once.
        let mut missing: Vec<String> = Vec::new();
        for (h, _) in photos {
            if !self.thumbs.contains_key(h) && !missing.contains(h) {
                missing.push(h.clone());
            }
        }
        if !missing.is_empty() {
            if self.thumbs.len() + missing.len() > MAX_THUMBS {
                self.thumbs.clear();
            }
            let mut got = store.get_many(MediaKind::Thumb, &missing);
            for h in missing {
                let tex = got.remove(&h).and_then(|b| decode_texture(ui.ctx(), &h, &b));
                self.thumbs.insert(h, tex);
            }
        }

        ui.add_space(6.0);
        ui.label(RichText::new(trf("Фото: {n}", &[("n", photos.len().to_string())])).weak());
        let accent = crate::theme::p().accent;
        ui.horizontal_wrapped(|ui| {
            for (i, (hash, _)) in photos.iter().enumerate() {
                let size = Vec2::new(GAL_W, GAL_H);
                let tex = self.thumbs.get(hash).and_then(|t| t.as_ref()).map(|t| t.id());
                let resp = match tex {
                    Some(tid) => ui.add(egui::Image::new((tid, size)).sense(Sense::click())),
                    None => {
                        let (rect, r) = ui.allocate_exact_size(size, Sense::click());
                        ui.painter().rect_filled(rect, 0.0, ui.visuals().faint_bg_color);
                        r
                    }
                };
                if i == self.current {
                    ui.painter().rect_stroke(resp.rect, 0.0_f32, egui::Stroke::new(2.5_f32, accent));
                }
                if resp.clicked() {
                    self.current = i;
                }
            }
        });
    }

    fn full_texture(&mut self, ctx: &egui::Context, store: &MediaStore, hash: &str) -> Option<(egui::TextureId, Vec2)> {
        if !self.full.contains_key(hash) {
            if self.full.len() >= MAX_FULL {
                self.full.clear();
            }
            let t = load_texture(ctx, store, MediaKind::Full, hash);
            self.full.insert(hash.to_string(), t);
        }
        self.full.get(hash)?.as_ref().map(|t| (t.id(), t.size_vec2()))
    }
}

fn decode_texture(ctx: &egui::Context, hash: &str, bytes: &[u8]) -> Option<TextureHandle> {
    let img = image::load_from_memory(bytes).ok()?.to_rgba8();
    let size = [img.width() as usize, img.height() as usize];
    let color = egui::ColorImage::from_rgba_unmultiplied(size, img.as_raw());
    Some(ctx.load_texture(format!("ps-thumb-{hash}"), color, egui::TextureOptions::LINEAR))
}

fn info(ui: &mut egui::Ui, tree: &Tree, p: &Person) -> Option<Event> {
    let mut event = None;
    ui.add_space(10.0);
    ui.label(RichText::new(full_name(p)).size(24.0).strong());

    let gender = match p.gender {
        Gender::Male => Some(tr("Мужской").to_string()),
        Gender::Female => Some(tr("Женский").to_string()),
        Gender::Unknown => None,
    };
    let rows: Vec<(&str, String)> = [
        (tr("Пол"), gender),
        (tr("Родился(ась)"), format_date(&p.birth)),
        (tr("Место рождения"), non_empty(&p.birth_place)),
        (tr("Умер(ла)"), format_date(&p.death)),
        (tr("Место смерти"), non_empty(&p.death_place)),
        (tr("Возраст"), age_text(&p.birth, &p.death, today())),
        (tr("Национальность"), nationalities_text(p)),
        (tr("Специальность"), non_empty(&p.profession)),
    ]
    .into_iter()
    .filter_map(|(k, v)| v.map(|v| (k, v)))
    .collect();
    // Label above value: long values wrap instead of running off a narrow screen.
    for (k, v) in rows {
        ui.add_space(6.0);
        ui.label(RichText::new(k).weak().small());
        ui.label(v);
    }

    let groups: [(&str, Vec<PersonId>); 3] = [
        (tr("Родители"), tree.parents_of(p.id)),
        (tr("Партнёры"), tree.partners_of(p.id)),
        (tr("Дети"), tree.children_of(p.id)),
    ];
    for (title, ids) in groups {
        if ids.is_empty() {
            continue;
        }
        ui.add_space(14.0);
        ui.label(RichText::new(title).strong());
        for rid in ids {
            let Some(r) = tree.persons.get(&rid) else { continue };
            let years = format_years(r);
            let text = if years.is_empty() {
                full_name(r)
            } else {
                format!("{} ({years})", full_name(r))
            };
            let btn = egui::Button::new(text);
            if ui.add_sized([ui.available_width(), 44.0], btn).clicked() {
                event = Some(Event::Goto(rid));
            }
        }
    }

    if !p.notes.trim().is_empty() {
        ui.add_space(14.0);
        ui.label(RichText::new(tr("Заметки")).strong());
        ui.label(p.notes.trim());
    }
    event
}
