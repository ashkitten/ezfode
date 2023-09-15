use core::ffi::c_void;

use fatfs::{IoBase, Read, Seek, SeekFrom, Write};
use gba::prelude::*;

use crate::delay;
use crate::dma::dma_copy;
use crate::ezflash::set_rompage;

#[repr(u16)]
pub enum SdControl {
    Disable = 0,
    Enable = 1,
    ReadState = 3,
}

#[link_section = ".iwram"]
pub unsafe fn set_sd_control(control: SdControl) {
    (0x9fe0000 as *mut u16).write_volatile(0xd200);
    (0x8000000 as *mut u16).write_volatile(0x1500);
    (0x8020000 as *mut u16).write_volatile(0xd200);
    (0x8040000 as *mut u16).write_volatile(0x1500);
    (0x9400000 as *mut u16).write_volatile(control as u16);
    (0x9fc0000 as *mut u16).write_volatile(0x1500);
}

#[link_section = ".iwram"]
pub unsafe fn sd_enable() {
    set_sd_control(SdControl::Enable);
}

#[link_section = ".iwram"]
pub unsafe fn sd_disable() {
    set_sd_control(SdControl::Disable);
}

#[link_section = ".iwram"]
pub unsafe fn sd_read_state() {
    set_sd_control(SdControl::ReadState);
}

#[link_section = ".iwram"]
pub unsafe fn sd_response() -> u16 {
    (0x9e00000 as *mut u16).read_volatile()
}

#[link_section = ".iwram"]
pub unsafe fn wait_sd_response() -> u32 {
    for _ in 0..0x100000 {
        if sd_response() != 0xeee1 {
            return 0;
        }
    }

    // timeout!
    BACKDROP_COLOR.write(Color::BLUE);
    return 1;
}

/// read `count` 512-byte sectors starting from `address` into `buffer`
#[link_section = ".iwram"]
pub unsafe fn read_sd_sectors(address: u32, count: u16, buffer: &mut [u8]) {
    sd_enable();

    for i in (0..count).step_by(4) {
        // read at most 4 blocks at a time
        let blocks = if count - i > 4 { 4 } else { count - i };
        let addr_l = ((address + i as u32) & 0x0000FFFF) as u16;
        let addr_h = (((address + i as u32) & 0xFFFF0000) >> 16) as u16;

        // try three times to read
        for _ in 0..2 {
            unsafe {
                (0x9fe0000 as *mut u16).write_volatile(0xd200);
                (0x8000000 as *mut u16).write_volatile(0x1500);
                (0x8020000 as *mut u16).write_volatile(0xd200);
                (0x8040000 as *mut u16).write_volatile(0x1500);
                (0x9600000 as *mut u16).write_volatile(addr_l);
                (0x9620000 as *mut u16).write_volatile(addr_h);
                (0x9640000 as *mut u16).write_volatile(blocks);
                (0x9fc0000 as *mut u16).write_volatile(0x1500);
            }

            sd_read_state();
            if wait_sd_response() == 0 {
                // successful read!
                break;
            }

            // try again
            sd_enable();
            delay(5000);
        }

        unsafe {
            let src = 0x9e00000 as *mut c_void;
            let dst = (buffer as *mut [u8] as *mut c_void).add(i as usize * 512);
            dma_copy(src, dst, blocks as u32 * 512);
        }
    }

    sd_disable();
}

pub struct SDCard {
    /// current stream position
    pos: u32,
    /// track the current page in the buffer
    page: Option<u16>,
    /// buffer containing the current page
    /// we can read at most four 512-byte blocks at a time
    buf: [u8; 2048],
}

impl SDCard {
    pub fn new() -> Self {
        Self {
            pos: 0,
            page: None,
            buf: [0; 2048],
        }
    }
}

impl IoBase for SDCard {
    type Error = ();
}

impl Read for SDCard {
    #[link_section = ".iwram"]
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, ()> {
        if !self.page.is_some_and(|p| p == (self.pos >> 11) as u16) {
            self.page = Some((self.pos >> 11) as u16);
            let page_start = self.pos ^ (u32::MAX >> 11);
            unsafe {
                set_rompage(0x8000); // OS mode
                read_sd_sectors(page_start, 4, &mut self.buf);
                set_rompage(0x200); // game mode
            }
        }

        // offset inside page
        let offset = (self.pos & (u32::MAX >> 11)) as usize;
        // end of the read
        let end = self.buf.len().min(offset + buf.len());

        let slice = &self.buf[offset..end];
        buf.copy_from_slice(slice);

        let len = end - offset;
        self.pos += len as u32;
        Ok(len)
    }
}

impl Write for SDCard {
    fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        todo!()
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        todo!()
    }
}

impl Seek for SDCard {
    fn seek(&mut self, pos: SeekFrom) -> Result<u64, ()> {
        match pos {
            SeekFrom::Start(addr) => {
                self.pos = addr as u32;
            }
            SeekFrom::End(addr) => {
                self.pos = u32::MAX - addr as u32;
            }
            SeekFrom::Current(addr) => {
                self.pos = self.pos + addr as u32;
            }
        }

        Ok(self.pos as u64)
    }
}
