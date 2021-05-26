use core::panic::PanicInfo;
use kernel::debug;

extern "C" {
    pub(crate) fn semihost_command(command: usize, arg0: usize, arg1: usize) -> !;
}

#[no_mangle]
#[panic_handler]
pub unsafe extern "C" fn panic_fmt(_pi: &PanicInfo) -> ! {
    // Exit QEMU with a return code of 1
    crate::tests::semihost_command(0x18, 1, 0);
}

#[test_case]
fn trivial_assertion() {
    debug!("trivial assertion... ");
    assert_eq!(1, 1);
    debug!("    [ok]");
}
