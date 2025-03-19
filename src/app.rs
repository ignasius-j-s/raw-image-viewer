use std::fs::File;
use std::io::SeekFrom::*;
use std::io::prelude::*;
use std::path::{Path, PathBuf};

use iced::{
    Element, Length,
    alignment::Vertical,
    keyboard::{Key, Modifiers, key::Named},
    widget::{
        Checkbox, Column, Container, Row, button, checkbox, column, container,
        image::{FilterMethod, Handle, viewer},
        row, text, text_input, vertical_space,
    },
};

mod image_format;
mod message;
mod pixel_format;

use crate::SPACING;
use image_format::{ImageFormat, PaletteInfo};
use message::{Message, TextInput};
use pixel_format::{Endian, PixelFormat, PixelFormatState};

#[derive(Debug, Default)]
pub struct App {
    filepath: Option<PathBuf>,
    width: String,
    height: String,
    offset: String,
    pixel_format: PixelFormatState,
    ignore_alpha: bool,
    image_format: ImageFormat,
    palette: PaletteInfo,
    image: Option<Handle>,
    error: Option<String>,
    filter_method: FilterMethod,
}

impl App {
    pub fn title(&self) -> String {
        String::from("Raw Image Viewer")
    }

    pub fn update(&mut self, message: Message) {
        match message {
            Message::PickFile => {
                let path = rfd::FileDialog::new().set_title("Open").pick_file();

                if path.is_some() {
                    self.filepath = path;
                }
            }
            Message::TextInputChanged(kind, input) => {
                if input.chars().all(char::is_numeric) || input.is_empty() {
                    match kind {
                        TextInput::Width => self.width = input,
                        TextInput::Height => self.height = input,
                        TextInput::Offset => self.offset = input,
                        TextInput::PaletteOffset => self.palette.offset = input,
                    }
                }
            }
            Message::PixelFormatChanged(pixel_format) => {
                self.pixel_format.selected = pixel_format;
                self.pixel_format.component_order = pixel_format.default_order()
            }
            Message::OrderChanged(order) => self.pixel_format.component_order = order,
            Message::EndianChanged(endian) => self.pixel_format.endian = endian,
            Message::IgnoreAlphaChanged(val) => self.ignore_alpha = val,
            Message::ImageFormatChanged(image_format) => self.image_format = image_format,
            Message::PaletteBppChanged(bpp) => self.palette.bpp = bpp,
            Message::ProcessImage => match process_image(&self) {
                Ok(handle) => {
                    self.image = Some(handle);
                    self.error = None
                }
                Err(message) => {
                    self.error = Some(message);
                }
            },
        }
    }

    pub fn view(&self) -> Element<Message> {
        let filepath_view = self.filepath_view();
        let dim_view = self.dimension_view();
        let offset = TextInput::Offset.view("Offset:", &self.offset);
        let pixel_format_view = self.pixel_format_view();
        let image_format_view = self.image_format_view();
        let process_button = self.process_button();
        let error_view = self.error_view();

        let image_viewer = self.image_view().width(Length::FillPortion(4));
        let left_view = column![
            filepath_view,
            dim_view,
            offset,
            pixel_format_view,
            image_format_view,
            vertical_space(),
            Column::new()
                .push_maybe(error_view)
                .push(process_button)
                .spacing(SPACING)
        ]
        .spacing(SPACING)
        .width(Length::FillPortion(3));

        let main_view = row![left_view, image_viewer].spacing(SPACING);

        container(main_view)
            .padding(SPACING)
            .style(container::transparent)
            .into()
    }

    pub fn theme(&self) -> iced::Theme {
        iced::theme::Theme::CatppuccinMocha
    }

    pub fn key_subs(&self) -> iced::Subscription<Message> {
        iced::keyboard::on_key_press(Self::on_key_enter)
    }
}

impl App {
    pub fn filepath_view(&self) -> Row<Message> {
        use iced::widget::text_input::Status;

        let path = self
            .filepath
            .as_deref()
            .and_then(Path::to_str)
            .unwrap_or_default();

        let label = text("File:");
        let input = text_input("", path)
            .width(Length::Fill)
            .style(|theme, _| text_input::default(theme, Status::Active));
        let button = button("...").on_press(Message::PickFile);

        row![label, input, button]
            .spacing(SPACING)
            .align_y(Vertical::Center)
    }

    pub fn dimension_view(&self) -> Row<Message> {
        row![
            TextInput::Width.view("Width:", &self.width),
            TextInput::Height.view("Height:", &self.height),
        ]
        .spacing(SPACING)
    }

