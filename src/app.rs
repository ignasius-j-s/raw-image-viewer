use std::fs::File;
use std::path::{Path, PathBuf};

use iced::{
    Element, Length,
    alignment::{Horizontal, Vertical},
    keyboard::{Key, Modifiers, key::Named},
    widget::{
        Checkbox, Column, Container, Row, Stack, button, checkbox, column, container,
        horizontal_rule, horizontal_space,
        image::{FilterMethod, Handle, viewer},
        radio, row, stack, text, text_input, vertical_space,
    },
};

mod image;
mod image_format;
mod message;
mod pixel_format;

use crate::SPACING;
use image::Image;
use image_format::{ImageFormat, PaletteInfo, TileInfo};
use message::{Message, SaveFormat, TextInput};
use pixel_format::PixelFormatState;

#[derive(Debug)]
pub struct App {
    filepath: Option<PathBuf>,
    width: String,
    height: String,
    offset: String,
    pixel_format: PixelFormatState,
    ignore_alpha: bool,
    image_format: ImageFormat,
    palette: PaletteInfo,
    tile: TileInfo,
    image: Option<Handle>,
    error: Option<String>,
    filter_method: FilterMethod,
}

impl Default for App {
    fn default() -> Self {
        Self {
            filepath: None,
            width: 2.to_string(),
            height: 2.to_string(),
            offset: 0.to_string(),
            pixel_format: Default::default(),
            ignore_alpha: false,
            image_format: Default::default(),
            palette: Default::default(),
            tile: TileInfo::default(),
            image: None,
            error: None,
            filter_method: FilterMethod::Nearest,
        }
    }
}

impl App {
    pub fn title(&self) -> String {
        String::from("Raw Image Viewer")
    }

