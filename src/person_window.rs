//! Большое окно человека: крупное фото, данные, родственники (кликабельные),
//! заметки и галерея фото. Только чтение (редактирование — шаг 9).

use crate::container::{MediaKind, MediaStore};
use crate::model::{Gender, PartialDate, Person, PersonId, Tree};
use crate::view::draw_placeholder;
use eframe::egui::{self, Align2, FontId, RichText, Sense, TextureHandle, Vec2};
use std::collections::HashMap;

/// Максимальный размер большого фото в окне.
const BIG_W: f32 = 340.0;
const BIG_H: f32 = 453.0;
/// Размер миниатюры в галерее.
const GAL_W: f32 = 66.0;
const GAL_H: f32 = 88.0;
/// Лимит кэша текстур окна.
const MAX_FULL: usize = 6;
const MAX_THUMBS: usize = 200;

/// Что попросило окно сделать приложение.
pub enum WindowEvent {
    None,
    Close,
    /// Открыть другого человека (клик по родственнику).
    Goto(PersonId),
}

#[derive(Default)]
pub struct PersonWindow {
    /// Для кого сейчас открыто (чтобы сбросить выбор фото при смене человека).
    for_id: Option<PersonId>,
    /// Какое фото показано крупно (индекс в списке фото человека).
    current: usize,
    full: HashMap<String, Option<TextureHandle>>,
    thumbs: HashMap<String, Option<TextureHandle>>,
}

impl PersonWindow {
    /// Сбросить кэш (после загрузки другого файла).
    pub fn reset(&mut self) {
        self.for_id = None;
        self.current = 0;
        self.full.clear();
        self.thumbs.clear();
    }