    pub fn pixel_format_view(&self) -> Column<Message> {
        let row = self.pixel_format.view();

        let checkbox: Option<Checkbox<Message>> = if self.pixel_format.selected.use_alpha() {
            checkbox("Ignore alpha", self.ignore_alpha)
                .on_toggle(Message::IgnoreAlphaChanged)
                .into()
        } else {
            None
        };

        column![row].spacing(SPACING).push_maybe(checkbox)
    }

    pub fn image_format_view(&self) -> Column<Message> {
        let image_format_view = self.image_format.view();
        let palette_info_view = self.palette.view(self.image_format.use_palette());

        column![image_format_view]
            .push_maybe(palette_info_view)
            .spacing(SPACING)
    }

    pub fn process_button(&self) -> button::Button<Message> {
        button("Process").on_press(Message::ProcessImage)
    }

    pub fn image_view(&self) -> Container<Message> {
        fn style(theme: &iced::Theme) -> container::Style {
            let color = iced::Color {
                r: 0.0,
                g: 100.0 / 255.0,
                b: 0.0,
                a: 1.0,
            };

            container::Style {
                background: Some(iced::Background::Color(color)),
                ..container::rounded_box(theme)
            }
        }

        let content: Element<Message> = match &self.image {
            Some(handle) => viewer(handle.clone())
                .width(Length::Fill)
                .height(Length::Fill)
                .content_fit(iced::ContentFit::ScaleDown)
                .filter_method(self.filter_method)
                .into(),
            None => text("no preview").into(),
        };

        container(content)
            .center(Length::Shrink)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(style)
    }

    fn error_view(&self) -> Option<Row<Message>> {
        let Some(message) = self.error.as_ref() else {
            return None;
        };

        Some(row![text(message).style(iced::widget::text::danger)].spacing(SPACING))
    }

    fn on_key_enter(key: Key, _: Modifiers) -> Option<Message> {
        match key {
            Key::Named(named) => match named {
                Named::Enter => return Some(Message::ProcessImage),
                _ => (),
            },
            _ => (),
        }

        return None;
    }
}

fn process_image(app: &App) -> Result<Handle, String> {
    let path = app.filepath.as_deref().ok_or("file is empty")?;
    let width = app.width.parse::<usize>().map_err(|_| "width is empty")?;
    let height = app.height.parse::<usize>().map_err(|_| "height is empty")?;
    let offset = app.offset.parse::<usize>().map_err(|_| "offset is empty")?;

    if width == 0 || height == 0 {
        return Err("width or height cannot be zero".into());
    }

    let file = File::open(path).map_err(|err| err.to_string())?;

    match app.image_format {
        ImageFormat::Linear => linear_image(app, file, width, height, offset),
        ImageFormat::LinearIndexed => Err("TODO".into()),
    }
}

