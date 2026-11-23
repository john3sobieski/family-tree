//! Export of the whole tree to PNG (raster) and PDF (vector).
//!
//! Pipeline: the same `Layout` as on screen → `Scene` (a flat list of vector primitives in world
//! units: lines, cards, photos, flags, text as glyph outlines) → one of two backends:
//! * PNG: `tiny-skia`, rendered in horizontal bands and streamed straight into the PNG file,
//!   so memory stays bounded even for huge images; the background may be transparent;
//! * PDF: written by hand (no PDF library): everything is vector, text is outlines (no fonts to
//!   embed, any script works), photos are the JPEG thumbnails embedded as they are.
//!
//! The work runs in a thread (`start`), the UI polls it (`ExportUi::show`).
//! Flags are not re-implemented: `flags::draw` is run on a headless egui context and the recorded
//! shapes are converted into primitives (`record_flag`).

use crate::container::{MediaKind, MediaStore};
use crate::flags::{FLAG_H, FLAG_W};
use crate::i18n::{tr, trf};
use crate::layout::{Layout, LayoutConfig};
use crate::model::{Gender, Tree};
use crate::view::{CARD_RADIUS, GENDER_STRIP, PHOTO_PAD};
use ab_glyph::{Font, FontVec, GlyphId, OutlineCurve};
use eframe::egui;
use std::collections::{HashMap, HashSet};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{mpsc, Arc};
use std::time::Duration;
use tiny_skia as sk;

/// Free space around the tree (world units).
const MARGIN: f32 = 100.0;
/// Wanted PNG scale (pixels per world unit): the 12.5-unit text becomes 25 px tall.
const PNG_SCALE: f32 = 2.0;
/// PNG limits: longest side and total pixels; the scale is reduced for huge trees.
const PNG_MAX_SIDE: f32 = 65_000.0;
const PNG_MAX_PIXELS: f64 = 400e6;
/// Below this scale a PNG would be unreadable, so it is refused (the PDF has no such limit).
const PNG_MIN_SCALE: f32 = 0.05;
/// Page size limit of PDF viewers (Acrobat): 200 inches.
const PDF_MAX_SIDE: f32 = 14_400.0;
/// Memory for one band of the PNG.
const BAND_BYTES: usize = 64 * 1024 * 1024;
/// Decoded photos kept while rendering the PNG.
const PHOTO_CACHE: usize = 300;

const LINE_W: f32 = 1.5;
const BORDER_W: f32 = 1.0;
const TEXT_SIZE: f32 = 12.5;

