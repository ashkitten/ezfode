const MAGIC_1: *mut u16 = 0x9fe0000 as *mut u16;
const MAGIC_2: *mut u16 = 0x8000000 as *mut u16;
const MAGIC_3: *mut u16 = 0x8020000 as *mut u16;
const MAGIC_4: *mut u16 = 0x8040000 as *mut u16;
// finalize txn
const MAGIC_5: *mut u16 = 0x9fc0000 as *mut u16;

const ROMPAGE: *mut u16 = 0x9880000 as *mut u16;
const PSRAMPAGE: *mut u16 = 0x9860000 as *mut u16;
const LED_CTRL: *mut u16 = 0x96e0000 as *mut u16;
const SD_CTRL: *mut u16 = 0x9400000 as *mut u16;
const SD_BUF: *mut u16 = 0x9e00000 as *mut u16;

#[repr(u16)]
pub enum SdControl {
    Disable = 0,
    Enable = 1,
    ReadState = 3,
}

#[link_section = ".iwram"]
unsafe fn start_txn() {
    MAGIC_1.write_volatile(0xd200);
    MAGIC_2.write_volatile(0x1500);
    MAGIC_3.write_volatile(0xd200);
    MAGIC_4.write_volatile(0x1500);
}

#[link_section = ".iwram"]
unsafe fn finish_txn() {
    MAGIC_5.write_volatile(0x1500);
}

#[link_section = ".iwram"]
pub unsafe fn set_rompage(page: u16) {
    start_txn();
    ROMPAGE.write_volatile(page); //C4
    finish_txn();
}

#[link_section = ".iwram"]
pub unsafe fn set_psrampage(page: u16) {
    start_txn();
    PSRAMPAGE.write_volatile(page); // C3
    finish_txn();
}

#[link_section = ".iwram"]
pub unsafe fn set_led_control(status: u16) {
    start_txn();
    LED_CTRL.write_volatile(status);
    finish_txn();
}

#[link_section = ".iwram"]
pub unsafe fn set_sd_control(control: SdControl) {
    start_txn();
    SD_CTRL.write_volatile(control as u16);
    finish_txn();
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
    SD_BUF.read_volatile()
}

#[link_section = ".iwram"]
pub unsafe fn wait_sd_response() -> Result<(), ()> {
    for _ in 0..100000 {
        if sd_response() != 0xeee1 {
            return Ok(());
        }
    }

    // timeout!
    Err(())
}
