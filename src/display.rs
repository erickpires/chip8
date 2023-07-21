pub(crate) const DISPLAY_WIDTH: usize = 64;
pub(crate) const DISPLAY_HEIGHT: usize = 32;

const PIXEL_FADE_RATE: u8 = 8;
pub(crate) struct Display {
    // TODO: Replacing this array with a Vec would
    // enable us to easily implement multiple screen sizes.
    pub(crate) data: [u8; DISPLAY_WIDTH * DISPLAY_HEIGHT],
}

impl Display {
    pub(crate) const fn new() -> Self {
        Self {
            data: [0; DISPLAY_WIDTH * DISPLAY_HEIGHT]
        }
    }

    pub(crate) fn clear(&mut self) {
        self.data = [0; DISPLAY_WIDTH * DISPLAY_HEIGHT]
    }

    pub(crate) fn fade_pixels(&mut self) {
        for pixel in &mut self.data {
            // TODO: We probably want to fade the pixels following an exponetial curve, not a linear one.
            if *pixel > 0 && *pixel < 0xFF {
                *pixel = if *pixel >= PIXEL_FADE_RATE { *pixel - PIXEL_FADE_RATE } else { 0 };
            }
        }
    }

    pub(crate) fn draw_sprite(&mut self, x_coord: u8, y_coord: u8, sprite: &[u8]) -> bool {
        let mut has_colision = false;
        
        let mut y_index = (y_coord as usize) % DISPLAY_HEIGHT;
        for sprite_line in sprite {
            if y_index >= DISPLAY_HEIGHT { break; }

            let mut x_index = (x_coord as usize) % DISPLAY_WIDTH;
            for column_index in 0..8u8 {
                if x_index >= DISPLAY_WIDTH { break; }

                let pixel_mask = 0x80 >> column_index;
                if sprite_line & pixel_mask != 0 {
                    has_colision |= self.flip_pixel_at(x_index, y_index);
                }

                x_index += 1;
            }

            y_index += 1;
        }

        has_colision
    }

    fn flip_pixel_at(&mut self, x: usize, y: usize) -> bool {
        let data_index = y * DISPLAY_WIDTH + x;

        if self.data[data_index] == 0xFF {
            self.data[data_index] = 0xFF - PIXEL_FADE_RATE;

            true
        } else {
            self.data[data_index] = 0xFF;

            false
        }
    }
}

impl Default for Display {
    fn default() -> Self {
        Self::new()
    }
}