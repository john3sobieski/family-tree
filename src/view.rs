//! Режим «Просмотр»: mindmap с zoom/pan, отрисовка только видимых карточек,
//! ленивая загрузка миниатюр в текстуры.

use crate::container::{MediaKind, MediaStore};
use crate::layout::{Layout, LayoutConfig};
use crate::model::{PersonId, Tree};
use eframe::egui::{
    self, Align2, Color32, FontId, Painter, Pos2, Rect, Sense, Stroke, TextureHandle, TextureId,
    Vec2,
};
use std::collections::HashMap;

const MIN_ZOOM: f32 = 0.03;
const MAX_ZOOM: f32 = 3.0;
/// Ниже этого масштаба — только прямоугольники (без фото и текста).
const DETAIL_ZOOM: f32 = 0.3;
/// Лимит текстур в памяти (миниатюра ~200 КБ).
const MAX_TEXTURES: usize = 300;
/// Сколько миниатюр загружать за кадр, чтобы не вешать интерфейс.
const LOADS_PER_FRAME: usize = 6;
/// Отступ фото внутри карточки (в единицах мира).
const PHOTO_PAD: f32 = 4.0;

pub struct ViewState {
    pub zoom: f32,
    /// Смещение мира относительно левого верхнего угла области (в пикселях).
    pub pan: Vec2,
    /// false → при следующем кадре вписать дерево в окно.
    pub fitted: bool,
    textures: HashMap<String, Option<TextureHandle>>,
}

impl Default for ViewState {
    fn default() -> Self {
        ViewState { zoom: 1.0, pan: Vec2::ZERO, fitted: false, textures: HashMap::new() }
    }
}

impl ViewState {
    /// Сбросить камеру и кэш (после загрузки другого дерева).
    pub fn reset(&mut self) {
        self.zoom = 1.0;
        self.pan = Vec2::ZERO;
        self.fitted = false;
        self.textures.clear();
    }

    fn fit(&mut self, rect: Rect, layout: &Layout) {
        let b = layout.bounds;
        let bw = (b.max.x - b.min.x).max(1.0);
        let bh = (b.max.y - b.min.y).max(1.0);
        let margin = 60.0;
        let z = ((rect.width() - margin) / bw)
            .min((rect.height() - margin) / bh)
            .clamp(MIN_ZOOM, 1.0);
        let cx = (b.min.x + b.max.x) / 2.0;
        let cy = (b.min.y + b.max.y) / 2.0;
        self.zoom = z;
        self.pan = Vec2::new(rect.width() / 2.0 - cx * z, rect.height() / 2.0 - cy * z);
    }

    fn zoom_at(&mut self, rect: Rect, pointer: Pos2, factor: f32) {
        let new_zoom = (self.zoom * factor).clamp(MIN_ZOOM, MAX_ZOOM);
        let k = new_zoom / self.zoom;
        let rel = pointer - rect.min;
        self.pan = rel - (rel - self.pan) * k;
        self.zoom = new_zoom;
    }

    fn texture_for(
        &mut self,
        ctx: &egui::Context,
        store: &MediaStore,
        hash: &str,
        budget: &mut usize,
        pending: &mut bool,
    ) -> Option<TextureId> {
        if let Some(entry) = self.textures.get(hash) {
            return entry.as_ref().map(|t| t.id());
        }
        if *budget == 0 {
            *pending = true;
            return None;
        }
        *budget -= 1;
        let tex = load_thumb(ctx, store, hash);
        if self.textures.len() >= MAX_TEXTURES {
            self.textures.clear();
        }
        let id = tex.as_ref().map(|t| t.id());
        self.textures.insert(hash.to_string(), tex);
        id
    }

