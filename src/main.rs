mod api;
mod app;
mod config;
mod cost;
mod db;
mod diff;
mod export;
mod model;
mod theme;
mod ui;

use app::ChatApp;

fn main() -> iced::Result {
    eprintln!("[stoa] starting...");

    let result = iced::application(ChatApp::new, ChatApp::update, ChatApp::view)
        .title("Stoa")
        .subscription(ChatApp::subscription)
        .theme(ChatApp::theme)
        .window_size((1200.0, 800.0))
        .run();

    eprintln!("[stoa] exited with: {result:?}");
    result
}
