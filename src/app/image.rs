use std::{
    fs::File,
    io::{SeekFrom::Start, prelude::*},
    slice::ChunksExact,
};

use iced::widget::image::Handle;

use super::pixel_format::{Endian, PixelFormat};
use super::{App, image_format::Bpp};

pub struct Image;

impl Image {
    fn new_handle(width: u32, height: u32, rgba: Vec<u8>) -> Handle {
        Handle::from_rgba(width, height, rgba)
    }

    pub fn linear(
        app: &App,
        mut file: File,
        w: usize,
        h: usize,
        offset: usize,
    ) -> Result<Handle, String> {
        let pixel_format = app.pixel_format.selected;
        let pixel_count = w * h;
        let bytes_per_pixel = pixel_format.bytes_per_pixel();

        let mut pixel_data = vec![0; pixel_count * bytes_per_pixel];
        file.seek(Start(offset as _))
            .map_err(|err| err.to_string())?;
        file.read_exact(&mut pixel_data)
            .map_err(|err| format!("failed to fill pixel data buffer. {}", err.kind()))?;

        let pixel_chunks = pixel_data.chunks_exact(bytes_per_pixel);
        let mut rgba = vec![0; w * h * 4];
        fill_rgba(app, &mut rgba, pixel_chunks)?;

        Ok(Self::new_handle(w as _, h as _, rgba))
    }

    pub fn indexed(
        app: &App,
        mut file: File,
        w: usize,
        h: usize,
        offset: usize,
    ) -> Result<Handle, String> {
        let palette = &app.palette;
        let palette_offset = palette.offset().map_err(|_| "palette offset is empty")?;

        let pixel_format = app.pixel_format.selected;
        let color_count = palette.color_count();
        let bytes_per_color = pixel_format.bytes_per_pixel();

        let mut palette_data = vec![0; color_count * bytes_per_color];
        file.seek(Start(palette_offset as _))
            .map_err(|err| err.to_string())?;
        file.read_exact(&mut palette_data)
            .map_err(|err| format!("failed to fill palette data buffer. {}", err.kind()))?;

        let color_chunks = palette_data.chunks_exact(bytes_per_color);
        let mut palette_rgba = vec![0; color_count * 4];
        fill_rgba(app, &mut palette_rgba, color_chunks)?;

        let mut pixel_data = match palette.bpp {
            Bpp::Bpp4 => vec![0; w * h / 2],
            Bpp::Bpp8 => vec![0; w * h],
        };
        file.seek(Start(offset as _))
            .map_err(|err| err.to_string())?;
        file.read_exact(&mut pixel_data)
            .map_err(|err| format!("failed to fill pixel data buffer. {}", err.kind()))?;

        let mut rgba = vec![0; w * h * 4];
        match palette.bpp {
            Bpp::Bpp4 => {
                for (i, pixels) in pixel_data.into_iter().enumerate() {
                    let src1 = (pixels & 0xF) as usize * 4;
                    let src2 = ((pixels & 0xF0) >> 4) as usize * 4;
                    let dst1 = i * 2 * 4;
                    let dst2 = dst1 + 4;

                    rgba[dst1..dst1 + 4].clone_from_slice(&palette_rgba[src1..src1 + 4]);
                    rgba[dst2..dst2 + 4].clone_from_slice(&palette_rgba[src2..src2 + 4]);
                }
            }
            Bpp::Bpp8 => {
                for (i, pixel) in pixel_data.into_iter().enumerate() {
                    let src = pixel as usize * 4;
                    let dst = i * 4;

                    rgba[dst..dst + 4].clone_from_slice(&palette_rgba[src..src + 4]);
                }
            }
        }

        Ok(Self::new_handle(w as _, h as _, rgba))
    }

    pub fn tiled(
        app: &App,
        mut file: File,
        w: usize,
        h: usize,
        offset: usize,
    ) -> Result<Handle, String> {
        let tile_w = app.tile.width().map_err(|_| "tile width is empty")?;
        let tile_h = app.tile.height().map_err(|_| "tile height is empty")?;

        if w % tile_w != 0 {
            return Err("width is not divisible by tile width".to_owned());
        }
        if h % tile_h != 0 {
            return Err("height is not divisible by tile height".to_owned());
        }

        let tile_row = w / tile_w;
        let tile_col = h / tile_h;
        let tile_count = tile_row * tile_col;

        let mut tiles = Vec::with_capacity(tile_count);

        let pixel_format = app.pixel_format.selected;
        let pixel_count = tile_w * tile_h;
        let bytes_per_pixel = pixel_format.bytes_per_pixel();

        let mut pixel_datas = vec![0; pixel_count * bytes_per_pixel * tile_count];
        file.seek(Start(offset as _))
            .map_err(|err| err.to_string())?;
        file.read_exact(&mut pixel_datas)
            .map_err(|err| format!("failed to fill pixel data buffer. {}", err.kind()))?;

        for pixel_data in pixel_datas.chunks_exact(pixel_count * bytes_per_pixel) {
            let mut tile_rgba = vec![0; tile_w * tile_h * 4];
            let chunks = pixel_data.chunks_exact(bytes_per_pixel);

            fill_rgba(app, &mut tile_rgba, chunks)?;

            tiles.push(tile_rgba);
        }

        let mut rgba = vec![0; w * h * 4];

        for y in 0..h {
            for x in 0..w {
                let tile_x = x / tile_w;
                let tile_y = y / tile_h;
                let tile = &tiles[tile_y * tile_row + tile_x];

                let src = ((y % tile_h) * tile_w + (x % tile_w)) * 4;
                let dst = (y * w + x) * 4;

                rgba[dst..dst + 4].copy_from_slice(&tile[src..src + 4]);
            }
        }

        Ok(Self::new_handle(w as _, h as _, rgba))
    }
}

fn fill_rgba(app: &App, rgba: &mut [u8], chunks: ChunksExact<u8>) -> Result<(), String> {
    use super::pixel_format::{rgb_order, rgba_order};

    let pixel_format = app.pixel_format.selected;
    let Some(order) = pixel_format.valid_order(&app.pixel_format.component_order) else {
        return Err("invalid component order".into());
    };

    match pixel_format {
        PixelFormat::RGBA8888 => {
            let (r_i, g_i, b_i, a_i) = rgba_order(&order)?;

            for (i, chunk) in chunks.enumerate() {
                let a = if app.ignore_alpha { 255 } else { chunk[a_i] };

                rgba[i * 4] = chunk[r_i];
                rgba[i * 4 + 1] = chunk[g_i];
                rgba[i * 4 + 2] = chunk[b_i];
                rgba[i * 4 + 3] = a;
            }
        }
        PixelFormat::RGB888 => {
            let (r_i, g_i, b_i) = rgb_order(&order)?;
            let a = 255;

            for (i, chunk) in chunks.enumerate() {
                rgba[i * 4] = chunk[r_i];
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

                rgba[i * 4] = color[r_i];
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

                rgba[i * 4] = color[r_i];
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

                rgba[i * 4] = color[r_i];
                rgba[i * 4 + 1] = color[g_i];
                rgba[i * 4 + 2] = color[b_i];
                rgba[i * 4 + 3] = a;
            }
        }
        PixelFormat::R8 => {
            for (i, chunk) in chunks.enumerate() {
                rgba[i * 4] = chunk[0];
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
                rgba[i * 4] = chunk[0];
                rgba[i * 4 + 1] = chunk[0];
                rgba[i * 4 + 2] = chunk[0];
                rgba[i * 4 + 3] = 255;
            }
        }
    }

    Ok(())
}
