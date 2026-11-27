//! Окно выбора области главного фото: весь снимок, поверх него рамка 3:4 (как у карточки).
//! Рамку можно двигать мышью, приближать колесом или ползунком. В карточку попадает только то, что в рамке.

use crate::i18n::tr;
use crate::media::{self, CropRect, CARD_ASPECT, MAX_ZOOM};
use crate::theme;
use eframe::egui::{self, Align, Color32, Layout, Pos2, Rect, RichText, Sense, Stroke, TextureHandle, Vec2};
use image::imageops::FilterType;
use image::DynamicImage;

const CANVAS_W: f32 = 560.0;
const CANVAS_H: f32 = 400.0;
const PREVIEW_H: f32 = 160.0;
/// Сторона экранной копии снимка: для рамки хватает, а гигантская текстура не нужна.
const PREVIEW_MAX_SIDE: u32 = 1400;
/// Скорость приближения колесом мыши.
const WHEEL_SPEED: f32 = 0.0015;

pub enum CropOutcome {
    /// Окно открыто, ждём пользователя.
    Open,
    Cancel,
    Done(CropRect),
}

pub struct CropDialog {
    img: DynamicImage,
    tex: TextureHandle,
    zoom: f32,
    /// Центр рамки в пикселях снимка.
    cx: f32,
    cy: f32,
}

impl CropDialog {
    pub fn new(ctx: &egui::Context, img: DynamicImage) -> Self {
        let rgba = if img.width().max(img.height()) > PREVIEW_MAX_SIDE {
            img.resize(PREVIEW_MAX_SIDE, PREVIEW_MAX_SIDE, FilterType::Triangle).to_rgba8()
        } else {
            img.to_rgba8()
        };
        let size = [rgba.width() as usize, rgba.height() as usize];
        let color = egui::ColorImage::from_rgba_unmultiplied(size, rgba.as_raw());
        let tex = ctx.load_texture("crop-source", color, egui::TextureOptions::LINEAR);
        let (cx, cy) = (img.width() as f32 / 2.0, img.height() as f32 / 2.0);
        CropDialog { img, tex, zoom: 1.0, cx, cy }
    }

    /// Исходный снимок (полного размера) — из него вырезается результат.
    pub fn image(&self) -> &DynamicImage {
        &self.img
    }

    fn reset(&mut self) {
        self.zoom = 1.0;
        self.cx = self.img.width() as f32 / 2.0;
        self.cy = self.img.height() as f32 / 2.0;
    }

    fn rect(&self) -> CropRect {
        media::crop_for(self.img.width() as f32, self.img.height() as f32, self.zoom, self.cx, self.cy)
    }

