mod api;
mod app;
mod config;
mod db;
mod model;
mod theme;
mod ui;

use app::ChatApp;

fn main() -> iced::Result {
    eprintln!("[rust-chat] starting...");

    let result = iced::application(ChatApp::new, ChatApp::update, ChatApp::view)
        .title("Morphe Chat")
        .subscription(ChatApp::subscription)
        .theme(ChatApp::theme)
        .window_size((1200.0, 800.0))
        .run();

    eprintln!("[rust-chat] exited with: {result:?}");
    result
}
