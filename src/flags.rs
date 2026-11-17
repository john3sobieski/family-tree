//! Европейские страны и их флаги. Флаги рисуются кодом (без картинок), эмблемы упрощены.

use eframe::egui::{pos2, Color32, Painter, Pos2, Rect, Sense, Shape, Stroke, Ui, Vec2};

/// Сколько национальностей может быть у одного человека.
pub const MAX_PER_PERSON: usize = 2;

/// Размер одного флага над карточкой (в единицах мира, 3:2).
pub const FLAG_W: f32 = 30.0;
pub const FLAG_H: f32 = 20.0;

/// (код ISO 3166-1 alpha-2, название). `XK` — Косово (неофициальный код). По алфавиту.
pub const COUNTRIES: &[(&str, &str)] = &[
    ("AT", "Австрия"),
    ("AL", "Албания"),
    ("AD", "Андорра"),
    ("BY", "Беларусь"),
    ("BE", "Бельгия"),
    ("BG", "Болгария"),
    ("BA", "Босния и Герцеговина"),
    ("VA", "Ватикан"),
    ("GB", "Великобритания"),
    ("HU", "Венгрия"),
    ("DE", "Германия"),
    ("GR", "Греция"),
    ("DK", "Дания"),
    ("IE", "Ирландия"),
    ("IS", "Исландия"),
    ("ES", "Испания"),
    ("IT", "Италия"),
    ("CY", "Кипр"),
    ("XK", "Косово"),
    ("LV", "Латвия"),
    ("LT", "Литва"),
    ("LI", "Лихтенштейн"),
    ("LU", "Люксембург"),
    ("MT", "Мальта"),
    ("MD", "Молдова"),
    ("MC", "Монако"),
    ("NL", "Нидерланды"),
    ("NO", "Норвегия"),
    ("PL", "Польша"),
    ("PT", "Португалия"),
    ("RU", "Россия"),
    ("RO", "Румыния"),
    ("SM", "Сан-Марино"),
    ("MK", "Северная Македония"),
    ("RS", "Сербия"),
    ("SK", "Словакия"),
    ("SI", "Словения"),
    ("UA", "Украина"),
    ("FI", "Финляндия"),
    ("FR", "Франция"),
    ("HR", "Хорватия"),
    ("ME", "Черногория"),
    ("CZ", "Чехия"),
    ("CH", "Швейцария"),
    ("SE", "Швеция"),
    ("EE", "Эстония"),
];

/// Название страны по коду.
pub fn name(code: &str) -> Option<&'static str> {
    COUNTRIES.iter().find(|(c, _)| *c == code).map(|(_, n)| *n)
}

/// Маленький флаг для интерфейса (редактор, окно человека).
pub fn icon(ui: &mut Ui, code: &str, h: f32) {
    let (rect, _) = ui.allocate_exact_size(Vec2::new(h * 1.5, h), Sense::hover());
    draw(ui.painter(), rect, code, true);
}

/// Флаги над карточкой: по центру, вплотную к верхней грани и друг к другу.
/// `card` — прямоугольник карточки на экране, `zoom` — текущий масштаб.
pub fn draw_row(painter: &Painter, card: Rect, zoom: f32, codes: &[String], detail: bool) {
    let n = codes.len().min(MAX_PER_PERSON);
    if n == 0 {
        return;
    }
    let (w, h) = (FLAG_W * zoom, FLAG_H * zoom);
    let x0 = card.center().x - w * n as f32 / 2.0;
    for (i, code) in codes.iter().take(n).enumerate() {
        let min = pos2(x0 + w * i as f32, card.min.y - h);
        draw(painter, Rect::from_min_size(min, Vec2::new(w, h)), code, detail);
    }
}

// ---------------------------------------------------------------- рисование

const WHITE: Color32 = Color32::WHITE;

fn rgb(r: u8, g: u8, b: u8) -> Color32 {
    Color32::from_rgb(r, g, b)
}

/// Точка внутри флага по долям ширины/высоты.
fn fr(r: Rect, x: f32, y: f32) -> Pos2 {
    pos2(r.min.x + r.width() * x, r.min.y + r.height() * y)
}

fn part(r: Rect, x0: f32, y0: f32, x1: f32, y1: f32) -> Rect {
    Rect::from_min_max(fr(r, x0, y0), fr(r, x1, y1))
}

fn fill(p: &Painter, r: Rect, c: Color32) {
    p.rect_filled(r, 0.0, c);
}

