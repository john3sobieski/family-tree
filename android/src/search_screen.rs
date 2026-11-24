// Full-screen search page for the phone: a field (text comes from the hidden Java EditText) and tappable suggestions.
// Matching itself is the desktop `search::find`, so results are the same as on PC.
use crate::i18n::tr;
use crate::model::{PersonId, Tree};
use crate::person_window::{format_years, full_name};
use eframe::egui::{self, Align, Align2, FontId, Layout, RichText, Sense, Vec2};

/// Temporary top margin (status bar); proper insets come in step A7.
const TOP_INSET: f32 = 42.0;
const BAR_H: f32 = 44.0;
const ROW_H: f32 = 56.0;

pub enum Event {
    None,
    /// "Back" button: close the search.
    Back,
    /// "×" button: erase the text.
    Clear,
    /// The field was tapped: bring the keyboard back.
    Focus,
    /// A suggestion was tapped.
    Pick(PersonId),
}

pub fn show(ctx: &egui::Context, tree: &Tree, text: &str, hits: &[PersonId]) -> Event {
    let mut event = Event::None;
    egui::TopBottomPanel::top("search-bar").frame(crate::theme::bar_frame(10.0, 6.0)).show(ctx, |ui| {
        ui.add_space(TOP_INSET);
        ui.horizontal(|ui| {
            if ui.add(egui::Button::new(tr("Назад")).min_size(Vec2::new(0.0, BAR_H))).clicked() {
                event = Event::Back;
            }
            let gap = ui.spacing().item_spacing.x;
            let clear_w = if text.is_empty() { 0.0 } else { BAR_H + gap };
            let w = (ui.available_width() - clear_w).max(60.0);
            let (rect, resp) = ui.allocate_exact_size(Vec2::new(w, BAR_H), Sense::click());
            let v = ui.visuals();
            ui.painter().rect(rect, 6.0, v.extreme_bg_color, v.widgets.inactive.bg_stroke);
            let (shown, color) = if text.is_empty() {
                (tr("🔍 Найти человека…").to_string(), v.weak_text_color())
            } else {
                (text.to_string(), v.text_color())
            };
            ui.painter_at(rect.shrink(2.0)).text(
                rect.left_center() + Vec2::new(10.0, 0.0),
                Align2::LEFT_CENTER,
                shown,
                FontId::proportional(16.0),
                color,
            );
            if resp.clicked() {
                event = Event::Focus;
            }
            if !text.is_empty() && ui.add(egui::Button::new("×").min_size(Vec2::splat(BAR_H))).clicked() {
                event = Event::Clear;
            }
        });
    });
    egui::CentralPanel::default().show(ctx, |ui| {
        if text.trim().is_empty() {
            return;
        }
        egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
            if hits.is_empty() {
                ui.label(RichText::new(tr("Никого не найдено")).weak());
                return;
            }
            ui.with_layout(Layout::top_down_justified(Align::Min), |ui| {
                ui.spacing_mut().interact_size.y = ROW_H;
                for id in hits {
                    let Some(p) = tree.persons.get(id) else { continue };
                    let years = format_years(p);
                    let label = if years.is_empty() { full_name(p) } else { format!("{}  ({years})", full_name(p)) };
                    if ui.selectable_label(false, RichText::new(label).size(16.0)).clicked() {
                        event = Event::Pick(*id);
                    }
                    ui.separator();
                }
            });
        });
    });
    event
}
