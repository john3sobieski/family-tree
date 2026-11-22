//! All countries of the world and their flags. Flags are drawn in code (no images), emblems are simplified.

use eframe::egui::{pos2, Color32, Mesh, Painter, Pos2, Rect, Sense, Shape, Stroke, Ui, Vec2};
use crate::i18n::Lang;
use std::f32::consts::{FRAC_PI_2, PI, TAU};

/// Сколько национальностей может быть у одного человека.
pub const MAX_PER_PERSON: usize = 2;

/// Размер одного флага над карточкой (в единицах мира, 3:2).
pub const FLAG_W: f32 = 30.0;
pub const FLAG_H: f32 = 20.0;

/// (ISO 3166-1 alpha-2 code, Russian name), sorted by name: 193 UN members, the Vatican,
/// Palestine, Taiwan and Kosovo (`XK`, unofficial code).
pub const COUNTRIES: &[(&str, &str)] = &[
    ("AU", "Австралия"),
    ("AT", "Австрия"),
    ("AZ", "Азербайджан"),
    ("AL", "Албания"),
    ("DZ", "Алжир"),
    ("AO", "Ангола"),
    ("AD", "Андорра"),
    ("AG", "Антигуа и Барбуда"),
    ("AR", "Аргентина"),
    ("AM", "Армения"),
    ("AF", "Афганистан"),
    ("BS", "Багамы"),
    ("BD", "Бангладеш"),
    ("BB", "Барбадос"),
    ("BH", "Бахрейн"),
    ("BY", "Беларусь"),
    ("BZ", "Белиз"),
    ("BE", "Бельгия"),
    ("BJ", "Бенин"),
    ("BG", "Болгария"),
    ("BO", "Боливия"),
    ("BA", "Босния и Герцеговина"),
    ("BW", "Ботсвана"),
    ("BR", "Бразилия"),
    ("BN", "Бруней"),
    ("BF", "Буркина-Фасо"),
    ("BI", "Бурунди"),
    ("BT", "Бутан"),
    ("VU", "Вануату"),
    ("VA", "Ватикан"),
    ("GB", "Великобритания"),
    ("HU", "Венгрия"),
    ("VE", "Венесуэла"),
    ("TL", "Восточный Тимор"),
    ("VN", "Вьетнам"),
    ("GA", "Габон"),
    ("HT", "Гаити"),
    ("GY", "Гайана"),
    ("GM", "Гамбия"),
    ("GH", "Гана"),
    ("GT", "Гватемала"),
    ("GN", "Гвинея"),
    ("GW", "Гвинея-Бисау"),
    ("DE", "Германия"),
    ("HN", "Гондурас"),
    ("GD", "Гренада"),
    ("GR", "Греция"),
    ("GE", "Грузия"),
    ("CD", "ДР Конго"),
    ("DK", "Дания"),
    ("DJ", "Джибути"),
    ("DM", "Доминика"),
    ("DO", "Доминиканская Республика"),
    ("EG", "Египет"),
    ("ZM", "Замбия"),
    ("ZW", "Зимбабве"),
    ("IL", "Израиль"),
    ("IN", "Индия"),
    ("ID", "Индонезия"),
    ("JO", "Иордания"),
    ("IQ", "Ирак"),
    ("IR", "Иран"),
    ("IE", "Ирландия"),
    ("IS", "Исландия"),
    ("ES", "Испания"),
    ("IT", "Италия"),
    ("YE", "Йемен"),
    ("CV", "Кабо-Верде"),
    ("KZ", "Казахстан"),
    ("KH", "Камбоджа"),
    ("CM", "Камерун"),
    ("CA", "Канада"),
    ("QA", "Катар"),
    ("KE", "Кения"),
    ("CY", "Кипр"),
    ("KG", "Киргизия"),
    ("KI", "Кирибати"),
    ("CN", "Китай"),
    ("CO", "Колумбия"),
    ("KM", "Коморы"),
    ("XK", "Косово"),
    ("CR", "Коста-Рика"),
    ("CI", "Кот-д'Ивуар"),
    ("CU", "Куба"),
    ("KW", "Кувейт"),
    ("LA", "Лаос"),
    ("LV", "Латвия"),
    ("LS", "Лесото"),
    ("LR", "Либерия"),
    ("LB", "Ливан"),
    ("LY", "Ливия"),
    ("LT", "Литва"),
    ("LI", "Лихтенштейн"),
    ("LU", "Люксембург"),
    ("MU", "Маврикий"),
    ("MR", "Мавритания"),
    ("MG", "Мадагаскар"),
    ("MW", "Малави"),
    ("MY", "Малайзия"),
    ("ML", "Мали"),
    ("MV", "Мальдивы"),
    ("MT", "Мальта"),
    ("MA", "Марокко"),
    ("MH", "Маршалловы Острова"),
    ("MX", "Мексика"),
    ("FM", "Микронезия"),
    ("MZ", "Мозамбик"),
    ("MD", "Молдова"),
    ("MC", "Монако"),
    ("MN", "Монголия"),
    ("MM", "Мьянма"),
    ("NA", "Намибия"),
    ("NR", "Науру"),
    ("NP", "Непал"),
    ("NE", "Нигер"),
    ("NG", "Нигерия"),
    ("NL", "Нидерланды"),
    ("NI", "Никарагуа"),
    ("NZ", "Новая Зеландия"),
    ("NO", "Норвегия"),
    ("AE", "Объединённые Арабские Эмираты"),
    ("OM", "Оман"),
    ("PK", "Пакистан"),
    ("PW", "Палау"),
    ("PS", "Палестина"),
    ("PA", "Панама"),
    ("PG", "Папуа — Новая Гвинея"),
    ("PY", "Парагвай"),
    ("PE", "Перу"),
    ("PL", "Польша"),
    ("PT", "Португалия"),
    ("CG", "Республика Конго"),
    ("RU", "Россия"),
    ("RW", "Руанда"),
    ("RO", "Румыния"),
    ("US", "США"),
    ("SV", "Сальвадор"),
    ("WS", "Самоа"),
    ("SM", "Сан-Марино"),
    ("ST", "Сан-Томе и Принсипи"),
    ("SA", "Саудовская Аравия"),
    ("KP", "Северная Корея"),
    ("MK", "Северная Македония"),
    ("SC", "Сейшелы"),
    ("SN", "Сенегал"),
    ("VC", "Сент-Винсент и Гренадины"),
    ("KN", "Сент-Китс и Невис"),
    ("LC", "Сент-Люсия"),
    ("RS", "Сербия"),
    ("SG", "Сингапур"),
    ("SY", "Сирия"),
    ("SK", "Словакия"),
    ("SI", "Словения"),
    ("SB", "Соломоновы Острова"),
    ("SO", "Сомали"),
    ("SD", "Судан"),
    ("SR", "Суринам"),
    ("SL", "Сьерра-Леоне"),
    ("TJ", "Таджикистан"),
    ("TH", "Таиланд"),
    ("TW", "Тайвань"),
    ("TZ", "Танзания"),
    ("TG", "Того"),
    ("TO", "Тонга"),
    ("TT", "Тринидад и Тобаго"),
    ("TV", "Тувалу"),
    ("TN", "Тунис"),
    ("TM", "Туркменистан"),
    ("TR", "Турция"),
    ("UG", "Уганда"),
    ("UZ", "Узбекистан"),
    ("UA", "Украина"),
    ("UY", "Уругвай"),
    ("FJ", "Фиджи"),
    ("PH", "Филиппины"),
    ("FI", "Финляндия"),
    ("FR", "Франция"),
    ("HR", "Хорватия"),
    ("CF", "Центральноафриканская Республика"),
    ("TD", "Чад"),
    ("ME", "Черногория"),
    ("CZ", "Чехия"),
    ("CL", "Чили"),
    ("CH", "Швейцария"),
    ("SE", "Швеция"),
    ("LK", "Шри-Ланка"),
    ("EC", "Эквадор"),
    ("GQ", "Экваториальная Гвинея"),
    ("ER", "Эритрея"),
    ("SZ", "Эсватини"),
    ("EE", "Эстония"),
    ("ET", "Эфиопия"),
    ("ZA", "ЮАР"),
    ("KR", "Южная Корея"),
    ("SS", "Южный Судан"),
    ("JM", "Ямайка"),
    ("JP", "Япония"),
];

/// Country name in the current interface language.
pub fn name(code: &str) -> Option<&'static str> {
    name_in(crate::i18n::current(), code)
}

pub fn name_in(lang: Lang, code: &str) -> Option<&'static str> {
    match lang.column() {
        None => COUNTRIES.iter().find(|(c, _)| *c == code).map(|(_, n)| *n),
        Some(col) => crate::countries::NAMES.iter().find(|(c, _)| *c == code).map(|(_, n)| n[col]),
    }
}

/// Lowercase and strip diacritics so that "Égypte" sorts under E and "Łotwa" under L.
fn fold(s: &str) -> String {
    s.chars()
        .flat_map(|c| c.to_lowercase())
        .map(|c| match c {
            'à' | 'á' | 'â' | 'ã' | 'ä' | 'å' | 'ą' => 'a',
            'ç' | 'ć' | 'č' => 'c',
            'è' | 'é' | 'ê' | 'ë' | 'ę' | 'ě' => 'e',
            'ì' | 'í' | 'î' | 'ï' => 'i',
            'ł' => 'l',
            'ñ' | 'ń' => 'n',
            'ò' | 'ó' | 'ô' | 'õ' | 'ö' | 'ø' => 'o',
            'ř' => 'r',
            'ś' | 'š' => 's',
            'ù' | 'ú' | 'û' | 'ü' | 'ů' => 'u',
            'ý' => 'y',
            'ź' | 'ż' | 'ž' => 'z',
            other => other,
        })
        .collect()
}

