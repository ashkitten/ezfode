#![feature(
    ascii_char,
    const_slice_from_raw_parts_mut,
    int_roundings,
    generic_const_exprs,
    panic_info_message,
    exclusive_range_pattern,
    const_mut_refs
)]
#![no_std]
#![no_main]

use ape_fatfs::fs::{FileSystem, FsOptions};
use ape_mbr::{PartitionId, MBR};
use core::{fmt::Write, str::from_utf8_unchecked};
use ezflash::set_led_control;
use fs::BufferedIo;
use gba::prelude::*;
use halfwidth::TextPainter;
use log::{debug, error, info, trace, warn, Level, Log};
use sd::SdCard;

mod dma;
mod ezflash;
mod fs;
mod halfwidth;
mod sd;

static mut PAINTER: TextPainter = TextPainter::new();
static LOGGER: ScreenLogger = ScreenLogger;

macro_rules! print {
    ($($args:expr),*) => {
        unsafe { write!(PAINTER, $($args),*).unwrap() }
    };
}

macro_rules! println {
    ($($args:expr),*) => {
        unsafe { writeln!(PAINTER, $($args),*).unwrap() }
    };
}

struct ScreenLogger;

impl Log for ScreenLogger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &log::Record) {
        let color = match record.level() {
            Level::Error => "\x1b[91m", // bright red
            Level::Warn => "\x1b[93m",  // bright yellow
            Level::Info => "\x1b[94m",  // bright blue
            Level::Debug => "\x1b[95m", // bright magenta
            Level::Trace => "\x1b[37m", // white
        };
        println!("{}{}\x1b[m", color, record.args());
    }

    fn flush(&self) {}
}

#[allow(unused_must_use)]
#[panic_handler]
#[link_section = ".iwram"]
fn panic_handler(info: &core::panic::PanicInfo) -> ! {
    unsafe {
        // red+green
        set_led_control(0b10100000);
    }

    // black text on red background
    print!("\x1b[97m\x1b[41m");

    if let Some(location) = info.location() {
        println!("panic at {}:{}:", location.file(), location.line());
    } else {
        println!("panic at unknown location:");
    };
    if let Some(msg) = info.message() {
        println!("{}", *msg);
    }

    // reset
    print!("\x1b[m");

    loop {
        VBlankIntrWait();
    }
}

#[link_section = ".iwram"]
extern "C" fn irq_handler(irq: IrqBits) {
    // maximum value of VCOUNT is 227
    const OFFSET_LUT: [u8; 228] = {
        let mut lut = [0u8; 228];

        let mut offset = 0;
        // can't use for loops in const exprs yet
        let mut i = 0;
        while i < 160 {
            if (i + 1) % 6 == 0 {
                offset = (i + 1) / 3;
            }
            lut[i as usize] = offset;
            i += 1;
        }

        lut
    };

    if irq.hblank() {
        let offset = OFFSET_LUT[VCOUNT.read() as usize];
        BG0VOFS.write(offset as u16);
        BG1VOFS.write(offset as u16);
        BG2VOFS.write(offset as u16);
        BG3VOFS.write(offset as u16);
    }
}

#[no_mangle]
extern "C" fn main() -> ! {
    RUST_IRQ_HANDLER.write(Some(irq_handler));
    DISPSTAT.write(
        DisplayStatus::new()
            .with_irq_vblank(true)
            .with_irq_hblank(true),
    );
    IE.write(IrqBits::new().with_vblank(true).with_hblank(true));
    IME.write(true);

    DISPCNT.write(DisplayControl::new().with_show_bg0(true));

    unsafe {
        // red+green + blue sd indicator
        set_led_control(0b10110001);

        PAINTER.setup_display();

        log::set_logger_racy(&LOGGER).unwrap();
        log::set_max_level_racy(log::LevelFilter::Trace);
    }

    println!("hello world!");

    trace!("this is a trace message");
    debug!("this is a debug message");
    info!("this is an info message");
    warn!("this is a warning message");
    error!("this is an error message");

    let mut mbr = MBR::new(BufferedIo::<512, 2048, _>::new(SdCard)).unwrap();
    let partition = mbr.get_partition(PartitionId::One).unwrap();
    let fs = FileSystem::new(partition, FsOptions::new()).unwrap();

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

        info!("{}", unsafe { from_utf8_unchecked(&bytes) });
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
