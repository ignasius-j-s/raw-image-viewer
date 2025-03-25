use iced::{
    Element,
    widget::{Row, column, radio, row},
};

use super::message::{Message, TextInput};
use crate::SPACING;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum ImageFormat {
    #[default]
    Linear,
    Indexed,
    Tiled,
}

impl ImageFormat {
    pub fn view(&self) -> Row<Message> {
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
        let tiled = radio(
            "Tiled",
            Self::Tiled,
            Some(*self),
            Message::ImageFormatChanged,
        );

        row![linear, linear_indexed, tiled].spacing(SPACING)
    }
}

#[derive(Debug, Clone)]
pub struct PaletteInfo {
    pub offset: String,
    pub bpp: Bpp,
}

impl Default for PaletteInfo {
    fn default() -> Self {
        Self {
            offset: 0.to_string(),
            bpp: Default::default(),
        }
    }
}

impl PaletteInfo {
    pub fn view(&self) -> Element<Message> {
        let pal_view = TextInput::PaletteOffset.view("Palette offset:", &self.offset);
        let bpp_view = self.bpp.view();

        column![pal_view, bpp_view].spacing(SPACING).into()
    }

    pub fn color_count(&self) -> usize {
        self.bpp.color_count()
    }

    pub fn offset(&self) -> Result<usize, std::num::ParseIntError> {
        self.offset.parse()
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

#[derive(Debug, Clone)]
pub struct TileInfo {
    pub width: String,
    pub height: String,
}

impl Default for TileInfo {
    fn default() -> Self {
        Self {
            width: 2.to_string(),
            height: 2.to_string(),
        }
    }
}

impl TileInfo {
    pub fn view(&self) -> Element<Message> {
        let width = TextInput::TileWidth.view("Tile width:", &self.width);
        let height = TextInput::TileHeight.view("Tile height:", &self.height);

        row![width, height].spacing(SPACING).into()
    }

    pub fn width(&self) -> Result<usize, std::num::ParseIntError> {
        self.width.parse()
    }

    pub fn height(&self) -> Result<usize, std::num::ParseIntError> {
        self.height.parse()
    }
}
