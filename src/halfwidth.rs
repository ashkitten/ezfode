use anstyle_parse::{DefaultCharAccumulator, Params, Parser, Perform};
use core::fmt::Write;
use gba::prelude::*;

const TAB_CHARS: usize = 4;

const ROWS: usize = 26;
const COLUMNS: usize = 60;

const DEFAULT_FG: u16 = 15;
const DEFAULT_BG: u16 = 0;

// 96 printable ascii chars, each using half of a 8x8 4bpp tile
const LIFONT: &'static [u8] = &include_aligned_bytes!("lifont-3x5-as-8x8.img.lz77").0;

// TODO: support for drawing to a region of the charblock/screenblock
// (i.e. not necessarily taking over the whole display)
pub struct TextPainter {
    row: usize,
    col: usize,
    fg: u16,
    bg: u16,
}

impl TextPainter {
    pub const fn new() -> Self {
        Self {
            row: 0,
            col: 0,
            fg: DEFAULT_FG,
            bg: DEFAULT_BG,
        }
    }

    pub fn setup_display(&mut self) {
        self.row = 0;
        self.col = 0;

        DISPCNT.write(
            DisplayControl::new()
                .with_show_bg0(true)
                .with_show_bg1(true)
                .with_show_bg2(true)
                .with_show_bg3(true),
        );

        BG0CNT.write(
            BackgroundControl::new()
                .with_size(0)
                .with_charblock(0)
                .with_screenblock(16),
        );
        BG0HOFS.write(0);
        BG0VOFS.write(0);

        BG1CNT.write(
            BackgroundControl::new()
                .with_size(0)
                .with_charblock(0)
                .with_screenblock(17),
        );
        BG1HOFS.write(252);
        BG1VOFS.write(0);

        BG2CNT.write(
            BackgroundControl::new()
                .with_size(0)
                .with_charblock(1)
                .with_screenblock(18),
        );
        BG2HOFS.write(0);
        BG2VOFS.write(0);

        BG3CNT.write(
            BackgroundControl::new()
                .with_size(0)
                .with_charblock(1)
                .with_screenblock(19),
        );
        BG3HOFS.write(252);
        BG3VOFS.write(0);

        unsafe {
            LZ77UnCompReadNormalWrite16bit(LIFONT.as_ptr(), CHARBLOCK0_4BPP.as_ptr() as *mut u16);
        }

        // one tile background
        CHARBLOCK1_4BPP.get(0).unwrap().write([0x0000ffff; 8]);
        for screenblock in 16..=19 {
            let frame = TEXT_SCREENBLOCKS.get_frame(screenblock).unwrap();
            for r in 0..32 {
                let row = frame.get_row(r).unwrap();
                for cell in row.iter() {
                    cell.write(TextEntry::new());
                }
            }
        }

        const COLORS: [Color; 16] = [
            Color::from_rgb(00, 00, 00), // black
            Color::from_rgb(20, 00, 00), // red
            Color::from_rgb(00, 20, 00), // green
            Color::from_rgb(20, 20, 00), // yelow
            Color::from_rgb(00, 00, 20), // blue
            Color::from_rgb(20, 00, 20), // magenta
            Color::from_rgb(00, 20, 20), // cyan
            Color::from_rgb(20, 20, 20), // white
            Color::from_rgb(10, 10, 10), // bright black
            Color::from_rgb(31, 10, 10), // bright red
            Color::from_rgb(10, 31, 10), // bright green
            Color::from_rgb(31, 31, 10), // bright yelow
            Color::from_rgb(10, 10, 31), // bright blue
            Color::from_rgb(31, 10, 31), // bright magenta
            Color::from_rgb(10, 31, 31), // bright cyan
            Color::from_rgb(31, 31, 31), // bright white
        ];

        for (i, color) in COLORS.iter().enumerate() {
            BG_PALETTE.index(i * 16 + 0).write(*color);
            BG_PALETTE.index(i * 16 + 1).write(Color::from_rgb(
                color.red() / 3 * 2,
                color.green() / 3 * 2,
                color.blue() / 3 * 2,
            ));
            BG_PALETTE.index(i * 16 + 2).write(*color);
        }
    }
}

impl Write for TextPainter {
    fn write_str(&mut self, text: &str) -> core::fmt::Result {
        let mut state = Parser::<DefaultCharAccumulator>::new();

        for c in text
            .chars()
            .map(|c| c.as_ascii().map(|a| a.to_u8()).unwrap_or(0x7f))
        {
            state.advance(self, c);
        }

        Ok(())
    }
}

impl Perform for TextPainter {
    fn print(&mut self, c: char) {
        if self.col >= COLUMNS {
            self.row += 1;
            self.col = 0;
        }

        let fg = TEXT_SCREENBLOCKS.get_frame(16 + (self.col & 1)).unwrap();
        let bg = TEXT_SCREENBLOCKS.get_frame(18 + (self.col & 1)).unwrap();

        fg.get_row(self.row)
            .unwrap()
            .get(self.col >> 1)
            .unwrap()
            .write(
                TextEntry::new()
                    .with_tile(c as u16 - 0x20)
                    .with_palbank(self.fg),
            );

        bg.get_row(self.row)
            .unwrap()
            .get(self.col >> 1)
            .unwrap()
            .write(TextEntry::new().with_tile(1).with_palbank(self.bg));

        self.col += 1;
    }

    fn execute(&mut self, byte: u8) {
        match byte {
            // '\b', backspace
            8 => {
                self.col = self.col.saturating_sub(1);
            }
            b'\t' => {
                self.col = (self.col + 1).next_multiple_of(TAB_CHARS);
            }
            b'\n' => {
                self.row += 1;
                self.col = 0;
            }
            b'\r' => {
                self.col = 0;
            }
            _ => {} // TODO?
        }
    }

    fn hook(&mut self, params: &Params, intermediates: &[u8], ignore: bool, action: u8) {}

    fn put(&mut self, byte: u8) {}

    fn unhook(&mut self) {}

    fn osc_dispatch(&mut self, params: &[&[u8]], bell_terminated: bool) {}

    fn csi_dispatch(&mut self, params: &Params, intermediates: &[u8], ignore: bool, action: u8) {
        match action {
            b'm' => {
                for param in params.iter() {
                    match param {
                        [0] => {
                            self.fg = DEFAULT_FG;
                            self.bg = DEFAULT_BG;
                        }
                        [30..=37] => self.fg = param[0] - 30,
                        [39] => self.fg = DEFAULT_FG,
                        [40..=47] => self.bg = param[0] - 40,
                        [49] => self.bg = DEFAULT_BG,
                        [90..=97] => self.fg = param[0] - 90 + 8,
                        [100..=107] => self.bg = param[0] - 100 + 8,
                        _ => (),
                    }
                }
            }
            _ => (),
        }
    }

    fn esc_dispatch(&mut self, intermediates: &[u8], ignore: bool, byte: u8) {}
}
