use iced::{
    alignment::Vertical,
    widget::{Row, row, text, text_input},
};

use super::{
    image_format::{Bpp, ImageFormat},
    pixel_format::PixelFormat,
};
use crate::SPACING;

#[derive(Debug, Clone)]
pub enum Message {
    PickFile,
    TextInputChanged(TextInput, String),
    PixelFormatChanged(PixelFormat),
    OrderChanged(String),
    IgnoreAlphaChanged(bool),
    ImageFormatChanged(ImageFormat),
    PaletteBppChanged(Bpp),
    ProcessImage,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextInput {
    Width,
    Height,
    Offset,
    PaletteOffset,
}

impl TextInput {
    pub fn view(&self, label: &'static str, input: &str) -> Row<Message> {
        let label = text(label).width(50);
        let input = text_input("", input)
            .on_input(|new_value| Message::TextInputChanged(*self, new_value))
            .width(80);

        row![label, input]
            .spacing(SPACING)
            .align_y(Vertical::Center)
    }
}