type Rgba = [u8; 4];
/// x0, y0, x1, y1.
type Rect4 = [f32; 4];

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Format {
    Png,
    Pdf,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Background {
    Transparent,
    White,
    Black,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Settings {
    pub format: Format,
    pub background: Background,
    /// Colors of the cards, lines and text: dark or light theme.
    pub dark: bool,
}

// ---------------------------------------------------------------- scene

#[derive(Clone, Copy, Debug)]
enum Seg {
    M(f32, f32),
    L(f32, f32),
    C(f32, f32, f32, f32, f32, f32),
    Z,
}

enum Op {
    Fill { segs: Vec<Seg>, color: Rgba },
    Line { segs: Vec<Seg>, width: f32, color: Rgba },
    /// Photo (thumbnail) in a rectangle with rounded corners.
    Photo { hash: String, rect: Rect4, radius: f32 },
    /// Small reusable picture (a flag, the photo placeholder) with its top-left corner at x, y.
    Sprite { key: String, x: f32, y: f32 },
}

struct Prim {
    op: Op,
    /// Bounding box (world units), generous for strokes: used to skip primitives outside a band.
    bbox: Rect4,
    clip: Option<Rect4>,
}

struct Sprite {
    w: f32,
    h: f32,
    prims: Vec<Prim>,
}

struct PhotoData {
    bytes: Vec<u8>,
    w: u32,
    h: u32,
    /// 1 (gray) or 3 (RGB).
    comps: u8,
}

pub struct Scene {
    prims: Vec<Prim>,
    sprites: HashMap<String, Sprite>,
    photos: HashMap<String, PhotoData>,
    /// The whole picture including the margins.
    bounds: Rect4,
}

fn grow(b: &mut Rect4, x: f32, y: f32) {
    b[0] = b[0].min(x);
    b[1] = b[1].min(y);
    b[2] = b[2].max(x);
    b[3] = b[3].max(y);
}

fn bbox_of(segs: &[Seg]) -> Rect4 {
    let mut b = [f32::MAX, f32::MAX, f32::MIN, f32::MIN];
    for s in segs {
        match *s {
            Seg::M(x, y) | Seg::L(x, y) => grow(&mut b, x, y),
            Seg::C(a, c, d, e, f, g) => {
                grow(&mut b, a, c);
                grow(&mut b, d, e);
                grow(&mut b, f, g);
            }
            Seg::Z => {}
        }
    }
    if b[0] > b[2] {
        [0.0; 4]
    } else {
        b
    }
}

fn fill_prim(segs: Vec<Seg>, color: Rgba) -> Prim {
    let bbox = bbox_of(&segs);
    Prim { op: Op::Fill { segs, color }, bbox, clip: None }
}

fn line_prim(segs: Vec<Seg>, width: f32, color: Rgba) -> Prim {
    let mut bbox = bbox_of(&segs);
    let pad = width * 2.0 + 1.0;
    bbox = [bbox[0] - pad, bbox[1] - pad, bbox[2] + pad, bbox[3] + pad];
    Prim { op: Op::Line { segs, width, color }, bbox, clip: None }
}

fn rect_contains(outer: Rect4, inner: Rect4) -> bool {
    outer[0] <= inner[0] && outer[1] <= inner[1] && outer[2] >= inner[2] && outer[3] >= inner[3]
}

fn rect_intersects(a: Rect4, b: Rect4) -> bool {
    a[0] < b[2] && b[0] < a[2] && a[1] < b[3] && b[1] < a[3]
}

/// Adds a clip rectangle only when the primitive actually sticks out of it.
fn with_clip(mut p: Prim, clip: Rect4) -> Option<Prim> {
    if !rect_intersects(clip, p.bbox) {
        return None;
    }
    if !rect_contains(clip, p.bbox) {
        p.clip = Some(clip);
    }
    Some(p)
}

const KAPPA: f32 = 0.552_284_75;

/// Rectangle with rounded corners: radii nw, ne, se, sw.
fn rrect_segs(x0: f32, y0: f32, x1: f32, y1: f32, r: [f32; 4]) -> Vec<Seg> {
    let max_r = ((x1 - x0).min(y1 - y0) / 2.0).max(0.0);
    let [nw, ne, se, sw] = r.map(|v| v.clamp(0.0, max_r));
    let k = KAPPA;
    let mut s = vec![Seg::M(x0 + nw, y0), Seg::L(x1 - ne, y0)];
    if ne > 0.0 {
        s.push(Seg::C(x1 - ne + ne * k, y0, x1, y0 + ne - ne * k, x1, y0 + ne));
    }
    s.push(Seg::L(x1, y1 - se));
    if se > 0.0 {
        s.push(Seg::C(x1, y1 - se + se * k, x1 - se + se * k, y1, x1 - se, y1));
    }
    s.push(Seg::L(x0 + sw, y1));
    if sw > 0.0 {
        s.push(Seg::C(x0 + sw - sw * k, y1, x0, y1 - sw + sw * k, x0, y1 - sw));
    }
    s.push(Seg::L(x0, y0 + nw));
    if nw > 0.0 {
        s.push(Seg::C(x0, y0 + nw - nw * k, x0 + nw - nw * k, y0, x0 + nw, y0));
    }
    s.push(Seg::Z);
    s
}

fn circle_segs(cx: f32, cy: f32, r: f32) -> Vec<Seg> {
    let k = KAPPA * r;
    vec![
        Seg::M(cx + r, cy),
        Seg::C(cx + r, cy + k, cx + k, cy + r, cx, cy + r),
        Seg::C(cx - k, cy + r, cx - r, cy + k, cx - r, cy),
        Seg::C(cx - r, cy - k, cx - k, cy - r, cx, cy - r),
        Seg::C(cx + k, cy - r, cx + r, cy - k, cx + r, cy),
        Seg::Z,
    ]
}

fn poly_segs(pts: &[egui::Pos2], closed: bool) -> Vec<Seg> {
    let mut s = Vec::with_capacity(pts.len() + 1);
    for (i, p) in pts.iter().enumerate() {
        s.push(if i == 0 { Seg::M(p.x, p.y) } else { Seg::L(p.x, p.y) });
    }
    if closed && !pts.is_empty() {
        s.push(Seg::Z);
    }
    s
}

fn rgba(c: egui::Color32) -> Rgba {
    c.to_srgba_unmultiplied()
}

fn mix(a: Rgba, b: Rgba) -> Rgba {
    let m = |x: u8, y: u8| ((x as u16 + y as u16) / 2) as u8;
    [m(a[0], b[0]), m(a[1], b[1]), m(a[2], b[2]), 255]
}

struct Colors {
    card: Rgba,
    border: Rgba,
    text: Rgba,
    line: Rgba,
    ph_fg: Rgba,
    ph_bg: Rgba,
    male: Rgba,
    female: Rgba,
}

/// The same colors the screen view uses (see `view.rs`), for a chosen theme.
fn colors(dark: bool) -> Colors {
    let p = crate::theme::palette(dark);
    Colors {
        card: rgba(p.surface),
        border: rgba(p.border),
        text: rgba(p.text_dim),
        line: mix(rgba(p.text_dim), rgba(p.surface)),
        ph_fg: rgba(p.border),
        ph_bg: rgba(p.faint),
        male: rgba(p.gender_male),
        female: rgba(p.gender_female),
    }
}

// ---------------------------------------------------------------- flags (recorded from egui)

/// Converts one recorded egui shape into primitives (flag-local coordinates).
fn convert_shape(shape: &egui::Shape, clip: Rect4, out: &mut Vec<Prim>) {
    use egui::epaint::ColorMode;
    use egui::Shape as S;
    fn add(out: &mut Vec<Prim>, p: Prim, clip: Rect4) {
        if let Some(p) = with_clip(p, clip) {
            out.push(p);
        }
    }
    match shape {
        S::Vec(list) => {
            for s in list {
                convert_shape(s, clip, out);
            }
        }
        S::Circle(c) => {
            if c.fill.a() > 0 {
                add(out, fill_prim(circle_segs(c.center.x, c.center.y, c.radius), rgba(c.fill)), clip);
            }
            if c.stroke.width > 0.0 && c.stroke.color.a() > 0 {
                add(out, line_prim(circle_segs(c.center.x, c.center.y, c.radius), c.stroke.width, rgba(c.stroke.color)), clip);
            }
        }
        S::LineSegment { points, stroke } => {
            if let ColorMode::Solid(col) = &stroke.color {
                if stroke.width > 0.0 && col.a() > 0 {
                    add(out, line_prim(poly_segs(points, false), stroke.width, rgba(*col)), clip);
                }
            }
        }
        S::Path(p) => {
            if p.closed && p.fill.a() > 0 {
                add(out, fill_prim(poly_segs(&p.points, true), rgba(p.fill)), clip);
            }
            if let ColorMode::Solid(col) = &p.stroke.color {
                if p.stroke.width > 0.0 && col.a() > 0 {
                    add(out, line_prim(poly_segs(&p.points, p.closed), p.stroke.width, rgba(*col)), clip);
                }
            }
        }
        S::Rect(r) => {
            let (x0, y0, x1, y1) = (r.rect.min.x, r.rect.min.y, r.rect.max.x, r.rect.max.y);
            let radii = [r.rounding.nw, r.rounding.ne, r.rounding.se, r.rounding.sw];
            if r.fill.a() > 0 {
                add(out, fill_prim(rrect_segs(x0, y0, x1, y1, radii), rgba(r.fill)), clip);
            }
            if r.stroke.width > 0.0 && r.stroke.color.a() > 0 {
                add(out, line_prim(rrect_segs(x0, y0, x1, y1, radii), r.stroke.width, rgba(r.stroke.color)), clip);
            }
        }
        S::Mesh(m) => {
            // Triangles of one color are merged into a single path: no seams between them.
            let mut groups: Vec<(Rgba, Vec<Seg>)> = Vec::new();
            for tri in m.indices.chunks(3) {
                if tri.len() < 3 {
                    continue;
                }
                let v = |i: u32| m.vertices[i as usize];
                let (a, b, c) = (v(tri[0]), v(tri[1]), v(tri[2]));
                let col = rgba(a.color);
                if col[3] == 0 {
                    continue;
                }
                let idx = match groups.iter().position(|g| g.0 == col) {
                    Some(i) => i,
                    None => {
                        groups.push((col, Vec::new()));
                        groups.len() - 1
                    }
                };
                let segs = &mut groups[idx].1;
                segs.push(Seg::M(a.pos.x, a.pos.y));
                segs.push(Seg::L(b.pos.x, b.pos.y));
                segs.push(Seg::L(c.pos.x, c.pos.y));
                segs.push(Seg::Z);
            }
            for (col, segs) in groups {
                add(out, fill_prim(segs, col), clip);
            }
        }
        // Not used by the flags.
        _ => {}
    }
}

/// Draws the flag with the screen code on a headless egui context and converts what it painted.
fn record_flag(ctx: &egui::Context, code: &str) -> Vec<Prim> {
    let rect = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(FLAG_W, FLAG_H));
    let output = ctx.run(egui::RawInput::default(), |ctx| {
        let painter = ctx.layer_painter(egui::LayerId::background());
        crate::flags::draw(&painter, rect, code, true);
    });
    let mut prims = Vec::new();
    for cs in &output.shapes {
        let c = cs.clip_rect.intersect(rect);
        convert_shape(&cs.shape, [c.min.x, c.min.y, c.max.x, c.max.y], &mut prims);
    }
    prims
}

// ---------------------------------------------------------------- text as outlines

struct Face {
    font: FontVec,
    upem: f32,
}

/// Fonts of the interface (Segoe UI, fallbacks, CJK when needed); text becomes glyph outlines.
struct Fonts {
    faces: Vec<Face>,
    /// Outlines in font units (y up), per (face, glyph).
    cache: HashMap<(usize, u16), Vec<Seg>>,
}

impl Fonts {
    fn load(cjk: bool) -> Fonts {
        let defs = crate::theme::font_definitions(cjk);
        let names = defs.families.get(&egui::FontFamily::Proportional).cloned().unwrap_or_default();
        let mut faces = Vec::new();
        for name in names {
            let Some(data) = defs.font_data.get(&name) else { continue };
            if let Ok(font) = FontVec::try_from_vec_and_index(data.font.to_vec(), data.index) {
                let upem = font.units_per_em().unwrap_or(1000.0).max(1.0);
                faces.push(Face { font, upem });
            }
        }
        Fonts { faces, cache: HashMap::new() }
    }

    fn face_for(&self, c: char) -> Option<(usize, GlyphId)> {
        self.faces.iter().enumerate().find_map(|(i, f)| {
            let g = f.font.glyph_id(c);
            (g.0 != 0).then_some((i, g))
        })
    }

    /// Distance from the top of a text row to the baseline, per unit of font size.
    fn ascent(&self) -> f32 {
        self.faces.first().map_or(0.9, |f| f.font.ascent_unscaled() / f.upem)
    }

    fn ensure_glyph(&mut self, face: usize, gid: GlyphId) {
        if self.cache.contains_key(&(face, gid.0)) {
            return;
        }
        let mut segs: Vec<Seg> = Vec::new();
        if let Some(outline) = self.faces[face].font.outline(gid) {
            let mut last: Option<(f32, f32)> = None;
            fn start(segs: &mut Vec<Seg>, last: &mut Option<(f32, f32)>, x: f32, y: f32) {
                if *last != Some((x, y)) {
                    if last.is_some() {
                        segs.push(Seg::Z);
                    }
                    segs.push(Seg::M(x, y));
                }
            }
            for curve in &outline.curves {
                match *curve {
                    OutlineCurve::Line(a, b) => {
                        start(&mut segs, &mut last, a.x, a.y);
                        segs.push(Seg::L(b.x, b.y));
                        last = Some((b.x, b.y));
                    }
                    OutlineCurve::Quad(a, c, b) => {
                        start(&mut segs, &mut last, a.x, a.y);
                        // Quadratic → cubic.
                        segs.push(Seg::C(
                            a.x + 2.0 / 3.0 * (c.x - a.x),
                            a.y + 2.0 / 3.0 * (c.y - a.y),
                            b.x + 2.0 / 3.0 * (c.x - b.x),
                            b.y + 2.0 / 3.0 * (c.y - b.y),
                            b.x,
                            b.y,
                        ));
                        last = Some((b.x, b.y));
                    }
                    OutlineCurve::Cubic(a, c1, c2, b) => {
                        start(&mut segs, &mut last, a.x, a.y);
                        segs.push(Seg::C(c1.x, c1.y, c2.x, c2.y, b.x, b.y));
                        last = Some((b.x, b.y));
                    }
                }
            }
            if last.is_some() {
                segs.push(Seg::Z);
            }
        }
        self.cache.insert((face, gid.0), segs);
    }

    /// One line of text at font size `size`: outlines with the origin at the left end of the
    /// baseline (y down), and the width of the line.
    fn run(&mut self, text: &str, size: f32) -> (Vec<Seg>, f32) {
        let mut out: Vec<Seg> = Vec::new();
        let mut x = 0.0_f32;
        for ch in text.chars() {
            if ch.is_control() {
                continue;
            }
            let Some((fi, gid)) = self.face_for(ch) else {
                x += size * 0.5;
                continue;
            };
            let k = size / self.faces[fi].upem;
            self.ensure_glyph(fi, gid);
            if let Some(g) = self.cache.get(&(fi, gid.0)) {
                for seg in g {
                    out.push(match *seg {
                        Seg::M(a, b) => Seg::M(x + a * k, -b * k),
                        Seg::L(a, b) => Seg::L(x + a * k, -b * k),
                        Seg::C(a, b, c, d, e, f) => Seg::C(x + a * k, -b * k, x + c * k, -d * k, x + e * k, -f * k),
                        Seg::Z => Seg::Z,
                    });
                }
            }
            x += self.faces[fi].font.h_advance_unscaled(gid) * k;
        }
        (out, x)
    }
}

fn offset(segs: &[Seg], dx: f32, dy: f32) -> Vec<Seg> {
    segs.iter()
        .map(|s| match *s {
            Seg::M(a, b) => Seg::M(a + dx, b + dy),
            Seg::L(a, b) => Seg::L(a + dx, b + dy),
            Seg::C(a, b, c, d, e, f) => Seg::C(a + dx, b + dy, c + dx, d + dy, e + dx, f + dy),
            Seg::Z => Seg::Z,
        })
        .collect()
}

/// Whether any name needs the CJK font (Chinese, Japanese, Korean).
fn needs_cjk(tree: &Tree, layout: &Layout) -> bool {
    let wide = |s: &str| s.chars().any(|c| c as u32 >= 0x2E80);
    layout.nodes.iter().any(|n| {
        tree.persons
            .get(&n.id)
            .map_or(false, |p| wide(&p.first_name) || wide(&p.last_name) || wide(&p.maiden_name) || wide(&p.patronymic))
    })
}

// ---------------------------------------------------------------- building the scene

/// Parses the header of a JPEG: (width, height, components). Only 8-bit gray/RGB is accepted.
fn jpeg_info(b: &[u8]) -> Option<(u32, u32, u8)> {
    if b.len() < 4 || b[0] != 0xFF || b[1] != 0xD8 {
        return None;
    }
    let mut i = 2;
    while i + 4 <= b.len() {
        if b[i] != 0xFF {
            i += 1;
            continue;
        }
        let m = b[i + 1];
        if m == 0xFF {
            i += 1;
            continue;
        }
        if m == 0xD8 || m == 0x01 || (0xD0..=0xD7).contains(&m) {
            i += 2;
            continue;
        }
        if m == 0xD9 {
            return None;
        }
        let len = u16::from_be_bytes([b[i + 2], b[i + 3]]) as usize;
        if matches!(m, 0xC0 | 0xC1 | 0xC2) {
            if i + 10 > b.len() || b[i + 4] != 8 {
                return None;
            }
            let h = u16::from_be_bytes([b[i + 5], b[i + 6]]) as u32;
            let w = u16::from_be_bytes([b[i + 7], b[i + 8]]) as u32;
            let comps = b[i + 9];
            if w == 0 || h == 0 || !(comps == 1 || comps == 3) {
                return None;
            }
            return Some((w, h, comps));
        }
        i += 2 + len;
    }
    None
}

/// Size of the whole picture in world units (cards, flags above them, margins).
pub fn world_size(layout: &Layout) -> (f32, f32) {
    let b = layout.bounds;
    let w = (b.max.x - b.min.x).max(1.0) + 2.0 * MARGIN;
    let h = (b.max.y - b.min.y).max(1.0) + FLAG_H + 2.0 * MARGIN;
    (w, h)
}

fn world_bounds(layout: &Layout) -> Rect4 {
    let b = layout.bounds;
    [b.min.x - MARGIN, b.min.y - FLAG_H - MARGIN, b.max.x + MARGIN, b.max.y + MARGIN]
}

struct Builder<'a> {
    tree: &'a Tree,
    layout: &'a Layout,
    cfg: &'a LayoutConfig,
    colors: Colors,
    fonts: Fonts,
    egui_ctx: egui::Context,
    thumbs: HashMap<String, Vec<u8>>,
    prims: Vec<Prim>,
    sprites: HashMap<String, Sprite>,
    photos: HashMap<String, PhotoData>,
}