fn hbands(p: &Painter, r: Rect, bands: &[(f32, Color32)]) {
    let total: f32 = bands.iter().map(|b| b.0).sum();
    let mut y = 0.0;
    for (i, (w, c)) in bands.iter().enumerate() {
        let y1 = if i + 1 == bands.len() { 1.0 } else { y + w / total };
        fill(p, part(r, 0.0, y, 1.0, y1), *c);
        y = y1;
    }
}

fn vbands(p: &Painter, r: Rect, bands: &[(f32, Color32)]) {
    let total: f32 = bands.iter().map(|b| b.0).sum();
    let mut x = 0.0;
    for (i, (w, c)) in bands.iter().enumerate() {
        let x1 = if i + 1 == bands.len() { 1.0 } else { x + w / total };
        fill(p, part(r, x, 0.0, x1, 1.0), *c);
        x = x1;
    }
}

/// Крест на всю ширину/высоту: вертикальная перекладина на доле `cx` ширины, толщина `th` (пиксели).
fn cross(p: &Painter, r: Rect, cx: f32, th: f32, c: Color32) {
    let x = r.min.x + r.width() * cx;
    p.rect_filled(Rect::from_min_max(pos2(x - th / 2.0, r.min.y), pos2(x + th / 2.0, r.max.y)), 0.0, c);
    let y = r.center().y;
    p.rect_filled(Rect::from_min_max(pos2(r.min.x, y - th / 2.0), pos2(r.max.x, y + th / 2.0)), 0.0, c);
}

/// Скандинавский крест (опционально с внутренним крестом другого цвета).
fn nordic(p: &Painter, r: Rect, bg: Color32, c: Color32, th: f32, inner: Option<(Color32, f32)>) {
    fill(p, r, bg);
    cross(p, r, 0.36, th, c);
    if let Some((ic, ith)) = inner {
        cross(p, r, 0.36, ith, ic);
    }
}

fn dot(p: &Painter, r: Rect, x: f32, y: f32, rad: f32, c: Color32) {
    p.circle_filled(fr(r, x, y), r.height() * rad, c);
}

/// Выпуклый многоугольник по долям.
fn poly(p: &Painter, r: Rect, pts: &[(f32, f32)], c: Color32) {
    let v: Vec<Pos2> = pts.iter().map(|&(x, y)| fr(r, x, y)).collect();
    p.add(Shape::convex_polygon(v, c, Stroke::NONE));
}

