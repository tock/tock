//! Low-level 8042 (i8042) controller bring-up.
//! Does not touch keyboard protocol settings.

use kernel::errorcode::ErrorCode;
use kernel::hil::ps2_traits::PS2Traits;

/// Run once, before any device-level init

pub fn init_controller<C: PS2Traits>() -> Result<(), ErrorCode> {
    // Disable keyboard (port 1) and aux (port 2)
    C::write_command(0xAD);
    C::write_command(0xA7);
    
    // Self-test: 0xAA - expect 0x55
    C::write_command(0xAA);
    C::wait_output_ready();
    if C::read_data() != 0x55 {
        return Err (ErrorCode::FAIL);
    }

    // Enable IRQ1 in config byte
    C::write_command(0x20);
    C::wait_output_ready();
    let mut cfg = C::read_data();
    cfg |= 1 << 0; //bit0 = IRQ1 enable
    C::write_command(0x60);
    C::write_data(cfg);

    // Port-1 interface test 0xAB - expect 0x00
    C::write_command(0xAB);
    C::wait_output_ready();
    if C::read_data() != 0x00 {
        return Err (ErrorCode::FAIL);
    }
    Ok(())

}