impl Builder<'_> {
    fn flag_sprite(&mut self, code: &str) -> String {
        let key = format!("fl:{code}");
        if !self.sprites.contains_key(&key) {
            let prims = record_flag(&self.egui_ctx, code);
            self.sprites.insert(key.clone(), Sprite { w: FLAG_W, h: FLAG_H, prims });
        }
        key
    }

    fn placeholder_sprite(&mut self, w: f32, h: f32) -> String {
        let key = "ph".to_string();
        if !self.sprites.contains_key(&key) {
            let c = &self.colors;
            let mut prims = vec![fill_prim(rrect_segs(0.0, 0.0, w, h, [CARD_RADIUS; 4]), c.ph_bg)];
            let clip = [0.0, 0.0, w, h];
            let head = fill_prim(circle_segs(w / 2.0, h * 0.36, w * 0.17), c.ph_fg);
            let body = fill_prim(circle_segs(w / 2.0, h * 0.98, w * 0.38), c.ph_fg);
            prims.extend(with_clip(head, clip));
            prims.extend(with_clip(body, clip));
            self.sprites.insert(key.clone(), Sprite { w, h, prims });
        }
        key
    }

    /// True if the photo has a usable thumbnail (and keeps it for the scene).
    fn photo_ok(&mut self, hash: &str) -> bool {
        if self.photos.contains_key(hash) {
            return true;
        }
        if let Some(bytes) = self.thumbs.remove(hash) {
            if let Some((w, h, comps)) = jpeg_info(&bytes) {
                self.photos.insert(hash.to_string(), PhotoData { bytes, w, h, comps });
                return true;
            }
        }
        false
    }

    /// One line centered at `cx`, `top` is the top of the text row. Too long lines get smaller.
    fn text_line(&mut self, cx: f32, top: f32, text: &str, max_w: f32) {
        if text.is_empty() {
            return;
        }
        let (_, w) = self.fonts.run(text, TEXT_SIZE);
        let mut k = if w > max_w && w > 0.0 { (max_w / w).max(0.7) } else { 1.0 };
        if w * k > max_w && w > 0.0 {
            k = max_w / w;
        }
        let size = TEXT_SIZE * k;
        let (segs, width) = self.fonts.run(text, size);
        if segs.is_empty() {
            return;
        }
        let baseline = top + self.fonts.ascent() * size;
        let placed = offset(&segs, cx - width / 2.0, baseline);
        self.prims.push(fill_prim(placed, self.colors.text));
    }

    fn edges(&mut self) {
        let line = self.colors.line;
        let layout = self.layout;
        for e in &layout.edges {
            let segs = if e.hang {
                vec![Seg::M(e.from.x, e.from.y), Seg::L(e.from.x, e.to.y), Seg::L(e.to.x, e.to.y)]
            } else {
                vec![
                    Seg::M(e.from.x, e.from.y),
                    Seg::L(e.from.x, e.bus),
                    Seg::L(e.to.x, e.bus),
                    Seg::L(e.to.x, e.to.y),
                ]
            };
            self.prims.push(line_prim(segs, LINE_W, line));
        }
        // Partner links: a short line between neighbouring cards of a unit.
        let (cw, ch) = (self.cfg.card_w, self.cfg.card_h);
        for u in &layout.units {
            for pair in u.members.windows(2) {
                if let (Some(a), Some(b)) = (layout.pos_of(pair[0]), layout.pos_of(pair[1])) {
                    let y = a.y - ch / 2.0 + (cw * 4.0 / 3.0) / 2.0;
                    let segs = vec![Seg::M(a.x + cw / 2.0, y), Seg::L(b.x - cw / 2.0, y)];
                    self.prims.push(line_prim(segs, LINE_W, line));
                }
            }
        }
    }

    fn card(&mut self, id: crate::model::PersonId, cx: f32, cy: f32) {
        let tree = self.tree;
        let Some(person) = tree.persons.get(&id) else { return };
        let (cw, ch) = (self.cfg.card_w, self.cfg.card_h);
        let (x0, y0, x1, y1) = (cx - cw / 2.0, cy - ch / 2.0, cx + cw / 2.0, cy + ch / 2.0);
        let strip = match person.gender {
            Gender::Male => Some(self.colors.male),
            Gender::Female => Some(self.colors.female),
            Gender::Unknown => None,
        };
        let r = CARD_RADIUS;
        match strip {
            Some(col) => {
                self.prims.push(fill_prim(rrect_segs(x0, y0, x1, y1, [r; 4]), col));
                let top = [r, r, 0.0, 0.0];
                self.prims.push(fill_prim(rrect_segs(x0, y0, x1, y1 - GENDER_STRIP, top), self.colors.card));
            }
            None => self.prims.push(fill_prim(rrect_segs(x0, y0, x1, y1, [r; 4]), self.colors.card)),
        }
        self.prims.push(line_prim(rrect_segs(x0, y0, x1, y1, [r; 4]), BORDER_W, self.colors.border));

        // Flags of nationalities: right above the card, centered.
        let n = person.nationalities.len().min(crate::flags::MAX_PER_PERSON);
        let fx0 = cx - FLAG_W * n as f32 / 2.0;
        for (i, code) in person.nationalities.iter().take(n).enumerate() {
            let key = self.flag_sprite(code);
            let (x, y) = (fx0 + FLAG_W * i as f32, y0 - FLAG_H);
            self.prims.push(Prim {
                op: Op::Sprite { key, x, y },
                bbox: [x, y, x + FLAG_W, y + FLAG_H],
                clip: None,
            });
        }

        // Photo 3:4.
        let pw = cw - 2.0 * PHOTO_PAD;
        let ph = pw * 4.0 / 3.0;
        let (px, py) = (x0 + PHOTO_PAD, y0 + PHOTO_PAD);
        let hash = person.main_photo.as_deref().filter(|h| self.photo_ok(h)).map(str::to_string);
        match hash {
            Some(hash) => self.prims.push(Prim {
                op: Op::Photo { hash, rect: [px, py, px + pw, py + ph], radius: r },
                bbox: [px, py, px + pw, py + ph],
                clip: None,
            }),
            None => {
                let key = self.placeholder_sprite(pw, ph);
                self.prims.push(Prim {
                    op: Op::Sprite { key, x: px, y: py },
                    bbox: [px, py, px + pw, py + ph],
                    clip: None,
                });
            }
        }

        // First name, surname (a woman: «Иванова (Петрова)»), patronymic.
        let max_w = cw - 2.0 * PHOTO_PAD;
        let y_text = py + ph + 3.0;
        let first = person.first_name.trim().to_string();
        let last = person.surname_display();
        let patr = person.patronymic.trim().to_string();
        if first.is_empty() && last.is_empty() && patr.is_empty() {
            self.text_line(cx, y_text, "—", max_w);
        } else {
            self.text_line(cx, y_text, &first, max_w);
            self.text_line(cx, y_text + 15.0, &last, max_w);
            self.text_line(cx, y_text + 30.0, &patr, max_w);
        }
    }
}

