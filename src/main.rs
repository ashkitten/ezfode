#![feature(int_roundings)]
#![feature(generic_const_exprs)]
#![no_std]
#![no_main]

use core::str::from_utf8_unchecked;

use ape_fatfs::fs::{FileSystem, FsOptions};
use ape_mbr::{PartitionId, MBR};
use ezflash::set_led_control;
use fs::BufferedIo;
use gba::prelude::*;
use log::Log;
use sd::SdCard;

mod dma;
mod ezflash;
mod fs;
mod sd;

#[panic_handler]
fn panic_handler(info: &core::panic::PanicInfo) -> ! {
    unsafe {
        // red+green
        set_led_control(0b10100000);
    }
    BACKDROP_COLOR.write(Color::RED);

    let mut itoa = itoa::Buffer::new();

    let mut pos = 0;
    let mut bytes = [0u8; 512];
    let mut write = |str: &str| {
        // truncate silently
        let end = bytes.len().min(pos + str.len());
        let len = end - pos;
        bytes[pos..end].copy_from_slice(&str.as_bytes()[..len]);
        pos = end;
    };

    write("panic at ");
    if let Some(location) = info.location() {
        write(location.file().rsplit_terminator('/').next().unwrap());
        write(" line ");
        write(itoa.format(location.line()));
    } else {
        write("unknown location");
    };

    draw_text(unsafe { core::str::from_utf8_unchecked(&bytes) });

    loop {}
}

// static LOGGER: ScreenLogger = ScreenLogger::new();

// struct ScreenLogger {
//     buf: GbaCell<[u8; 1024]>,
//     len: GbaCell<u16>,
//     pos: GbaCell<u16>,
// }

// impl ScreenLogger {
//     fn new() -> Self {
//         Self {
//             buf: GbaCell::new([0; 1024]),
//             len: GbaCell::new(0),
//             pos: GbaCell::new(0),
//         }
//     }
// }

// impl Log for ScreenLogger {
//     fn enabled(&self, metadata: &log::Metadata) -> bool {
//         true
//     }

//     fn log(&self, record: &log::Record) {}

//     fn flush(&self) {
//         todo!()
//     }
// }

extern "C" fn irq_handler(_: IrqBits) {}

fn draw_text(text: &str) {
    Cga8x8Thick.bitunpack_4bpp(CHARBLOCK0_4BPP.as_region(), 0);
    BG0CNT.write(BackgroundControl::new().with_screenblock(8));

    let screenblock = TEXT_SCREENBLOCKS.get_frame(8).unwrap();
    for x in 0..32 {
        for y in 0..32 {
            screenblock
                .get(x, y)
                .unwrap()
                .write(TextEntry::new().with_tile(0));
        }
    }

    for (y, line) in text.split_terminator("\n").take(32).enumerate() {
        let row = screenblock.get_row(y).unwrap();
        for (x, byte) in line.bytes().take(32).enumerate() {
            let text_entry = TextEntry::new().with_tile(byte as u16);
            row.get(x).unwrap().write(text_entry);
        }
    }

    DISPCNT.write(DisplayControl::new().with_show_bg0(true));
}

#[no_mangle]
extern "C" fn main() -> ! {
    RUST_IRQ_HANDLER.write(Some(irq_handler));
    DISPSTAT.write(DisplayStatus::new().with_irq_vblank(true));
    IE.write(IrqBits::VBLANK);
    IME.write(true);

    DISPCNT.write(DisplayControl::new().with_show_bg0(true));
    BACKDROP_COLOR.write(Color::YELLOW);
    unsafe {
        // red+green + blue sd indicator
        set_led_control(0b10110001);
    }

    draw_text("hello world!");

    let mut mbr = MBR::new(BufferedIo::<512, 2048, _>::new(SdCard)).unwrap();
    let partition = mbr.get_partition(PartitionId::One).unwrap();
    let fs = FileSystem::new(partition, FsOptions::new()).unwrap();

    BACKDROP_COLOR.write(Color::GREEN);
    unsafe {
        // green + blue sd indicator
        set_led_control(0b10010001);
    }

    {
        let mut pos = 0;
        let mut bytes = [0u8; 512];
        for entry in fs.root_dir().iter() {
            let entry = entry.unwrap();
            if let Some(name) = entry.long_file_name_as_ucs2_units() {
                pos += ucs2::decode(name, &mut bytes[pos..]).unwrap()
            } else {
                let name = entry.short_file_name_as_bytes();
                bytes[pos..pos + name.len()].copy_from_slice(&name);
                pos += name.len();
            }
            if entry.is_dir() {
                bytes[pos..pos + 1].copy_from_slice("/".as_bytes());
                pos += 1;
            }
            bytes[pos..pos + 1].copy_from_slice("\n".as_bytes());
            pos += 1;
        }

        draw_text(unsafe { from_utf8_unchecked(&bytes) });
    }

    loop {}
}

pub fn delay(count: u32) {
    let mut i = count;
    let i = &mut i as *mut u32;
    unsafe {
        while i.read_volatile() > 0 {
            i.write_volatile(i.read_volatile() - 1);
        }
    }
}
