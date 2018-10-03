use kernel::common::cells::VolatileCell;

use setup::ddi;
use aux;
use prcm;
use rtc;

const FLASH_STANDBY_BASE: *mut VolatileCell<u32> = 0x4003_0024 as *mut VolatileCell<u32>; // Flash CFG Address
const 


pub fn setup_device_trim() {
    let mut fcfg_revision: u32 = 0;
    let mut aon_sys_reset_ctl: u32 = 0;

    // Enable flash standby
    
}

pub fn trim_after_wakeup() {

}

pub fn trim_after_shutdown() {

}

pub fn trim_after_reset() {

}

pub fn setup_cache_mode() {
    
}

