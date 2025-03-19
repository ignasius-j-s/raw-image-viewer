use app::App;
use iced::{Settings, Size};

mod app;

const SPACING: u16 = 10;
const LABEL_WIDTH: u16 = 50;

fn main() -> Result<(), iced::Error> {
    let size = Size {
        width: 720.,
        height: 460.,
    };
    let setting = Settings {
        default_text_size: 13.0.into(),
        ..Default::default()
    };

    iced::application(App::title, App::update, App::view)
        .window_size(size)
        .theme(App::theme)
        .settings(setting)
        .subscription(App::key_subs)
        .run()
}