    pub fn show(
        &mut self,
        ctx: &egui::Context,
        tree: &Tree,
        store: &MediaStore,
        id: PersonId,
    ) -> WindowEvent {
        let Some(person) = tree.persons.get(&id) else {
            return WindowEvent::Close;
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

        let mut open = true;
        let mut event = WindowEvent::None;
        let title = full_name(person);
        egui::Window::new(title)
            .id(egui::Id::new("person-window"))
            .open(&mut open)
            .collapsible(false)
            .resizable(true)
            .default_size([820.0, 600.0])
            .min_size([560.0, 360.0])
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
                    ui.horizontal_top(|ui| {
                        ui.vertical(|ui| {
                            ui.set_max_width(BIG_W);
                            self.big_photo(ui, store, &photos);
                            self.gallery(ui, store, &photos);
                        });
                        ui.add_space(12.0);
                        ui.vertical(|ui| {
                            if let Some(e) = info_panel(ui, tree, person) {
                                event = e;
                            }
                        });
                    });
                });
            });
        if !open {
            return WindowEvent::Close;
        }
        event
    }

    fn big_photo(&mut self, ui: &mut egui::Ui, store: &MediaStore, photos: &[(String, String)]) {
        let tex = photos
            .get(self.current)
            .and_then(|(h, _)| self.full_texture(ui.ctx(), store, h));
        match tex {
            Some((tid, size)) => {
                let k = (BIG_W / size.x).min(BIG_H / size.y).min(1.0).max(0.01);
                ui.image((tid, size * k));
            }
            None => {
                let (rect, _) = ui.allocate_exact_size(Vec2::new(BIG_W, BIG_H), Sense::hover());
                let v = ui.visuals();
                draw_placeholder(
                    ui.painter(),
                    rect,
                    v.widgets.noninteractive.bg_stroke.color,
                    v.faint_bg_color,
                );
                if !photos.is_empty() {
                    ui.painter().text(
                        rect.center(),
                        Align2::CENTER_CENTER,
                        "не удалось загрузить фото",
                        FontId::proportional(13.0),
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
        ui.add_space(6.0);
        ui.label(RichText::new(format!("Фото: {}", photos.len())).weak());
        let accent = crate::theme::ACCENT;
        ui.horizontal_wrapped(|ui| {
            for (i, (hash, _)) in photos.iter().enumerate() {
                let tex = self.thumb_texture(ui.ctx(), store, hash);
                let size = Vec2::new(GAL_W, GAL_H);
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
                } else if resp.hovered() {
                    ui.painter().rect_stroke(resp.rect, 0.0_f32, egui::Stroke::new(1.5_f32, accent));
                }
                if resp.on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
                    self.current = i;
                }
            }
        });
    }

    fn full_texture(
        &mut self,
        ctx: &egui::Context,
        store: &MediaStore,
        hash: &str,
    ) -> Option<(egui::TextureId, Vec2)> {
        if !self.full.contains_key(hash) {
            if self.full.len() >= MAX_FULL {
                self.full.clear();
            }
            let t = load_texture(ctx, store, MediaKind::Full, hash);
            self.full.insert(hash.to_string(), t);
        }
        self.full.get(hash)?.as_ref().map(|t| (t.id(), t.size_vec2()))
    }

    fn thumb_texture(
        &mut self,
        ctx: &egui::Context,
        store: &MediaStore,
        hash: &str,
    ) -> Option<egui::TextureId> {
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

pub(crate) fn load_texture(
    ctx: &egui::Context,
    store: &MediaStore,
    kind: MediaKind,
    hash: &str,
) -> Option<TextureHandle> {
    let bytes = store.get(kind, hash).ok()?;
    let img = image::load_from_memory(&bytes).ok()?.to_rgba8();
    let size = [img.width() as usize, img.height() as usize];
    let color = egui::ColorImage::from_rgba_unmultiplied(size, img.as_raw());
    Some(ctx.load_texture(format!("pw-{kind:?}-{hash}"), color, egui::TextureOptions::LINEAR))
}

/// Все фото человека: главное (если его нет среди остальных) + остальные.
fn photo_list(p: &Person) -> Vec<(String, String)> {
    let mut v: Vec<(String, String)> = Vec::new();
    if let Some(m) = &p.main_photo {
        if !p.photos.iter().any(|ph| &ph.hash == m) {
            v.push((m.clone(), String::new()));
        }
    }
    for ph in &p.photos {
        v.push((ph.hash.clone(), ph.caption.clone()));
    }
    v
}

pub(crate) fn full_name(p: &Person) -> String {
    let s = format!("{} {}", p.first_name.trim(), p.last_name.trim());
    let s = s.trim();
    if s.is_empty() {
        "Без имени".to_string()
    } else {
        s.to_string()
    }
}

fn info_panel(ui: &mut egui::Ui, tree: &Tree, p: &Person) -> Option<WindowEvent> {
    let mut event = None;
    ui.set_min_width(260.0);
    ui.label(RichText::new(full_name(p)).size(24.0).strong());
    ui.add_space(4.0);

    let gender = match p.gender {
        Gender::Male => Some("Мужской"),
        Gender::Female => Some("Женский"),
        Gender::Unknown => None,
    };
    let (ty, tm, td) = today();
    let rows: Vec<(&str, String)> = [
        ("Пол", gender.map(|g| g.to_string())),
        ("Родился(ась)", format_date(&p.birth)),
        ("Место рождения", non_empty(&p.birth_place)),
        ("Умер(ла)", format_date(&p.death)),
        ("Место смерти", non_empty(&p.death_place)),
        ("Возраст", age_text(&p.birth, &p.death, (ty, tm, td))),
        ("Национальность", nationalities_text(p)),
        ("Специальность", non_empty(&p.profession)),
    ]
    .into_iter()
    .filter_map(|(k, v)| v.map(|v| (k, v)))
    .collect();

    if !rows.is_empty() {
        egui::Grid::new("person-info").num_columns(2).spacing([14.0, 6.0]).show(ui, |ui| {
            for (k, v) in rows {
                ui.label(RichText::new(k).weak());
                ui.label(v);
                ui.end_row();
            }
        });
    }

    let groups: [(&str, Vec<PersonId>); 3] = [
        ("Родители", tree.parents_of(p.id)),
        ("Партнёры", tree.partners_of(p.id)),
        ("Дети", tree.children_of(p.id)),
    ];
    for (title, ids) in groups {
        if ids.is_empty() {
            continue;
        }
        ui.add_space(10.0);
        ui.label(RichText::new(title).strong());
        ui.horizontal_wrapped(|ui| {
            for rid in ids {
                let Some(r) = tree.persons.get(&rid) else { continue };
                let years = format_years(r);
                let text = if years.is_empty() {
                    full_name(r)
                } else {
                    format!("{} ({years})", full_name(r))
                };
                if ui.link(text).clicked() {
                    event = Some(WindowEvent::Goto(rid));
                }
            }
        });
    }

    if !p.notes.trim().is_empty() {
        ui.add_space(10.0);
        ui.label(RichText::new("Заметки").strong());
        ui.label(p.notes.trim());
    }
    event
}

fn nationalities_text(p: &Person) -> Option<String> {
    let names: Vec<&str> = p
        .nationalities
        .iter()
        .take(crate::flags::MAX_PER_PERSON)
        .filter_map(|c| crate::flags::name(c))
        .collect();
    if names.is_empty() {
        None
    } else {
        Some(names.join(", "))
    }
}

fn non_empty(s: &str) -> Option<String> {
    let t = s.trim();
    if t.is_empty() {
        None
    } else {
        Some(t.to_string())
    }
}

/// «1985», «03.1985», «14.03.1985»; None — дата неизвестна.
pub fn format_date(d: &PartialDate) -> Option<String> {
    let y = d.year?;
    Some(match (d.month, d.day) {
        (Some(m), Some(day)) => format!("{day:02}.{m:02}.{y}"),
        (Some(m), None) => format!("{m:02}.{y}"),
        _ => y.to_string(),
    })
}

/// «1958–2020», «р. 1985», «…»; для ссылок на родственников.
pub(crate) fn format_years(p: &Person) -> String {
    match (p.birth.year, p.death.year) {
        (Some(b), Some(d)) => format!("{b}–{d}"),
        (Some(b), None) => format!("р. {b}"),
        (None, Some(d)) => format!("ум. {d}"),
        (None, None) => String::new(),
    }
}

/// Возраст (или сколько прожил). Точный, если известны день и месяц, иначе «≈».
pub fn age_text(
    birth: &PartialDate,
    death: &PartialDate,
    today: (i32, u8, u8),
) -> Option<String> {
    let by = birth.year?;
    let (end, dead) = if death.year.is_some() {
        (death.clone(), true)
    } else {
        (
            PartialDate { year: Some(today.0), month: Some(today.1), day: Some(today.2) },
            false,
        )
    };
    let ey = end.year?;
    let mut years = ey - by;
    let exact = birth.month.is_some() && birth.day.is_some() && end.month.is_some() && end.day.is_some();
    if exact {
        let b = (birth.month.unwrap(), birth.day.unwrap());
        let e = (end.month.unwrap(), end.day.unwrap());
        if e < b {
            years -= 1;
        }
    }
    if years < 0 {
        return None;
    }
    let word = years_word(years);
    let prefix = if exact { "" } else { "≈ " };
    Some(if dead {
        format!("{prefix}{years} {word} (прожил(а))")
    } else {
        format!("{prefix}{years} {word}")
    })
}

fn years_word(n: i32) -> &'static str {
    let (a, b) = (n % 10, n % 100);
    if (11..=14).contains(&b) {
        "лет"
    } else if a == 1 {
        "год"
    } else if (2..=4).contains(&a) {
        "года"
    } else {
        "лет"
    }
}

/// Сегодняшняя дата (UTC) без внешних крейтов.
pub(crate) fn today() -> (i32, u8, u8) {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    civil_from_days(secs.div_euclid(86_400))
}

/// Дни от 1970-01-01 → (год, месяц, день).
fn civil_from_days(z: i64) -> (i32, u8, u8) {
    let z = z + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u8;
    let m = (if mp < 10 { mp + 3 } else { mp - 9 }) as u8;
    ((if m <= 2 { y + 1 } else { y }) as i32, m, d)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(y: Option<i32>, m: Option<u8>, day: Option<u8>) -> PartialDate {
        PartialDate { year: y, month: m, day }
    }

    #[test]
    fn date_formats() {
        assert_eq!(format_date(&d(None, None, None)), None);
        assert_eq!(format_date(&d(Some(1985), None, None)).unwrap(), "1985");
        assert_eq!(format_date(&d(Some(1985), Some(3), None)).unwrap(), "03.1985");
        assert_eq!(format_date(&d(Some(1985), Some(3), Some(4))).unwrap(), "04.03.1985");
    }

    #[test]
    fn age_exact_and_approx() {
        let none = d(None, None, None);
        let b = d(Some(1990), Some(6), Some(15));
        assert_eq!(age_text(&b, &none, (2026, 6, 14)).unwrap(), "35 лет");
        assert_eq!(age_text(&b, &none, (2026, 6, 15)).unwrap(), "36 лет");
        let by = d(Some(1990), None, None);
        assert_eq!(age_text(&by, &none, (2026, 6, 14)).unwrap(), "≈ 36 лет");
        let dd = d(Some(2000), Some(1), Some(1));
        assert_eq!(age_text(&b, &dd, (2026, 1, 1)).unwrap(), "9 лет (прожил(а))");
        assert!(age_text(&none, &none, (2026, 1, 1)).is_none());
    }

    #[test]
    fn year_words() {
        assert_eq!(years_word(1), "год");
        assert_eq!(years_word(2), "года");
        assert_eq!(years_word(5), "лет");
        assert_eq!(years_word(11), "лет");
        assert_eq!(years_word(21), "год");
        assert_eq!(years_word(112), "лет");
    }

    #[test]
    fn civil_date_conversion() {
        assert_eq!(civil_from_days(0), (1970, 1, 1));
        assert_eq!(civil_from_days(19_723), (2024, 1, 1));
        assert_eq!(civil_from_days(20_527), (2026, 3, 15));
    }
}
