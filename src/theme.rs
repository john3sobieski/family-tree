//! Theme: dark (OLED black) and light, with thin borders (Resend-like). The choice is saved in a small file.

use eframe::egui::{self, Color32, FontFamily, FontId, Rounding, Stroke, TextStyle};
use std::sync::atomic::{AtomicBool, Ordering};

/// Colors that differ between the dark and the light theme.
pub struct Palette {
    pub bg: Color32,
    pub surface: Color32,
    pub surface2: Color32,
    pub faint: Color32,
    pub border: Color32,
    pub border_hi: Color32,
    pub text: Color32,
    pub text_dim: Color32,
    pub muted: Color32,
    /// Main accent (primary button, selected card outline).
    pub accent: Color32,
    /// Text on top of `accent`.
    pub on_accent: Color32,
    pub select_bg: Color32,
    pub hover: Color32,
    pub active: Color32,
    pub danger: Color32,
    /// Card bottom strip by gender.
    pub gender_male: Color32,
    pub gender_female: Color32,
}

const DARK: Palette = Palette {
    bg: Color32::from_rgb(0, 0, 0),
    surface: Color32::from_rgb(10, 10, 10),
    surface2: Color32::from_rgb(20, 20, 20),
    faint: Color32::from_rgb(14, 14, 14),
    border: Color32::from_rgb(34, 34, 34),
    border_hi: Color32::from_rgb(70, 70, 76),
    text: Color32::from_rgb(237, 237, 237),
    text_dim: Color32::from_rgb(200, 200, 200),
    muted: Color32::from_rgb(140, 140, 140),
    accent: Color32::from_rgb(237, 237, 237),
    on_accent: Color32::from_rgb(0, 0, 0),
    select_bg: Color32::from_rgb(38, 38, 42),
    hover: Color32::from_rgb(30, 30, 30),
    active: Color32::from_rgb(42, 42, 42),
    danger: Color32::from_rgb(229, 72, 77),
    gender_male: Color32::from_rgb(98, 148, 196),
    gender_female: Color32::from_rgb(200, 128, 158),
};

const LIGHT: Palette = Palette {
    bg: Color32::from_rgb(255, 255, 255),
    surface: Color32::from_rgb(244, 244, 245),
    surface2: Color32::from_rgb(236, 236, 238),
    faint: Color32::from_rgb(248, 248, 249),
    border: Color32::from_rgb(212, 212, 216),
    border_hi: Color32::from_rgb(161, 161, 170),
    text: Color32::from_rgb(17, 17, 17),
    text_dim: Color32::from_rgb(63, 63, 70),
    muted: Color32::from_rgb(113, 113, 122),
    accent: Color32::from_rgb(17, 17, 17),
    on_accent: Color32::from_rgb(255, 255, 255),
    select_bg: Color32::from_rgb(228, 228, 232),
    hover: Color32::from_rgb(228, 228, 231),
    active: Color32::from_rgb(212, 212, 216),
    danger: Color32::from_rgb(220, 38, 38),
    gender_male: Color32::from_rgb(70, 120, 175),
    gender_female: Color32::from_rgb(196, 100, 138),
};

static IS_DARK: AtomicBool = AtomicBool::new(true);

/// Colors of the current theme.
pub fn p() -> &'static Palette {
    if IS_DARK.load(Ordering::Relaxed) {
        &DARK
    } else {
        &LIGHT
    }
}

pub fn is_dark() -> bool {
    IS_DARK.load(Ordering::Relaxed)
}

/// Colors of a given theme (not necessarily the current one): the export may use either.
pub fn palette(dark: bool) -> &'static Palette {
    if dark {
        &DARK
    } else {
        &LIGHT
    }
}

fn settings_file() -> Option<std::path::PathBuf> {
    Some(crate::i18n::settings_dir()?.join("theme.txt"))
}

/// Reads the saved choice (dark by default). Call once at startup, before `apply`.
pub fn init() {
    let light = settings_file()
        .and_then(|f| std::fs::read_to_string(f).ok())
        .map_or(false, |s| s.trim() == "light");
    IS_DARK.store(!light, Ordering::Relaxed);
}

/// Switches the theme, remembers it and restyles the whole UI.
pub fn set_dark(ctx: &egui::Context, dark: bool) {
    IS_DARK.store(dark, Ordering::Relaxed);
    if let Some(f) = settings_file() {
        if let Some(dir) = f.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        let _ = std::fs::write(f, if dark { "dark" } else { "light" });
    }
    apply_visuals(ctx);
    ctx.request_repaint();
}

const R: f32 = 6.0;

fn wv(bg: Color32, weak: Color32, stroke: Color32, fg: Color32) -> egui::style::WidgetVisuals {
    egui::style::WidgetVisuals {
        bg_fill: bg,
        weak_bg_fill: weak,
        bg_stroke: Stroke::new(1.0_f32, stroke),
        rounding: Rounding::same(R),
        fg_stroke: Stroke::new(1.0_f32, fg),
        expansion: 0.0,
    }
}

pub fn apply(ctx: &egui::Context) {
    set_fonts(ctx, crate::i18n::current());
    apply_visuals(ctx);
}

