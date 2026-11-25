// System state pushed from Java (`FamActivity.nativeEnv`): dark mode and the window insets
// (status bar, cutout, gesture/navigation bar) that overlap our content.
use eframe::egui::{self, Frame};
use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};

static DARK: AtomicBool = AtomicBool::new(true);
/// Left, top, right, bottom in pixels.
static INSETS: [AtomicI32; 4] = [AtomicI32::new(0), AtomicI32::new(0), AtomicI32::new(0), AtomicI32::new(0)];

/// Called from the JNI callback (Java UI thread).
pub fn set(dark: bool, left: i32, top: i32, right: i32, bottom: i32) {
    DARK.store(dark, Ordering::SeqCst);
    for (slot, v) in INSETS.iter().zip([left, top, right, bottom]) {
        slot.store(v.max(0), Ordering::SeqCst);
    }
}

/// Startup value read directly from Java, so the first frame already has the right theme.
pub fn init_dark(dark: bool) {
    DARK.store(dark, Ordering::SeqCst);
}

pub fn is_dark() -> bool {
    DARK.load(Ordering::SeqCst)
}

/// Insets in egui points: [left, top, right, bottom].
fn insets(ctx: &egui::Context) -> [f32; 4] {
    let k = ctx.pixels_per_point().max(0.5);
    let mut out = [0.0_f32; 4];
    for (o, slot) in out.iter_mut().zip(INSETS.iter()) {
        *o = slot.load(Ordering::SeqCst) as f32 / k;
    }
    out
}

/// Empty panels that keep every screen out of the status bar, the cutout and the navigation bar.
/// Call first in every frame, before any other panel.
pub fn show_insets(ctx: &egui::Context) {
    let [l, t, r, b] = insets(ctx);
    let frame = || Frame::none().fill(crate::theme::p().bg);
    if l > 0.5 {
        egui::SidePanel::left("inset-left")
            .resizable(false)
            .exact_width(l)
            .frame(frame())
            .show_separator_line(false)
            .show(ctx, |_| {});
    }
    if r > 0.5 {
        egui::SidePanel::right("inset-right")
            .resizable(false)
            .exact_width(r)
            .frame(frame())
            .show_separator_line(false)
            .show(ctx, |_| {});
    }
    if t > 0.5 {
        egui::TopBottomPanel::top("inset-top")
            .exact_height(t)
            .frame(frame())
            .show_separator_line(false)
            .show(ctx, |_| {});
    }
    if b > 0.5 {
        egui::TopBottomPanel::bottom("inset-bottom")
            .exact_height(b)
            .frame(frame())
            .show_separator_line(false)
            .show(ctx, |_| {});
    }
}

/// Sun (in the dark theme) or moon (in the light theme) as a 44 px touch button. True when tapped.
pub fn theme_button(ui: &mut egui::Ui) -> bool {
    let (rect, resp) = ui.allocate_exact_size(egui::vec2(44.0, 44.0), egui::Sense::click());
    let v = ui.visuals();
    let fill = if resp.is_pointer_button_down_on() { v.widgets.active.bg_fill } else { v.widgets.inactive.bg_fill };
    let painter = ui.painter();
    painter.rect(rect, 6.0, fill, v.widgets.inactive.bg_stroke);
    let col = v.widgets.inactive.fg_stroke.color;
    let stroke = egui::Stroke::new(1.8_f32, col);
    let m = rect.center();
    if crate::theme::is_dark() {
        painter.circle_stroke(m, 5.5, stroke);
        for i in 0..8 {
            let a = i as f32 * std::f32::consts::FRAC_PI_4;
            let d = egui::vec2(a.cos(), a.sin());
            painter.line_segment([m + d * 9.0, m + d * 12.5], stroke);
        }
    } else {
        painter.circle_filled(m, 9.0, col);
        painter.circle_filled(m + egui::vec2(4.0, -3.5), 7.5, fill);
    }
    resp.clicked()
}