    /// Рисует дерево. Возвращает человека, по карточке которого кликнули.
    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        tree: &Tree,
        layout: &Layout,
        cfg: &LayoutConfig,
        store: &MediaStore,
    ) -> Option<PersonId> {
        let (response, painter) = ui.allocate_painter(ui.available_size(), Sense::click_and_drag());
        let rect = response.rect;
        let visuals = ui.visuals().clone();
        painter.rect_filled(rect, 0.0, visuals.extreme_bg_color);

        if layout.nodes.is_empty() {
            painter.text(
                rect.center(),
                Align2::CENTER_CENTER,
                "Дерево пусто. Откройте файл или нажмите «Демо».",
                FontId::proportional(18.0),
                visuals.weak_text_color(),
            );
            return None;
        }
        if !self.fitted {
            self.fit(rect, layout);
            self.fitted = true;
        }

        // Ввод: перетаскивание — сдвиг, колесо/щипок — масштаб вокруг курсора.
        if response.dragged() {
            self.pan += response.drag_delta();
        }
        if response.hovered() {
            let (scroll, zoom_delta, ptr) =
                ui.input(|i| (i.smooth_scroll_delta.y, i.zoom_delta(), i.pointer.hover_pos()));
            let factor = zoom_delta * (scroll * 0.002).exp();
            if (factor - 1.0).abs() > 1e-4 {
                if let Some(p) = ptr {
                    self.zoom_at(rect, p, factor);
                }
            }
        }

        let zoom = self.zoom;
        let pan = self.pan;
        let origin = rect.min;
        let s = |x: f32, y: f32| Pos2::new(origin.x + pan.x + x * zoom, origin.y + pan.y + y * zoom);
        // Видимая область в координатах мира.
        let inv = 1.0 / zoom;
        let vis = Rect::from_min_max(
            Pos2::new(-pan.x * inv, -pan.y * inv),
            Pos2::new((rect.width() - pan.x) * inv, (rect.height() - pan.y) * inv),
        );

        let line_color = visuals.weak_text_color();
        let line = Stroke::new((1.5 * zoom).clamp(0.6, 2.5), line_color);

        // Связи родитель → ребёнок.
        for e in &layout.edges {
            if !vis.intersects(Rect::from_two_pos(
                Pos2::new(e.from.x, e.from.y),
                Pos2::new(e.to.x, e.to.y),
            )) {
                continue;
            }
            if e.hang {
                // «Лесенка»: вниз по общей линии, затем вбок к карточке.
                painter.add(egui::Shape::line(
                    vec![s(e.from.x, e.from.y), s(e.from.x, e.to.y), s(e.to.x, e.to.y)],
                    line,
                ));
                continue;
            }
            let mid_y = (e.from.y + e.to.y) / 2.0;
            painter.add(egui::Shape::line(
                vec![s(e.from.x, e.from.y), s(e.from.x, mid_y), s(e.to.x, mid_y), s(e.to.x, e.to.y)],
                line,
            ));
        }
        // Связи партнёров.
        for u in &layout.units {
            if u.members.len() < 2 {
                continue;
            }
            let ur = Rect::from_center_size(
                Pos2::new(u.center.x, u.center.y),
                Vec2::new(u.width, cfg.card_h),
            );
            if !vis.intersects(ur) {
                continue;
            }
            for pair in u.members.windows(2) {
                if let (Some(a), Some(b)) = (layout.pos_of(pair[0]), layout.pos_of(pair[1])) {
                    let y = a.y - cfg.card_h / 2.0 + (cfg.card_w * 4.0 / 3.0) / 2.0;
                    painter.line_segment(
                        [s(a.x + cfg.card_w / 2.0, y), s(b.x - cfg.card_w / 2.0, y)],
                        line,
                    );
                }
            }
        }

        // Карточки (только видимые).
        let hover_pos = response.hover_pos();
        let mut hovered_id: Option<PersonId> = None;
        let mut budget = LOADS_PER_FRAME;
        let mut pending = false;
        let ctx = ui.ctx().clone();
        let card_bg = visuals.widgets.noninteractive.bg_fill;
        let card_stroke = visuals.widgets.noninteractive.bg_stroke;
        let accent = visuals.selection.bg_fill;
        let text_color = visuals.text_color();
        let ph_fg = visuals.widgets.noninteractive.bg_stroke.color;
        let ph_bg = visuals.faint_bg_color;

        for n in &layout.nodes {
            let world = Rect::from_center_size(
                Pos2::new(n.pos.x, n.pos.y),
                Vec2::new(cfg.card_w, cfg.card_h),
            );
            if !vis.intersects(world) {
                continue;
            }
            let card = Rect::from_min_max(s(world.min.x, world.min.y), s(world.max.x, world.max.y));
            let is_hover = hover_pos.map_or(false, |p| card.contains(p));
            if is_hover {
                hovered_id = Some(n.id);
            }
            let stroke = if is_hover { Stroke::new(2.5, accent) } else { card_stroke };

            if zoom < DETAIL_ZOOM {
                painter.rect_filled(card, 2.0, card_bg);
                painter.rect_stroke(card, 2.0, stroke);
                continue;
            }
            let Some(person) = tree.persons.get(&n.id) else { continue };

            painter.rect_filled(card, 4.0 * zoom, card_bg);
            painter.rect_stroke(card, 4.0 * zoom, stroke);

            // Фото 3:4.
            let pad = PHOTO_PAD * zoom;
            let pw = card.width() - 2.0 * pad;
            let photo = Rect::from_min_size(
                card.min + Vec2::splat(pad),
                Vec2::new(pw, pw * 4.0 / 3.0),
            );
            let tex = person
                .main_photo
                .as_deref()
                .and_then(|h| self.texture_for(&ctx, store, h, &mut budget, &mut pending));
            match tex {
                Some(id) => {
                    painter.image(
                        id,
                        photo,
                        Rect::from_min_max(Pos2::new(0.0, 0.0), Pos2::new(1.0, 1.0)),
                        Color32::WHITE,
                    );
                }
                None => draw_placeholder(&painter, photo, ph_fg, ph_bg),
            }

            // Имя и фамилия.
            let clip = painter.with_clip_rect(card);
            let font = FontId::proportional(12.5 * zoom);
            let cx = card.center().x;
            let y1 = photo.max.y + 3.0 * zoom;
            let (first, last) = (person.first_name.trim(), person.last_name.trim());
            if first.is_empty() && last.is_empty() {
                clip.text(Pos2::new(cx, y1), Align2::CENTER_TOP, "—", font, text_color);
            } else {
                clip.text(Pos2::new(cx, y1), Align2::CENTER_TOP, first, font.clone(), text_color);
                clip.text(
                    Pos2::new(cx, y1 + 15.0 * zoom),
                    Align2::CENTER_TOP,
                    last,
                    font,
                    text_color,
                );
            }
        }

        if pending {
            ctx.request_repaint();
        }
        if hovered_id.is_some() && !response.dragged() {
            ctx.set_cursor_icon(egui::CursorIcon::PointingHand);
        }
        if response.clicked() {
            return hovered_id;
        }
        None
    }
}

pub(crate) fn draw_placeholder(painter: &Painter, r: Rect, fg: Color32, bg: Color32) {
    painter.rect_filled(r, 0.0, bg);
    let clip = painter.with_clip_rect(r);
    let cx = r.center().x;
    clip.circle_filled(Pos2::new(cx, r.top() + r.height() * 0.36), r.width() * 0.17, fg);
    clip.circle_filled(Pos2::new(cx, r.top() + r.height() * 0.98), r.width() * 0.38, fg);
}

fn load_thumb(ctx: &egui::Context, store: &MediaStore, hash: &str) -> Option<TextureHandle> {
    let bytes = store.get(MediaKind::Thumb, hash).ok()?;
    let img = image::load_from_memory(&bytes).ok()?.to_rgba8();
    let size = [img.width() as usize, img.height() as usize];
    let color = egui::ColorImage::from_rgba_unmultiplied(size, img.as_raw());
    Some(ctx.load_texture(format!("thumb-{hash}"), color, egui::TextureOptions::LINEAR))
}
