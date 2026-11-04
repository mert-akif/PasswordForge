mod app;
mod generator;
mod typing;
mod theme;

use app::PasswordApp;
use eframe::egui;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([560.0, 560.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Password Forge",
        options,
        Box::new(|_cc| {
            Ok(Box::new(PasswordApp::default()))
        }),
    )
}