fn build_scene(
    tree: &Tree,
    layout: &Layout,
    cfg: &LayoutConfig,
    thumbs: HashMap<String, Vec<u8>>,
    dark: bool,
    progress: &AtomicU32,
) -> Scene {
    let mut b = Builder {
        tree,
        layout,
        cfg,
        colors: colors(dark),
        fonts: Fonts::load(needs_cjk(tree, layout)),
        egui_ctx: egui::Context::default(),
        thumbs,
        prims: Vec::new(),
        sprites: HashMap::new(),
        photos: HashMap::new(),
    };
    progress.store(40, Ordering::Relaxed);
    b.edges();
    let total = layout.nodes.len().max(1);
    for (i, n) in layout.nodes.iter().enumerate() {
        b.card(n.id, n.pos.x, n.pos.y);
        if i % 64 == 0 {
            progress.store(40 + (i * 60 / total) as u32, Ordering::Relaxed);
        }
    }
    Scene { prims: b.prims, sprites: b.sprites, photos: b.photos, bounds: world_bounds(layout) }
}

// ---------------------------------------------------------------- PNG (tiny-skia)

/// (scale, width, height) of the PNG for a picture of `w` × `h` world units; `None` if it would
/// be too small to read.
pub fn plan_png(w: f32, h: f32) -> Option<(f32, u32, u32)> {
    let mut s = PNG_SCALE;
    s = s.min(PNG_MAX_SIDE / w.max(h));
    s = s.min((PNG_MAX_PIXELS / (w as f64 * h as f64)).sqrt() as f32);
    if s < PNG_MIN_SCALE {
        return None;
    }
    Some((s, (w * s).ceil() as u32, (h * s).ceil() as u32))
}

fn solid(c: Rgba) -> sk::Paint<'static> {
    let mut p = sk::Paint::default();
    p.set_color_rgba8(c[0], c[1], c[2], c[3]);
    p.anti_alias = true;
    p
}

fn to_path(segs: &[Seg], ox: f32, oy: f32, s: f32) -> Option<sk::Path> {
    let mut pb = sk::PathBuilder::new();
    let px = |x: f32| (x - ox) * s;
    let py = |y: f32| (y - oy) * s;
    for seg in segs {
        match *seg {
            Seg::M(x, y) => pb.move_to(px(x), py(y)),
            Seg::L(x, y) => pb.line_to(px(x), py(y)),
            Seg::C(a, b, c, d, e, f) => pb.cubic_to(px(a), py(b), px(c), py(d), px(e), py(f)),
            Seg::Z => pb.close(),
        }
    }
    pb.finish()
}

fn make_mask(w: u32, h: u32, c: Rect4, ox: f32, oy: f32, s: f32) -> Option<sk::Mask> {
    let mut m = sk::Mask::new(w, h)?;
    let r = sk::Rect::from_ltrb((c[0] - ox) * s, (c[1] - oy) * s, (c[2] - ox) * s, (c[3] - oy) * s)?;
    let path = sk::PathBuilder::from_rect(r);
    m.fill_path(&path, sk::FillRule::Winding, true, sk::Transform::identity());
    Some(m)
}

fn decode_pixmap(bytes: &[u8]) -> Option<sk::Pixmap> {
    let img = image::load_from_memory(bytes).ok()?.to_rgba8();
    let (w, h) = img.dimensions();
    let mut pm = sk::Pixmap::new(w, h)?;
    // A JPEG is opaque, so premultiplied and plain RGBA are the same.
    pm.data_mut().copy_from_slice(img.as_raw());
    Some(pm)
}

struct Raster<'a> {
    scene: &'a Scene,
    s: f32,
    sprites: HashMap<String, Option<sk::Pixmap>>,
    photos: HashMap<String, Option<sk::Pixmap>>,
}

