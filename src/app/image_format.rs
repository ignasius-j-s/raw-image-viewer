use iced::widget::{Column, Row, column, radio, row, text};

use super::message::{Message, TextInput};
use crate::SPACING;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum ImageFormat {
    #[default]
    Linear,
    Indexed,
}

impl ImageFormat {
    pub fn view(&self) -> Row<Message> {
        let label = text("Image Format:");
        let linear = radio(
            "Linear",
            Self::Linear,
            Some(*self),
            Message::ImageFormatChanged,
        );
        let linear_indexed = radio(
            "Indexed",
            Self::Indexed,
            Some(*self),
            Message::ImageFormatChanged,
        );

        row![label, linear, linear_indexed].spacing(SPACING)
    }

    pub fn use_palette(&self) -> bool {
        *self == Self::Indexed
    }
}

#[derive(Debug, Clone, Default)]
pub struct PaletteInfo {
    pub offset: String,
    pub bpp: Bpp,
}

impl PaletteInfo {
    pub fn view(&self, show: bool) -> Option<Column<Message>> {
        if !show {
            return None;
        }

        let pal_view = TextInput::PaletteOffset.view("Palette offset:", &self.offset);
        let bpp_view = self.bpp.view();

        column![pal_view, bpp_view].spacing(SPACING).into()
    }

    pub fn color_count(&self) -> usize {
        self.bpp.color_count()
    }

    pub fn offset(&self) -> Result<usize, std::num::ParseIntError> {
        self.offset.parse::<usize>()
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Bpp {
    Bpp4,
    #[default]
    Bpp8,
}

impl Bpp {
    fn view(&self) -> Row<Message> {
        let bpp4 = radio("4bpp", Self::Bpp4, Some(*self), Message::PaletteBppChanged);
        let bpp8 = radio("8bpp", Self::Bpp8, Some(*self), Message::PaletteBppChanged);

        row![bpp4, bpp8].spacing(SPACING)
    }

    fn color_count(&self) -> usize {
        match self {
            Bpp::Bpp4 => 16,
            Bpp::Bpp8 => 256,
        }
    }
}
