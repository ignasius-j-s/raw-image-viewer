use iced::{
    alignment::Vertical,
    widget::{Row, image::FilterMethod, row, text, text_input},
};

use super::{
    image_format::{Bpp, ImageFormat},
    pixel_format::{Endian, PixelFormat},
};
use crate::{LABEL_WIDTH, SPACING};

#[derive(Debug, Clone)]
pub enum Message {
    PickFile,
    TextInputChanged(TextInput, String),
    PixelFormatChanged(PixelFormat),
    OrderChanged(String),
    EndianChanged(Endian),
    IgnoreAlphaChanged(bool),
    ImageFormatChanged(ImageFormat),
    PaletteBppChanged(Bpp),
    ProcessImage,
    FilterChanged(FilterMethod),
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
        let label = text(label).width(LABEL_WIDTH);
        let input = text_input("", input)
            .on_input(|new_value| Message::TextInputChanged(*self, new_value))
            .width(80);

        row![label, input]
            .spacing(SPACING)
            .align_y(Vertical::Center)
    }
}