impl<'a> Raster<'a> {
    fn new(scene: &'a Scene, s: f32) -> Self {
        Raster { scene, s, sprites: HashMap::new(), photos: HashMap::new() }
    }

    /// Draws a primitive on `pm`; (ox, oy) is the world point at the top-left pixel.
    fn draw(&mut self, pm: &mut sk::Pixmap, p: &Prim, ox: f32, oy: f32) {
        let s = self.s;
        let mask = p.clip.and_then(|c| make_mask(pm.width(), pm.height(), c, ox, oy, s));
        let id = sk::Transform::identity();
        match &p.op {
            Op::Fill { segs, color } => {
                if let Some(path) = to_path(segs, ox, oy, s) {
                    pm.fill_path(&path, &solid(*color), sk::FillRule::Winding, id, mask.as_ref());
                }
            }
            Op::Line { segs, width, color } => {
                if let Some(path) = to_path(segs, ox, oy, s) {
                    let stroke = sk::Stroke { width: width * s, ..Default::default() };
                    pm.stroke_path(&path, &solid(*color), &stroke, id, mask.as_ref());
                }
            }
            Op::Photo { hash, rect, radius } => {
                if !self.photos.contains_key(hash) {
                    if self.photos.len() >= PHOTO_CACHE {
                        self.photos.clear();
                    }
                    let decoded = self.scene.photos.get(hash).and_then(|d| decode_pixmap(&d.bytes));
                    self.photos.insert(hash.clone(), decoded);
                }
                let Some(Some(photo)) = self.photos.get(hash) else { return };
                let (dx, dy) = ((rect[0] - ox) * s, (rect[1] - oy) * s);
                let (dw, dh) = ((rect[2] - rect[0]) * s, (rect[3] - rect[1]) * s);
                let ts = sk::Transform::from_row(
                    dw / photo.width() as f32,
                    0.0,
                    0.0,
                    dh / photo.height() as f32,
                    dx,
                    dy,
                );
                let mut paint = sk::Paint::default();
                paint.shader = sk::Pattern::new(photo.as_ref(), sk::SpreadMode::Pad, sk::FilterQuality::Bicubic, 1.0, ts);
                paint.anti_alias = true;
                let segs = rrect_segs(rect[0], rect[1], rect[2], rect[3], [*radius; 4]);
                if let Some(path) = to_path(&segs, ox, oy, s) {
                    pm.fill_path(&path, &paint, sk::FillRule::Winding, id, mask.as_ref());
                }
            }
            Op::Sprite { key, x, y } => {
                if !self.sprites.contains_key(key) {
                    let rendered = self.render_sprite(key);
                    self.sprites.insert(key.clone(), rendered);
                }
                let Some(Some(sprite)) = self.sprites.get(key) else { return };
                // Snapped to whole pixels: sprites stay sharp (the error is under half a pixel).
                let ix = ((x - ox) * s).round() as i32;
                let iy = ((y - oy) * s).round() as i32;
                pm.draw_pixmap(ix, iy, sprite.as_ref(), &sk::PixmapPaint::default(), id, None);
            }
        }
    }

    fn render_sprite(&mut self, key: &str) -> Option<sk::Pixmap> {
        let scene = self.scene;
        let sp = scene.sprites.get(key)?;
        let w = (sp.w * self.s).ceil().max(1.0) as u32;
        let h = (sp.h * self.s).ceil().max(1.0) as u32;
        let mut pm = sk::Pixmap::new(w, h)?;
        for p in &sp.prims {
            self.draw(&mut pm, p, 0.0, 0.0);
        }
        Some(pm)
    }
}

fn tmp_path(p: &Path) -> PathBuf {
    let mut s = p.as_os_str().to_owned();
    s.push(".tmp");
    PathBuf::from(s)
}

fn write_png(
    scene: &Scene,
    bg: Background,
    path: &Path,
    progress: &AtomicU32,
    cancel: &AtomicBool,
) -> Result<Done, String> {
    let b = scene.bounds;
    let (w, h) = (b[2] - b[0], b[3] - b[1]);
    let Some((s, wpx, hpx)) = plan_png(w, h) else {
        return Err(tr("Дерево слишком большое для PNG, используйте PDF.").to_string());
    };
    let band_h = ((BAND_BYTES / (wpx as usize * 4)).clamp(16, 512) as u32).min(hpx);
    let tmp = tmp_path(path);
    let result = (|| -> Result<(), String> {
        let file = std::fs::File::create(&tmp).map_err(|e| e.to_string())?;
        let mut enc = png::Encoder::new(BufWriter::new(file), wpx, hpx);
        enc.set_color(png::ColorType::Rgba);
        enc.set_depth(png::BitDepth::Eight);
        enc.set_compression(png::Compression::Fast);
        let mut writer = enc.write_header().map_err(|e| e.to_string())?;
        let mut stream = writer.stream_writer().map_err(|e| e.to_string())?;

        let mut pm = sk::Pixmap::new(wpx, band_h).ok_or_else(|| tr("Не хватает памяти.").to_string())?;
        let fill = match bg {
            Background::Transparent => sk::Color::TRANSPARENT,
            Background::White => sk::Color::WHITE,
            Background::Black => sk::Color::BLACK,
        };
        let mut raster = Raster::new(scene, s);
        let mut out: Vec<u8> = Vec::new();
        let bands = (hpx + band_h - 1) / band_h;
        for band in 0..bands {
            if cancel.load(Ordering::Relaxed) {
                return Err(tr("Экспорт отменён.").to_string());
            }
            let row0 = band * band_h;
            let rows = band_h.min(hpx - row0);
            pm.fill(fill);
            let oy = b[1] + row0 as f32 / s;
            let ox = b[0];
            // World rectangle of this band (a little wider, for antialiasing).
            let view = [ox - 2.0, oy - 2.0, ox + wpx as f32 / s + 2.0, oy + (rows as f32 + 2.0) / s];
            for p in &scene.prims {
                if rect_intersects(view, p.bbox) {
                    raster.draw(&mut pm, p, ox, oy);
                }
            }
            // Premultiplied → plain RGBA, only the rows that belong to the image.
            let n = rows as usize * wpx as usize * 4;
            out.clear();
            out.extend_from_slice(&pm.data()[..n]);
            for px in out.chunks_exact_mut(4) {
                let a = px[3] as u32;
                if a == 0 {
                    px[0] = 0;
                    px[1] = 0;
                    px[2] = 0;
                } else if a < 255 {
                    for c in &mut px[..3] {
                        *c = ((*c as u32 * 255 + a / 2) / a).min(255) as u8;
                    }
                }
            }
            stream.write_all(&out).map_err(|e| e.to_string())?;
            progress.store(100 + (band as u64 * 880 / bands as u64) as u32, Ordering::Relaxed);
        }
        stream.finish().map_err(|e| e.to_string())?;
        writer.finish().map_err(|e| e.to_string())?;
        Ok(())
    })();
    match result {
        Ok(()) => std::fs::rename(&tmp, path).map_err(|e| {
            let _ = std::fs::remove_file(&tmp);
            e.to_string()
        })?,
        Err(e) => {
            let _ = std::fs::remove_file(&tmp);
            return Err(e);
        }
    }
    Ok(Done { path: path.to_path_buf(), format: Format::Png, width: wpx, height: hpx })
}

// ---------------------------------------------------------------- PDF (written by hand)

/// (scale, width, height) in points: one world unit is one point unless the page would be too big.
pub fn plan_pdf(w: f32, h: f32) -> (f32, f32, f32) {
    let s = (PDF_MAX_SIDE / w.max(h)).min(1.0);
    (s, w * s, h * s)
}

fn num(v: f32) -> String {
    let mut s = format!("{v:.3}");
    while s.ends_with('0') {
        s.pop();
    }
    if s.ends_with('.') {
        s.pop();
    }
    if s == "-0" {
        s = "0".to_string();
    }
    s
}

fn color_ops(c: Rgba, stroke: bool) -> String {
    let f = |v: u8| num(v as f32 / 255.0);
    format!("{} {} {} {}\n", f(c[0]), f(c[1]), f(c[2]), if stroke { "RG" } else { "rg" })
}

