//! Provides a struct that enables access to the unique 120 bit serial number stored in read-only
//! flash on the sam4l.

use kernel::common::StaticRef;

// The sam4l stores a unique 120 bit serial number readable from address 0x0080020C to 0x0080021A
// This value cannot be written to normally, and instead requires special instructions to overwrite,
// which we do not implement. Because this number is stored in the user page of flash memory,
// it is not cleared by chip erase.
#[repr(C)]
struct sam4lSerialRegister {
    serial_num: [u8; 15],
}

const SERIAL_NUM_ADDRESS: StaticRef<sam4lSerialRegister> =
    unsafe { StaticRef::new(0x0080020C as *const sam4lSerialRegister) };

/// Struct that can be used to get the unique serial number of the sam4l
pub struct SerialNum {
    regs: StaticRef<sam4lSerialRegister>,
}

impl SerialNum {
    /// Returns a struct that can read the serial number of the sam4l
    pub fn new() -> SerialNum {
        SerialNum {
            regs: SERIAL_NUM_ADDRESS,
        }
    }

    /// Returns the 120-bit serial number of the sam4l in a byte array
    pub fn get(&self) -> [u8; 15] {
        self.regs.serial_num
    }

    /// Helper function for simply returning the lower 64 bits of the serial number
    /// as a u64 rather than a byte array
    pub fn get_lower_64(&self) -> u64 {
        let full_num = self.regs.serial_num;
        let mut lower_64 = 0;
        // Below could use transmute, but didn't want to add unsafe code
        for i in 7..15 {
            lower_64 = lower_64 + ((full_num[i] as u64) << ((14 - i) * 8));
        }
        lower_64
    }
}
