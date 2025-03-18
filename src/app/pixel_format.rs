use iced::{
    alignment::Vertical,
    widget::{Row, combo_box, row, text, text_input},
};

use super::message::Message;
use crate::SPACING;
use PixelFormat::*;

#[derive(Debug)]
pub struct PixelFormatState {
    pub state: combo_box::State<PixelFormat>,
    pub selected: PixelFormat,
    pub component_order: String,
}

impl Default for PixelFormatState {
    fn default() -> Self {
        let default = PixelFormat::default();
        Self {
            state: combo_box::State::new(PixelFormat::all()),
            selected: default,
            component_order: default.default_order(),
        }
    }
}

impl PixelFormatState {
    pub fn view(&self) -> Row<Message> {
        let label = text("Format:").width(50);
        let combo_box = combo_box(
            &self.state,
            "",
            Some(&self.selected),
            Message::PixelFormatChanged,
        )
        .width(80);

        let order: Option<Row<Message>> = if self.is_orderable() {
            let label = text("Order:").width(50);
            let input = text_input("", &self.component_order)
                .width(80)
                .on_input(Message::OrderChanged);

            row![label, input]
                .spacing(SPACING)
                .align_y(Vertical::Center)
                .into()
        } else {
            None
        };

        row![label, combo_box]
            .push_maybe(order)
            .spacing(SPACING)
            .align_y(Vertical::Center)
    }

    pub fn is_orderable(&self) -> bool {
        self.selected.is_orderable()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PixelFormat {
    #[default]
    RGBA8888,
    RGB888,

    RGBA4444,
    RGBA5551,
    RGB565,

    R8,
    G8,
    B8,
    L8,
}

impl PixelFormat {
    fn all() -> Vec<Self> {
        vec![RGBA8888, RGB888, RGBA4444, RGBA5551, RGB565, R8, G8, B8, L8]
    }

    fn is_orderable(&self) -> bool {
        match self {
            RGBA8888 | RGB888 | RGBA4444 | RGBA5551 | RGB565 => true,
            _ => false,
        }
    }

    pub fn use_alpha(&self) -> bool {
        match self {
            RGBA8888 | RGBA4444 | RGBA5551 => true,
            _ => false,
        }
    }

    pub fn default_order(&self) -> String {
        match self {
            RGBA8888 | RGBA4444 | RGBA5551 => String::from("RGBA"),
            RGB888 | RGB565 => String::from("RGB"),
            _ => String::new(),
        }
    }

    pub fn valid_order(&self, order: &str) -> Option<Vec<char>> {
        let order: Vec<char> = order.to_ascii_lowercase().chars().collect();

        match self {
            RGBA8888 | RGBA4444 | RGBA5551 => {
                if order.len() == 4 && ['r', 'g', 'b', 'a'].iter().all(|chr| order.contains(chr)) {
                    return Some(order);
                }
            }

            RGB888 | RGB565 => {
                if order.len() == 3 && ['r', 'g', 'b'].iter().all(|chr| order.contains(chr)) {
                    return Some(order);
                }
            }
            _ => return Some(vec![]),
        }

        return None;
    }

    pub fn bytes_per_pixel(&self) -> usize {
        match self {
            RGBA8888 => 4,
            RGB888 => 3,
            RGBA4444 | RGBA5551 | RGB565 => 2,
            R8 | G8 | B8 | L8 => 1,
        }
    }
}

impl std::fmt::Display for PixelFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

pub fn rgba_order(order: &Vec<char>) -> Result<(usize, usize, usize, usize), String> {
    let err_message = || String::from("invalid rgba order");

    Ok((
        order
            .iter()
            .position(|c| *c == 'r')
            .ok_or_else(err_message)?,
        order
            .iter()
            .position(|c| *c == 'g')
            .ok_or_else(err_message)?,
        order
            .iter()
            .position(|c| *c == 'b')
            .ok_or_else(err_message)?,
        order
            .iter()
            .position(|c| *c == 'a')
            .ok_or_else(err_message)?,
    ))
}

pub fn rgb_order(order: &Vec<char>) -> Result<(usize, usize, usize), String> {
    let err_message = || String::from("invalid rgb order");

    Ok((
        order
            .iter()
            .position(|c| *c == 'r')
            .ok_or_else(err_message)?,
        order
            .iter()
            .position(|c| *c == 'g')
            .ok_or_else(err_message)?,
        order
            .iter()
            .position(|c| *c == 'b')
            .ok_or_else(err_message)?,
    ))
}
