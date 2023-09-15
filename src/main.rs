#![feature(int_roundings)]
#![no_std]
#![no_main]

use fatfs::{FileSystem, FsOptions};
use gba::prelude::*;
use sd::SDCard;

mod dma;
mod ezflash;
mod sd;

#[panic_handler]
#[link_section = ".iwram"]
fn panic_handler(_: &core::panic::PanicInfo) -> ! {
    BACKDROP_COLOR.write(Color::RED);
    loop {}
}

#[no_mangle]
extern "C" fn main() -> ! {
    DISPCNT.write(DisplayControl::new().with_show_bg0(true));
    BACKDROP_COLOR.write(Color::YELLOW);

    let sd = SDCard::new();
    let fs = FileSystem::new(sd, FsOptions::new()).unwrap();

    BACKDROP_COLOR.write(Color::GREEN);

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
