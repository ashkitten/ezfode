#![feature(int_roundings)]
#![feature(generic_const_exprs)]
#![no_std]
#![no_main]

use ape_fatfs::fs::{FileSystem, FsOptions};
use ape_mbr::{PartitionId, MBR};
use fs::BufferedIo;
use gba::prelude::*;
use sd::SdCard;

mod dma;
mod ezflash;
mod fs;
mod sd;

#[panic_handler]
fn panic_handler(_: &core::panic::PanicInfo) -> ! {
    BACKDROP_COLOR.write(Color::RED);
    loop {}
}

#[no_mangle]
extern "C" fn main() -> ! {
    DISPCNT.write(DisplayControl::new().with_show_bg0(true));
    BACKDROP_COLOR.write(Color::YELLOW);

    let mut mbr = MBR::new(BufferedIo::<512, 2048, _>::new(SdCard)).unwrap();
    let partition = mbr.get_partition(PartitionId::One).unwrap();
    let fs = FileSystem::new(partition, FsOptions::new()).unwrap();

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
