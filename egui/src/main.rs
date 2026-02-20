mod api;
mod app;
mod config;
mod model;
mod streaming;
mod theme;
mod ui;

use app::ChatApp;

fn main() -> eframe::Result<()> {
    eprintln!("[rust-chat-egui] starting...");

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_title("Morphe Chat"),
        ..Default::default()
    };

    eframe::run_native(
        "Morphe Chat",
        options,
        Box::new(|cc| Ok(Box::new(ChatApp::new(cc)))),
    )
}
