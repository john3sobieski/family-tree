//! Режим «Список»: таблица всех людей с поиском, фильтром по полу и сортировкой
//! по клику на заголовок. Строки рисуются виртуально (только видимые) — быстро на тысячах людей.

use crate::container::{MediaKind, MediaStore};
use crate::model::{Gender, Person, PersonId, Tree};
use crate::person_window::{age_text, format_date, full_name, load_texture, today};
use eframe::egui::{self, Align2, Color32, FontId, Pos2, Rect, Sense, TextureHandle, Vec2};
use std::cmp::Ordering;
use std::collections::HashMap;

const ROW_H: f32 = 46.0;
const HEADER_H: f32 = 26.0;
const PHOTO_COL_W: f32 = 44.0;
const MAX_THUMBS: usize = 300;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Col {
    First,
    Last,
    Gender,
    Birth,
    Death,
    Age,
    Profession,
}

/// (колонка, заголовок, ширина; 0 — занимает остаток).
const COLUMNS: [(Col, &str, f32); 7] = [
    (Col::First, "Имя", 130.0),
    (Col::Last, "Фамилия", 150.0),
    (Col::Gender, "Пол", 90.0),
    (Col::Birth, "Рождение", 100.0),
    (Col::Death, "Смерть", 100.0),
    (Col::Age, "Возраст", 150.0),
    (Col::Profession, "Специальность", 0.0),
];

#[derive(Clone, Copy, PartialEq, Eq)]
enum GenderFilter {
    All,
    Male,
    Female,
    Unknown,
}

impl GenderFilter {
    fn label(self) -> &'static str {
        match self {
            GenderFilter::All => "Все",
            GenderFilter::Male => "Мужчины",
            GenderFilter::Female => "Женщины",
            GenderFilter::Unknown => "Пол не указан",
        }
    }

    fn matches(self, g: &Gender) -> bool {
        match self {
            GenderFilter::All => true,
            GenderFilter::Male => *g == Gender::Male,
            GenderFilter::Female => *g == Gender::Female,
            GenderFilter::Unknown => *g == Gender::Unknown,
        }
    }
}

pub struct ListState {
    pub search: String,
    gender: GenderFilter,
    sort: Col,
    asc: bool,
    order: Vec<PersonId>,
    /// Ключ, для которого посчитан `order`.
    key: Option<(String, u8, Col, bool, usize)>,
    stale: bool,
    thumbs: HashMap<String, Option<TextureHandle>>,
}

impl Default for ListState {
    fn default() -> Self {
        ListState {
            search: String::new(),
            gender: GenderFilter::All,
            sort: Col::Last,
            asc: true,
            order: Vec::new(),
            key: None,
            stale: true,
            thumbs: HashMap::new(),
        }
    }
}

fn gender_code(g: GenderFilter) -> u8 {
    match g {
        GenderFilter::All => 0,
        GenderFilter::Male => 1,
        GenderFilter::Female => 2,
        GenderFilter::Unknown => 3,
    }
}

/// Сравнение необязательных значений: неизвестные всегда в конце, при любом порядке.
fn cmp_opt<T: Ord>(a: &Option<T>, b: &Option<T>, asc: bool) -> Ordering {
    match (a, b) {
        (None, None) => Ordering::Equal,
        (None, Some(_)) => Ordering::Greater,
        (Some(_), None) => Ordering::Less,
        (Some(x), Some(y)) => {
            if asc {
                x.cmp(y)
            } else {
                y.cmp(x)
            }
        }
    }
}

fn date_key(d: &crate::model::PartialDate) -> Option<(i32, u8, u8)> {
    d.year.map(|y| (y, d.month.unwrap_or(0), d.day.unwrap_or(0)))
}

/// Возраст в годах для сортировки (по годам).
fn age_years(p: &Person, now_year: i32) -> Option<i32> {
    let by = p.birth.year?;
    let end = p.death.year.unwrap_or(now_year);
    Some(end - by)
}

fn gender_rank(g: &Gender) -> u8 {
    match g {
        Gender::Male => 0,
        Gender::Female => 1,
        Gender::Unknown => 2,
    }
}

fn gender_text(g: &Gender) -> &'static str {
    match g {
        Gender::Male => "Мужской",
        Gender::Female => "Женский",
        Gender::Unknown => "",
    }
}

impl ListState {
    /// Пересчитать список при следующем показе (дерево могло измениться).
    pub fn invalidate(&mut self) {
        self.stale = true;
    }

    pub fn reset(&mut self) {
        self.thumbs.clear();
        self.order.clear();
        self.key = None;
        self.stale = true;
    }

