use log::{info, debug};
use stoa::app::ChatApp;

fn main() -> iced::Result {
    env_logger::init();
    info!("starting...");

    let result = iced::application(ChatApp::new, ChatApp::update, ChatApp::view)
        .title("Stoa")
        .subscription(ChatApp::subscription)
        .theme(ChatApp::theme)
        .window_size((1200.0, 800.0))
        .run();

    debug!("exited with: {result:?}");
    result
}
