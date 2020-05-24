use crate::cpu::mem::{MemoryRegister, Memory};
use crate::cpu::CPU;
use sdl2::Sdl;
use sdl2::VideoSubsystem;
use sdl2::render::WindowCanvas;
use sdl2::rect::{Point, Rect};
use crate::util::{check_lcdc_bit, color_to_sdl};
use sdl2::pixels::Color;
use crate::cpu::int::Interrupt;

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum GbColor {
    White,
    LightGray,
    DarkGray,
    Black,
}

pub enum BGTileIndexingMethod {
    Unsigned8000,
    Signed8800,
}


pub struct VideoDrv {
    video: VideoSubsystem,
    canvas: WindowCanvas,
    bg_palette: [GbColor; 4],
    sprite0_palette: [GbColor; 3],
    sprite1_palette: [GbColor; 3],
    hblank_acc: u16,
    vblank_acc: u16,
    tile_buffer: Vec<(u8, u16)>,
    scale_factor: i32,
    disabled: bool,
}

impl VideoDrv {
    pub fn new(sdl: &Sdl) -> VideoDrv {
        let scale_factor = 4;
        let video = sdl.video().unwrap();
        let mut wnd = video.window("gbemu", 160 * scale_factor, 144 * scale_factor).borderless().position_centered().build().unwrap();
        wnd.show();
        let mut canvas = wnd.into_canvas().present_vsync().build().unwrap();
        canvas.set_draw_color(Color::WHITE);
        canvas.clear();
        canvas.present();
        VideoDrv {
            video,
            canvas,
            bg_palette: [GbColor::White; 4],
            sprite0_palette: [GbColor::White; 3],
            sprite1_palette: [GbColor::White; 3],
            hblank_acc: 0,
            vblank_acc: 0,
            tile_buffer: Vec::new(),
            scale_factor: scale_factor as i32,
            disabled: true,
        }
    }

    /// Scans a line
    /// If this function returns true, the caller should dispatch INT $40 (VBLANK)
    pub fn tick(&mut self, mem: &mut Memory) -> bool {
        if self.hblank_acc != 0 {
            self.hblank_acc -= 1;
            return false;
        }
        if self.vblank_acc != 0 {
            if self.vblank_acc == 1 {
                self.canvas.present();
            }
            self.hblank_acc = 456;
            self.vblank_acc -= 1;
            if self.vblank_acc == 0 {
                // Reset LY if this is the end of the vblank period
                mem.set_register(MemoryRegister::LY, 0);
            } else {
                // Increment LY if there's still blanking to go
                let l = mem.get_register(MemoryRegister::LY);
                mem.set_register(MemoryRegister::LY, l + 1);
            }
            return false;
        }
        let lcdc = mem.get_register(MemoryRegister::LCDC);
        let bgp = mem.get_register(MemoryRegister::BGP);

        // LCD is disabled
        if !check_lcdc_bit(lcdc, 7) {
            if !self.disabled {
                // Clear the screen and exit
                self.canvas.set_draw_color(Color::WHITE);
                self.canvas.clear();
                self.canvas.present();
                self.disabled = true;
            }
            return false;
        }

        self.disabled = false;

        let line = mem.get_register(MemoryRegister::LY);

        self.update_bg_palette(bgp);

        // Get state from mmap registers. Offset of the screen, indexing mode, and tile map base address.
        let scy = mem.get_register(MemoryRegister::SCY) as u16;
        let scx = mem.get_register(MemoryRegister::SCX) as u16;
        let tile_map_base = if check_lcdc_bit(lcdc, 3) { 0x9800 } else { 0x9C00 };
        let indexing_mode = if check_lcdc_bit(lcdc, 4) { BGTileIndexingMethod::Unsigned8000 } else { BGTileIndexingMethod::Signed8800 };

        // Find the address for the start of this scanline
        let first_tile = 4 * scy + scx / 8;

        if line % 8 == 0 {
            self.tile_buffer.clear();
            // Push all the pending tiles to be handled as we continue scanning
            for tile in first_tile..first_tile + 20 {
                self.tile_buffer.push((line, tile));
            }
        }

        for (ly, map_addr) in self.tile_buffer.iter() {

            // Get the tile from the tile map
            let n = mem.get_addr(tile_map_base + *map_addr);
            let tile = match indexing_mode {
                BGTileIndexingMethod::Unsigned8000 => 0x8000 + n as u16,
                BGTileIndexingMethod::Signed8800 => (0x9000 + n as i32) as u16,
            };

            // Figure out the offset into the tile, and get the 2 bytes for this line.
            let line_start = tile + 2 * (line - *ly) as u16;
            let t1 = mem.get_addr(line_start);
            let t2 = mem.get_addr(line_start + 1);

            let mut line_data = [GbColor::White; 8];

            // Go through the bytes for this line of tile data and get the color data from it
            // store the color data in line_data.
            for (j, i) in (0..8).rev().enumerate() {
                let low = t1 >> i & 1;
                let high = t2 >> i & 1;
                let c = self.bg_palette[(low | high << 1) as usize];
                line_data[j] = c;
            }


            // Draw each pixel in line_data
            for (i, c) in line_data.iter().enumerate() {
                self.canvas.set_draw_color(color_to_sdl(*c));
                let line_inset = (*map_addr as u16 - first_tile);
                if self.scale_factor == 1 {
                    let pt = Point::new(8 * line_inset as i32 + i as i32, line as i32);
                    self.canvas.draw_point(pt).unwrap();
                } else {
                    let rect = Rect::new(self.scale_factor * (8 * line_inset as i32 + i as i32), self.scale_factor * line as i32, self.scale_factor as u32, self.scale_factor as u32);
                    self.canvas.fill_rect(rect).unwrap();
                }
            }
        }


        // Update LY to hold the next line that will be scanned.
        return if line == 144 {
            self.vblank_acc = 10;
            mem.set_register(MemoryRegister::LY, line + 1);
            self.tile_buffer.clear();
            true
        } else {
            self.hblank_acc = 204;
            mem.set_register(MemoryRegister::LY, line + 1);
            false
        }
    }

    /// Updates the internal palette for background colours based on
    /// the contents of the Background Palette (BGP) register
    fn update_bg_palette(&mut self, bgp: u8) {
        let mask = 0b11;

        for i in 0..4 {
            let mask = mask << 2 * i as u8;
            self.bg_palette[i] = match (bgp & mask) >> 2 * i as u8 {
                0 => GbColor::White,
                1 => GbColor::LightGray,
                2 => GbColor::DarkGray,
                3 => GbColor::Black,
                _ => unreachable!(),
            };
        }
    }

    fn update_sprite_palettes(&mut self, obp0: u8, obp1: u8) {
        let mut mask = 0b11 << 2;

        for i in 0..3 {
            mask <<= 2 * i as u8;
            self.sprite0_palette[i] = match (obp0 & mask) >> (2 * i as u8 + 2) {
                0 => GbColor::White,
                1 => GbColor::LightGray,
                2 => GbColor::DarkGray,
                3 => GbColor::Black,
                _ => unreachable!(),
            };

            self.sprite1_palette[i] = match (obp1 & mask) >> (2 * i as u8 + 2) {
                0 => GbColor::White,
                1 => GbColor::LightGray,
                2 => GbColor::DarkGray,
                3 => GbColor::Black,
                _ => unreachable!(),
            };
        }
    }

}