/// Рисует флаг страны `code` в прямоугольнике `r`. `detail == false` — только основа
/// (полосы и фон, без эмблем): для мелкого масштаба.
pub fn draw(painter: &Painter, r: Rect, code: &str, detail: bool) {
    let clipped = painter.with_clip_rect(painter.clip_rect().intersect(r));
    let p = &clipped;
    let h = r.height();
    match code {
        "AL" => {
            fill(p, r, rgb(224, 30, 42));
            if detail {
                let k = rgb(0, 0, 0);
                poly(p, r, &[(0.50, 0.28), (0.22, 0.40), (0.30, 0.62), (0.50, 0.80), (0.70, 0.62), (0.78, 0.40)], k);
                dot(p, r, 0.43, 0.27, 0.07, k);
                dot(p, r, 0.57, 0.27, 0.07, k);
            }
        }
        "AD" => {
            vbands(p, r, &[(8.0, rgb(16, 6, 159)), (11.0, rgb(254, 221, 0)), (8.0, rgb(213, 0, 50))]);
            if detail {
                dot(p, r, 0.5, 0.5, 0.17, rgb(190, 140, 40));
                dot(p, r, 0.5, 0.5, 0.10, rgb(213, 0, 50));
            }
        }
        "AT" => hbands(p, r, &[(1.0, rgb(237, 41, 57)), (1.0, WHITE), (1.0, rgb(237, 41, 57))]),
        "BY" => {
            hbands(p, r, &[(2.0, rgb(200, 49, 55)), (1.0, rgb(0, 135, 63))]);
            fill(p, part(r, 0.0, 0.0, 0.11, 1.0), WHITE);
            if detail {
                for k in 0..5 {
                    dot(p, r, 0.055, (k as f32 + 0.5) / 5.0, 0.07, rgb(200, 49, 55));
                }
            }
        }
        "BE" => vbands(p, r, &[(1.0, rgb(0, 0, 0)), (1.0, rgb(253, 218, 36)), (1.0, rgb(239, 51, 64))]),
        "BA" => {
            fill(p, r, rgb(0, 45, 160));
            poly(p, r, &[(0.30, 0.0), (0.75, 0.0), (0.75, 1.0)], rgb(254, 203, 0));
            if detail {
                for i in 0..7 {
                    let t = (i as f32 + 0.5) / 7.0;
                    dot(p, r, 0.23 + 0.45 * t, t, 0.045, WHITE);
                }
            }
        }
        "BG" => hbands(p, r, &[(1.0, WHITE), (1.0, rgb(0, 150, 110)), (1.0, rgb(214, 38, 18))]),
        "HR" => {
            hbands(p, r, &[(1.0, rgb(255, 0, 0)), (1.0, WHITE), (1.0, rgb(23, 23, 150))]);
            if detail {
                fill(p, part(r, 0.42, 0.28, 0.58, 0.72), WHITE);
                for cx in 0..2 {
                    for cy in 0..3 {
                        if (cx + cy) % 2 == 0 {
                            let (x0, y0) = (0.42 + cx as f32 * 0.08, 0.28 + cy as f32 * 0.1467);
                            fill(p, part(r, x0, y0, x0 + 0.08, y0 + 0.1467), rgb(255, 0, 0));
                        }
                    }
                }
                p.rect_stroke(part(r, 0.42, 0.28, 0.58, 0.72), 0.0, Stroke::new(1.0_f32, rgb(255, 0, 0)));
            }
        }
        "CY" => {
            fill(p, r, WHITE);
            poly(
                p,
                r,
                &[(0.27, 0.42), (0.50, 0.33), (0.73, 0.42), (0.68, 0.52), (0.50, 0.54), (0.30, 0.52)],
                rgb(213, 125, 50),
            );
            if detail {
                let g = Stroke::new(1.5_f32, rgb(60, 140, 60));
                p.line_segment([fr(r, 0.35, 0.68), fr(r, 0.50, 0.78)], g);
                p.line_segment([fr(r, 0.50, 0.78), fr(r, 0.65, 0.68)], g);
            }
        }
        "CZ" => {
            hbands(p, r, &[(1.0, WHITE), (1.0, rgb(215, 20, 26))]);
            poly(p, r, &[(0.0, 0.0), (0.5, 0.5), (0.0, 1.0)], rgb(17, 69, 126));
        }
        "DK" => nordic(p, r, rgb(200, 16, 46), WHITE, h * 0.2, None),
        "EE" => hbands(p, r, &[(1.0, rgb(0, 114, 206)), (1.0, rgb(0, 0, 0)), (1.0, WHITE)]),
        "FI" => nordic(p, r, WHITE, rgb(0, 53, 128), h * 0.2, None),
        "FR" => vbands(p, r, &[(1.0, rgb(0, 85, 164)), (1.0, WHITE), (1.0, rgb(239, 65, 53))]),
        "DE" => hbands(p, r, &[(1.0, rgb(0, 0, 0)), (1.0, rgb(221, 0, 0)), (1.0, rgb(255, 206, 0))]),
        "GR" => {
            let blue = rgb(13, 94, 175);
            for i in 0..9 {
                let c = if i % 2 == 0 { blue } else { WHITE };
                fill(p, part(r, 0.0, i as f32 / 9.0, 1.0, (i as f32 + 1.0) / 9.0), c);
            }
            let side = h * 5.0 / 9.0;
            fill(p, Rect::from_min_size(r.min, Vec2::new(side, side)), blue);
            let (t, half) = (h / 9.0, side / 2.0);
            let (cx, cy) = (r.min.x + half, r.min.y + half);
            p.rect_filled(Rect::from_min_max(pos2(cx - t / 2.0, cy - half), pos2(cx + t / 2.0, cy + half)), 0.0, WHITE);
            p.rect_filled(Rect::from_min_max(pos2(cx - half, cy - t / 2.0), pos2(cx + half, cy + t / 2.0)), 0.0, WHITE);
        }
        "HU" => hbands(p, r, &[(1.0, rgb(205, 42, 62)), (1.0, WHITE), (1.0, rgb(67, 111, 77))]),
        "IS" => nordic(p, r, rgb(2, 82, 156), WHITE, h * 0.26, Some((rgb(220, 30, 53), h * 0.13))),
        "IE" => vbands(p, r, &[(1.0, rgb(22, 155, 98)), (1.0, WHITE), (1.0, rgb(255, 136, 62))]),
        "IT" => vbands(p, r, &[(1.0, rgb(0, 146, 70)), (1.0, WHITE), (1.0, rgb(206, 43, 55))]),
        "XK" => {
            fill(p, r, rgb(36, 77, 157));
            poly(
                p,
                r,
                &[(0.30, 0.56), (0.38, 0.40), (0.55, 0.34), (0.70, 0.44), (0.68, 0.60), (0.50, 0.68)],
                rgb(209, 168, 0),
            );
            if detail {
                for i in 0..6 {
                    let t = i as f32 / 5.0;
                    let x = 0.30 + 0.40 * t;
                    let y = 0.20 - 0.07 * (1.0 - (2.0 * t - 1.0).powi(2));
                    dot(p, r, x, y, 0.04, WHITE);
                }
            }
        }
        "LV" => hbands(p, r, &[(2.0, rgb(158, 48, 57)), (1.0, WHITE), (2.0, rgb(158, 48, 57))]),
        "LI" => {
            hbands(p, r, &[(1.0, rgb(0, 43, 127)), (1.0, rgb(206, 17, 38))]);
            if detail {
                let y = rgb(255, 215, 0);
                fill(p, part(r, 0.17, 0.20, 0.31, 0.32), y);
                dot(p, r, 0.24, 0.17, 0.05, y);
            }
        }
        "LT" => hbands(p, r, &[(1.0, rgb(253, 185, 19)), (1.0, rgb(0, 106, 68)), (1.0, rgb(193, 39, 45))]),
        "LU" => hbands(p, r, &[(1.0, rgb(237, 41, 57)), (1.0, WHITE), (1.0, rgb(0, 163, 224))]),
        "MT" => {
            vbands(p, r, &[(1.0, WHITE), (1.0, rgb(207, 20, 43))]);
            if detail {
                let g = rgb(150, 150, 150);
                fill(p, part(r, 0.10, 0.08, 0.14, 0.32), g);
                fill(p, part(r, 0.06, 0.16, 0.18, 0.24), g);
            }
        }
        "MD" => {
            vbands(p, r, &[(1.0, rgb(0, 70, 170)), (1.0, rgb(255, 209, 0)), (1.0, rgb(204, 9, 47))]);
            if detail {
                fill(p, part(r, 0.44, 0.36, 0.56, 0.64), rgb(150, 90, 40));
            }
        }
        "MC" => hbands(p, r, &[(1.0, rgb(206, 17, 38)), (1.0, WHITE)]),
        "ME" => {
            fill(p, r, rgb(213, 170, 64));
            fill(p, r.shrink(h * 0.06), rgb(200, 16, 46));
            if detail {
                dot(p, r, 0.5, 0.5, 0.16, rgb(213, 170, 64));
            }
        }
        "NL" => hbands(p, r, &[(1.0, rgb(174, 28, 40)), (1.0, WHITE), (1.0, rgb(33, 70, 139))]),
        "MK" => {
            let y = rgb(255, 230, 0);
            fill(p, r, rgb(216, 32, 32));
            if detail {
                let c = r.center();
                for i in 0..8 {
                    let a = i as f32 * std::f32::consts::FRAC_PI_4;
                    p.line_segment([c, c + Vec2::angled(a) * r.width()], Stroke::new(h * 0.09, y));
                }
            }
            dot(p, r, 0.5, 0.5, 0.14, y);
        }
        "NO" => nordic(p, r, rgb(200, 16, 46), WHITE, h * 0.26, Some((rgb(0, 40, 104), h * 0.13))),
        "PL" => hbands(p, r, &[(1.0, WHITE), (1.0, rgb(220, 20, 60))]),
        "PT" => {
            vbands(p, r, &[(2.0, rgb(0, 102, 0)), (3.0, rgb(255, 0, 0))]);
            if detail {
                dot(p, r, 0.4, 0.5, 0.17, rgb(255, 204, 0));
                dot(p, r, 0.4, 0.5, 0.09, rgb(220, 30, 30));
            }
        }
        "RO" => vbands(p, r, &[(1.0, rgb(0, 43, 127)), (1.0, rgb(252, 209, 22)), (1.0, rgb(206, 17, 38))]),
        "RU" => hbands(p, r, &[(1.0, WHITE), (1.0, rgb(0, 57, 166)), (1.0, rgb(213, 43, 30))]),
        "SM" => {
            hbands(p, r, &[(1.0, WHITE), (1.0, rgb(94, 182, 228))]);
            if detail {
                dot(p, r, 0.5, 0.5, 0.13, rgb(220, 180, 60));
            }
        }
        "RS" => {
            hbands(p, r, &[(1.0, rgb(198, 54, 60)), (1.0, rgb(12, 64, 118)), (1.0, WHITE)]);
            if detail {
                fill(p, part(r, 0.23, 0.22, 0.37, 0.56), rgb(198, 54, 60));
                p.rect_stroke(part(r, 0.23, 0.22, 0.37, 0.56), 0.0, Stroke::new(1.0_f32, rgb(220, 180, 60)));
            }
        }
        "SK" => {
            hbands(p, r, &[(1.0, WHITE), (1.0, rgb(11, 78, 162)), (1.0, rgb(238, 28, 37))]);
            if detail {
                fill(p, part(r, 0.21, 0.18, 0.39, 0.72), rgb(238, 28, 37));
                fill(p, part(r, 0.29, 0.26, 0.31, 0.46), WHITE);
                fill(p, part(r, 0.25, 0.32, 0.35, 0.36), WHITE);
                fill(p, part(r, 0.21, 0.56, 0.39, 0.72), rgb(11, 78, 162));
                p.rect_stroke(part(r, 0.21, 0.18, 0.39, 0.72), 0.0, Stroke::new(1.0_f32, WHITE));
            }
        }
        "SI" => {
            hbands(p, r, &[(1.0, WHITE), (1.0, rgb(0, 82, 165)), (1.0, rgb(237, 28, 36))]);
            if detail {
                fill(p, part(r, 0.18, 0.10, 0.32, 0.50), rgb(0, 82, 165));
                p.rect_stroke(part(r, 0.18, 0.10, 0.32, 0.50), 0.0, Stroke::new(1.0_f32, rgb(237, 28, 36)));
            }
        }
        "ES" => {
            hbands(p, r, &[(1.0, rgb(170, 21, 27)), (2.0, rgb(241, 191, 0)), (1.0, rgb(170, 21, 27))]);
            if detail {
                fill(p, part(r, 0.20, 0.38, 0.30, 0.62), rgb(170, 21, 27));
            }
        }
        "SE" => nordic(p, r, rgb(0, 106, 167), rgb(254, 204, 0), h * 0.2, None),
        "CH" => {
            fill(p, r, rgb(218, 41, 28));
            let (c, arm, t) = (r.center(), h * 0.3, h * 0.2);
            p.rect_filled(Rect::from_min_max(pos2(c.x - t / 2.0, c.y - arm), pos2(c.x + t / 2.0, c.y + arm)), 0.0, WHITE);
            p.rect_filled(Rect::from_min_max(pos2(c.x - arm, c.y - t / 2.0), pos2(c.x + arm, c.y + t / 2.0)), 0.0, WHITE);
        }
        "UA" => hbands(p, r, &[(1.0, rgb(0, 87, 183)), (1.0, rgb(255, 215, 0))]),
        "GB" => {
            let red = rgb(200, 16, 46);
            fill(p, r, rgb(1, 33, 105));
            p.line_segment([r.left_top(), r.right_bottom()], Stroke::new(h * 0.2, WHITE));
            p.line_segment([r.right_top(), r.left_bottom()], Stroke::new(h * 0.2, WHITE));
            if detail {
                p.line_segment([r.left_top(), r.right_bottom()], Stroke::new(h * 0.07, red));
                p.line_segment([r.right_top(), r.left_bottom()], Stroke::new(h * 0.07, red));
            }
            cross(p, r, 0.5, h * 0.33, WHITE);
            cross(p, r, 0.5, h * 0.2, red);
        }
        "VA" => {
            vbands(p, r, &[(1.0, rgb(255, 224, 0)), (1.0, WHITE)]);
            if detail {
                p.line_segment([fr(r, 0.68, 0.30), fr(r, 0.82, 0.70)], Stroke::new(1.5_f32, rgb(200, 160, 0)));
                p.line_segment([fr(r, 0.82, 0.30), fr(r, 0.68, 0.70)], Stroke::new(1.5_f32, rgb(150, 150, 150)));
                dot(p, r, 0.75, 0.30, 0.06, rgb(200, 30, 30));
            }
        }
        _ => fill(p, r, rgb(90, 90, 90)),
    }
    painter.rect_stroke(r, 0.0, Stroke::new(1.0_f32, crate::theme::BORDER_HI));
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn country_codes_are_unique_and_named() {
        let mut seen = HashSet::new();
        for (code, name) in COUNTRIES {
            assert!(seen.insert(*code), "дубль кода {code}");
            assert!(!name.is_empty());
            assert_eq!(code.len(), 2);
        }
        assert_eq!(name("GB"), Some("Великобритания"));
        assert_eq!(name("ZZ"), None);
    }

    #[test]
    fn countries_are_sorted_by_name() {
        let names: Vec<&str> = COUNTRIES.iter().map(|c| c.1).collect();
        let mut sorted = names.clone();
        sorted.sort();
        assert_eq!(names, sorted);
    }
}
