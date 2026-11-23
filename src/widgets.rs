//! Small shared widgets: painted icons (no font glyphs, so they never turn into squares),
//! icon buttons and the framed section used by the edit panel.

use crate::theme;
use eframe::egui::{self, Align2, Color32, Margin, Pos2, Rect, Response, RichText, Rounding, Sense, Stroke, Ui};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Icon {
    Plus,
    Cross,
    Check,
}

/// Paints an icon centered at `c`; `r` is its half-size.
pub fn paint_icon(painter: &egui::Painter, c: Pos2, icon: Icon, r: f32, color: Color32) {
    let st = Stroke::new(1.7_f32, color);
    match icon {
        Icon::Plus => {
            painter.line_segment([Pos2::new(c.x - r, c.y), Pos2::new(c.x + r, c.y)], st);
            painter.line_segment([Pos2::new(c.x, c.y - r), Pos2::new(c.x, c.y + r)], st);
        }
        Icon::Cross => {
            let d = r * 0.8;
            painter.line_segment([Pos2::new(c.x - d, c.y - d), Pos2::new(c.x + d, c.y + d)], st);
            painter.line_segment([Pos2::new(c.x - d, c.y + d), Pos2::new(c.x + d, c.y - d)], st);
        }
        Icon::Check => {
            painter.line_segment([Pos2::new(c.x - r, c.y + r * 0.1), Pos2::new(c.x - r * 0.3, c.y + r * 0.7)], st);
            painter.line_segment([Pos2::new(c.x - r * 0.3, c.y + r * 0.7), Pos2::new(c.x + r, c.y - r * 0.6)], st);
        }
    }
}

/// A non-clickable icon in the text flow (e.g. a check mark before a sentence).
pub fn inline_icon(ui: &mut Ui, icon: Icon, color: Color32) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(16.0, 16.0), Sense::hover());
    paint_icon(ui.painter(), rect.center(), icon, 5.5, color);
}

/// Small square button with a painted icon. `danger`: the icon turns red on hover.
pub fn icon_button(ui: &mut Ui, icon: Icon, tip: &str, danger: bool) -> Response {
    let (rect, resp) = ui.allocate_exact_size(egui::vec2(24.0, 24.0), Sense::click());
    if ui.is_rect_visible(rect) {
        let v = ui.style().interact(&resp);
        let color = if danger && resp.hovered() { theme::p().danger } else { v.fg_stroke.color };
        ui.painter().rect(rect, v.rounding, v.weak_bg_fill, v.bg_stroke);
        paint_icon(ui.painter(), rect.center(), icon, 5.0, color);
    }
    resp.on_hover_text(tip).on_hover_cursor(egui::CursorIcon::PointingHand)
}

/// Button with a painted icon and a text. `width`: fixed width (content is centered), or as wide as needed.
pub fn icon_text_button(ui: &mut Ui, icon: Icon, text: &str, width: Option<f32>) -> Response {
    const ICON_W: f32 = 12.0;
    const GAP: f32 = 6.0;
    let font = egui::TextStyle::Button.resolve(ui.style());
    let galley = ui.painter().layout_no_wrap(text.to_string(), font, Color32::PLACEHOLDER);
    let pad = ui.spacing().button_padding;
    let content_w = ICON_W + GAP + galley.size().x;
    let size = egui::vec2(
        width.unwrap_or(content_w + pad.x * 2.0),
        (galley.size().y + pad.y * 2.0).max(ui.spacing().interact_size.y),
    );
    let (rect, resp) = ui.allocate_exact_size(size, Sense::click());
    if ui.is_rect_visible(rect) {
        let v = ui.style().interact(&resp);
        ui.painter().rect(rect, v.rounding, v.weak_bg_fill, v.bg_stroke);
        let x0 = rect.center().x - content_w / 2.0;
        paint_icon(ui.painter(), Pos2::new(x0 + ICON_W / 2.0, rect.center().y), icon, 5.0, v.fg_stroke.color);
        let pos = Pos2::new(x0 + ICON_W + GAP, rect.center().y - galley.size().y / 2.0);
        ui.painter().galley(pos, galley, v.fg_stroke.color);
    }
    resp.on_hover_cursor(egui::CursorIcon::PointingHand)
}

/// A framed block (card) across the full width, with an optional title.
pub fn section(ui: &mut Ui, title: Option<&str>, add_contents: impl FnOnce(&mut Ui)) {
    let p = theme::p();
    egui::Frame::none()
        .fill(p.surface)
        .stroke(Stroke::new(1.0_f32, p.border))
        .rounding(Rounding::same(8.0))
        .inner_margin(Margin::same(12.0))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            if let Some(t) = title {
                ui.label(RichText::new(t).strong());
                ui.add_space(6.0);
            }
            add_contents(ui);
        });
    ui.add_space(8.0);
}

/// Centered text inside a rectangle (placeholders).
pub fn centered_text(ui: &Ui, rect: Rect, text: &str) {
    ui.painter().text(
        rect.center(),
        Align2::CENTER_CENTER,
        text,
        egui::FontId::proportional(14.0),
        theme::p().muted,
    );
}