fn path_ops(segs: &[Seg], out: &mut String) {
    for seg in segs {
        match *seg {
            Seg::M(x, y) => out.push_str(&format!("{} {} m\n", num(x), num(y))),
            Seg::L(x, y) => out.push_str(&format!("{} {} l\n", num(x), num(y))),
            Seg::C(a, b, c, d, e, f) => out.push_str(&format!(
                "{} {} {} {} {} {} c\n",
                num(a),
                num(b),
                num(c),
                num(d),
                num(e),
                num(f)
            )),
            Seg::Z => out.push_str("h\n"),
        }
    }
}

/// Compressing sink for a content stream.
struct Sink {
    enc: flate2::write::ZlibEncoder<Vec<u8>>,
    buf: String,
}

impl Sink {
    fn new() -> Self {
        Sink {
            enc: flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::default()),
            buf: String::new(),
        }
    }

    fn text(&mut self) -> &mut String {
        if self.buf.len() > 1 << 20 {
            let _ = self.enc.write_all(self.buf.as_bytes());
            self.buf.clear();
        }
        &mut self.buf
    }

    fn finish(mut self) -> std::io::Result<Vec<u8>> {
        self.enc.write_all(self.buf.as_bytes())?;
        self.enc.finish()
    }
}

struct PdfNames {
    sprites: HashMap<String, usize>,
    photos: HashMap<String, usize>,
}

fn emit_prim(sink: &mut Sink, p: &Prim, names: &PdfNames) {
    let clipped = p.clip.is_some();
    if let Some(c) = p.clip {
        let t = sink.text();
        t.push_str(&format!("q\n{} {} {} {} re W n\n", num(c[0]), num(c[1]), num(c[2] - c[0]), num(c[3] - c[1])));
    }
    match &p.op {
        Op::Fill { segs, color } => {
            if color[3] > 0 {
                let t = sink.text();
                t.push_str(&color_ops(*color, false));
                path_ops(segs, t);
                t.push_str("f\n");
            }
        }
        Op::Line { segs, width, color } => {
            if color[3] > 0 {
                let t = sink.text();
                t.push_str(&color_ops(*color, true));
                t.push_str(&format!("{} w\n", num(*width)));
                path_ops(segs, t);
                t.push_str("S\n");
            }
        }
        Op::Photo { hash, rect, radius } => {
            if let Some(i) = names.photos.get(hash) {
                let t = sink.text();
                t.push_str("q\n");
                path_ops(&rrect_segs(rect[0], rect[1], rect[2], rect[3], [*radius; 4]), t);
                t.push_str("W n\n");
                // The image occupies the unit square with y up; the page is flipped, so flip back.
                t.push_str(&format!(
                    "{} 0 0 {} {} {} cm\n/Im{} Do\nQ\n",
                    num(rect[2] - rect[0]),
                    num(-(rect[3] - rect[1])),
                    num(rect[0]),
                    num(rect[3]),
                    i
                ));
            }
        }
        Op::Sprite { key, x, y } => {
            if let Some(i) = names.sprites.get(key) {
                let t = sink.text();
                t.push_str(&format!("q\n1 0 0 1 {} {} cm\n/Fl{} Do\nQ\n", num(*x), num(*y), i));
            }
        }
    }
    if clipped {
        sink.text().push_str("Q\n");
    }
}

struct PdfOut {
    w: BufWriter<std::fs::File>,
    pos: u64,
    /// Byte offset of every object, indexed by object number (0 is unused).
    offsets: Vec<u64>,
}

impl PdfOut {
    fn put(&mut self, data: &[u8]) -> std::io::Result<()> {
        self.w.write_all(data)?;
        self.pos += data.len() as u64;
        Ok(())
    }

    fn begin(&mut self, id: usize) -> std::io::Result<()> {
        self.offsets[id] = self.pos;
        self.put(format!("{id} 0 obj\n").as_bytes())
    }

    fn end(&mut self) -> std::io::Result<()> {
        self.put(b"\nendobj\n")
    }

    fn object(&mut self, id: usize, body: &str) -> std::io::Result<()> {
        self.begin(id)?;
        self.put(body.as_bytes())?;
        self.end()
    }

    fn stream(&mut self, id: usize, dict: &str, data: &[u8]) -> std::io::Result<()> {
        self.begin(id)?;
        self.put(format!("<< {dict} /Length {} >>\nstream\n", data.len()).as_bytes())?;
        self.put(data)?;
        self.put(b"\nendstream")?;
        self.end()
    }
}

fn write_pdf(
    scene: &Scene,
    bg: Background,
    path: &Path,
    progress: &AtomicU32,
    cancel: &AtomicBool,
) -> Result<Done, String> {
    let b = scene.bounds;
    let (w, h) = (b[2] - b[0], b[3] - b[1]);
    let (s, wpt, hpt) = plan_pdf(w, h);

    // Object numbers: 1 catalog, 2 pages, 3 page, 4 contents, then sprites, then photos.
    let mut sprite_keys: Vec<&String> = scene.sprites.keys().collect();
    sprite_keys.sort();
    let mut photo_keys: Vec<&String> = scene.photos.keys().collect();
    photo_keys.sort();
    let sprite_id = |i: usize| 5 + i;
    let photo_id = |i: usize| 5 + sprite_keys.len() + i;
    let total_objects = 5 + sprite_keys.len() + photo_keys.len();
    let names = PdfNames {
        sprites: sprite_keys.iter().enumerate().map(|(i, k)| ((*k).clone(), i)).collect(),
        photos: photo_keys.iter().enumerate().map(|(i, k)| ((*k).clone(), i)).collect(),
    };

    let tmp = tmp_path(path);
    let result = (|| -> Result<(), String> {
        let io = |e: std::io::Error| e.to_string();
        let file = std::fs::File::create(&tmp).map_err(io)?;
        let mut out = PdfOut { w: BufWriter::new(file), pos: 0, offsets: vec![0; total_objects] };
        out.put(b"%PDF-1.5\n%\xE2\xE3\xCF\xD3\n").map_err(io)?;

        // Resources of the page.
        let mut xobj = String::new();
        for i in 0..sprite_keys.len() {
            xobj.push_str(&format!("/Fl{} {} 0 R ", i, sprite_id(i)));
        }
        for i in 0..photo_keys.len() {
            xobj.push_str(&format!("/Im{} {} 0 R ", i, photo_id(i)));
        }
        out.object(1, "<< /Type /Catalog /Pages 2 0 R >>").map_err(io)?;
        out.object(2, "<< /Type /Pages /Kids [3 0 R] /Count 1 >>").map_err(io)?;
        out.object(
            3,
            &format!(
                "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 {} {}] /Contents 4 0 R /Resources << /XObject << {}>> >> >>",
                num(wpt),
                num(hpt),
                xobj
            ),
        )
        .map_err(io)?;
        progress.store(150, Ordering::Relaxed);

        // Page content: background, then the flipped world coordinate system, then everything.
        let mut sink = Sink::new();
        {
            let t = sink.text();
            match bg {
                Background::Transparent => {}
                Background::White => t.push_str(&format!("1 1 1 rg\n0 0 {} {} re f\n", num(wpt), num(hpt))),
                Background::Black => t.push_str(&format!("0 0 0 rg\n0 0 {} {} re f\n", num(wpt), num(hpt))),
            }
            // World (x, y down) → page (points, y up).
            t.push_str(&format!(
                "{} 0 0 {} {} {} cm\n0 J 0 j 4 M\n",
                num(s),
                num(-s),
                num(-b[0] * s),
                num((b[1] + h) * s)
            ));
        }
        let count = scene.prims.len().max(1);
        for (i, p) in scene.prims.iter().enumerate() {
            emit_prim(&mut sink, p, &names);
            if i % 2048 == 0 {
                if cancel.load(Ordering::Relaxed) {
                    return Err(tr("Экспорт отменён.").to_string());
                }
                progress.store(150 + (i as u64 * 600 / count as u64) as u32, Ordering::Relaxed);
            }
        }
        let content = sink.finish().map_err(io)?;
        out.stream(4, "/Filter /FlateDecode", &content).map_err(io)?;
        progress.store(760, Ordering::Relaxed);

        // Sprites (flags, placeholder): form XObjects in the same flipped coordinates.
        for (i, key) in sprite_keys.iter().enumerate() {
            let sp = &scene.sprites[*key];
            let mut sink = Sink::new();
            for p in &sp.prims {
                emit_prim(&mut sink, p, &names);
            }
            let data = sink.finish().map_err(io)?;
            let dict = format!(
                "/Type /XObject /Subtype /Form /BBox [0 0 {} {}] /Resources << >> /Filter /FlateDecode",
                num(sp.w),
                num(sp.h)
            );
            out.stream(sprite_id(i), &dict, &data).map_err(io)?;
        }

        // Photos: the JPEG files as they are.
        for (i, key) in photo_keys.iter().enumerate() {
            let ph = &scene.photos[*key];
            let space = if ph.comps == 1 { "/DeviceGray" } else { "/DeviceRGB" };
            let dict = format!(
                "/Type /XObject /Subtype /Image /Width {} /Height {} /ColorSpace {} /BitsPerComponent 8 /Filter /DCTDecode",
                ph.w, ph.h, space
            );
            out.stream(photo_id(i), &dict, &ph.bytes).map_err(io)?;
        }
        progress.store(950, Ordering::Relaxed);

        // Cross-reference table.
        let xref_pos = out.pos;
        let mut xref = format!("xref\n0 {}\n0000000000 65535 f \n", total_objects);
        for id in 1..total_objects {
            xref.push_str(&format!("{:010} 00000 n \n", out.offsets[id]));
        }
        xref.push_str(&format!(
            "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{}\n%%EOF\n",
            total_objects, xref_pos
        ));
        out.put(xref.as_bytes()).map_err(io)?;
        out.w.flush().map_err(io)?;
        Ok(())
    })();
    match result {
        Ok(()) => std::fs::rename(&tmp, path).map_err(|e| {
            let _ = std::fs::remove_file(&tmp);
            e.to_string()
        })?,
        Err(e) => {
            let _ = std::fs::remove_file(&tmp);
            return Err(e);
        }
    }
    Ok(Done { path: path.to_path_buf(), format: Format::Pdf, width: wpt.round() as u32, height: hpt.round() as u32 })
}