    pub fn update(&mut self, message: Message) {
        let mut process = false;
        let mut save: Option<SaveFormat> = None;

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
                        TextInput::TileWidth => self.tile.width = input,
                        TextInput::TileHeight => self.tile.height = input,
                    }
                }
            }
            Message::PixelFormatChanged(pixel_format) => {
                self.pixel_format.selected = pixel_format;
                self.pixel_format.component_order = pixel_format.default_order();
            }
            Message::OrderChanged(order) => self.pixel_format.component_order = order,
            Message::EndianChanged(endian) => self.pixel_format.endian = endian,
            Message::IgnoreAlphaChanged(val) => self.ignore_alpha = val,
            Message::ImageFormatChanged(image_format) => self.image_format = image_format,
            Message::PaletteBppChanged(bpp) => self.palette.bpp = bpp,
            Message::ProcessImage => process = true,
            Message::SaveImage(format) => save = Some(format),
            Message::FilterChanged(filter_method) => {
                self.filter_method = filter_method;
                return;
            }
        }

        if self.image.is_some() {
            process = true;
        }

        if process {
            match self.process_image() {
                Ok(handle) => {
                    self.image = Some(handle);
                    self.error = None
                }
                Err(message) => {
                    self.error = Some(message);
                }
            }
        }

        if let Some(format) = save {
            let Some(handle) = self.image.as_ref() else {
                self.error = Some("no image to save".to_string());
                return;
            };

            let Some(path) = rfd::FileDialog::new()
                .set_title("Save")
                .add_filter("", format.extension())
                .save_file()
            else {
                return;
            };

            if let Err(message) = App::save_image(handle, format, path) {
                self.error = Some(format!("failed to save image. {message}"));
            }
        }
    }

    pub fn view(&self) -> Element<Message> {
        let filepath_view = self.filepath_view();
        let dim_view = self.dimension_view();
        let offset = TextInput::Offset.view("Offset:", &self.offset);
        let pixel_format_view = self.pixel_format_view();
        let image_format_view = self.image_format_view();
        let buttons_view = self.buttons_view();
        let error_view = self.error_view();

        let image_viewer = self.image_view().width(Length::Fill);
        let left_view = column![
            filepath_view,
            dim_view,
            offset,
            pixel_format_view,
            horizontal_rule(1),
            image_format_view,
            vertical_space(),
            Column::new()
                .push_maybe(error_view)
                .push(buttons_view)
                .spacing(SPACING)
        ]
        .spacing(SPACING)
        .width(280);

        let main_view = row![left_view, image_viewer].spacing(SPACING);

        container(main_view)
            .padding(SPACING)
            .style(container::transparent)
            .into()
    }

    pub fn theme(&self) -> iced::Theme {
        iced::theme::Theme::CatppuccinMacchiato
    }

    pub fn key_subs(&self) -> iced::Subscription<Message> {
        iced::keyboard::on_key_press(Self::on_key_enter)
    }

    fn process_image(&self) -> Result<Handle, String> {
        let path = self.filepath.as_deref().ok_or("file is empty")?;
        let width: usize = self.width.parse().map_err(|_| "width is empty")?;
        let height: usize = self.height.parse().map_err(|_| "height is empty")?;
        let offset: usize = self.offset.parse().map_err(|_| "offset is empty")?;

        if width == 0 || height == 0 {
            return Err("width or height cannot be zero".into());
        }

        let file = File::open(path).map_err(|err| err.to_string())?;

        match self.image_format {
            ImageFormat::Linear => Image::linear(self, file, width, height, offset),
            ImageFormat::Indexed => Image::linear_indexed(self, file, width, height, offset),
            ImageFormat::Tiled => Image::tiled(self, file, width, height, offset),
            ImageFormat::TiledIndexed => Image::tiled_indexed(self, file, width, height, offset),
        }
    }

    fn save_image(handle: &Handle, format: SaveFormat, path: PathBuf) -> Result<(), String> {
        let Handle::Rgba {
            width,
            height,
            pixels,
            ..
        } = &handle
        else {
            unreachable!();
        };

        match format {
            SaveFormat::Rgba => {
                std::fs::write(path, pixels).map_err(|err| err.kind().to_string())?;
            }
            SaveFormat::Png => {
                let file = std::fs::File::create(path).map_err(|err| err.kind().to_string())?;
                let mut encoder = png::Encoder::new(file, *width, *height);

                encoder.set_color(png::ColorType::Rgba);
                encoder.set_depth(png::BitDepth::Eight);
                encoder
                    .write_header()
                    .and_then(|mut wr| wr.write_image_data(pixels))
                    .map_err(|err| err.to_string())?;
            }
        };

        Ok(())
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

        let view: Option<Element<Message>> = match self.image_format {
            ImageFormat::Linear => None,
            ImageFormat::Indexed => self.palette.view().into(),
            ImageFormat::Tiled => self.tile.view().into(),
            ImageFormat::TiledIndexed => {
                let tile_view = self.tile.view();
                let pal_view = self.palette.view();

                Some(column![tile_view, pal_view].spacing(SPACING).into())
            }
        };

        column![image_format_view].push_maybe(view).spacing(SPACING)
    }

    pub fn buttons_view(&self) -> Row<Message> {
        let rgba_save = button("Save (rgba)")
            .on_press(Message::SaveImage(SaveFormat::Rgba))
            .style(button::secondary);
        let png_save = button("Save (png)")
            .on_press(Message::SaveImage(SaveFormat::Png))
            .style(button::success);
        let process = button("Process").on_press(Message::ProcessImage);

        row![process, horizontal_space(), rgba_save, png_save].spacing(SPACING)
    }

    pub fn image_view(&self) -> Stack<Message> {
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

        let container = container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_y(Vertical::Center)
            .align_x(Horizontal::Center)
            .style(style);

        let filter_view = self.filter_view();

        stack([container.into(), filter_view.into()])
    }

    fn filter_view(&self) -> Container<Message> {
        let linear = radio(
            "Linear",
            FilterMethod::Linear,
            Some(self.filter_method),
            Message::FilterChanged,
        );
        let nearest = radio(
            "Nearest",
            FilterMethod::Nearest,
            Some(self.filter_method),
            Message::FilterChanged,
        );

        let content = row![linear, nearest].spacing(SPACING);

        container(content)
            .padding(SPACING)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(container::transparent)
            .align_y(Vertical::Bottom)
            .align_x(Horizontal::Center)
    }

    fn error_view(&self) -> Option<Row<Message>> {
        let message = self.error.as_ref()?;

        Some(row![text(message).style(iced::widget::text::danger)].spacing(SPACING))
    }

    fn on_key_enter(key: Key, _: Modifiers) -> Option<Message> {
        match key {
            Key::Named(Named::Enter) => Some(Message::ProcessImage),
            _ => None,
        }
    }
}
