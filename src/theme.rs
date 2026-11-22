//! Тема: OLED-чёрный, строгая монохромная палитра с тонкими границами (в духе Resend).

use eframe::egui::{self, Color32, FontFamily, FontId, Rounding, Stroke, TextStyle};

pub const BG: Color32 = Color32::from_rgb(0, 0, 0);
pub const SURFACE: Color32 = Color32::from_rgb(10, 10, 10);
pub const SURFACE2: Color32 = Color32::from_rgb(20, 20, 20);
pub const BORDER: Color32 = Color32::from_rgb(34, 34, 34);
pub const BORDER_HI: Color32 = Color32::from_rgb(70, 70, 76);
pub const TEXT: Color32 = Color32::from_rgb(237, 237, 237);
pub const TEXT_DIM: Color32 = Color32::from_rgb(200, 200, 200);
pub const MUTED: Color32 = Color32::from_rgb(140, 140, 140);
/// Основной акцент (кнопки, рамка выбранной карточки).
pub const ACCENT: Color32 = Color32::from_rgb(237, 237, 237);
/// Фон выбранной строки/вкладки.
pub const SELECT_BG: Color32 = Color32::from_rgb(38, 38, 42);
pub const DANGER: Color32 = Color32::from_rgb(229, 72, 77);
/// Нижняя полоса карточки по полу: приглушённые голубой и розовый.
pub const GENDER_MALE: Color32 = Color32::from_rgb(98, 148, 196);
pub const GENDER_FEMALE: Color32 = Color32::from_rgb(200, 128, 158);

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

    let mut v = egui::Visuals::dark();
    v.override_text_color = None;
    v.panel_fill = BG;
    v.window_fill = SURFACE;
    v.extreme_bg_color = BG; // фон схемы и полей ввода
    v.faint_bg_color = Color32::from_rgb(14, 14, 14);
    v.code_bg_color = SURFACE2;
    v.window_stroke = Stroke::new(1.0_f32, BORDER);
    v.window_rounding = Rounding::same(10.0);
    v.menu_rounding = Rounding::same(8.0);
    v.window_shadow.color = Color32::TRANSPARENT;
    v.popup_shadow.color = Color32::TRANSPARENT;
    v.hyperlink_color = TEXT;
    v.error_fg_color = DANGER;
    v.selection.bg_fill = SELECT_BG;
    v.selection.stroke = Stroke::new(1.0_f32, TEXT);

    v.widgets.noninteractive = wv(SURFACE, SURFACE, BORDER, TEXT_DIM);
    v.widgets.inactive = wv(SURFACE2, SURFACE2, BORDER, TEXT_DIM);
    v.widgets.hovered = wv(Color32::from_rgb(30, 30, 30), Color32::from_rgb(30, 30, 30), BORDER_HI, TEXT);
    v.widgets.active = wv(Color32::from_rgb(42, 42, 42), Color32::from_rgb(42, 42, 42), MUTED, Color32::WHITE);
    v.widgets.open = wv(Color32::from_rgb(30, 30, 30), Color32::from_rgb(30, 30, 30), BORDER_HI, TEXT);
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

/// Segoe UI from Windows (Cyrillic, modern look); a CJK fallback font is added only for Chinese.
pub fn set_fonts(ctx: &egui::Context, lang: crate::i18n::Lang) {
    let mut fonts = egui::FontDefinitions::default();
    if let Ok(bytes) = std::fs::read(r"C:\Windows\Fonts\segoeui.ttf") {
        fonts.font_data.insert("segoe".to_owned(), egui::FontData::from_owned(bytes));
        if let Some(list) = fonts.families.get_mut(&FontFamily::Proportional) {
            list.insert(0, "segoe".to_owned());
        }
    }
    if lang == crate::i18n::Lang::Zh {
        for name in ["msyh.ttc", "msyh.ttf", "simsun.ttc", "simhei.ttf"] {
            if let Ok(bytes) = std::fs::read(format!(r"C:\Windows\Fonts\{name}")) {
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
    ctx.set_fonts(fonts);
}

/// Светлая основная кнопка (чёрный текст на белом).
pub fn primary_button(ui: &mut egui::Ui, text: &str) -> egui::Response {
    ui.add(
        egui::Button::new(egui::RichText::new(text).color(Color32::BLACK).strong())
            .fill(ACCENT)
            .stroke(Stroke::NONE),
    )
}

/// Рамка панелей-баров: чёрный фон, поля.
pub fn bar_frame(h: f32, v: f32) -> egui::Frame {
    egui::Frame::none().fill(BG).inner_margin(egui::Margin::symmetric(h, v))
}