    pub fn show(&mut self, ctx: &egui::Context) -> CropOutcome {
        let mut out = CropOutcome::Open;
        egui::Window::new(tr("Область фото на карточке"))
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.label(RichText::new(tr("Перетаскивайте рамку; масштаб — колесо мыши или ползунок")).color(theme::p().muted));
                ui.add_space(6.0);
                ui.horizontal_top(|ui| {
                    self.canvas(ui);
                    ui.vertical(|ui| {
                        ui.label(RichText::new(tr("Карточка")).color(theme::p().muted));
                        self.preview(ui);
                    });
                });
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    ui.label(tr("Масштаб"));
                    ui.spacing_mut().slider_width = 260.0;
                    ui.add(egui::Slider::new(&mut self.zoom, 1.0..=MAX_ZOOM).logarithmic(true).show_value(false));
                    ui.label(format!("×{:.1}", self.zoom));
                    if ui.button(tr("Сбросить")).clicked() {
                        self.reset();
                    }
                });
                ui.add_space(6.0);
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if theme::primary_button(ui, tr("Готово")).clicked() {
                        out = CropOutcome::Done(self.rect());
                    }
                    if ui.button(tr("Отмена")).clicked() {
                        out = CropOutcome::Cancel;
                    }
                });
            });
        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            out = CropOutcome::Cancel;
        }
        out
    }

    /// Снимок целиком + рамка. Рамка двигается перетаскиванием, масштаб — колесом.
    fn canvas(&mut self, ui: &mut egui::Ui) {
        let (iw, ih) = (self.img.width() as f32, self.img.height() as f32);
        let (rect, resp) = ui.allocate_exact_size(Vec2::new(CANVAS_W, CANVAS_H), Sense::click_and_drag());
        // Масштаб «снимок → экран»: снимок целиком вписан в область.
        let s = (CANVAS_W / iw).min(CANVAS_H / ih);
        let img_rect = Rect::from_center_size(rect.center(), Vec2::new(iw * s, ih * s));

        if resp.dragged() {
            let d = resp.drag_delta();
            self.cx += d.x / s;
            self.cy += d.y / s;
        }
        if resp.hovered() {
            let scroll = ui.input(|i| i.smooth_scroll_delta.y);
            if scroll != 0.0 {
                self.zoom *= (scroll * WHEEL_SPEED).exp();
            }
        }
        self.zoom = self.zoom.clamp(1.0, MAX_ZOOM);
        // Прижимаем центр, чтобы рамка не вылезала за снимок.
        let c = self.rect();
        self.cx = c.x + c.w / 2.0;
        self.cy = c.y + c.h / 2.0;

        if resp.dragged() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::Grabbing);
        } else if resp.hovered() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::Grab);
        }

        let painter = ui.painter_at(rect);
        painter.rect_filled(rect, 4.0, theme::p().surface2);
        painter.image(
            self.tex.id(),
            img_rect,
            Rect::from_min_max(Pos2::new(0.0, 0.0), Pos2::new(1.0, 1.0)),
            Color32::WHITE,
        );

        let frame = Rect::from_min_size(
            Pos2::new(img_rect.min.x + c.x * s, img_rect.min.y + c.y * s),
            Vec2::new(c.w * s, c.h * s),
        );
        // Затемняем всё, что вне рамки (четыре полосы вокруг неё).
        let dim = Color32::from_black_alpha(160);
        let (a, b) = (img_rect, frame);
        painter.rect_filled(Rect::from_min_max(a.min, Pos2::new(a.max.x, b.min.y)), 0.0, dim);
        painter.rect_filled(Rect::from_min_max(Pos2::new(a.min.x, b.max.y), a.max), 0.0, dim);
        painter.rect_filled(Rect::from_min_max(Pos2::new(a.min.x, b.min.y), Pos2::new(b.min.x, b.max.y)), 0.0, dim);
        painter.rect_filled(Rect::from_min_max(Pos2::new(b.max.x, b.min.y), Pos2::new(a.max.x, b.max.y)), 0.0, dim);

        // Рамка и сетка третей (помогает поставить лицо).
        let grid = Stroke::new(1.0_f32, Color32::from_white_alpha(70));
        for k in 1..3 {
            let t = k as f32 / 3.0;
            let x = b.min.x + b.width() * t;
            let y = b.min.y + b.height() * t;
            painter.line_segment([Pos2::new(x, b.min.y), Pos2::new(x, b.max.y)], grid);
            painter.line_segment([Pos2::new(b.min.x, y), Pos2::new(b.max.x, y)], grid);
        }
        painter.rect_stroke(frame, 0.0, Stroke::new(2.0_f32, Color32::WHITE));
    }

    /// Как будет выглядеть фото на карточке.
    fn preview(&self, ui: &mut egui::Ui) {
        let (iw, ih) = (self.img.width() as f32, self.img.height() as f32);
        let c = self.rect();
        let size = Vec2::new(PREVIEW_H * CARD_ASPECT, PREVIEW_H);
        let (rect, _) = ui.allocate_exact_size(size, Sense::hover());
        let uv = Rect::from_min_max(
            Pos2::new(c.x / iw, c.y / ih),
            Pos2::new((c.x + c.w) / iw, (c.y + c.h) / ih),
        );
        egui::Image::new((self.tex.id(), size)).uv(uv).rounding(6.0).paint_at(ui, rect);
        ui.painter().rect_stroke(rect, 6.0, Stroke::new(1.0_f32, theme::p().border_hi));
    }
}
