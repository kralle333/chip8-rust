extern crate sdl2;

use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;

use crate::mem;

pub const SCREEN_WIDTH: usize = 64;
pub const SCREEN_HEIGHT: usize = 32;
pub const PIXEL_SIZE: u32 = 16;

pub struct Video {
    pixels: [u8; SCREEN_WIDTH * SCREEN_HEIGHT],
}

impl Video {
    pub fn new() -> Video {
        Video {
            pixels: [0; SCREEN_WIDTH * SCREEN_HEIGHT],
        }
    }
    pub fn clear(&mut self) {
        for i in 0..self.pixels.len() {
            self.pixels[i] = 0;
        }
    }

    pub fn draw_sprite(&mut self, x: u8, y: u8, n: u16, i: u16, mem: &mem::Memory) -> bool {
        let mut flipped = false;
        let x = x as usize;
        let y = y as usize;

        for yline in 0..n {
            let pixel = mem.get_byte(i + yline);
            for xline in 0..8 {
                if (pixel & (0x80 >> xline)) != 0 {
                    let x = (x + xline) % SCREEN_WIDTH;
                    let y = (y + yline as usize) % SCREEN_HEIGHT;
                    if self.pixels[x + y * SCREEN_WIDTH] == 1 {
                        flipped = true
                    }
                    self.pixels[x + y * SCREEN_WIDTH] ^= 1;
                }
            }
        }

        flipped
    }
    pub fn draw(&self, canvas: &mut Canvas<Window>) {
        for x in 0..SCREEN_WIDTH {
            for y in 0..SCREEN_HEIGHT {
                let color = match self.pixels[y * SCREEN_WIDTH + x] {
                    0 => Color::BLACK,
                    _ => Color::WHITE,
                };

                canvas.set_draw_color(color);
                canvas
                    .fill_rect(Rect::new(
                        (x as i32) * (PIXEL_SIZE as i32),
                        (y as i32) * (PIXEL_SIZE as i32),
                        PIXEL_SIZE,
                        PIXEL_SIZE,
                    ))
                    .expect("Unable to draw rect!");
            }
        }
    }
}
