use gba::prelude::*;

const FONT_WIDTH: usize = 4;
const FONT_HEIGHT: usize = 6;
const TAB_CHARS: usize = 8;

const MAP_WIDTH: usize = 32;
const WRAP_COL: usize = 60;

//const LIFONT_LZ77: &[u8] = include_bytes!("lifont-3x5.lz77");

#[link_section = ".ewram"]
static mut PAINTER: TextPainter = TextPainter::new();

type DoubleHalfTile = [u16; 2*8];

pub unsafe fn text_painter() -> &'static mut TextPainter {
    //PAINTER.get_or_insert_with(TextPainter::new)
    /*if PAINTER.font[1][1] == 0 {
        LZ77UnCompReadNormalWrite16bit(
            LIFONT_LZ77.as_ptr(),
            PAINTER.font.as_mut_ptr() as *mut u16,
        );
    }*/
    &mut PAINTER
}

#[repr(C, align(4))]
pub struct TextPainter {
    // 96 printable ascii chars, each using half of a 8x8 4bpp tile
    font: [u8; 96*16],
    pixel_row: usize,
    pixel_col: usize,
}

impl TextPainter {
    const fn new() -> Self {
        Self {
            font: *include_bytes!("lifont-3x5.4bpp"),
            pixel_row: 0,
            pixel_col: 0,
        }
    }

    fn font(&self) -> &'static [DoubleHalfTile] {
        unsafe { core::slice::from_raw_parts(self.font.as_ptr() as *const DoubleHalfTile, 96/2) }
    }

    fn charblock_write(&self, tile_index: usize, tile_row_index: usize, value: u16) {
        unsafe {
            let tile_ptr = (CHARBLOCK0_4BPP.as_ptr() as *mut DoubleHalfTile)
                .add(tile_index);
            (*tile_ptr).as_mut_ptr().add(tile_row_index).write_volatile(value);
        }
    }

    pub fn setup_display(&mut self) {
        self.pixel_row = 0;
        self.pixel_col = 0;
        let dispcnt = DisplayControl::new()
            .with_show_bg0(true);
        DISPCNT.write(dispcnt);
        let bg0cnt = BackgroundControl::new()
            .with_size(0)
            .with_charblock(0)
            .with_screenblock(16);
        BG0CNT.write(bg0cnt);
        BG0HOFS.write(0);
        BG0VOFS.write(0);
        let screenblock = TEXT_SCREENBLOCKS.get_frame(16).unwrap();
        let mut x = 0;
        for r in 0..32 {
            let row = screenblock.get_row(r).unwrap();
            for cell in row.iter() {
                cell.write(TextEntry::new().with_tile(x));
                x += 1;
            }
        }
        for tile in CHARBLOCK0_4BPP.iter() {
            tile.write([0; 8]);
        }
        BG_PALETTE.index(1).write(Color::new().with_red(22).with_green(16).with_blue(16));
        BG_PALETTE.index(2).write(Color::new().with_red(31).with_green(31).with_blue(31));
        BG_PALETTE.index(0).write(Color::new().with_red(8));
    }

    #[link_section = ".iwram"]
    pub fn paint_text(&mut self, text: &str) {
        for c in text.chars()
            .map(|c| c.as_ascii().map(|a| a.to_u8()).unwrap_or(0x7f) as usize)
        {
            if self.pixel_col >= FONT_WIDTH * WRAP_COL {
                self.pixel_row += FONT_HEIGHT;
                self.pixel_col = 0;
            }
            if (self.pixel_row >> 3) > MAP_WIDTH {
                break;
            }
            if c < 0x20 {
                match c as u8 {
                    // '\b', backspace
                    8 => {
                        self.pixel_col = self.pixel_col.saturating_sub(FONT_WIDTH);
                    }
                    b'\t' => {
                        self.pixel_col = (self.pixel_col + 1).next_multiple_of(TAB_CHARS * FONT_WIDTH);
                    }
                    b'\n' => {
                        self.pixel_row += FONT_HEIGHT;
                        self.pixel_col = 0; // assume cooked
                    }
                    b'\r' => {
                        self.pixel_col = 0;
                    }
                    _ => {} // TODO?
                }
            } else {
                let font_tile = &self.font()[(c - 0x20) >> 1];
                let out_tile_col = self.pixel_col >> 3;
                let mut out_pixel_row = self.pixel_row;
                for font_row in font_tile.iter().skip(c & 1).step_by(2).take(FONT_HEIGHT).copied() {
                    let out_tile_row = out_pixel_row >> 3;
                    let out_tile_pixel_row = ((out_pixel_row & 7) << 1) + ((self.pixel_col & 4) >> 2);
                    let idx = out_tile_row * MAP_WIDTH + out_tile_col;
                    self.charblock_write(idx, out_tile_pixel_row, font_row);
                    out_pixel_row += 1;
                    VBlankIntrWait();
                }
                self.pixel_col += FONT_WIDTH;
            }
        }
    }
}

