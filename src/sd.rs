use core::ffi::c_void;
use core::fmt;

use gba::prelude::BACKDROP_COLOR;
use gba::video::Color;

use crate::delay;
use crate::dma::dma_copy;
use crate::ezflash::{sd_disable, sd_enable, sd_read_state, set_rompage, wait_sd_response};

pub type Lba = u32;

pub trait BlockIo<const BS: usize>
where
    Self::Error: fmt::Debug,
{
    type Error;
    fn read_blocks(&mut self, start_lba: Lba, buffer: &mut [u8]) -> Result<(), Self::Error>;
    fn write_blocks(&mut self, start_lba: Lba, buffer: &[u8]) -> Result<(), Self::Error>;
}

#[derive(Debug)]
pub enum BlockIoError {}

pub struct SdCard;

impl SdCard {
    pub fn partition(&mut self, start: Lba, end: Lba) -> Partition<'_, 512, Self> {
        Partition {
            disk: self,
            start,
            end,
        }
    }
}

impl BlockIo<512> for SdCard {
    type Error = BlockIoError;

    #[link_section = ".iwram"]
    fn read_blocks(&mut self, start_lba: Lba, buffer: &mut [u8]) -> Result<(), Self::Error> {
        unsafe {
            set_rompage(0x8000); // OS mode
            sd_enable();

            // we can't overrun, and we need whole blocks
            // 2 ^ 9 = 512
            let count = (buffer.len() >> 9) as u32;
            'chunks: for i in (0..count).step_by(4) {
                // read at most 4 blocks at a time
                let blocks = 4.min(count - i) as u16;
                // low and high 16 bits of the address
                let addr_l = (start_lba + i) as u16;
                let addr_h = ((start_lba + i) >> 16) as u16;

                // try three times to read
                for _ in 0..2 {
                    sd_enable();

                    (0x9fe0000 as *mut u16).write_volatile(0xd200);
                    (0x8000000 as *mut u16).write_volatile(0x1500);
                    (0x8020000 as *mut u16).write_volatile(0xd200);
                    (0x8040000 as *mut u16).write_volatile(0x1500);
                    (0x9600000 as *mut u16).write_volatile(addr_l);
                    (0x9620000 as *mut u16).write_volatile(addr_h);
                    (0x9640000 as *mut u16).write_volatile(blocks);
                    (0x9fc0000 as *mut u16).write_volatile(0x1500);

                    sd_read_state();
                    if wait_sd_response().is_ok() {
                        sd_enable();
                        // successful read!
                        let src = 0x9e00000 as *mut c_void;
                        let dst = &mut buffer[i as usize * 512] as *mut u8 as *mut c_void;
                        dma_copy(src, dst, blocks as u32 * 512);

                        // keep copying chunks
                        continue 'chunks;
                    } else {
                        // read timed out, try again
                        delay(5000);
                    }
                }

                // oh no! we couldn't read!
                BACKDROP_COLOR.write(Color::BLUE);
                loop {}
            }

            sd_disable();
            set_rompage(0x200); // game mode
        }

        Ok(())
    }

    fn write_blocks(&mut self, start_lba: Lba, src: &[u8]) -> Result<(), Self::Error> {
        todo!()
    }
}

pub struct Partition<'d, const BS: usize, D: BlockIo<BS>> {
    disk: &'d mut D,
    start: Lba,
    end: Lba,
}

impl<'d, const BS: usize, D: BlockIo<BS>> BlockIo<BS> for Partition<'d, BS, D> {
    type Error = D::Error;

    fn read_blocks(&mut self, start_lba: Lba, dst: &mut [u8]) -> Result<(), Self::Error> {
        let count = dst.len() / BS;
        self.disk.read_blocks(start_lba + self.start, dst)
    }

    fn write_blocks(&mut self, start_lba: Lba, src: &[u8]) -> Result<(), Self::Error> {
        let count = src.len() / BS;
        self.disk.write_blocks(start_lba + self.start, src)
    }
}