pub fn linear_image(
    app: &App,
    mut file: File,
    w: usize,
    h: usize,
    offset: usize,
) -> Result<Handle, String> {
    use pixel_format::{rgb_order, rgba_order};

    let pixel_count = w * h;
    let pixel_format = app.pixel_format.selected;
    let bytes_per_pixel = pixel_format.bytes_per_pixel();
    let mut pixel_data = vec![0; pixel_count * bytes_per_pixel];

    let Some(order) = pixel_format.valid_order(&app.pixel_format.component_order) else {
        return Err("invalid component order".into());
    };

    file.seek(Start(offset as _))
        .map_err(|err| err.to_string())?;
    file.read_exact(&mut pixel_data)
        .map_err(|err| format!("failed to fill pixel data buffer. {}", err.kind()))?;

    let mut rgba = vec![0; w * h * 4];
    let chunks = pixel_data.chunks_exact(bytes_per_pixel);
    match pixel_format {
        PixelFormat::RGBA8888 => {
            let (r_i, g_i, b_i, a_i) = rgba_order(&order)?;

            for (i, chunk) in chunks.enumerate() {
                let a = if app.ignore_alpha { 255 } else { chunk[a_i] };

                rgba[i * 4 + 0] = chunk[r_i];
                rgba[i * 4 + 1] = chunk[g_i];
                rgba[i * 4 + 2] = chunk[b_i];
                rgba[i * 4 + 3] = a;
            }
        }
        PixelFormat::RGB888 => {
            let (r_i, g_i, b_i) = rgb_order(&order)?;
            let a = 255;

            for (i, chunk) in chunks.enumerate() {
                rgba[i * 4 + 0] = chunk[r_i];
                rgba[i * 4 + 1] = chunk[g_i];
                rgba[i * 4 + 2] = chunk[b_i];
                rgba[i * 4 + 3] = a;
            }
        }
        PixelFormat::RGBA4444 => {
            let (r_i, g_i, b_i, a_i) = rgba_order(&order)?;
            let mut color = [0, 0, 0, 0];

            for (i, chunk) in chunks.enumerate() {
                let pixel = match app.pixel_format.endian {
                    Endian::LE => u16::from_le_bytes([chunk[0], chunk[1]]),
                    Endian::BE => u16::from_be_bytes([chunk[0], chunk[1]]),
                };

                color[0] = (pixel & 0xF) as u8 * 17;
                color[1] = ((pixel & 0xF0) >> 4) as u8 * 17;
                color[2] = ((pixel & 0xF00) >> 8) as u8 * 17;
                color[3] = ((pixel & 0xF000) >> 12) as u8 * 17;

                let a = if app.ignore_alpha { 255 } else { color[a_i] };

                rgba[i * 4 + 0] = color[r_i];
                rgba[i * 4 + 1] = color[g_i];
                rgba[i * 4 + 2] = color[b_i];
                rgba[i * 4 + 3] = a;
            }
        }
        PixelFormat::RGBA5551 => {
            let (r_i, g_i, b_i, a_i) = rgba_order(&order)?;
            let mut color = [0, 0, 0, 0];

            for (i, chunk) in chunks.enumerate() {
                let pixel = match app.pixel_format.endian {
                    Endian::LE => u16::from_le_bytes([chunk[0], chunk[1]]),
                    Endian::BE => u16::from_be_bytes([chunk[0], chunk[1]]),
                };

                color[0] = (pixel & 0x1F) as u8 * 8;
                color[1] = ((pixel & 0x3E0) >> 5) as u8 * 8;
                color[2] = ((pixel & 0x7C00) >> 10) as u8 * 8;
                color[3] = ((pixel & 0x8000) >> 15) as u8 * 255;

                color[0] += color[0] / 32;
                color[1] += color[1] / 32;
                color[2] += color[2] / 32;

                let a = if app.ignore_alpha { 255 } else { color[a_i] };

                rgba[i * 4 + 0] = color[r_i];
                rgba[i * 4 + 1] = color[g_i];
                rgba[i * 4 + 2] = color[b_i];
                rgba[i * 4 + 3] = a;
            }
        }
        PixelFormat::RGB565 => {
            let (r_i, g_i, b_i) = rgb_order(&order)?;
            let mut color = [0, 0, 0];
            let a = 255;

            for (i, chunk) in chunks.enumerate() {
                let pixel = match app.pixel_format.endian {
                    Endian::LE => u16::from_le_bytes([chunk[0], chunk[1]]),
                    Endian::BE => u16::from_be_bytes([chunk[0], chunk[1]]),
                };

                color[0] = (pixel & 0x1F) as u8 * 8;
                color[1] = ((pixel & 0x7E0) >> 5) as u8 * 4;
                color[2] = ((pixel & 0xF800) >> 11) as u8 * 8;

                color[0] += color[0] / 32;
                color[1] += color[1] / 64;
                color[2] += color[2] / 32;

                rgba[i * 4 + 0] = color[r_i];
                rgba[i * 4 + 1] = color[g_i];
                rgba[i * 4 + 2] = color[b_i];
                rgba[i * 4 + 3] = a;
            }
        }
        PixelFormat::R8 => {
            for (i, chunk) in chunks.enumerate() {
                rgba[i * 4 + 0] = chunk[0];
                rgba[i * 4 + 3] = 255;
            }
        }
        PixelFormat::G8 => {
            for (i, chunk) in chunks.enumerate() {
                rgba[i * 4 + 1] = chunk[0];
                rgba[i * 4 + 3] = 255;
            }
        }
        PixelFormat::B8 => {
            for (i, chunk) in chunks.enumerate() {
                rgba[i * 4 + 2] = chunk[0];
                rgba[i * 4 + 3] = 255;
            }
        }
        PixelFormat::L8 => {
            for (i, chunk) in chunks.enumerate() {
                rgba[i * 4 + 0] = chunk[0];
                rgba[i * 4 + 1] = chunk[0];
                rgba[i * 4 + 2] = chunk[0];
                rgba[i * 4 + 3] = 255;
            }
        }
    }

    Ok(Handle::from_rgba(w as _, h as _, rgba))
}
