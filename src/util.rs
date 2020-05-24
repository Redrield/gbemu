use crate::peripherals::video::GbColor;
use sdl2::pixels::Color;

pub fn color_to_sdl(c: GbColor) -> Color {
    match c {
        GbColor::Black => Color::BLACK,
        GbColor::LightGray => Color::GRAY,
        GbColor::DarkGray => Color::RGBA(55, 55, 55, 55),
        GbColor::White => Color::WHITE,
    }
}

pub fn check_lcdc_bit(lcdc: u8, bit: u8) -> bool {
    let mask = 1 << bit;

    lcdc & mask == mask
}