// ---------------------------------------------------------------- background job

pub struct Request {
    pub tree: Tree,
    pub layout: Layout,
    pub cfg: LayoutConfig,
    /// JPEG thumbnails by hash (read from the file before the thread starts).
    pub thumbs: HashMap<String, Vec<u8>>,
    pub settings: Settings,
    pub path: PathBuf,
}

#[derive(Clone, Debug)]
pub struct Done {
    pub path: PathBuf,
    pub format: Format,
    /// PNG: pixels; PDF: points.
    pub width: u32,
    pub height: u32,
}

pub struct Job {
    /// 0..=1000.
    progress: Arc<AtomicU32>,
    cancel: Arc<AtomicBool>,
    rx: mpsc::Receiver<Result<Done, String>>,
}

fn run(req: Request, progress: &AtomicU32, cancel: &AtomicBool) -> Result<Done, String> {
    progress.store(10, Ordering::Relaxed);
    if req.layout.nodes.is_empty() {
        return Err(tr("Нечего экспортировать: дерево пустое.").to_string());
    }
    let scene = build_scene(&req.tree, &req.layout, &req.cfg, req.thumbs, req.settings.dark, progress);
    progress.store(100, Ordering::Relaxed);
    if cancel.load(Ordering::Relaxed) {
        return Err(tr("Экспорт отменён.").to_string());
    }
    match req.settings.format {
        Format::Png => write_png(&scene, req.settings.background, &req.path, progress, cancel),
        Format::Pdf => write_pdf(&scene, req.settings.background, &req.path, progress, cancel),
    }
}

/// Starts the export in a thread.
pub fn start(req: Request, ctx: egui::Context) -> Job {
    let progress = Arc::new(AtomicU32::new(0));
    let cancel = Arc::new(AtomicBool::new(false));
    let (tx, rx) = mpsc::channel();
    let (p, c) = (progress.clone(), cancel.clone());
    std::thread::spawn(move || {
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| run(req, &p, &c)))
            .unwrap_or_else(|_| Err(tr("Внутренняя ошибка при экспорте.").to_string()));
        let _ = tx.send(result);
        ctx.request_repaint();
    });
    Job { progress, cancel, rx }
}

// ---------------------------------------------------------------- dialog

pub struct ExportUi {
    pub open: bool,
    settings: Settings,
    job: Option<Job>,
    result: Option<Result<Done, String>>,
}

impl Default for ExportUi {
    fn default() -> Self {
        ExportUi {
            open: false,
            settings: Settings { format: Format::Png, background: Background::White, dark: false },
            job: None,
            result: None,
        }
    }
}

fn scale_text(s: f32) -> String {
    num(s)
}

impl ExportUi {
    /// Opens the dialog; colors and background start like the current theme.
    pub fn open(&mut self) {
        self.open = true;
        if self.job.is_none() {
            self.settings.dark = crate::theme::is_dark();
            self.settings.background = if self.settings.dark { Background::Black } else { Background::White };
            self.result = None;
        }
    }

    /// Text for the status bar while the export is running.
    pub fn status_text(&self) -> Option<String> {
        let job = self.job.as_ref()?;
        let percent = job.progress.load(Ordering::Relaxed) / 10;
        Some(trf("Экспорт… {percent}%", &[("percent", percent.to_string())]))
    }

    fn pick_path(&self, base: &str) -> Option<PathBuf> {
        let (ext, label) = match self.settings.format {
            Format::Png => ("png", tr("Изображение PNG")),
            Format::Pdf => ("pdf", tr("Документ PDF")),
        };
        let mut p = rfd::FileDialog::new()
            .add_filter(label, &[ext])
            .set_file_name(format!("{base}.{ext}"))
            .save_file()?;
        if p.extension().is_none() {
            p.set_extension(ext);
        }
        Some(p)
    }

    fn begin(&mut self, ctx: &egui::Context, tree: &Tree, layout: &Layout, cfg: &LayoutConfig, store: &MediaStore, path: PathBuf) {
        let mut seen = HashSet::new();
        let hashes: Vec<String> = layout
            .nodes
            .iter()
            .filter_map(|n| tree.persons.get(&n.id)?.main_photo.clone())
            .filter(|h| seen.insert(h.clone()))
            .collect();
        let thumbs = store.get_many(MediaKind::Thumb, &hashes);
        let req = Request { tree: tree.clone(), layout: layout.clone(), cfg: cfg.clone(), thumbs, settings: self.settings, path };
        self.result = None;
        self.job = Some(start(req, ctx.clone()));
    }

