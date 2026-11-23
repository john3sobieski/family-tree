// FamTree Android viewer: skeleton (step A1). Shows an empty window to prove the build works.
#[cfg(target_os = "android")]
use winit::platform::android::activity::AndroidApp;

struct App;

impl eframe::App for App {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        eframe::egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("FamTree");
            ui.label("Android viewer skeleton (A1)");
        });
    }
}

// Entry point called by the Android runtime (NativeActivity).
#[cfg(target_os = "android")]
#[no_mangle]
fn android_main(app: AndroidApp) {
    android_logger::init_once(
        android_logger::Config::default().with_max_level(log::LevelFilter::Info),
    );
    let options = eframe::NativeOptions {
        android_app: Some(app),
        ..Default::default()
    };
    if let Err(e) = eframe::run_native(
        "FamTree",
        options,
        Box::new(|_cc| Ok(Box::new(App))),
    ) {
        log::error!("eframe failed: {e}");
    }
}
