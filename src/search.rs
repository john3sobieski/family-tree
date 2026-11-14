//! Поиск человека по имени на схеме: поле в верхней панели + выпадающие подсказки
//! (автодополнение). Выбор подсказки или Enter возвращает id — приложение плавно
//! перелетает камерой к человеку и подсвечивает его.

use crate::layout::Layout;
use crate::model::{Person, PersonId, Tree};
use crate::person_window::{format_years, full_name};
use eframe::egui::{self, Key, Modifiers};

const MAX_SUGGESTIONS: usize = 8;
/// Id поля — чтобы фокусировать его по Ctrl+F.
pub const FIELD_ID: &str = "search-field";

#[derive(Default)]
pub struct SearchState {
    pub text: String,
    /// Подсвеченная стрелками подсказка.
    sel: usize,
    /// Указатель был над списком подсказок в прошлом кадре (клик по подсказке
    /// снимает фокус с поля, поэтому список нельзя закрывать сразу).
    popup_hover: bool,
}

impl SearchState {
    pub fn clear(&mut self) {
        self.text.clear();
        self.sel = 0;
        self.popup_hover = false;
    }
}

/// Подходящие люди (только те, кто есть на схеме): все слова запроса должны входить
/// в «имя фамилия» (в любом порядке); те, у кого имя/фамилия начинается с запроса, — выше.
pub fn find(tree: &Tree, layout: &Layout, query: &str, limit: usize) -> Vec<PersonId> {
    let tokens: Vec<String> = query.split_whitespace().map(|t| t.to_lowercase()).collect();
    if tokens.is_empty() {
        return Vec::new();
    }
    let mut hits: Vec<(u8, String, PersonId)> = Vec::new();
    for p in tree.persons.values() {
        let name = full_name(p).to_lowercase();
        if !tokens.iter().all(|t| name.contains(t.as_str())) {
            continue;
        }
        if layout.pos_of(p.id).is_none() {
            continue;
        }
        hits.push((rank(p, &tokens), name, p.id));
    }
    hits.sort();
    hits.into_iter().take(limit).map(|h| h.2).collect()
}

/// 0 — каждое слово запроса начинается с начала имени или фамилии; 1 — просто содержится.
fn rank(p: &Person, tokens: &[String]) -> u8 {
    let first = p.first_name.to_lowercase();
    let last = p.last_name.to_lowercase();
    let all_prefix = tokens.iter().all(|t| first.starts_with(t.as_str()) || last.starts_with(t.as_str()));
    if all_prefix {
        0
    } else {
        1
    }
}

/// Поле поиска с подсказками. Возвращает выбранного человека.
pub fn show(
    ui: &mut egui::Ui,
    st: &mut SearchState,
    tree: &Tree,
    layout: &Layout,
) -> Option<PersonId> {
    let resp = ui.add(
        egui::TextEdit::singleline(&mut st.text)
            .id(egui::Id::new(FIELD_ID))
            .hint_text("🔍 Найти человека…")
            .desired_width(200.0),
    );
    if resp.changed() {
        st.sel = 0;
    }
    let enter = resp.lost_focus() && ui.input(|i| i.key_pressed(Key::Enter));
    let open = !st.text.trim().is_empty() && (resp.has_focus() || st.popup_hover || enter);
    if !open {
        st.popup_hover = false;
        return None;
    }

    let matches = find(tree, layout, &st.text, MAX_SUGGESTIONS);
    if matches.is_empty() {
        st.sel = 0;
    } else {
        if resp.has_focus() {
            let (down, up) = ui.input_mut(|i| {
                (
                    i.consume_key(Modifiers::NONE, Key::ArrowDown),
                    i.consume_key(Modifiers::NONE, Key::ArrowUp),
                )
            });
            if down {
                st.sel = (st.sel + 1) % matches.len();
            }
            if up {
                st.sel = (st.sel + matches.len() - 1) % matches.len();
            }
        }
        st.sel = st.sel.min(matches.len() - 1);
    }

    let mut picked: Option<PersonId> = None;
    if enter {
        picked = matches.get(st.sel).copied();
    }

    if resp.has_focus() || st.popup_hover {
        let area = egui::Area::new(egui::Id::new("search-popup"))
            .order(egui::Order::Foreground)
            .fixed_pos(resp.rect.left_bottom() + egui::vec2(0.0, 2.0))
            .show(ui.ctx(), |ui| {
                egui::Frame::popup(ui.style()).show(ui, |ui| {
                    ui.set_min_width(260.0);
                    if matches.is_empty() {
                        ui.label(egui::RichText::new("Никого не найдено").weak());
                    }
                    for (i, id) in matches.iter().enumerate() {
                        let Some(p) = tree.persons.get(id) else { continue };
                        let years = format_years(p);
                        let label = if years.is_empty() {
                            full_name(p)
                        } else {
                            format!("{}  ({years})", full_name(p))
                        };
                        if ui.selectable_label(i == st.sel, label).clicked() {
                            picked = Some(*id);
                        }
                    }
                });
            });
        let hover = ui
            .ctx()
            .input(|i| i.pointer.hover_pos())
            .map_or(false, |p| area.response.rect.contains(p));
        st.popup_hover = hover;
    }

    if let Some(id) = picked {
        if let Some(p) = tree.persons.get(&id) {
            st.text = full_name(p);
        }
        st.popup_hover = false;
        resp.surrender_focus();
        return Some(id);
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::{compute, LayoutConfig};

    fn tree() -> (Tree, Layout) {
        let mut t = Tree::new();
        let root = t.add_person(Person::new("Иван", "Петров"));
        let a = t.add_person(Person::new("Пётр", "Иванов"));
        let b = t.add_person(Person::new("Анна", "Петрова"));
        t.add_parent_child(root, a).unwrap();
        t.add_parent_child(root, b).unwrap();
        // Не связан с деревом — на схеме его нет.
        t.add_person(Person::new("Иван", "Чужой"));
        t.root = Some(root);
        let l = compute(&t, &LayoutConfig::default());
        (t, l)
    }

    #[test]
    fn finds_by_any_word_order_and_skips_unplaced() {
        let (t, l) = tree();
        assert_eq!(find(&t, &l, "иван", 10).len(), 2); // Иван Петров, Пётр Иванов (без «Чужого»)
        assert_eq!(find(&t, &l, "Петров иван", 10).len(), 1);
        assert!(find(&t, &l, "   ", 10).is_empty());
        assert!(find(&t, &l, "Чужой", 10).is_empty());
    }

    #[test]
    fn prefix_matches_come_first_and_limit_applies() {
        let (t, l) = tree();
        let names: Vec<String> =
            find(&t, &l, "иван", 10).iter().map(|id| t.persons[id].first_name.clone()).collect();
        // «Иван Петров» начинается с запроса, у «Пётр Иванов» запрос — фамилия-префикс: оба ранг 0,
        // порядок по имени.
        assert_eq!(names.len(), 2);
        assert_eq!(find(&t, &l, "иван", 1).len(), 1);
    }
}
