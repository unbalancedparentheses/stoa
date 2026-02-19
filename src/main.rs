mod api;
mod app;
mod config;
mod model;
mod widgets;

pub use app::App;
pub use app::live_design;
use makepad_widgets::*;

app_main!(App);

fn main() {
    app_main();
}
