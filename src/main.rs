#![feature(ascii_char)]
#![feature(const_slice_from_raw_parts_mut)]
#![feature(int_roundings)]
#![feature(generic_const_exprs)]
#![feature(panic_info_message)]
#![no_std]
#![no_main]

use ape_fatfs::fs::{FileSystem, FsOptions};
use ape_mbr::{PartitionId, MBR};
use core::{fmt::Write, str::from_utf8_unchecked};
use ezflash::set_led_control;
use fs::BufferedIo;
use gba::prelude::*;
use halfwidth::TextPainter;
use log::{error, info, Log};
use sd::SdCard;

mod dma;
mod ezflash;
mod fs;
mod halfwidth;
mod sd;

static mut PAINTER: TextPainter = TextPainter::new();
static LOGGER: ScreenLogger = ScreenLogger;

struct ScreenLogger;

impl Log for ScreenLogger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &log::Record) {
        unsafe { writeln!(PAINTER, "{} - {}", record.level(), record.args()).unwrap() };
    }

    fn flush(&self) {}
}

#[allow(unused_must_use)]
#[panic_handler]
fn panic_handler(info: &core::panic::PanicInfo) -> ! {
    unsafe {
        // red+green
        set_led_control(0b10100000);
    }
    BACKDROP_COLOR.write(Color::new().with_red(8));

    if let Some(location) = info.location() {
        error!("panic at {}:{}:", location.file(), location.line());
    } else {
        error!("panic at unknown location:");
    };
    if let Some(msg) = info.message() {
        error!("{}", *msg);
    }

    loop {
        VBlankIntrWait();
    }
}

extern "C" fn irq_handler(irq: IrqBits) {
    if irq.hblank() {
        let next_vcount = VCOUNT.read() + 1;
        // overscan region
        if next_vcount > 160 {
            BG0VOFS.write(0);
            BG1VOFS.write(0);
            BG2VOFS.write(0);
            BG3VOFS.write(0);
        } else if next_vcount % 6 == 0 {
            BG0VOFS.write(next_vcount / 3);
            BG1VOFS.write(next_vcount / 3);
            BG2VOFS.write(next_vcount / 3);
            BG3VOFS.write(next_vcount / 3);
        }
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
        PAINTER.setup_display();
        BACKDROP_COLOR.write(Color::new().with_green(8));
        log::set_logger_racy(&LOGGER).unwrap();
        log::set_max_level_racy(log::LevelFilter::Trace);
    }

    //panic!("According to all known laws of aviation, there is no way for a bee to fly.\n\tHowever\rThe bee flies anyway because fuck you that's why.");

    unsafe {
        // red+green + blue sd indicator
        set_led_control(0b10110001);
    }

    info!("hello world!");

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
