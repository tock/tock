use crate::CHIP;
use kernel::{debug, Chip};

extern "C" {
    pub(crate) fn semihost_command(command: usize, arg0: usize, arg1: usize) -> !;
}

#[test_case]
fn trivial_assertion() {
    debug!("trivial assertion... ");
    assert_eq!(1, 1);
    debug!("    [ok]");
}

#[test_case]
fn check_epmp_regions() {
    debug!("check epmp regions... ");
    unsafe {
        debug!("{}", CHIP.unwrap().pmp);
    }
    debug!("    [ok]");
}

#[test_case]
fn check_pending_interrupts() {
    debug!("check pending interrupts... ");
    unsafe {
        assert_eq!(CHIP.unwrap().has_pending_interrupts(), false);
    }
    debug!("    [ok]");
}
