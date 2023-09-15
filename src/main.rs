#![feature(int_roundings)]
#![feature(generic_const_exprs)]
#![no_std]
#![no_main]

use ape_fatfs::{
    error::Error,
    fs::{FileSystem, FsOptions},
};
use ape_mbr::{PartitionId, MBR};
use embedded_io::blocking::Read;
use fs::BufferedIo;
use gba::prelude::*;
use sd::SdCard;

use crate::sd::BlockIo;

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

    let fs = {
        // let mut mbr = MBR::new(BufferedIo::<512, 2048, _>::new(SdCard)).unwrap();
        // let mut partition = mbr.get_partition(PartitionId::One).unwrap();
        let mut buf = [0; 2048];
        SdCard.read_blocks(0, &mut buf).unwrap();
        // assert!(buf.into_iter().any(|b| b != 0));
        // let start: [u8; 4] = buf[0x01be + 0x08..0x01be + 0x08 + 4].try_into().unwrap();
        // let start: u32 = u32::from_le_bytes(start);
        // assert!(start > 0);
        // let mut buf = [0; 39];
        // assert!(partition.read(&mut buf).unwrap() == 0);
        // let fs = match FileSystem::new(partition, FsOptions::new()) {
        //     Ok(_) => BACKDROP_COLOR.write(Color::GREEN),
        //     Err(Error::Io(fs::ErrorKind::ReadExactError)) => BACKDROP_COLOR.write(Color::MAGENTA),
        //     Err(Error::CorruptedFileSystem) => BACKDROP_COLOR.write(Color::BLUE),
        //     _ => unreachable!(),
        // };
    };

    BACKDROP_COLOR.write(Color::GREEN);

    // let mut sd = SdCard;
    // let buf_io = BufferedIo::new(sd.partition(lba_range));
    // let fs = FileSystem::new(buf_io, FsOptions::new()).unwrap();

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
