pub unsafe fn set_rompage(page: u16) {
    (0x9fe0000 as *mut u16).write_volatile(0xd200);
    (0x8000000 as *mut u16).write_volatile(0x1500);
    (0x8020000 as *mut u16).write_volatile(0xd200);
    (0x8040000 as *mut u16).write_volatile(0x1500);
    (0x9880000 as *mut u16).write_volatile(page); //C4
    (0x9fc0000 as *mut u16).write_volatile(0x1500);
}

pub unsafe fn set_led_control(status: u16) {
    (0x9fe0000 as *mut u16).write_volatile(0xd200);
    (0x8000000 as *mut u16).write_volatile(0x1500);
    (0x8020000 as *mut u16).write_volatile(0xd200);
    (0x8040000 as *mut u16).write_volatile(0x1500);
    (0x96E0000 as *mut u16).write_volatile(status);
    (0x9fc0000 as *mut u16).write_volatile(0x1500);
}