    fn rebuild(&mut self, tree: &Tree) {
        let q = self.search.trim().to_lowercase();
        let key = (q.clone(), gender_code(self.gender), self.sort, self.asc, tree.persons.len());
        if !self.stale && self.key.as_ref() == Some(&key) {
            return;
        }
        let gender = self.gender;
        let mut people: Vec<&Person> = tree
            .persons
            .values()
            .filter(|p| gender.matches(&p.gender))
            .filter(|p| {
                q.is_empty()
                    || full_name(p).to_lowercase().contains(&q)
                    || p.profession.to_lowercase().contains(&q)
                    || p.notes.to_lowercase().contains(&q)
            })
            .collect();
        let (asc, now_year) = (self.asc, today().0);
        match self.sort {
            Col::First => people.sort_by_cached_key(|p| p.first_name.to_lowercase()),
            Col::Last => people.sort_by_cached_key(|p| p.last_name.to_lowercase()),
            Col::Gender => people.sort_by_key(|p| gender_rank(&p.gender)),
            Col::Birth => people.sort_by(|a, b| cmp_opt(&date_key(&a.birth), &date_key(&b.birth), asc)),
            Col::Death => people.sort_by(|a, b| cmp_opt(&date_key(&a.death), &date_key(&b.death), asc)),
            Col::Age => people.sort_by(|a, b| {
                cmp_opt(&age_years(a, now_year), &age_years(b, now_year), asc)
            }),
            Col::Profession => people.sort_by_cached_key(|p| p.profession.to_lowercase()),
        }
        // Для строковых колонок порядок «убывание» — разворот (стабильная сортировка сохранена
        // за счёт предварительной сортировки по id в BTreeMap).
        if !asc && matches!(self.sort, Col::First | Col::Last | Col::Gender | Col::Profession) {
            people.reverse();
        }
        self.order = people.into_iter().map(|p| p.id).collect();
        self.key = Some(key);
        self.stale = false;
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

/// Рисует режим «Список». Возвращает человека, по строке которого кликнули.
pub fn show(
    ui: &mut egui::Ui,
    st: &mut ListState,
    tree: &Tree,
    store: &MediaStore,
    selected: Option<PersonId>,
) -> Option<PersonId> {
    let mut clicked = None;

    ui.add_space(4.0);
    ui.horizontal(|ui| {
        ui.add_space(6.0);
        ui.label("Поиск:");
        ui.add(
            egui::TextEdit::singleline(&mut st.search)
                .hint_text("имя, специальность, заметки…")
                .desired_width(280.0),
        );
        ui.label("Пол:");
        egui::ComboBox::from_id_salt("list-gender").selected_text(st.gender.label()).show_ui(ui, |ui| {
            for g in [GenderFilter::All, GenderFilter::Male, GenderFilter::Female, GenderFilter::Unknown] {
                ui.selectable_value(&mut st.gender, g, g.label());
            }
        });
        if !st.search.is_empty() && ui.button("✕").on_hover_text("Очистить поиск").clicked() {
            st.search.clear();
        }
    });
    st.rebuild(tree);
    ui.horizontal(|ui| {
        ui.add_space(6.0);
        ui.weak(format!("Показано: {} из {}", st.order.len(), tree.persons.len()));
    });
    ui.add_space(2.0);

    // Ширины колонок: последняя занимает остаток.
    let fixed: f32 = PHOTO_COL_W + COLUMNS.iter().map(|c| c.2).sum::<f32>();
    let rest = (ui.available_width() - fixed - 16.0).max(180.0);
    let widths: Vec<f32> = COLUMNS.iter().map(|c| if c.2 > 0.0 { c.2 } else { rest }).collect();
    let total_w = PHOTO_COL_W + widths.iter().sum::<f32>();

    // Заголовок.
    let (hrect, _) = ui.allocate_exact_size(Vec2::new(total_w, HEADER_H), Sense::hover());
    ui.painter().rect_filled(hrect, 0.0, ui.visuals().widgets.noninteractive.bg_fill);
    let mut x = hrect.min.x + PHOTO_COL_W;
    for (i, (col, title, _)) in COLUMNS.iter().enumerate() {
        let cell = Rect::from_min_size(Pos2::new(x, hrect.min.y), Vec2::new(widths[i], HEADER_H));
        let id = ui.id().with(("list-header", i));
        let resp = ui.interact(cell, id, Sense::click());
        if resp.hovered() {
            ui.painter().rect_filled(cell, 0.0, ui.visuals().widgets.hovered.bg_fill);
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        }
        let arrow = if st.sort == *col {
            if st.asc { " ▲" } else { " ▼" }
        } else {
            ""
        };
        ui.painter().with_clip_rect(cell).text(
            Pos2::new(x + 6.0, cell.center().y),
            Align2::LEFT_CENTER,
            format!("{title}{arrow}"),
            FontId::proportional(14.0),
            ui.visuals().strong_text_color(),
        );
        if resp.clicked() {
            if st.sort == *col {
                st.asc = !st.asc;
            } else {
                st.sort = *col;
                st.asc = true;
            }
        }
        x += widths[i];
    }

    // Строки (только видимые).
    let order = st.order.clone();
    let now = today();
    let text_color = ui.visuals().text_color();
    let weak_color = ui.visuals().weak_text_color();
    egui::ScrollArea::vertical().auto_shrink([false, false]).show_rows(
        ui,
        ROW_H,
        order.len(),
        |ui, range| {
            for i in range {
                let id = order[i];
                let Some(p) = tree.persons.get(&id) else { continue };
                let (rect, resp) = ui.allocate_exact_size(Vec2::new(total_w, ROW_H), Sense::click());
                let painter = ui.painter().clone();
                if selected == Some(id) {
                    painter.rect_filled(rect, 0.0, crate::theme::SELECT_BG);
                } else if resp.hovered() {
                    painter.rect_filled(rect, 0.0, ui.visuals().widgets.hovered.bg_fill);
                } else if i % 2 == 1 {
                    painter.rect_filled(rect, 0.0, ui.visuals().faint_bg_color);
                }
                if resp.hovered() {
                    ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                }

                // Фото.
                let photo = Rect::from_min_size(rect.min + Vec2::new(5.0, 3.0), Vec2::new(32.0, 40.0));
                let tex = p.main_photo.as_deref().and_then(|h| st.thumb(ui.ctx(), store, h));
                match tex {
                    Some(tid) => {
                        painter.image(
                            tid,
                            photo,
                            Rect::from_min_max(Pos2::new(0.0, 0.0), Pos2::new(1.0, 1.0)),
                            Color32::WHITE,
                        );
                    }
                    None => crate::view::draw_placeholder(
                        &painter,
                        photo,
                        ui.visuals().widgets.noninteractive.bg_stroke.color,
                        ui.visuals().faint_bg_color,
                    ),
                }

                // Ячейки.
                let age = age_text(&p.birth, &p.death, now).unwrap_or_default();
                let cells: [String; 7] = [
                    p.first_name.clone(),
                    p.last_name.clone(),
                    gender_text(&p.gender).to_string(),
                    format_date(&p.birth).unwrap_or_default(),
                    format_date(&p.death).unwrap_or_default(),
                    age,
                    p.profession.clone(),
                ];
                let mut x = rect.min.x + PHOTO_COL_W;
                for (ci, text) in cells.iter().enumerate() {
                    let cell = Rect::from_min_size(Pos2::new(x, rect.min.y), Vec2::new(widths[ci], ROW_H));
                    let (txt, color) = if text.is_empty() { ("—", weak_color) } else { (text.as_str(), text_color) };
                    painter.with_clip_rect(cell).text(
                        Pos2::new(x + 6.0, rect.center().y),
                        Align2::LEFT_CENTER,
                        txt,
                        FontId::proportional(14.0),
                        color,
                    );
                    x += widths[ci];
                }

                if resp.clicked() {
                    clicked = Some(id);
                }
            }
        },
    );
    clicked
}

#[cfg(test)]
mod tests {
    use super::*;

    fn person(first: &str, last: &str, birth: Option<i32>, g: Gender) -> Person {
        let mut p = Person::new(first, last);
        p.birth.year = birth;
        p.gender = g;
        p
    }

    fn sample() -> Tree {
        let mut t = Tree::new();
        t.add_person(person("Борис", "Яковлев", Some(1950), Gender::Male));
        t.add_person(person("Анна", "Белова", None, Gender::Female));
        t.add_person(person("Виктор", "Алексеев", Some(1990), Gender::Male));
        t
    }

    fn names(st: &ListState, t: &Tree) -> Vec<String> {
        st.order.iter().map(|id| t.persons[id].first_name.clone()).collect()
    }

    #[test]
    fn sort_by_last_name_both_directions() {
        let t = sample();
        let mut st = ListState::default();
        st.rebuild(&t);
        assert_eq!(names(&st, &t), ["Виктор", "Анна", "Борис"].map(String::from));
        st.asc = false;
        st.rebuild(&t);
        assert_eq!(names(&st, &t), ["Борис", "Анна", "Виктор"].map(String::from));
    }

    #[test]
    fn unknown_birth_is_always_last() {
        let t = sample();
        let mut st = ListState::default();
        st.sort = Col::Birth;
        st.rebuild(&t);
        assert_eq!(names(&st, &t), ["Борис", "Виктор", "Анна"].map(String::from));
        st.asc = false;
        st.rebuild(&t);
        assert_eq!(names(&st, &t), ["Виктор", "Борис", "Анна"].map(String::from));
    }

    #[test]
    fn search_and_gender_filter() {
        let t = sample();
        let mut st = ListState::default();
        st.search = "  АННА ".to_string();
        st.rebuild(&t);
        assert_eq!(names(&st, &t), ["Анна".to_string()]);
        st.search.clear();
        st.gender = GenderFilter::Male;
        st.rebuild(&t);
        assert_eq!(st.order.len(), 2);
    }

    #[test]
    fn invalidate_picks_up_new_people() {
        let mut t = sample();
        let mut st = ListState::default();
        st.rebuild(&t);
        assert_eq!(st.order.len(), 3);
        t.add_person(person("Игорь", "Орлов", None, Gender::Male));
        st.rebuild(&t);
        assert_eq!(st.order.len(), 4);
    }
}