/// Colors and spacing only (no font reload), so it is cheap to call on a theme switch.
fn apply_visuals(ctx: &egui::Context) {
    let c = p();
    let mut v = if is_dark() { egui::Visuals::dark() } else { egui::Visuals::light() };
    v.override_text_color = None;
    v.panel_fill = c.bg;
    v.window_fill = c.surface;
    v.extreme_bg_color = c.bg; // canvas and input fields
    v.faint_bg_color = c.faint;
    v.code_bg_color = c.surface2;
    v.window_stroke = Stroke::new(1.0_f32, c.border);
    v.window_rounding = Rounding::same(10.0);
    v.menu_rounding = Rounding::same(8.0);
    v.window_shadow.color = Color32::TRANSPARENT;
    v.popup_shadow.color = Color32::TRANSPARENT;
    v.hyperlink_color = c.text;
    v.error_fg_color = c.danger;
    v.selection.bg_fill = c.select_bg;
    v.selection.stroke = Stroke::new(1.0_f32, c.text);

    let strong = if is_dark() { Color32::WHITE } else { Color32::BLACK };
    v.widgets.noninteractive = wv(c.surface, c.surface, c.border, c.text_dim);
    v.widgets.inactive = wv(c.surface2, c.surface2, c.border, c.text_dim);
    v.widgets.hovered = wv(c.hover, c.hover, c.border_hi, c.text);
    v.widgets.active = wv(c.active, c.active, c.muted, strong);
    v.widgets.open = wv(c.hover, c.hover, c.border_hi, c.text);
    ctx.set_visuals(v);

    ctx.style_mut(|s| {
        s.text_styles.insert(TextStyle::Heading, FontId::new(20.0, FontFamily::Proportional));
        s.text_styles.insert(TextStyle::Body, FontId::new(14.0, FontFamily::Proportional));
        s.text_styles.insert(TextStyle::Button, FontId::new(14.0, FontFamily::Proportional));
        s.text_styles.insert(TextStyle::Small, FontId::new(12.0, FontFamily::Proportional));
        s.spacing.item_spacing = egui::vec2(8.0, 6.0);
        s.spacing.button_padding = egui::vec2(12.0, 5.0);
        s.spacing.interact_size.y = 28.0;
        s.spacing.window_margin = egui::Margin::same(14.0);
    });
}

/// Sun (in the dark theme) or moon (in the light theme): click to switch. Returns true when clicked.
pub fn mode_toggle(ui: &mut egui::Ui) -> bool {
    let c = p();
    let (rect, resp) = ui.allocate_exact_size(egui::vec2(28.0, 20.0), egui::Sense::click());
    let painter = ui.painter();
    if resp.hovered() {
        painter.rect_filled(rect, 5.0, c.hover);
    }
    let col = if resp.hovered() { c.text } else { c.muted };
    let m = rect.center();
    if is_dark() {
        painter.circle_stroke(m, 3.5, Stroke::new(1.4_f32, col));
        for i in 0..8 {
            let a = i as f32 * std::f32::consts::FRAC_PI_4;
            let d = egui::vec2(a.cos(), a.sin());
            painter.line_segment([m + d * 5.8, m + d * 8.0], Stroke::new(1.2_f32, col));
        }
    } else {
        painter.circle_filled(m, 5.5, col);
        let bg = if resp.hovered() { c.hover } else { c.bg };
        painter.circle_filled(m + egui::vec2(2.6, -2.2), 4.6, bg);
    }
    let tip = if is_dark() { crate::i18n::tr("Светлая тема") } else { crate::i18n::tr("Тёмная тема") };
    resp.on_hover_text(tip).clicked()
}

/// Set once the language menu has been opened: its "中文" entry needs the CJK font in any language.
static CJK_REQUESTED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// Loads the CJK font on demand (the file is ~19 MB, so not at every startup).
pub fn ensure_cjk(ctx: &egui::Context, lang: crate::i18n::Lang) {
    if !CJK_REQUESTED.swap(true, std::sync::atomic::Ordering::Relaxed) {
        set_fonts(ctx, lang);
        ctx.request_repaint();
    }
}

/// Segoe UI from Windows (Cyrillic, modern look); a CJK fallback font is added for Chinese and on demand.
pub fn set_fonts(ctx: &egui::Context, lang: crate::i18n::Lang) {
    let cjk = lang == crate::i18n::Lang::Zh || CJK_REQUESTED.load(std::sync::atomic::Ordering::Relaxed);
    ctx.set_fonts(font_definitions(cjk));
}

/// The fonts of the interface (also used by the PDF/PNG export to turn text into outlines).
pub fn font_definitions(cjk: bool) -> egui::FontDefinitions {
    let mut fonts = egui::FontDefinitions::default();
    if let Ok(bytes) = std::fs::read(crate::platform::system_font("segoeui.ttf")) {
        fonts.font_data.insert("segoe".to_owned(), egui::FontData::from_owned(bytes));
        if let Some(list) = fonts.families.get_mut(&FontFamily::Proportional) {
            list.insert(0, "segoe".to_owned());
        }
    }
    if cjk {
        for name in ["msyh.ttc", "msyh.ttf", "simsun.ttc", "simhei.ttf"] {
            if let Ok(bytes) = std::fs::read(crate::platform::system_font(name)) {
                fonts.font_data.insert("cjk".to_owned(), egui::FontData::from_owned(bytes));
                for family in [FontFamily::Proportional, FontFamily::Monospace] {
                    if let Some(list) = fonts.families.get_mut(&family) {
                        list.push("cjk".to_owned());
                    }
                }
                break;
            }
        }
    }
    fonts
}

/// Светлая основная кнопка (чёрный текст на белом).
pub fn primary_button(ui: &mut egui::Ui, text: &str) -> egui::Response {
    ui.add(
        egui::Button::new(egui::RichText::new(text).color(p().on_accent).strong())
            .fill(p().accent)
            .stroke(Stroke::NONE),
    )
}

/// Рамка панелей-баров: чёрный фон, поля.
pub fn bar_frame(h: f32, v: f32) -> egui::Frame {
    egui::Frame::none().fill(p().bg).inner_margin(egui::Margin::symmetric(h, v))
}
