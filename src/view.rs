//! Режим «Просмотр»: mindmap с zoom/pan, отрисовка только видимых карточек,
//! ленивая загрузка миниатюр в текстуры.

use crate::container::{MediaKind, MediaStore};
use crate::i18n::tr;
use crate::layout::{Layout, LayoutConfig};
use crate::model::{Gender, PersonId, Tree};
use eframe::egui::{
    self, Align2, Color32, FontId, Painter, Pos2, Rect, Rounding, Sense, Stroke, TextureHandle,
    TextureId, Vec2,
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
pub(crate) const PHOTO_PAD: f32 = 4.0;
/// Скругление углов карточки; такое же у главного фото на ней (в единицах мира).
pub(crate) const CARD_RADIUS: f32 = 8.0;
/// Толщина цветной полосы пола внизу карточки (в единицах мира).
pub(crate) const GENDER_STRIP: f32 = 6.0;

/// Плавный перелёт камеры: центр (в мире) и масштаб интерполируются по времени.
#[derive(Clone, Copy)]
struct Anim {
    t0: f64,
    dur: f64,
    from_zoom: f32,
    to_zoom: f32,
    from_c: Pos2,
    to_c: Pos2,
    /// Насколько «отъехать» в середине пути (для дальних перелётов).
    arc: f32,
}

/// Масштаб, к которому подлетает камера (карточка хорошо видна).
const FLY_ZOOM: f32 = 0.9;
/// Сколько секунд пульсирует подсветка найденного человека.
const FLASH_SECS: f64 = 3.0;

pub struct ViewState {
    pub zoom: f32,
    /// Смещение мира относительно левого верхнего угла области (в пикселях).
    pub pan: Vec2,
    /// false → при следующем кадре вписать дерево в окно.
    pub fitted: bool,
    /// Человек, выделенный рамкой (выбранный).
    pub highlight: Option<PersonId>,
    /// Показать этого человека по центру при следующем кадре.
    pub pending_center: Option<PersonId>,
    /// Вместе с перелётом включить пульсирующую подсветку.
    pending_flash: bool,
    anim: Option<Anim>,
    /// Пульсирующая подсветка: (кто, время старта).
    flash: Option<(PersonId, f64)>,
    textures: HashMap<String, Option<TextureHandle>>,
}

impl Default for ViewState {
    fn default() -> Self {
        ViewState {
            zoom: 1.0,
            pan: Vec2::ZERO,
            fitted: false,
            highlight: None,
            pending_center: None,
            pending_flash: false,
            anim: None,
            flash: None,
            textures: HashMap::new(),
        }
    }
}

impl ViewState {
    /// Сбросить камеру и кэш (после загрузки другого дерева).
    pub fn reset(&mut self) {
        self.zoom = 1.0;
        self.pan = Vec2::ZERO;
        self.fitted = false;
        self.highlight = None;
        self.pending_center = None;
        self.pending_flash = false;
        self.anim = None;
        self.flash = None;
        self.textures.clear();
    }

    /// Плавно перелететь камерой к человеку (`flash` — ещё и подсветить).
    pub fn fly_to(&mut self, id: PersonId, flash: bool) {
        self.pending_center = Some(id);
        self.pending_flash = flash;
    }

    fn fit(&mut self, rect: Rect, layout: &Layout) {
        self.anim = None;
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
                tr("Дерево пусто. Откройте файл или добавьте первого человека в режиме «Редакция»."),
                FontId::proportional(18.0),
                visuals.weak_text_color(),
            );
            return None;
        }
        if !self.fitted {
            self.fit(rect, layout);
            self.fitted = true;
        }
        let now = ui.input(|i| i.time);
        if let Some(id) = self.pending_center.take() {
            let flash = std::mem::take(&mut self.pending_flash);
            if let Some(p) = layout.pos_of(id) {
                let from_c = Pos2::new(
                    (rect.width() / 2.0 - self.pan.x) / self.zoom,
                    (rect.height() / 2.0 - self.pan.y) / self.zoom,
                );
                let to_c = Pos2::new(p.x, p.y);
                // Расстояние в «экранах» (при текущем масштабе).
                let dist = (to_c - from_c).length() * self.zoom / rect.width().max(1.0);
                let arc = dist.min(6.0).ln_1p() * 0.6;
                self.anim = Some(Anim {
                    t0: now,
                    dur: (0.55 + 0.25 * dist.min(6.0).ln_1p() as f64).min(1.4),
                    from_zoom: self.zoom,
                    to_zoom: FLY_ZOOM,
                    from_c,
                    to_c,
                    arc,
                });
                if flash {
                    self.flash = Some((id, now));
                }
            }
        }

        // Ввод: перетаскивание — сдвиг, колесо/щипок — масштаб вокруг курсора.
        if response.dragged() {
            self.pan += response.drag_delta();
            self.anim = None;
        }
        if response.hovered() {
            let (scroll, zoom_delta, ptr) =
                ui.input(|i| (i.smooth_scroll_delta.y, i.zoom_delta(), i.pointer.hover_pos()));
            let factor = zoom_delta * (scroll * 0.002).exp();
            if (factor - 1.0).abs() > 1e-4 {
                if let Some(p) = ptr {
                    self.zoom_at(rect, p, factor);
                    self.anim = None;
                }
            }
        }
        // Шаг анимации камеры.
        if let Some(a) = self.anim {
            let t = (((now - a.t0) / a.dur).clamp(0.0, 1.0)) as f32;
            let e = t * t * (3.0 - 2.0 * t);
            let lz = a.from_zoom.ln() + (a.to_zoom.ln() - a.from_zoom.ln()) * e
                - a.arc * (std::f32::consts::PI * e).sin();
            let z = lz.exp().clamp(MIN_ZOOM, MAX_ZOOM);
            let c = a.from_c + (a.to_c - a.from_c) * e;
            self.zoom = z;
            self.pan = Vec2::new(rect.width() / 2.0 - c.x * z, rect.height() / 2.0 - c.y * z);
            if t >= 1.0 {
                self.anim = None;
            } else {
                ui.ctx().request_repaint();
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
            let mid_y = e.bus;
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
        let accent = crate::theme::p().accent;
        let text_color = visuals.text_color();
        let ph_fg = visuals.widgets.noninteractive.bg_stroke.color;
        let ph_bg = visuals.faint_bg_color;

        for n in &layout.nodes {
            let world = Rect::from_center_size(
                Pos2::new(n.pos.x, n.pos.y),
                Vec2::new(cfg.card_w, cfg.card_h),
            );
            // Флаги стоят над карточкой, поэтому область видимости расширена вверх.
            let with_flags = Rect::from_min_max(
                Pos2::new(world.min.x, world.min.y - crate::flags::FLAG_H),
                world.max,
            );
            if !vis.intersects(with_flags) {
                continue;
            }
            let card = Rect::from_min_max(s(world.min.x, world.min.y), s(world.max.x, world.max.y));
            let is_hover = hover_pos.map_or(false, |p| card.contains(p));
            if is_hover {
                hovered_id = Some(n.id);
            }
            let stroke = if is_hover || self.highlight == Some(n.id) { Stroke::new(2.5_f32, accent) } else { card_stroke };

            let Some(person) = tree.persons.get(&n.id) else { continue };

            let strip_color = match person.gender {
                Gender::Male => Some(crate::theme::p().gender_male),
                Gender::Female => Some(crate::theme::p().gender_female),
                Gender::Unknown => None,
            };

            if zoom < DETAIL_ZOOM {
                let strip = (GENDER_STRIP * zoom).max(1.0);
                draw_card_body(&painter, card, 2.0, strip, strip_color, card_bg);
                painter.rect_stroke(card, 2.0, stroke);
                crate::flags::draw_row(&painter, card, zoom, &person.nationalities, false);
                continue;
            }

            let radius = CARD_RADIUS * zoom;
            draw_card_body(&painter, card, radius, GENDER_STRIP * zoom, strip_color, card_bg);
            painter.rect_stroke(card, radius, stroke);
            // Флаги национальностей: над карточкой, вплотную, по центру.
            crate::flags::draw_row(&painter, card, zoom, &person.nationalities, true);

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
                    // Фото скруглено тем же радиусом, что и карточка.
                    egui::Image::new((id, photo.size()))
                        .rounding(radius)
                        .paint_at(ui, photo);
                }
                None => draw_placeholder_rounded(&painter, photo, radius, ph_fg, ph_bg),
            }

            // Имя, фамилия (у женщин с фамилией при рождении — «Иванова (Петрова)»), отчество.
            let clip = painter.with_clip_rect(card);
            let cx = card.center().x;
            let y1 = photo.max.y + 3.0 * zoom;
            let size = 12.5 * zoom;
            let max_w = card.width() - 2.0 * pad;
            let first = person.first_name.trim();
            let last = person.surname_display();
            let patr = person.patronymic.trim();
            if first.is_empty() && last.is_empty() && patr.is_empty() {
                clip.text(
                    Pos2::new(cx, y1),
                    Align2::CENTER_TOP,
                    "—",
                    FontId::proportional(size),
                    text_color,
                );
            } else {
                draw_fitted(&clip, cx, y1, first, size, max_w, text_color);
                draw_fitted(&clip, cx, y1 + 15.0 * zoom, &last, size, max_w, text_color);
                draw_fitted(&clip, cx, y1 + 30.0 * zoom, patr, size, max_w, text_color);
            }
        }

        // Пульсирующая подсветка найденного человека.
        if let Some((fid, t0)) = self.flash {
            let age = now - t0;
            match layout.pos_of(fid) {
                Some(p) if age < FLASH_SECS => {
                    let fade = (1.0 - age / FLASH_SECS) as f32;
                    let card = Rect::from_min_max(
                        s(p.x - cfg.card_w / 2.0, p.y - cfg.card_h / 2.0),
                        s(p.x + cfg.card_w / 2.0, p.y + cfg.card_h / 2.0),
                    );
                    for k in 0..2 {
                        let phase = ((age * 1.1 + k as f64 * 0.5) % 1.0) as f32;
                        let alpha = ((1.0 - phase) * fade * 255.0) as u8;
                        let color = Color32::from_rgba_unmultiplied(255, 170, 0, alpha);
                        painter.rect_stroke(
                            card.expand(phase * 45.0 * zoom),
                            (6.0 + phase * 20.0) * zoom,
                            Stroke::new(3.0_f32, color),
                        );
                    }
                    ctx.request_repaint();
                }
                _ => self.flash = None,
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

/// Строка по центру; если шире `max_w`, шрифт уменьшается (не меньше 70%).
fn draw_fitted(painter: &Painter, cx: f32, y: f32, text: &str, size: f32, max_w: f32, color: Color32) {
    if text.is_empty() {
        return;
    }
    let w = painter
        .layout_no_wrap(text.to_string(), FontId::proportional(size), color)
        .size()
        .x;
    let k = if w > max_w && w > 0.0 { (max_w / w).max(0.7) } else { 1.0 };
    painter.text(
        Pos2::new(cx, y),
        Align2::CENTER_TOP,
        text,
        FontId::proportional(size * k),
        color,
    );
}

/// Фон карточки. Если задан цвет полосы — нижняя часть (`strip`) окрашена им
/// и служит частью обрамления: внешний контур остаётся скруглённым.
fn draw_card_body(
    painter: &Painter,
    card: Rect,
    radius: f32,
    strip: f32,
    strip_color: Option<Color32>,
    bg: Color32,
) {
    match strip_color {
        Some(color) => {
            painter.rect_filled(card, radius, color);
            let inner = Rect::from_min_max(card.min, Pos2::new(card.max.x, card.max.y - strip));
            let top = Rounding { nw: radius, ne: radius, sw: 0.0, se: 0.0 };
            painter.rect_filled(inner, top, bg);
        }
        None => {
            painter.rect_filled(card, radius, bg);
        }
    }
}

pub(crate) fn draw_placeholder(painter: &Painter, r: Rect, fg: Color32, bg: Color32) {
    draw_placeholder_rounded(painter, r, 0.0, fg, bg);
}

/// Заглушка-силуэт со скруглёнными углами.
pub(crate) fn draw_placeholder_rounded(
    painter: &Painter,
    r: Rect,
    radius: f32,
    fg: Color32,
    bg: Color32,
) {
    painter.rect_filled(r, radius, bg);
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