/// All countries as (code, name in `lang`), sorted for that language.
/// Chinese has no alphabet order here, so it follows the English names.
pub fn sorted(lang: Lang) -> Vec<(&'static str, &'static str)> {
    let mut v: Vec<(&'static str, &'static str)> =
        COUNTRIES.iter().map(|(c, ru)| (*c, name_in(lang, c).unwrap_or(*ru))).collect();
    match lang {
        Lang::Ru => {}
        Lang::Zh => v.sort_by_cached_key(|(c, _)| fold(name_in(Lang::En, c).unwrap_or(""))),
        _ => v.sort_by_cached_key(|(_, n)| fold(n)),
    }
    v
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
const BLACK: Color32 = Color32::BLACK;

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

/// Fills a star-shaped polygon (ring seen from `centre`) with a triangle fan; no seams between triangles.
fn fan(p: &Painter, centre: Pos2, ring: &[Pos2], c: Color32) {
    let n = ring.len() as u32;
    if n < 3 {
        return;
    }
    let mut m = Mesh::default();
    m.colored_vertex(centre, c);
    for q in ring {
        m.colored_vertex(*q, c);
    }
    for i in 0..n {
        m.add_triangle(0, 1 + i, 1 + (i + 1) % n);
    }
    p.add(Shape::mesh(m));
}

/// Star with `n` points, one point up. `outer` is in pixels, `inner` is the inner/outer radius ratio.
fn star_px(p: &Painter, c: Pos2, outer: f32, n: usize, inner: f32, col: Color32) {
    let ring: Vec<Pos2> = (0..n * 2)
        .map(|i| {
            let a = -FRAC_PI_2 + i as f32 * PI / n as f32;
            let rad = if i % 2 == 0 { outer } else { outer * inner };
            c + Vec2::angled(a) * rad
        })
        .collect();
    fan(p, c, &ring, col);
}

/// Five-pointed star at a flag-relative position; `rad` is the outer radius as a share of the height.
fn star(p: &Painter, r: Rect, x: f32, y: f32, rad: f32, c: Color32) {
    star_px(p, fr(r, x, y), r.height() * rad, 5, 0.382, c);
}

/// Star with `n` points (suns, rayed stars).
fn star_n(p: &Painter, r: Rect, x: f32, y: f32, rad: f32, n: usize, inner: f32, c: Color32) {
    star_px(p, fr(r, x, y), r.height() * rad, n, inner, c);
}

/// Pentagram drawn with lines only (Morocco, Ethiopia-style outline stars).
fn pentagram(p: &Painter, r: Rect, x: f32, y: f32, rad: f32, width: f32, c: Color32) {
    let (ctr, big) = (fr(r, x, y), r.height() * rad);
    let v = |k: usize| ctr + Vec2::angled(-FRAC_PI_2 + k as f32 * TAU / 5.0) * big;
    p.add(Shape::closed_line(vec![v(0), v(2), v(4), v(1), v(3)], Stroke::new(width, c)));
}

/// Crescent moon in colour `c`; `dir` is the direction the opening faces. Needs no background colour.
fn crescent(p: &Painter, r: Rect, x: f32, y: f32, rad: f32, dir: Vec2, c: Color32) {
    let (big, d, small) = (r.height() * rad, 0.3_f32, 0.8_f32);
    let o = fr(r, x, y);
    let u = dir.normalized();
    let v = Vec2::new(-u.y, u.x);
    // Intersection of the outer circle (radius 1) and the cut-out circle (radius `small`, offset `d`).
    let phi = ((d * d + 1.0 - small * small) / (2.0 * d)).acos();
    let psi = phi.sin().atan2(phi.cos() - d);
    let at = |a: f32, rr: f32, off: f32| o + (u * (off + rr * a.cos()) + v * (rr * a.sin())) * big;
    let n = 24;
    let mut m = Mesh::default();
    for i in 0..=n {
        let t = i as f32 / n as f32;
        m.colored_vertex(at(phi + t * (TAU - 2.0 * phi), 1.0, 0.0), c);
        m.colored_vertex(at(psi + t * (TAU - 2.0 * psi), small, d), c);
    }
    for i in 0..n as u32 {
        let k = i * 2;
        m.add_triangle(k, k + 1, k + 2);
        m.add_triangle(k + 1, k + 2, k + 3);
    }
    p.add(Shape::mesh(m));
}

/// Circle outline.
fn ring(p: &Painter, r: Rect, x: f32, y: f32, rad: f32, width: f32, c: Color32) {
    p.circle_stroke(fr(r, x, y), r.height() * rad, Stroke::new(width, c));
}

/// Diamond (rhombus) with half-sizes `dx`, `dy` in flag fractions.
fn diamond(p: &Painter, r: Rect, x: f32, y: f32, dx: f32, dy: f32, c: Color32) {
    poly(p, r, &[(x, y - dy), (x + dx, y), (x, y + dy), (x - dx, y)], c);
}

/// Thick diagonal band between two opposite corners of `r`.
fn diag(p: &Painter, from: Pos2, to: Pos2, width: f32, c: Color32) {
    p.line_segment([from, to], Stroke::new(width, c));
}

/// Union Jack (also used as a canton), clipped to `r`.
fn union_jack(p: &Painter, r: Rect, detail: bool) {
    let q = p.with_clip_rect(p.clip_rect().intersect(r));
    let (red, h) = (rgb(200, 16, 46), r.height());
    fill(&q, r, rgb(1, 33, 105));
    diag(&q, r.left_top(), r.right_bottom(), h * 0.2, WHITE);
    diag(&q, r.right_top(), r.left_bottom(), h * 0.2, WHITE);
    if detail {
        diag(&q, r.left_top(), r.right_bottom(), h * 0.07, red);
        diag(&q, r.right_top(), r.left_bottom(), h * 0.07, red);
    }
    cross(&q, r, 0.5, h * 0.33, WHITE);
    cross(&q, r, 0.5, h * 0.2, red);
}

/// Maple leaf outline (unit coordinates, y down), star-shaped around the origin.
const MAPLE: [(f32, f32); 23] = [
    (0.0, -1.0),
    (0.13, -0.66),
    (0.33, -0.78),
    (0.27, -0.36),
    (0.6, -0.5),
    (0.52, -0.22),
    (0.84, -0.04),
    (0.44, 0.18),
    (0.5, 0.4),
    (0.2, 0.32),
    (0.05, 0.55),
    (0.04, 0.9),
    (-0.04, 0.9),
    (-0.05, 0.55),
    (-0.2, 0.32),
    (-0.5, 0.4),
    (-0.44, 0.18),
    (-0.84, -0.04),
    (-0.52, -0.22),
    (-0.6, -0.5),
    (-0.27, -0.36),
    (-0.33, -0.78),
    (-0.13, -0.66),
];

/// Rows of stars for the US canton: 9 rows of 6 and 5 stars, alternating.
fn us_stars(p: &Painter, canton: Rect) {
    for row in 0..9 {
        let (count, shift) = if row % 2 == 0 { (6, 0.5) } else { (5, 1.0) };
        for k in 0..count {
            let x = (k as f32 + shift) / 6.0;
            let y = (row as f32 + 1.0) / 10.0;
            star(p, canton, x, y, 0.06, WHITE);
        }
    }
}

/// Rays of a wheel (India's chakra): `spokes` lines through the centre.
fn wheel(p: &Painter, r: Rect, x: f32, y: f32, rad: f32, spokes: usize, c: Color32) {
    let (ctr, big) = (fr(r, x, y), r.height() * rad);
    for k in 0..spokes {
        let d = Vec2::angled(k as f32 * PI / spokes as f32) * big;
        p.line_segment([ctr - d, ctr + d], Stroke::new(1.0_f32, c));
    }
    p.circle_stroke(ctr, big, Stroke::new(big * 0.12 + 0.5, c));
}

/// Draws the flag of country `code` in rectangle `r`. `detail == false` — только основа
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
        "GB" => union_jack(p, r, detail),
        "VA" => {
            vbands(p, r, &[(1.0, rgb(255, 224, 0)), (1.0, WHITE)]);
            if detail {
                p.line_segment([fr(r, 0.68, 0.30), fr(r, 0.82, 0.70)], Stroke::new(1.5_f32, rgb(200, 160, 0)));
                p.line_segment([fr(r, 0.82, 0.30), fr(r, 0.68, 0.70)], Stroke::new(1.5_f32, rgb(150, 150, 150)));
                dot(p, r, 0.75, 0.30, 0.06, rgb(200, 30, 30));
            }
        }
        "NG" => vbands(p, r, &[(1.0, rgb(0, 135, 81)), (1.0, WHITE), (1.0, rgb(0, 135, 81))]),
        "AE" => {
            hbands(p, r, &[(1.0, rgb(0, 115, 47)), (1.0, WHITE), (1.0, BLACK)]);
            fill(p, part(r, 0.0, 0.0, 0.25, 1.0), rgb(255, 0, 0));
        }
        "AF" => {
            vbands(p, r, &[(1.0, BLACK), (1.0, rgb(190, 0, 0)), (1.0, rgb(0, 122, 54))]);
            if detail {
                ring(p, r, 0.5, 0.5, 0.17, 1.5, WHITE);
            }
        }
        "AG" => {
            fill(p, r, rgb(206, 17, 38));
            let xl = |y: f32| 0.5 * y / 0.62;
            let bands = [(0.0, 0.2, BLACK), (0.2, 0.4, rgb(0, 114, 198)), (0.4, 0.62, WHITE)];
            for (y0, y1, c) in bands {
                poly(p, r, &[(xl(y0), y0), (1.0 - xl(y0), y0), (1.0 - xl(y1), y1), (xl(y1), y1)], c);
            }
            if detail {
                dot(p, r, 0.5, 0.12, 0.07, rgb(252, 209, 22));
            }
        }
        "AM" => hbands(p, r, &[(1.0, rgb(217, 0, 18)), (1.0, rgb(0, 51, 160)), (1.0, rgb(242, 168, 0))]),
        "AO" => {
            hbands(p, r, &[(1.0, rgb(204, 9, 47)), (1.0, BLACK)]);
            if detail {
                star(p, r, 0.5, 0.5, 0.15, rgb(255, 203, 0));
            }
        }
        "AR" => {
            hbands(p, r, &[(1.0, rgb(117, 170, 219)), (1.0, WHITE), (1.0, rgb(117, 170, 219))]);
            if detail {
                dot(p, r, 0.5, 0.5, 0.08, rgb(246, 180, 14));
            }
        }
        "AU" => {
            fill(p, r, rgb(0, 0, 139));
            union_jack(p, part(r, 0.0, 0.0, 0.5, 0.5), detail);
            if detail {
                star_n(p, r, 0.25, 0.76, 0.13, 7, 0.45, WHITE);
                for (x, y, s) in [(0.75, 0.12, 0.06), (0.63, 0.45, 0.06), (0.88, 0.38, 0.06), (0.75, 0.82, 0.06), (0.8, 0.58, 0.035)] {
                    star(p, r, x, y, s, WHITE);
                }
            }
        }
        "AZ" => {
            hbands(p, r, &[(1.0, rgb(0, 181, 226)), (1.0, rgb(239, 51, 64)), (1.0, rgb(80, 158, 47))]);
            if detail {
                crescent(p, r, 0.46, 0.5, 0.15, Vec2::new(1.0, 0.0), WHITE);
                star_n(p, r, 0.58, 0.5, 0.06, 8, 0.45, WHITE);
            }
        }
        "BB" => {
            vbands(p, r, &[(1.0, rgb(0, 38, 100)), (1.0, rgb(255, 199, 38)), (1.0, rgb(0, 38, 100))]);
            if detail {
                fill(p, part(r, 0.485, 0.28, 0.515, 0.72), BLACK);
                fill(p, part(r, 0.43, 0.28, 0.57, 0.36), BLACK);
            }
        }
        "BD" => {
            fill(p, r, rgb(0, 106, 78));
            dot(p, r, 0.45, 0.5, 0.3, rgb(244, 42, 65));
        }
        "BF" => {
            hbands(p, r, &[(1.0, rgb(239, 43, 45)), (1.0, rgb(0, 158, 73))]);
            if detail {
                star(p, r, 0.5, 0.5, 0.2, rgb(252, 209, 22));
            }
        }
        "BH" => {
            fill(p, r, rgb(206, 17, 38));
            fill(p, part(r, 0.0, 0.0, 0.27, 1.0), WHITE);
            for i in 0..5 {
                let (y0, y1) = (i as f32 / 5.0, (i as f32 + 1.0) / 5.0);
                poly(p, r, &[(0.27, y0), (0.42, (y0 + y1) / 2.0), (0.27, y1)], WHITE);
            }
        }
        "BI" => {
            let (red, green) = (rgb(206, 17, 38), rgb(30, 181, 58));
            fill(p, r, red);
            poly(p, r, &[(0.0, 0.0), (0.5, 0.5), (0.0, 1.0)], green);
            poly(p, r, &[(1.0, 0.0), (0.5, 0.5), (1.0, 1.0)], green);
            diag(p, r.left_top(), r.right_bottom(), h * 0.14, WHITE);
            diag(p, r.right_top(), r.left_bottom(), h * 0.14, WHITE);
            if detail {
                dot(p, r, 0.5, 0.5, 0.25, WHITE);
                for (x, y) in [(0.5, 0.4), (0.43, 0.55), (0.57, 0.55)] {
                    star(p, r, x, y, 0.06, red);
                }
            }
        }
        "BJ" => {
            fill(p, part(r, 0.0, 0.0, 0.4, 1.0), rgb(0, 135, 81));
            fill(p, part(r, 0.4, 0.0, 1.0, 0.5), rgb(252, 209, 22));
            fill(p, part(r, 0.4, 0.5, 1.0, 1.0), rgb(232, 17, 45));
        }
        "BN" => {
            fill(p, r, rgb(247, 224, 23));
            diag(p, fr(r, 0.0, 0.2), fr(r, 1.0, 0.7), h * 0.14, WHITE);
            diag(p, fr(r, 0.0, 0.33), fr(r, 1.0, 0.83), h * 0.1, BLACK);
            if detail {
                dot(p, r, 0.5, 0.5, 0.13, rgb(207, 8, 33));
            }
        }
        "BO" => hbands(p, r, &[(1.0, rgb(217, 16, 35)), (1.0, rgb(247, 224, 0)), (1.0, rgb(0, 122, 61))]),
        "BR" => {
            fill(p, r, rgb(0, 155, 58));
            poly(p, r, &[(0.5, 0.1), (0.9, 0.5), (0.5, 0.9), (0.1, 0.5)], rgb(254, 223, 0));
            dot(p, r, 0.5, 0.5, 0.24, rgb(0, 39, 118));
            if detail {
                let band = vec![fr(r, 0.35, 0.52), fr(r, 0.5, 0.48), fr(r, 0.65, 0.44)];
                p.add(Shape::line(band, Stroke::new(h * 0.05, WHITE)));
            }
        }
        "BS" => {
            hbands(p, r, &[(1.0, rgb(0, 171, 201)), (1.0, rgb(255, 199, 44)), (1.0, rgb(0, 171, 201))]);
            poly(p, r, &[(0.0, 0.0), (0.45, 0.5), (0.0, 1.0)], BLACK);
        }
        "BT" => {
            poly(p, r, &[(0.0, 0.0), (1.0, 0.0), (0.0, 1.0)], rgb(255, 205, 0));
            poly(p, r, &[(1.0, 0.0), (1.0, 1.0), (0.0, 1.0)], rgb(255, 103, 31));
            if detail {
                dot(p, r, 0.5, 0.5, 0.14, WHITE);
            }
        }
        "BW" => hbands(
            p,
            r,
            &[(37.0, rgb(117, 170, 219)), (6.0, WHITE), (14.0, BLACK), (6.0, WHITE), (37.0, rgb(117, 170, 219))],
        ),
        "BZ" => {
            fill(p, r, rgb(0, 82, 156));
            fill(p, part(r, 0.0, 0.0, 1.0, 0.1), rgb(206, 17, 38));
            fill(p, part(r, 0.0, 0.9, 1.0, 1.0), rgb(206, 17, 38));
            dot(p, r, 0.5, 0.5, 0.33, WHITE);
            if detail {
                ring(p, r, 0.5, 0.5, 0.29, 1.5, rgb(0, 122, 61));
            }
        }
        "CA" => {
            let red = rgb(216, 6, 33);
            vbands(p, r, &[(1.0, red), (2.0, WHITE), (1.0, red)]);
            if detail {
                let c = fr(r, 0.5, 0.5);
                let outline: Vec<Pos2> = MAPLE.iter().map(|&(x, y)| c + Vec2::new(x, y) * (h * 0.42)).collect();
                fan(p, c, &outline, red);
            }
        }
        "CD" => {
            fill(p, r, rgb(0, 127, 255));
            diag(p, r.left_bottom(), r.right_top(), h * 0.34, rgb(247, 214, 24));
            diag(p, r.left_bottom(), r.right_top(), h * 0.22, rgb(206, 16, 33));
            if detail {
                star(p, r, 0.17, 0.2, 0.16, rgb(247, 214, 24));
            }
        }
        "CF" => {
            hbands(p, r, &[(1.0, rgb(0, 40, 104)), (1.0, WHITE), (1.0, rgb(0, 154, 68)), (1.0, rgb(252, 209, 22))]);
            fill(p, part(r, 0.43, 0.0, 0.57, 1.0), rgb(206, 17, 38));
            if detail {
                star(p, r, 0.2, 0.12, 0.1, rgb(252, 209, 22));
            }
        }
        "CG" => {
            poly(p, r, &[(0.0, 0.0), (0.66, 0.0), (0.0, 1.0)], rgb(0, 122, 61));
            poly(p, r, &[(0.66, 0.0), (1.0, 0.0), (0.34, 1.0), (0.0, 1.0)], rgb(251, 222, 74));
            poly(p, r, &[(1.0, 0.0), (1.0, 1.0), (0.34, 1.0)], rgb(206, 17, 38));
        }
        "CI" => vbands(p, r, &[(1.0, rgb(247, 127, 0)), (1.0, WHITE), (1.0, rgb(0, 154, 68))]),
        "CL" => {
            hbands(p, r, &[(1.0, WHITE), (1.0, rgb(213, 43, 30))]);
            fill(p, part(r, 0.0, 0.0, 0.33, 0.5), rgb(0, 57, 166));
            if detail {
                star(p, r, 0.165, 0.25, 0.12, WHITE);
            }
        }
        "CM" => {
            vbands(p, r, &[(1.0, rgb(0, 122, 61)), (1.0, rgb(206, 17, 38)), (1.0, rgb(252, 209, 22))]);
            if detail {
                star(p, r, 0.5, 0.5, 0.16, rgb(252, 209, 22));
            }
        }
        "CN" => {
            let gold = rgb(255, 222, 0);
            fill(p, r, rgb(238, 28, 37));
            if detail {
                star(p, r, 0.17, 0.27, 0.17, gold);
                for (x, y) in [(0.33, 0.1), (0.4, 0.2), (0.4, 0.35), (0.33, 0.45)] {
                    star(p, r, x, y, 0.05, gold);
                }
            }
        }
        "CO" => hbands(p, r, &[(2.0, rgb(252, 209, 22)), (1.0, rgb(0, 56, 147)), (1.0, rgb(206, 17, 38))]),
        "CR" => hbands(
            p,
            r,
            &[(1.0, rgb(0, 43, 127)), (1.0, WHITE), (2.0, rgb(206, 17, 38)), (1.0, WHITE), (1.0, rgb(0, 43, 127))],
        ),
        "CU" => {
            for i in 0..5 {
                let c = if i % 2 == 0 { rgb(0, 42, 143) } else { WHITE };
                fill(p, part(r, 0.0, i as f32 / 5.0, 1.0, (i as f32 + 1.0) / 5.0), c);
            }
            poly(p, r, &[(0.0, 0.0), (0.42, 0.5), (0.0, 1.0)], rgb(207, 20, 43));
            if detail {
                star(p, r, 0.13, 0.5, 0.14, WHITE);
            }
        }
        "CV" => {
            let blue = rgb(0, 56, 147);
            hbands(p, r, &[(8.0, blue), (1.0, WHITE), (2.0, rgb(207, 32, 46)), (1.0, WHITE), (4.0, blue)]);
            if detail {
                for k in 0..10 {
                    let c = fr(r, 0.35, 0.5) + Vec2::angled(k as f32 * TAU / 10.0) * (h * 0.27);
                    star_px(p, c, h * 0.04, 5, 0.382, rgb(255, 205, 0));
                }
            }
        }
        "DJ" => {
            hbands(p, r, &[(1.0, rgb(106, 178, 231)), (1.0, rgb(18, 173, 43))]);
            poly(p, r, &[(0.0, 0.0), (0.5, 0.5), (0.0, 1.0)], WHITE);
            if detail {
                star(p, r, 0.13, 0.5, 0.1, rgb(215, 20, 26));
            }
        }
        "DM" => {
            fill(p, r, rgb(0, 106, 76));
            cross(p, r, 0.5, h * 0.3, rgb(252, 209, 22));
            cross(p, r, 0.5, h * 0.2, BLACK);
            cross(p, r, 0.5, h * 0.1, WHITE);
            if detail {
                dot(p, r, 0.5, 0.5, 0.2, rgb(210, 16, 52));
            }
        }
        "DO" => {
            let (blue, red) = (rgb(0, 45, 98), rgb(206, 17, 38));
            fill(p, part(r, 0.0, 0.0, 0.5, 0.5), blue);
            fill(p, part(r, 0.5, 0.0, 1.0, 0.5), red);
            fill(p, part(r, 0.0, 0.5, 0.5, 1.0), red);
            fill(p, part(r, 0.5, 0.5, 1.0, 1.0), blue);
            cross(p, r, 0.5, h * 0.2, WHITE);
        }
        "DZ" => {
            vbands(p, r, &[(1.0, rgb(0, 102, 51)), (1.0, WHITE)]);
            if detail {
                let red = rgb(210, 16, 52);
                crescent(p, r, 0.48, 0.5, 0.27, Vec2::new(1.0, 0.0), red);
                star(p, r, 0.55, 0.5, 0.1, red);
            }
        }
        "EC" => {
            hbands(p, r, &[(2.0, rgb(255, 209, 0)), (1.0, rgb(0, 56, 168)), (1.0, rgb(206, 17, 38))]);
            if detail {
                dot(p, r, 0.5, 0.5, 0.1, rgb(140, 120, 90));
            }
        }
        "EG" => {
            hbands(p, r, &[(1.0, rgb(206, 17, 38)), (1.0, WHITE), (1.0, BLACK)]);
            if detail {
                dot(p, r, 0.5, 0.5, 0.09, rgb(193, 150, 40));
            }
        }
        "ER" => {
            fill(p, part(r, 0.0, 0.0, 1.0, 0.5), rgb(18, 173, 43));
            fill(p, part(r, 0.0, 0.5, 1.0, 1.0), rgb(65, 143, 205));
            poly(p, r, &[(0.0, 0.0), (1.0, 0.5), (0.0, 1.0)], rgb(234, 18, 38));
            if detail {
                ring(p, r, 0.2, 0.5, 0.1, 1.5, rgb(255, 199, 44));
            }
        }
        "ET" => {
            hbands(p, r, &[(1.0, rgb(7, 137, 63)), (1.0, rgb(252, 221, 9)), (1.0, rgb(218, 18, 26))]);
            if detail {
                dot(p, r, 0.5, 0.5, 0.2, rgb(15, 71, 175));
                star(p, r, 0.5, 0.5, 0.15, rgb(252, 221, 9));
            }
        }
        "FJ" => {
            fill(p, r, rgb(104, 191, 229));
            union_jack(p, part(r, 0.0, 0.0, 0.5, 0.5), detail);
            if detail {
                fill(p, part(r, 0.6, 0.2, 0.85, 0.62), WHITE);
                fill(p, part(r, 0.6, 0.2, 0.85, 0.34), rgb(206, 17, 38));
            }
        }
        "FM" => {
            fill(p, r, rgb(117, 178, 221));
            if detail {
                for (x, y) in [(0.5, 0.2), (0.5, 0.8), (0.37, 0.5), (0.63, 0.5)] {
                    star(p, r, x, y, 0.1, WHITE);
                }
            }
        }
        "GA" => hbands(p, r, &[(1.0, rgb(0, 158, 96)), (1.0, rgb(252, 209, 22)), (1.0, rgb(58, 117, 196))]),
        "GD" => {
            let (red, yellow, green) = (rgb(206, 17, 38), rgb(252, 209, 22), rgb(0, 122, 94));
            fill(p, r, red);
            let inner = r.shrink(h * 0.1);
            poly(p, inner, &[(0.0, 0.0), (1.0, 0.0), (0.5, 0.5)], yellow);
            poly(p, inner, &[(0.0, 1.0), (1.0, 1.0), (0.5, 0.5)], yellow);
            poly(p, inner, &[(0.0, 0.0), (0.5, 0.5), (0.0, 1.0)], green);
            poly(p, inner, &[(1.0, 0.0), (0.5, 0.5), (1.0, 1.0)], green);
            if detail {
                dot(p, r, 0.5, 0.5, 0.17, red);
                star(p, r, 0.5, 0.5, 0.13, yellow);
                for x in [0.3, 0.5, 0.7] {
                    star(p, r, x, 0.05, 0.04, yellow);
                    star(p, r, x, 0.95, 0.04, yellow);
                }
            }
        }
        "GE" => {
            let red = rgb(237, 28, 36);
            fill(p, r, WHITE);
            cross(p, r, 0.5, h * 0.2, red);
            if detail {
                for (x, y) in [(0.25, 0.25), (0.75, 0.25), (0.25, 0.75), (0.75, 0.75)] {
                    cross(p, Rect::from_center_size(fr(r, x, y), Vec2::splat(h * 0.22)), 0.5, h * 0.06, red);
                }
            }
        }
        "GH" => {
            hbands(p, r, &[(1.0, rgb(206, 17, 38)), (1.0, rgb(252, 209, 22)), (1.0, rgb(0, 107, 63))]);
            if detail {
                star(p, r, 0.5, 0.5, 0.17, BLACK);
            }
        }
        "GM" => hbands(
            p,
            r,
            &[(6.0, rgb(206, 17, 38)), (1.0, WHITE), (4.0, rgb(12, 28, 140)), (1.0, WHITE), (6.0, rgb(58, 170, 53))],
        ),
        "GN" => vbands(p, r, &[(1.0, rgb(206, 17, 38)), (1.0, rgb(252, 209, 22)), (1.0, rgb(0, 148, 96))]),
        "GQ" => {
            hbands(p, r, &[(1.0, rgb(62, 154, 0)), (1.0, WHITE), (1.0, rgb(232, 17, 45))]);
            poly(p, r, &[(0.0, 0.0), (0.36, 0.5), (0.0, 1.0)], rgb(0, 115, 206));
        }
        "GT" => {
            vbands(p, r, &[(1.0, rgb(73, 151, 208)), (1.0, WHITE), (1.0, rgb(73, 151, 208))]);
            if detail {
                ring(p, r, 0.5, 0.5, 0.1, 1.5, rgb(0, 122, 61));
            }
        }
        "GW" => {
            fill(p, part(r, 0.0, 0.0, 0.33, 1.0), rgb(206, 17, 38));
            fill(p, part(r, 0.33, 0.0, 1.0, 0.5), rgb(252, 209, 22));
            fill(p, part(r, 0.33, 0.5, 1.0, 1.0), rgb(0, 155, 72));
            if detail {
                star(p, r, 0.165, 0.5, 0.15, BLACK);
            }
        }
        "GY" => {
            fill(p, r, rgb(0, 158, 73));
            poly(p, r, &[(0.0, 0.0), (1.0, 0.5), (0.0, 1.0)], WHITE);
            poly(p, r, &[(0.0, 0.05), (0.94, 0.5), (0.0, 0.95)], rgb(252, 209, 22));
            poly(p, r, &[(0.0, 0.0), (0.4, 0.5), (0.0, 1.0)], rgb(206, 17, 38));
            poly(p, r, &[(0.0, 0.07), (0.32, 0.5), (0.0, 0.93)], BLACK);
        }
        "HN" => {
            let blue = rgb(0, 188, 212);
            hbands(p, r, &[(1.0, blue), (1.0, WHITE), (1.0, blue)]);
            if detail {
                for (x, y) in [(0.4, 0.43), (0.6, 0.43), (0.5, 0.5), (0.4, 0.57), (0.6, 0.57)] {
                    star(p, r, x, y, 0.05, blue);
                }
            }
        }
        "HT" => {
            hbands(p, r, &[(1.0, rgb(0, 32, 159)), (1.0, rgb(210, 16, 52))]);
            if detail {
                fill(p, part(r, 0.38, 0.3, 0.62, 0.7), WHITE);
                dot(p, r, 0.5, 0.5, 0.07, rgb(0, 150, 60));
            }
        }
        "ID" => hbands(p, r, &[(1.0, rgb(206, 17, 38)), (1.0, WHITE)]),
        "IL" => {
            let blue = rgb(0, 56, 184);
            fill(p, r, WHITE);
            fill(p, part(r, 0.0, 0.09, 1.0, 0.2), blue);
            fill(p, part(r, 0.0, 0.8, 1.0, 0.91), blue);
            if detail {
                let (c, big) = (fr(r, 0.5, 0.5), h * 0.17);
                for s in [-1.0_f32, 1.0] {
                    let tri: Vec<Pos2> =
                        (0..3).map(|k| c + Vec2::angled(-FRAC_PI_2 * s + k as f32 * TAU / 3.0) * big).collect();
                    p.add(Shape::closed_line(tri, Stroke::new(h * 0.045, blue)));
                }
            }
        }
        "IN" => {
            hbands(p, r, &[(1.0, rgb(255, 153, 51)), (1.0, WHITE), (1.0, rgb(19, 136, 8))]);
            if detail {
                wheel(p, r, 0.5, 0.5, 0.1, 12, rgb(0, 0, 128));
            }
        }
        "IQ" => {
            hbands(p, r, &[(1.0, rgb(206, 17, 38)), (1.0, WHITE), (1.0, BLACK)]);
            if detail {
                fill(p, part(r, 0.4, 0.44, 0.6, 0.56), rgb(0, 122, 61));
            }
        }
        "IR" => {
            hbands(p, r, &[(1.0, rgb(35, 159, 64)), (1.0, WHITE), (1.0, rgb(218, 0, 0))]);
            if detail {
                ring(p, r, 0.5, 0.5, 0.1, 1.5, rgb(218, 0, 0));
            }
        }
        "JM" => {
            let gold = rgb(254, 209, 0);
            fill(p, r, rgb(0, 155, 58));
            poly(p, r, &[(0.0, 0.0), (0.5, 0.5), (0.0, 1.0)], BLACK);
            poly(p, r, &[(1.0, 0.0), (0.5, 0.5), (1.0, 1.0)], BLACK);
            diag(p, r.left_top(), r.right_bottom(), h * 0.2, gold);
            diag(p, r.right_top(), r.left_bottom(), h * 0.2, gold);
        }
        "JO" => {
            hbands(p, r, &[(1.0, BLACK), (1.0, WHITE), (1.0, rgb(0, 122, 61))]);
            poly(p, r, &[(0.0, 0.0), (0.42, 0.5), (0.0, 1.0)], rgb(206, 17, 38));
            if detail {
                star_n(p, r, 0.12, 0.5, 0.07, 7, 0.45, WHITE);
            }
        }
        "JP" => {
            fill(p, r, WHITE);
            dot(p, r, 0.5, 0.5, 0.3, rgb(188, 0, 45));
        }
        "KE" => {
            hbands(
                p,
                r,
                &[(3.0, BLACK), (1.0, WHITE), (3.0, rgb(187, 10, 30)), (1.0, WHITE), (3.0, rgb(0, 102, 0))],
            );
            if detail {
                dot(p, r, 0.5, 0.5, 0.22, BLACK);
                dot(p, r, 0.5, 0.5, 0.15, rgb(187, 10, 30));
            }
        }
        "KG" => {
            fill(p, r, rgb(227, 30, 36));
            if detail {
                dot(p, r, 0.5, 0.5, 0.27, rgb(255, 239, 0));
                dot(p, r, 0.5, 0.5, 0.18, rgb(227, 30, 36));
            }
        }
        "KH" => {
            hbands(p, r, &[(1.0, rgb(3, 46, 161)), (2.0, rgb(224, 0, 37)), (1.0, rgb(3, 46, 161))]);
            if detail {
                fill(p, part(r, 0.38, 0.55, 0.62, 0.62), WHITE);
                fill(p, part(r, 0.4, 0.42, 0.46, 0.55), WHITE);
                fill(p, part(r, 0.47, 0.34, 0.53, 0.55), WHITE);
                fill(p, part(r, 0.54, 0.42, 0.6, 0.55), WHITE);
            }
        }
        "KI" => {
            fill(p, r, rgb(0, 40, 104));
            fill(p, part(r, 0.0, 0.0, 1.0, 0.55), rgb(206, 17, 38));
            for y in [0.64, 0.78, 0.92] {
                fill(p, part(r, 0.0, y, 1.0, y + 0.06), WHITE);
            }
            if detail {
                let top = p.with_clip_rect(p.clip_rect().intersect(part(r, 0.0, 0.0, 1.0, 0.55)));
                dot(&top, r, 0.5, 0.55, 0.2, rgb(252, 209, 22));
            }
        }
        "KM" => {
            hbands(
                p,
                r,
                &[(1.0, rgb(255, 195, 0)), (1.0, WHITE), (1.0, rgb(206, 17, 38)), (1.0, rgb(59, 93, 190))],
            );
            poly(p, r, &[(0.0, 0.0), (0.5, 0.5), (0.0, 1.0)], rgb(61, 162, 36));
            if detail {
                crescent(p, r, 0.14, 0.5, 0.17, Vec2::new(1.0, 0.0), WHITE);
            }
        }
        "KN" => {
            poly(p, r, &[(0.0, 0.0), (1.0, 0.0), (0.0, 1.0)], rgb(0, 166, 81));
            poly(p, r, &[(1.0, 0.0), (1.0, 1.0), (0.0, 1.0)], rgb(206, 17, 38));
            diag(p, r.left_bottom(), r.right_top(), h * 0.38, rgb(252, 209, 22));
            diag(p, r.left_bottom(), r.right_top(), h * 0.26, BLACK);
            if detail {
                star(p, r, 0.36, 0.62, 0.09, WHITE);
                star(p, r, 0.64, 0.38, 0.09, WHITE);
            }
        }
        "KP" => {
            let (blue, red) = (rgb(2, 81, 162), rgb(237, 28, 36));
            hbands(p, r, &[(2.0, blue), (0.35, WHITE), (5.3, red), (0.35, WHITE), (2.0, blue)]);
            if detail {
                dot(p, r, 0.33, 0.5, 0.27, WHITE);
                star(p, r, 0.33, 0.5, 0.22, red);
            }
        }
        "KR" => {
            fill(p, r, WHITE);
            if detail {
                let (red, blue) = (rgb(205, 46, 58), rgb(0, 71, 160));
                let (c, big) = (fr(r, 0.5, 0.5), h * 0.25);
                p.circle_filled(c, big, red);
                let low = p.with_clip_rect(p.clip_rect().intersect(Rect::from_min_max(pos2(r.min.x, c.y), r.max)));
                low.circle_filled(c, big, blue);
                p.circle_filled(c + Vec2::new(-big / 2.0, 0.0), big / 2.0, red);
                p.circle_filled(c + Vec2::new(big / 2.0, 0.0), big / 2.0, blue);
                // Simplified trigrams: three bars in each corner, facing the centre.
                for (sx, sy) in [(-1.0_f32, -1.0_f32), (1.0, -1.0), (-1.0, 1.0), (1.0, 1.0)] {
                    let q = c + Vec2::new(sx * h * 0.62, sy * h * 0.38);
                    let d = (c - q).normalized();
                    let n = Vec2::new(-d.y, d.x);
                    for j in -1..=1 {
                        let m = q + d * (j as f32 * h * 0.07);
                        p.line_segment([m - n * h * 0.1, m + n * h * 0.1], Stroke::new(h * 0.04, BLACK));
                    }
                }
            }
        }
        "KW" => {
            hbands(p, r, &[(1.0, rgb(0, 122, 61)), (1.0, WHITE), (1.0, rgb(206, 17, 38))]);
            poly(p, r, &[(0.0, 0.0), (0.28, 0.33), (0.28, 0.67), (0.0, 1.0)], BLACK);
        }
        "KZ" => {
            fill(p, r, rgb(0, 175, 202));
            if detail {
                let gold = rgb(254, 202, 0);
                star_n(p, r, 0.52, 0.4, 0.2, 16, 0.65, gold);
                fill(p, part(r, 0.05, 0.1, 0.09, 0.9), gold);
                fill(p, part(r, 0.34, 0.72, 0.7, 0.77), gold);
            }
        }
        "LA" => {
            hbands(p, r, &[(1.0, rgb(206, 17, 38)), (2.0, rgb(0, 40, 104)), (1.0, rgb(206, 17, 38))]);
            if detail {
                dot(p, r, 0.5, 0.5, 0.2, WHITE);
            }
        }
        "LB" => {
            hbands(p, r, &[(1.0, rgb(237, 28, 36)), (2.0, WHITE), (1.0, rgb(237, 28, 36))]);
            if detail {
                let green = rgb(0, 150, 60);
                poly(p, r, &[(0.5, 0.26), (0.4, 0.46), (0.6, 0.46)], green);
                poly(p, r, &[(0.5, 0.38), (0.37, 0.62), (0.63, 0.62)], green);
                fill(p, part(r, 0.485, 0.62, 0.515, 0.72), rgb(120, 70, 30));
            }
        }
        "LC" => {
            fill(p, r, rgb(102, 204, 255));
            poly(p, r, &[(0.5, 0.1), (0.32, 0.88), (0.68, 0.88)], WHITE);
            poly(p, r, &[(0.5, 0.2), (0.37, 0.84), (0.63, 0.84)], BLACK);
            poly(p, r, &[(0.5, 0.52), (0.39, 0.84), (0.61, 0.84)], rgb(255, 209, 0));
        }
        "LK" => {
            let gold = rgb(255, 190, 41);
            fill(p, r, gold);
            fill(p, part(r, 0.04, 0.06, 0.145, 0.94), rgb(0, 106, 78));
            fill(p, part(r, 0.145, 0.06, 0.25, 0.94), rgb(235, 116, 0));
            fill(p, part(r, 0.27, 0.06, 0.96, 0.94), rgb(141, 21, 58));
            if detail {
                dot(p, r, 0.6, 0.5, 0.17, gold);
            }
        }
        "LR" => {
            for i in 0..11 {
                let c = if i % 2 == 0 { rgb(191, 10, 48) } else { WHITE };
                fill(p, part(r, 0.0, i as f32 / 11.0, 1.0, (i as f32 + 1.0) / 11.0), c);
            }
            fill(p, part(r, 0.0, 0.0, 0.303, 5.0 / 11.0), rgb(0, 40, 104));
            if detail {
                star(p, r, 0.1515, 5.0 / 22.0, 0.12, WHITE);
            }
        }
        "LS" => {
            hbands(p, r, &[(3.0, rgb(0, 32, 159)), (4.0, WHITE), (3.0, rgb(0, 155, 58))]);
            if detail {
                poly(p, r, &[(0.5, 0.35), (0.43, 0.65), (0.57, 0.65)], BLACK);
            }
        }
        "LY" => {
            hbands(p, r, &[(1.0, rgb(231, 0, 19)), (2.0, BLACK), (1.0, rgb(35, 158, 53))]);
            if detail {
                crescent(p, r, 0.47, 0.5, 0.17, Vec2::new(1.0, 0.0), WHITE);
                star(p, r, 0.54, 0.5, 0.06, WHITE);
            }
        }
        "MA" => {
            fill(p, r, rgb(193, 39, 45));
            if detail {
                pentagram(p, r, 0.5, 0.5, 0.27, h * 0.045, rgb(0, 98, 51));
            }
        }
        "MG" => {
            fill(p, part(r, 0.0, 0.0, 0.33, 1.0), WHITE);
            fill(p, part(r, 0.33, 0.0, 1.0, 0.5), rgb(252, 61, 50));
            fill(p, part(r, 0.33, 0.5, 1.0, 1.0), rgb(0, 126, 58));
        }
        "MH" => {
            fill(p, r, rgb(0, 57, 166));
            poly(p, r, &[(0.0, 1.0), (1.0, 0.0), (1.0, 0.28)], rgb(221, 117, 0));
            poly(p, r, &[(0.0, 1.0), (1.0, 0.28), (1.0, 0.56)], WHITE);
            if detail {
                star_n(p, r, 0.18, 0.2, 0.15, 12, 0.6, WHITE);
            }
        }
        "ML" => vbands(p, r, &[(1.0, rgb(20, 181, 58)), (1.0, rgb(252, 209, 22)), (1.0, rgb(206, 17, 38))]),
        "MM" => {
            hbands(p, r, &[(1.0, rgb(254, 203, 0)), (1.0, rgb(52, 178, 51)), (1.0, rgb(234, 40, 57))]);
            if detail {
                star(p, r, 0.5, 0.54, 0.33, WHITE);
            }
        }
        "MN" => {
            vbands(p, r, &[(1.0, rgb(218, 32, 50)), (1.0, rgb(0, 102, 177)), (1.0, rgb(218, 32, 50))]);
            if detail {
                let gold = rgb(255, 217, 0);
                dot(p, r, 0.165, 0.28, 0.06, gold);
                fill(p, part(r, 0.15, 0.4, 0.18, 0.78), gold);
                fill(p, part(r, 0.11, 0.42, 0.14, 0.7), gold);
                fill(p, part(r, 0.19, 0.42, 0.22, 0.7), gold);
            }
        }
        "MR" => {
            fill(p, r, rgb(0, 122, 61));
            fill(p, part(r, 0.0, 0.0, 1.0, 0.14), rgb(210, 16, 52));
            fill(p, part(r, 0.0, 0.86, 1.0, 1.0), rgb(210, 16, 52));
            if detail {
                crescent(p, r, 0.5, 0.45, 0.27, Vec2::new(0.0, -1.0), rgb(255, 194, 0));
                star(p, r, 0.5, 0.33, 0.08, rgb(255, 194, 0));
            }
        }
        "MU" => hbands(
            p,
            r,
            &[(1.0, rgb(234, 40, 57)), (1.0, rgb(26, 32, 102)), (1.0, rgb(255, 213, 0)), (1.0, rgb(0, 166, 81))],
        ),
        "MV" => {
            fill(p, r, rgb(210, 16, 52));
            fill(p, part(r, 0.2, 0.25, 0.8, 0.75), rgb(0, 122, 61));
            if detail {
                crescent(p, r, 0.5, 0.5, 0.2, Vec2::new(1.0, 0.0), WHITE);
            }
        }
        "MW" => {
            hbands(p, r, &[(1.0, BLACK), (1.0, rgb(206, 17, 38)), (1.0, rgb(0, 153, 70))]);
            if detail {
                let top = p.with_clip_rect(p.clip_rect().intersect(part(r, 0.0, 0.0, 1.0, 1.0 / 3.0)));
                dot(&top, r, 0.5, 1.0 / 3.0, 0.2, rgb(206, 17, 38));
            }
        }
        "MX" => {
            vbands(p, r, &[(1.0, rgb(0, 104, 71)), (1.0, WHITE), (1.0, rgb(206, 17, 38))]);
            if detail {
                dot(p, r, 0.5, 0.5, 0.14, rgb(150, 110, 60));
                ring(p, r, 0.5, 0.5, 0.17, 1.2, rgb(0, 104, 71));
            }
        }
        "MY" => {
            for i in 0..14 {
                let c = if i % 2 == 0 { rgb(204, 0, 0) } else { WHITE };
                fill(p, part(r, 0.0, i as f32 / 14.0, 1.0, (i as f32 + 1.0) / 14.0), c);
            }
            fill(p, part(r, 0.0, 0.0, 0.5, 8.0 / 14.0), rgb(0, 0, 102));
            if detail {
                let gold = rgb(255, 204, 0);
                crescent(p, r, 0.19, 0.285, 0.17, Vec2::new(1.0, 0.0), gold);
                star_n(p, r, 0.3, 0.285, 0.1, 14, 0.5, gold);
            }
        }
        "MZ" => {
            hbands(
                p,
                r,
                &[(3.0, rgb(0, 122, 61)), (0.4, WHITE), (3.0, BLACK), (0.4, WHITE), (3.0, rgb(252, 209, 22))],
            );
            poly(p, r, &[(0.0, 0.0), (0.5, 0.5), (0.0, 1.0)], rgb(206, 17, 38));
            if detail {
                star(p, r, 0.14, 0.5, 0.14, rgb(252, 209, 22));
            }
        }
        "NA" => {
            poly(p, r, &[(0.0, 0.0), (1.0, 0.0), (0.0, 1.0)], rgb(0, 56, 147));
            poly(p, r, &[(1.0, 0.0), (1.0, 1.0), (0.0, 1.0)], rgb(0, 158, 73));
            diag(p, r.left_bottom(), r.right_top(), h * 0.38, WHITE);
            diag(p, r.left_bottom(), r.right_top(), h * 0.26, rgb(206, 17, 38));
            if detail {
                star_n(p, r, 0.16, 0.2, 0.13, 12, 0.6, rgb(255, 205, 0));
            }
        }
        "NE" => {
            hbands(p, r, &[(1.0, rgb(224, 82, 6)), (1.0, WHITE), (1.0, rgb(13, 141, 57))]);
            if detail {
                dot(p, r, 0.5, 0.5, 0.14, rgb(224, 82, 6));
            }
        }
        "NI" => {
            hbands(p, r, &[(1.0, rgb(0, 103, 198)), (1.0, WHITE), (1.0, rgb(0, 103, 198))]);
            if detail {
                poly(p, r, &[(0.5, 0.38), (0.42, 0.6), (0.58, 0.6)], rgb(180, 160, 40));
            }
        }
        "NP" => {
            let (blue, red) = (rgb(0, 56, 147), rgb(220, 20, 60));
            fill(p, r, WHITE);
            poly(p, r, &[(0.0, 0.0), (0.72, 0.48), (0.0, 0.48)], blue);
            poly(p, r, &[(0.0, 0.48), (0.3, 0.48), (0.72, 1.0), (0.0, 1.0)], blue);
            poly(p, r, &[(0.03, 0.06), (0.58, 0.45), (0.03, 0.45)], red);
            poly(p, r, &[(0.03, 0.52), (0.3, 0.52), (0.58, 0.93), (0.03, 0.93)], red);
            if detail {
                dot(p, r, 0.11, 0.32, 0.05, WHITE);
                dot(p, r, 0.11, 0.75, 0.07, WHITE);
            }
        }
        "NR" => {
            fill(p, r, rgb(0, 40, 104));
            fill(p, part(r, 0.0, 0.47, 1.0, 0.53), rgb(255, 198, 30));
            if detail {
                star_n(p, r, 0.22, 0.75, 0.12, 12, 0.55, WHITE);
            }
        }
        "NZ" => {
            fill(p, r, rgb(0, 36, 125));
            union_jack(p, part(r, 0.0, 0.0, 0.5, 0.5), detail);
            if detail {
                for (x, y, s) in [(0.75, 0.2, 0.07), (0.62, 0.45, 0.07), (0.88, 0.4, 0.07), (0.75, 0.82, 0.08)] {
                    star(p, r, x, y, s + 0.02, WHITE);
                    star(p, r, x, y, s, rgb(204, 20, 43));
                }
            }
        }
        "OM" => {
            hbands(p, r, &[(1.0, WHITE), (1.0, rgb(219, 22, 46)), (1.0, rgb(0, 121, 52))]);
            fill(p, part(r, 0.0, 0.0, 0.25, 1.0), rgb(219, 22, 46));
            if detail {
                dot(p, r, 0.125, 0.17, 0.06, WHITE);
            }
        }
        "PA" => {
            let (red, blue) = (rgb(210, 16, 52), rgb(0, 82, 147));
            fill(p, part(r, 0.0, 0.0, 0.5, 0.5), WHITE);
            fill(p, part(r, 0.5, 0.0, 1.0, 0.5), red);
            fill(p, part(r, 0.0, 0.5, 0.5, 1.0), blue);
            fill(p, part(r, 0.5, 0.5, 1.0, 1.0), WHITE);
            if detail {
                star(p, r, 0.25, 0.25, 0.14, blue);
                star(p, r, 0.75, 0.75, 0.14, red);
            }
        }
        "PE" => vbands(p, r, &[(1.0, rgb(217, 16, 35)), (1.0, WHITE), (1.0, rgb(217, 16, 35))]),
        "PG" => {
            poly(p, r, &[(0.0, 0.0), (1.0, 0.0), (1.0, 1.0)], rgb(206, 17, 38));
            poly(p, r, &[(0.0, 0.0), (1.0, 1.0), (0.0, 1.0)], BLACK);
            if detail {
                dot(p, r, 0.72, 0.28, 0.1, rgb(252, 209, 22));
                for (x, y) in [(0.2, 0.55), (0.3, 0.72), (0.13, 0.78), (0.22, 0.88), (0.15, 0.65)] {
                    star(p, r, x, y, 0.06, WHITE);
                }
            }
        }
        "PH" => {
            hbands(p, r, &[(1.0, rgb(0, 56, 168)), (1.0, rgb(206, 17, 38))]);
            poly(p, r, &[(0.0, 0.0), (0.5, 0.5), (0.0, 1.0)], WHITE);
            if detail {
                let gold = rgb(252, 209, 22);
                dot(p, r, 0.14, 0.5, 0.07, gold);
                for (x, y) in [(0.04, 0.12), (0.04, 0.88), (0.38, 0.5)] {
                    star(p, r, x, y, 0.06, gold);
                }
            }
        }
        "PK" => {
            fill(p, r, rgb(1, 65, 28));
            fill(p, part(r, 0.0, 0.0, 0.25, 1.0), WHITE);
            if detail {
                crescent(p, r, 0.61, 0.52, 0.26, Vec2::new(0.8, -0.6), WHITE);
                star(p, r, 0.73, 0.35, 0.08, WHITE);
            }
        }
        "PS" => {
            hbands(p, r, &[(1.0, BLACK), (1.0, WHITE), (1.0, rgb(0, 122, 61))]);
            poly(p, r, &[(0.0, 0.0), (0.3, 0.5), (0.0, 1.0)], rgb(206, 17, 38));
        }
        "PW" => {
            fill(p, r, rgb(74, 173, 214));
            dot(p, r, 0.42, 0.5, 0.31, rgb(255, 222, 0));
        }
        "PY" => {
            hbands(p, r, &[(1.0, rgb(213, 43, 30)), (1.0, WHITE), (1.0, rgb(0, 56, 168))]);
            if detail {
                ring(p, r, 0.5, 0.5, 0.1, 1.2, rgb(120, 120, 120));
            }
        }
        "QA" => {
            fill(p, r, rgb(138, 21, 56));
            fill(p, part(r, 0.0, 0.0, 0.27, 1.0), WHITE);
            for i in 0..9 {
                let (y0, y1) = (i as f32 / 9.0, (i as f32 + 1.0) / 9.0);
                poly(p, r, &[(0.27, y0), (0.4, (y0 + y1) / 2.0), (0.27, y1)], WHITE);
            }
        }
        "RW" => {
            hbands(p, r, &[(2.0, rgb(0, 163, 224)), (1.0, rgb(250, 210, 1)), (1.0, rgb(32, 96, 61))]);
            if detail {
                star_n(p, r, 0.82, 0.22, 0.12, 12, 0.5, rgb(250, 210, 1));
            }
        }
        "SA" => {
            fill(p, r, rgb(0, 108, 53));
            if detail {
                fill(p, part(r, 0.2, 0.3, 0.8, 0.4), WHITE);
                fill(p, part(r, 0.2, 0.62, 0.8, 0.66), WHITE);
            }
        }
        "SB" => {
            poly(p, r, &[(0.0, 0.0), (1.0, 0.0), (0.0, 1.0)], rgb(0, 81, 162));
            poly(p, r, &[(1.0, 0.0), (1.0, 1.0), (0.0, 1.0)], rgb(33, 111, 75));
            diag(p, r.left_bottom(), r.right_top(), h * 0.1, rgb(252, 209, 22));
            if detail {
                for (x, y) in [(0.07, 0.1), (0.21, 0.1), (0.14, 0.2), (0.07, 0.3), (0.21, 0.3)] {
                    star(p, r, x, y, 0.06, WHITE);
                }
            }
        }
        "SC" => {
            poly(p, r, &[(0.0, 1.0), (0.0, 0.0), (0.35, 0.0)], rgb(0, 61, 135));
            poly(p, r, &[(0.0, 1.0), (0.35, 0.0), (0.7, 0.0)], rgb(252, 216, 86));
            poly(p, r, &[(0.0, 1.0), (0.7, 0.0), (1.0, 0.0), (1.0, 0.33)], rgb(212, 40, 53));
            poly(p, r, &[(0.0, 1.0), (1.0, 0.33), (1.0, 0.66)], WHITE);
            poly(p, r, &[(0.0, 1.0), (1.0, 0.66), (1.0, 1.0)], rgb(0, 122, 61));
        }
        "SD" => {
            hbands(p, r, &[(1.0, rgb(210, 16, 52)), (1.0, WHITE), (1.0, BLACK)]);
            poly(p, r, &[(0.0, 0.0), (0.4, 0.5), (0.0, 1.0)], rgb(0, 122, 61));
        }
        "SG" => {
            hbands(p, r, &[(1.0, rgb(237, 41, 57)), (1.0, WHITE)]);
            if detail {
                crescent(p, r, 0.17, 0.25, 0.17, Vec2::new(1.0, 0.0), WHITE);
                let c = fr(r, 0.31, 0.25);
                for k in 0..5 {
                    let a = -FRAC_PI_2 + k as f32 * TAU / 5.0;
                    star_px(p, c + Vec2::angled(a) * (h * 0.1), h * 0.045, 5, 0.382, WHITE);
                }
            }
        }
        "SL" => hbands(p, r, &[(1.0, rgb(30, 181, 58)), (1.0, WHITE), (1.0, rgb(0, 114, 198))]),
        "SN" => {
            vbands(p, r, &[(1.0, rgb(0, 133, 63)), (1.0, rgb(253, 239, 66)), (1.0, rgb(227, 27, 35))]);
            if detail {
                star(p, r, 0.5, 0.5, 0.16, rgb(0, 133, 63));
            }
        }
        "SO" => {
            fill(p, r, rgb(65, 137, 221));
            if detail {
                star(p, r, 0.5, 0.5, 0.22, WHITE);
            }
        }
        "SR" => {
            hbands(
                p,
                r,
                &[(2.0, rgb(55, 126, 60)), (1.0, WHITE), (4.0, rgb(176, 16, 45)), (1.0, WHITE), (2.0, rgb(55, 126, 60))],
            );
            if detail {
                star(p, r, 0.5, 0.5, 0.22, rgb(255, 215, 0));
            }
        }
        "SS" => {
            hbands(
                p,
                r,
                &[(3.0, BLACK), (0.5, WHITE), (3.0, rgb(218, 18, 26)), (0.5, WHITE), (3.0, rgb(7, 109, 41))],
            );
            poly(p, r, &[(0.0, 0.0), (0.4, 0.5), (0.0, 1.0)], rgb(15, 71, 175));
            if detail {
                star(p, r, 0.11, 0.5, 0.1, rgb(252, 209, 22));
            }
        }
        "ST" => {
            hbands(p, r, &[(1.0, rgb(18, 173, 43)), (2.0, rgb(255, 206, 0)), (1.0, rgb(18, 173, 43))]);
            poly(p, r, &[(0.0, 0.0), (0.35, 0.5), (0.0, 1.0)], rgb(200, 16, 46));
            if detail {
                star(p, r, 0.52, 0.5, 0.12, BLACK);
                star(p, r, 0.72, 0.5, 0.12, BLACK);
            }
        }
        "SV" => {
            hbands(p, r, &[(1.0, rgb(15, 71, 175)), (1.0, WHITE), (1.0, rgb(15, 71, 175))]);
            if detail {
                dot(p, r, 0.5, 0.5, 0.1, rgb(200, 170, 60));
            }
        }
        "SY" => {
            hbands(p, r, &[(1.0, rgb(206, 17, 38)), (1.0, WHITE), (1.0, BLACK)]);
            if detail {
                star(p, r, 0.38, 0.5, 0.1, rgb(0, 122, 61));
                star(p, r, 0.62, 0.5, 0.1, rgb(0, 122, 61));
            }
        }
        "SZ" => {
            let blue = rgb(62, 94, 185);
            hbands(p, r, &[(3.0, blue), (1.0, rgb(255, 217, 0)), (6.0, rgb(176, 16, 45)), (1.0, rgb(255, 217, 0)), (3.0, blue)]);
            if detail {
                dot(p, r, 0.5, 0.5, 0.17, BLACK);
                dot(p, r, 0.5, 0.5, 0.09, WHITE);
            }
        }
        "TD" => vbands(p, r, &[(1.0, rgb(0, 38, 100)), (1.0, rgb(254, 203, 0)), (1.0, rgb(198, 12, 48))]),
        "TG" => {
            for i in 0..5 {
                let c = if i % 2 == 0 { rgb(0, 106, 78) } else { rgb(255, 206, 0) };
                fill(p, part(r, 0.0, i as f32 / 5.0, 1.0, (i as f32 + 1.0) / 5.0), c);
            }
            fill(p, part(r, 0.0, 0.0, 0.4, 0.6), rgb(210, 16, 52));
            if detail {
                star(p, r, 0.2, 0.3, 0.15, WHITE);
            }
        }
        "TH" => hbands(
            p,
            r,
            &[(1.0, rgb(165, 25, 49)), (1.0, WHITE), (2.0, rgb(45, 42, 74)), (1.0, WHITE), (1.0, rgb(165, 25, 49))],
        ),
        "TJ" => {
            hbands(p, r, &[(2.0, rgb(204, 0, 0)), (3.0, WHITE), (2.0, rgb(0, 102, 0))]);
            if detail {
                dot(p, r, 0.5, 0.5, 0.08, rgb(240, 190, 0));
            }
        }
        "TL" => {
            fill(p, r, rgb(220, 36, 31));
            poly(p, r, &[(0.0, 0.0), (0.5, 0.5), (0.0, 1.0)], rgb(255, 199, 0));
            poly(p, r, &[(0.0, 0.0), (0.33, 0.5), (0.0, 1.0)], BLACK);
            if detail {
                star(p, r, 0.1, 0.5, 0.1, WHITE);
            }
        }
        "TM" => {
            fill(p, r, rgb(0, 132, 61));
            fill(p, part(r, 0.08, 0.0, 0.26, 1.0), rgb(210, 16, 52));
            if detail {
                for k in 0..5 {
                    dot(p, r, 0.17, (k as f32 + 0.5) / 5.0, 0.04, rgb(255, 205, 0));
                }
                crescent(p, r, 0.34, 0.2, 0.12, Vec2::new(1.0, 0.0), WHITE);
            }
        }
        "TN" => {
            let red = rgb(231, 0, 19);
            fill(p, r, red);
            dot(p, r, 0.5, 0.5, 0.28, WHITE);
            if detail {
                crescent(p, r, 0.49, 0.5, 0.19, Vec2::new(1.0, 0.0), red);
                star(p, r, 0.52, 0.5, 0.1, red);
            }
        }
        "TO" => {
            let red = rgb(200, 16, 46);
            fill(p, r, red);
            let canton = part(r, 0.0, 0.0, 0.36, 0.5);
            fill(p, canton, WHITE);
            cross(p, canton, 0.5, h * 0.14, red);
        }
        "TR" => {
            fill(p, r, rgb(227, 10, 23));
            if detail {
                crescent(p, r, 0.38, 0.5, 0.25, Vec2::new(1.0, 0.0), WHITE);
                star(p, r, 0.54, 0.5, 0.1, WHITE);
            }
        }
        "TT" => {
            fill(p, r, rgb(200, 16, 46));
            diag(p, r.left_top(), r.right_bottom(), h * 0.5, WHITE);
            diag(p, r.left_top(), r.right_bottom(), h * 0.3, BLACK);
        }
        "TV" => {
            fill(p, r, rgb(65, 157, 201));
            union_jack(p, part(r, 0.0, 0.0, 0.5, 0.5), detail);
            if detail {
                let pts = [(0.65, 0.15), (0.85, 0.2), (0.58, 0.4), (0.78, 0.45), (0.92, 0.55), (0.65, 0.65), (0.8, 0.78), (0.58, 0.85), (0.92, 0.88)];
                for (x, y) in pts {
                    star(p, r, x, y, 0.05, rgb(255, 215, 0));
                }
            }
        }
        "TW" => {
            let blue = rgb(0, 0, 149);
            fill(p, r, rgb(254, 0, 0));
            fill(p, part(r, 0.0, 0.0, 0.5, 0.5), blue);
            if detail {
                star_n(p, r, 0.25, 0.25, 0.17, 12, 0.7, WHITE);
                dot(p, r, 0.25, 0.25, 0.1, blue);
                dot(p, r, 0.25, 0.25, 0.085, WHITE);
            }
        }
        "TZ" => {
            poly(p, r, &[(0.0, 0.0), (1.0, 0.0), (0.0, 1.0)], rgb(30, 181, 58));
            poly(p, r, &[(1.0, 0.0), (1.0, 1.0), (0.0, 1.0)], rgb(0, 163, 221));
            diag(p, r.left_bottom(), r.right_top(), h * 0.44, rgb(252, 209, 22));
            diag(p, r.left_bottom(), r.right_top(), h * 0.32, BLACK);
        }
        "UG" => {
            let cols = [BLACK, rgb(252, 220, 4), rgb(217, 16, 35)];
            for i in 0..6 {
                fill(p, part(r, 0.0, i as f32 / 6.0, 1.0, (i as f32 + 1.0) / 6.0), cols[i % 3]);
            }
            if detail {
                dot(p, r, 0.5, 0.5, 0.2, WHITE);
                dot(p, r, 0.5, 0.5, 0.1, rgb(130, 130, 130));
            }
        }
        "US" => {
            for i in 0..13 {
                let c = if i % 2 == 0 { rgb(178, 34, 52) } else { WHITE };
                fill(p, part(r, 0.0, i as f32 / 13.0, 1.0, (i as f32 + 1.0) / 13.0), c);
            }
            let canton = part(r, 0.0, 0.0, 0.4, 7.0 / 13.0);
            fill(p, canton, rgb(60, 59, 110));
            if detail {
                us_stars(p, canton);
            }
        }
        "UY" => {
            for i in 0..9 {
                let c = if i % 2 == 0 { WHITE } else { rgb(0, 56, 168) };
                fill(p, part(r, 0.0, i as f32 / 9.0, 1.0, (i as f32 + 1.0) / 9.0), c);
            }
            fill(p, part(r, 0.0, 0.0, 0.296, 4.0 / 9.0), WHITE);
            if detail {
                star_n(p, r, 0.148, 2.0 / 9.0, 0.12, 16, 0.6, rgb(252, 209, 22));
            }
        }
        "UZ" => {
            let red = rgb(206, 17, 38);
            hbands(p, r, &[(4.0, rgb(0, 153, 181)), (0.2, red), (4.0, WHITE), (0.2, red), (4.0, rgb(30, 181, 58))]);
            if detail {
                crescent(p, r, 0.13, 0.15, 0.1, Vec2::new(1.0, 0.0), WHITE);
                for (row, count) in [(0, 3), (1, 4), (2, 5)] {
                    for k in 0..count {
                        let x = 0.25 + 0.025 * (2 - row) as f32 + 0.05 * k as f32;
                        star(p, r, x, 0.07 + 0.07 * row as f32, 0.03, WHITE);
                    }
                }
            }
        }
        "VC" => {
            vbands(p, r, &[(1.0, rgb(0, 41, 156)), (2.0, rgb(252, 209, 22)), (1.0, rgb(0, 158, 73))]);
            if detail {
                let green = rgb(0, 158, 73);
                diamond(p, r, 0.42, 0.32, 0.04, 0.12, green);
                diamond(p, r, 0.58, 0.32, 0.04, 0.12, green);
                diamond(p, r, 0.5, 0.64, 0.04, 0.12, green);
            }
        }
        "VE" => {
            hbands(p, r, &[(1.0, rgb(255, 204, 0)), (1.0, rgb(0, 36, 125)), (1.0, rgb(207, 20, 43))]);
            if detail {
                for k in 0..8 {
                    let t = k as f32 / 7.0;
                    let u = 2.0 * t - 1.0;
                    star(p, r, 0.27 + 0.46 * t, 0.58 - 0.1 * (1.0 - u * u), 0.04, WHITE);
                }
            }
        }
        "VN" => {
            fill(p, r, rgb(218, 37, 29));
            if detail {
                star(p, r, 0.5, 0.52, 0.3, rgb(255, 255, 0));
            }
        }
        "VU" => {
            let yellow = rgb(253, 206, 18);
            hbands(p, r, &[(1.0, rgb(206, 17, 38)), (1.0, rgb(0, 148, 96))]);
            fill(p, part(r, 0.0, 0.4, 1.0, 0.6), BLACK);
            fill(p, part(r, 0.0, 0.43, 1.0, 0.57), yellow);
            poly(p, r, &[(0.0, 0.0), (0.5, 0.5), (0.0, 1.0)], yellow);
            poly(p, r, &[(0.0, 0.1), (0.38, 0.5), (0.0, 0.9)], BLACK);
            if detail {
                ring(p, r, 0.1, 0.5, 0.07, 1.5, yellow);
            }
        }
        "WS" => {
            fill(p, r, rgb(206, 17, 38));
            fill(p, part(r, 0.0, 0.0, 0.5, 0.5), rgb(0, 32, 159));
            if detail {
                let pts = [(0.25, 0.1, 0.06), (0.14, 0.27, 0.06), (0.37, 0.22, 0.06), (0.26, 0.4, 0.06), (0.31, 0.3, 0.035)];
                for (x, y, s) in pts {
                    star(p, r, x, y, s, WHITE);
                }
            }
        }
        "YE" => hbands(p, r, &[(1.0, rgb(206, 17, 38)), (1.0, WHITE), (1.0, BLACK)]),
        "ZA" => {
            let (green, gold) = (rgb(0, 122, 77), rgb(255, 184, 28));
            hbands(p, r, &[(1.0, rgb(224, 60, 49)), (1.0, rgb(0, 35, 149))]);
            poly(p, r, &[(0.0, 0.0), (0.0, 1.0), (0.4, 0.5)], BLACK);
            let (a, b, c, d) = (r.left_top(), r.left_bottom(), fr(r, 0.4, 0.5), r.right_center());
            for (w, col) in [(h * 0.44, WHITE), (h * 0.34, gold), (h * 0.24, green)] {
                diag(p, a, c, w, col);
                diag(p, b, c, w, col);
                diag(p, c, d, w, col);
                p.circle_filled(c, w / 2.0, col);
            }
        }
        "ZM" => {
            fill(p, r, rgb(25, 138, 0));
            vbands(p, part(r, 0.6, 0.5, 1.0, 1.0), &[(1.0, rgb(222, 32, 16)), (1.0, BLACK), (1.0, rgb(239, 125, 0))]);
            if detail {
                dot(p, r, 0.8, 0.25, 0.09, rgb(239, 125, 0));
            }
        }
        "ZW" => {
            let (green, gold, red) = (rgb(0, 100, 0), rgb(255, 210, 0), rgb(215, 20, 26));
            hbands(p, r, &[(1.0, green), (1.0, gold), (1.0, red), (1.0, BLACK), (1.0, red), (1.0, gold), (1.0, green)]);
            poly(p, r, &[(0.0, 0.0), (0.43, 0.5), (0.0, 1.0)], BLACK);
            poly(p, r, &[(0.0, 0.05), (0.38, 0.5), (0.0, 0.95)], WHITE);
            if detail {
                star(p, r, 0.12, 0.5, 0.11, red);
            }
        }
        _ => fill(p, r, rgb(90, 90, 90)),
    }
    painter.rect_stroke(r, 0.0, Stroke::new(1.0_f32, crate::theme::p().border_hi));
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
    fn all_world_countries_are_listed() {
        // 193 UN members + Vatican, Palestine, Taiwan, Kosovo.
        assert_eq!(COUNTRIES.len(), 197);
    }

    #[test]
    fn every_country_has_its_own_flag() {
        // Every code must have an explicit arm in `draw`, otherwise the grey fallback would be shown.
        let src = include_str!("flags.rs");
        for (code, _) in COUNTRIES {
            assert!(src.contains(&format!("\"{code}\" =>")), "no flag drawn for {code}");
        }
    }

    #[test]
    fn translated_names_cover_every_country() {
        assert_eq!(crate::countries::NAMES.len(), COUNTRIES.len());
        for ((code, _), (code2, names)) in COUNTRIES.iter().zip(crate::countries::NAMES) {
            assert_eq!(code, code2, "order differs from COUNTRIES");
            assert!(names.iter().all(|n| !n.trim().is_empty()), "empty name for {code}");
        }
    }

    #[test]
    fn names_are_unique_in_every_language() {
        for lang in Lang::ALL {
            let mut seen = HashSet::new();
            for (code, _) in COUNTRIES {
                let n = name_in(lang, code).unwrap();
                assert!(seen.insert(n), "{lang:?}: duplicate name {n}");
            }
        }
    }

    #[test]
    fn sorting_ignores_diacritics() {
        let pos = |lang: Lang, name: &str| sorted(lang).iter().position(|c| c.1 == name).unwrap();
        assert!(pos(Lang::Fr, "Égypte") < pos(Lang::Fr, "Espagne"));
        assert!(pos(Lang::Fr, "Équateur") < pos(Lang::Fr, "Érythrée"));
        assert!(pos(Lang::De, "Ägypten") < pos(Lang::De, "Albanien"));
        assert!(pos(Lang::Pl, "Łotwa") < pos(Lang::Pl, "Malta"));
        assert_eq!(sorted(Lang::Zh).len(), COUNTRIES.len());
    }

    #[test]
    fn countries_are_sorted_by_name() {
        let names: Vec<&str> = COUNTRIES.iter().map(|c| c.1).collect();
        let mut sorted = names.clone();
        sorted.sort();
        assert_eq!(names, sorted);
    }
}
