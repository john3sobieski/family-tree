//! Своя шапка окна (без рамки Windows): перетаскивание, свернуть/развернуть/закрыть.

use crate::theme;
use eframe::egui::{self, pos2, vec2, Align2, Color32, FontId, Id, Rect, Sense, Stroke, ViewportCommand};

const BAR_H: f32 = 36.0;
const BTN_W: f32 = 46.0;

pub fn title_bar(ctx: &egui::Context, file: &str, dirty: bool) {
    egui::TopBottomPanel::top("titlebar")
        .exact_height(BAR_H)
        .frame(egui::Frame::none().fill(theme::BG))
        .show(ctx, |ui| {
            let rect = ui.max_rect();
            let maximized = ctx.input(|i| i.viewport().maximized.unwrap_or(false));

            // Область перетаскивания — вся шапка; кнопки ниже перекрывают её.
            let drag = ui.interact(rect, Id::new("titlebar-drag"), Sense::click_and_drag());
            if drag.double_clicked() {
                ctx.send_viewport_cmd(ViewportCommand::Maximized(!maximized));
            } else if drag.drag_started_by(egui::PointerButton::Primary) {
                ctx.send_viewport_cmd(ViewportCommand::StartDrag);
            }

            let p = ui.painter();
            // Логотип: белый квадрат с «деревом».
            let logo = Rect::from_min_size(pos2(rect.left() + 14.0, rect.center().y - 9.0), vec2(18.0, 18.0));
            p.rect_filled(logo, 5.0, theme::ACCENT);
            let c = logo.center();
            let k = Stroke::new(1.5_f32, theme::BG);
            p.line_segment([pos2(c.x, c.y - 3.0), pos2(c.x, c.y + 1.0)], k);
            p.line_segment([pos2(c.x - 4.0, c.y + 1.0), pos2(c.x + 4.0, c.y + 1.0)], k);
            for dx in [-4.0_f32, 0.0, 4.0] {
                p.circle_filled(pos2(c.x + dx, c.y + 4.0), 1.6, theme::BG);
            }
            p.circle_filled(pos2(c.x, c.y - 4.5), 2.0, theme::BG);

            let name_pos = pos2(logo.right() + 10.0, rect.center().y);
            let r1 = p.text(name_pos, Align2::LEFT_CENTER, "Семейное дерево", FontId::proportional(13.0), theme::TEXT);
            let label = format!("{}{}", file, if dirty { "  ●" } else { "" });
            p.text(
                pos2(r1.right() + 12.0, rect.center().y),
                Align2::LEFT_CENTER,
                label,
                FontId::proportional(13.0),
                theme::MUTED,
            );

            // Кнопки окна справа: [—][□][×].
            let icon = Stroke::new(1.2_f32, theme::TEXT);
            for i in 0..3 {
                let r = Rect::from_min_size(
                    pos2(rect.right() - BTN_W * (3 - i) as f32, rect.top()),
                    vec2(BTN_W, BAR_H),
                );
                let resp = ui.interact(r, Id::new(("winbtn", i)), Sense::click());
                let is_close = i == 2;
                if resp.hovered() {
                    let bg = if is_close { Color32::from_rgb(196, 43, 28) } else { Color32::from_rgb(34, 34, 34) };
                    ui.painter().rect_filled(r, 0.0, bg);
                }
                let c = r.center();
                let p = ui.painter();
                match i {
                    0 => {
                        p.line_segment([pos2(c.x - 5.0, c.y), pos2(c.x + 5.0, c.y)], icon);
                    }
                    1 if maximized => {
                        p.rect_stroke(Rect::from_min_size(pos2(c.x - 5.0, c.y - 2.0), vec2(7.0, 7.0)), 0.0_f32, icon);
                        p.line_segment([pos2(c.x - 2.0, c.y - 5.0), pos2(c.x + 5.0, c.y - 5.0)], icon);
                        p.line_segment([pos2(c.x + 5.0, c.y - 5.0), pos2(c.x + 5.0, c.y + 2.0)], icon);
                    }
                    1 => {
                        p.rect_stroke(Rect::from_center_size(c, vec2(10.0, 10.0)), 0.0_f32, icon);
                    }
                    _ => {
                        p.line_segment([pos2(c.x - 5.0, c.y - 5.0), pos2(c.x + 5.0, c.y + 5.0)], icon);
                        p.line_segment([pos2(c.x - 5.0, c.y + 5.0), pos2(c.x + 5.0, c.y - 5.0)], icon);
                    }
                }
                if resp.clicked() {
                    match i {
                        0 => ctx.send_viewport_cmd(ViewportCommand::Minimized(true)),
                        1 => ctx.send_viewport_cmd(ViewportCommand::Maximized(!maximized)),
                        // Закрытие идёт через close_requested → подтверждение несохранённого.
                        _ => ctx.send_viewport_cmd(ViewportCommand::Close),
                    }
                }
            }
        });
}

/// Тонкая рамка по периметру окна (у окна без системной рамки).
pub fn window_border(ctx: &egui::Context) {
    if ctx.input(|i| i.viewport().maximized.unwrap_or(false)) {
        return;
    }
    let painter = ctx.layer_painter(egui::LayerId::new(egui::Order::Foreground, Id::new("window-border")));
    painter.rect_stroke(ctx.screen_rect(), 0.0_f32, Stroke::new(1.0_f32, theme::BORDER_HI));
}
