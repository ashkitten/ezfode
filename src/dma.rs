use core::ffi::c_void;

use gba::prelude::{DmaControl, DmaStartTime, DMA3_CONTROL, DMA3_COUNT, DMA3_DEST, DMA3_SRC};

/// this is supposed to mimic dmaCopy from libgba
/// size is a u32 because it needs to represent a 17-bit count (of u8)
/// but DMA3_COUNT takes a count of u16
#[inline(always)]
pub unsafe fn dma_copy(src: *mut c_void, dst: *mut c_void, size: u32) {
    DMA3_SRC.write(src);
    DMA3_DEST.write(dst);
    DMA3_COUNT.write((size >> 1) as u16);
    DMA3_CONTROL.write(DmaControl::new().with_enabled(true));
}