    /// Call every frame (also while the dialog is closed: it collects the result of the thread).
    pub fn show(
        &mut self,
        ctx: &egui::Context,
        tree: &Tree,
        layout: &Layout,
        cfg: &LayoutConfig,
        store: &MediaStore,
        base_name: &str,
    ) {
        if let Some(job) = &self.job {
            match job.rx.try_recv() {
                Ok(r) => {
                    self.result = Some(r);
                    self.job = None;
                    self.open = true;
                }
                Err(mpsc::TryRecvError::Empty) => ctx.request_repaint_after(Duration::from_millis(100)),
                Err(mpsc::TryRecvError::Disconnected) => {
                    self.result = Some(Err(tr("Внутренняя ошибка при экспорте.").to_string()));
                    self.job = None;
                    self.open = true;
                }
            }
        }
        if !self.open {
            return;
        }

        let mut keep_open = true;
        let mut close = false;
        let mut start_clicked = false;
        egui::Window::new(tr("Экспорт"))
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .open(&mut keep_open)
            .show(ctx, |ui| {
                let busy = self.job.is_some();
                ui.add_enabled_ui(!busy, |ui| {
                    ui.label(tr("Формат"));
                    ui.horizontal(|ui| {
                        ui.selectable_value(&mut self.settings.format, Format::Png, "PNG");
                        ui.selectable_value(&mut self.settings.format, Format::Pdf, "PDF");
                    });
                    ui.add_space(6.0);
                    ui.label(tr("Цвета"));
                    ui.horizontal(|ui| {
                        ui.selectable_value(&mut self.settings.dark, true, tr("Тёмные"));
                        ui.selectable_value(&mut self.settings.dark, false, tr("Светлые"));
                    });
                    ui.add_space(6.0);
                    ui.label(tr("Фон"));
                    ui.horizontal(|ui| {
                        ui.selectable_value(&mut self.settings.background, Background::Transparent, tr("Прозрачный"));
                        ui.selectable_value(&mut self.settings.background, Background::White, tr("Белый"));
                        ui.selectable_value(&mut self.settings.background, Background::Black, tr("Чёрный"));
                    });
                });
                ui.add_space(8.0);

                // What will come out.
                let (w, h) = world_size(layout);
                let weak = |t: String| egui::RichText::new(t).small().color(crate::theme::p().muted);
                if layout.nodes.is_empty() {
                    ui.label(weak(tr("Нечего экспортировать: дерево пустое.").to_string()));
                } else if self.settings.format == Format::Png {
                    match plan_png(w, h) {
                        Some((s, wpx, hpx)) => {
                            ui.label(weak(trf(
                                "PNG: {w} × {h} px (масштаб ×{scale}). Всё дерево целиком, с отступами по краям.",
                                &[("w", wpx.to_string()), ("h", hpx.to_string()), ("scale", scale_text(s))],
                            )));
                            if s < 1.0 {
                                ui.label(weak(
                                    tr("Дерево очень большое, детали будут мелкими. Для таких деревьев лучше PDF.").to_string(),
                                ));
                            }
                        }
                        None => {
                            ui.label(
                                egui::RichText::new(tr("Дерево слишком большое для PNG, используйте PDF."))
                                    .small()
                                    .color(crate::theme::p().danger),
                            );
                        }
                    }
                } else {
                    let (_, wpt, hpt) = plan_pdf(w, h);
                    ui.label(weak(trf(
                        "PDF: векторный, масштабируется без потери качества. Страница {w} × {h} pt.",
                        &[("w", (wpt.round() as u32).to_string()), ("h", (hpt.round() as u32).to_string())],
                    )));
                }
                ui.add_space(8.0);

                if let Some(job) = &self.job {
                    let p = job.progress.load(Ordering::Relaxed) as f32 / 1000.0;
                    ui.add(egui::ProgressBar::new(p.clamp(0.0, 1.0)).show_percentage());
                    ui.add_space(4.0);
                    if ui.button(tr("Отмена")).clicked() {
                        job.cancel.store(true, Ordering::Relaxed);
                    }
                } else {
                    match &self.result {
                        Some(Ok(d)) => {
                            let size = match d.format {
                                Format::Png => format!("{} × {} px", d.width, d.height),
                                Format::Pdf => format!("{} × {} pt", d.width, d.height),
                            };
                            ui.label(trf(
                                "Готово: {path} ({size})",
                                &[("path", d.path.display().to_string()), ("size", size)],
                            ));
                        }
                        Some(Err(e)) => {
                            ui.colored_label(
                                crate::theme::p().danger,
                                trf("Не удалось экспортировать: {err}", &[("err", e.clone())]),
                            );
                        }
                        None => {}
                    }
                    ui.add_space(4.0);
                    ui.horizontal(|ui| {
                        let can = !layout.nodes.is_empty();
                        if ui.add_enabled(can, egui::Button::new(tr("Экспортировать…"))).clicked() {
                            start_clicked = true;
                        }
                        if ui.button(tr("Закрыть")).clicked() {
                            close = true;
                        }
                    });
                }
            });
        if start_clicked {
            if let Some(path) = self.pick_path(base_name) {
                self.begin(ctx, tree, layout, cfg, store, path);
            }
        }
        self.open = keep_open && !close;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::container::MediaStore;

    fn small() -> (Tree, Layout, LayoutConfig) {
        let tree = crate::demo::sample_tree();
        let cfg = LayoutConfig::default();
        let layout = crate::layout::compute(&tree, &cfg);
        (tree, layout, cfg)
    }

    fn scene_of(tree: &Tree, layout: &Layout, cfg: &LayoutConfig, thumbs: HashMap<String, Vec<u8>>) -> Scene {
        build_scene(tree, layout, cfg, thumbs, true, &AtomicU32::new(0))
    }

    #[test]
    fn every_country_flag_converts_to_primitives() {
        let ctx = egui::Context::default();
        for (code, _) in crate::flags::COUNTRIES {
            let prims = record_flag(&ctx, code);
            assert!(!prims.is_empty(), "flag {code} produced nothing");
        }
    }

    #[test]
    fn rounded_rect_is_closed_and_inside() {
        let segs = rrect_segs(0.0, 0.0, 100.0, 50.0, [8.0, 8.0, 0.0, 0.0]);
        assert!(matches!(segs.first(), Some(Seg::M(..))));
        assert!(matches!(segs.last(), Some(Seg::Z)));
        let b = bbox_of(&segs);
        assert!(b[0] >= -0.001 && b[1] >= -0.001 && b[2] <= 100.001 && b[3] <= 50.001);
    }

    #[test]
    fn jpeg_header_is_parsed() {
        let img = image::RgbImage::from_pixel(30, 20, image::Rgb([200, 10, 10]));
        let mut buf = Vec::new();
        img.write_with_encoder(image::codecs::jpeg::JpegEncoder::new_with_quality(&mut buf, 85)).unwrap();
        assert_eq!(jpeg_info(&buf), Some((30, 20, 3)));
        assert_eq!(jpeg_info(b"not a jpeg"), None);
    }

    #[test]
    fn png_plan_respects_limits() {
        let (s, w, h) = plan_png(1000.0, 500.0).unwrap();
        assert_eq!(s, PNG_SCALE);
        assert_eq!((w, h), (2000, 1000));
        let (s, w, _) = plan_png(200_000.0, 3_000.0).unwrap();
        assert!(s < PNG_SCALE && w as f32 <= PNG_MAX_SIDE + 1.0);
        assert!(plan_png(2_000_000.0, 2_000_000.0).is_none());
        let (s, wp, hp) = plan_pdf(60_000.0, 3_000.0);
        assert!(s < 1.0 && wp <= PDF_MAX_SIDE + 1.0 && hp > 0.0);
    }

    #[test]
    fn scene_has_cards_text_and_flags() {
        let (tree, layout, cfg) = small();
        let scene = scene_of(&tree, &layout, &cfg, HashMap::new());
        assert!(scene.prims.len() > layout.nodes.len() * 4);
        assert!(scene.prims.iter().any(|p| matches!(p.op, Op::Sprite { .. })));
        let b = scene.bounds;
        assert!(b[2] > b[0] && b[3] > b[1]);
    }

    #[test]
    fn png_and_pdf_files_are_written() {
        let (tree, layout, cfg) = small();
        let scene = scene_of(&tree, &layout, &cfg, HashMap::new());
        let dir = std::env::temp_dir().join(format!("famtree-export-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let progress = AtomicU32::new(0);
        let cancel = AtomicBool::new(false);

        let png_path = dir.join("tree.png");
        let done = write_png(&scene, Background::Transparent, &png_path, &progress, &cancel).unwrap();
        let img = image::open(&png_path).unwrap().to_rgba8();
        assert_eq!((img.width(), img.height()), (done.width, done.height));
        // The corner is empty (transparent background), the picture has content somewhere.
        assert_eq!(img.get_pixel(0, 0)[3], 0);
        assert!(img.pixels().any(|p| p[3] > 0));

        let pdf_path = dir.join("tree.pdf");
        write_pdf(&scene, Background::White, &pdf_path, &progress, &cancel).unwrap();
        let pdf = std::fs::read(&pdf_path).unwrap();
        assert!(pdf.starts_with(b"%PDF-1.5"));
        assert!(pdf.ends_with(b"%%EOF\n"));
        let text = String::from_utf8_lossy(&pdf);
        assert!(text.contains("/Type /Page "));
        assert!(text.contains("startxref"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn big_demo_scene_builds() {
        let tree = crate::demo::big_tree(300);
        let cfg = LayoutConfig::default();
        let layout = crate::layout::compute(&tree, &cfg);
        let scene = scene_of(&tree, &layout, &cfg, HashMap::new());
        assert!(scene.prims.len() > 300);
        let _ = MediaStore::new();
    }
}
