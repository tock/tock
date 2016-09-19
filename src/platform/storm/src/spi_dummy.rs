//! A dummy SPI client to test the SPI implementation

use hil::gpio;
use hil::spi::{self, SpiMaster};
use sam4l;

#[allow(unused_variables,dead_code)]
pub struct DummyCB {
    val: u8,
}

pub static mut FLOP: bool = false;
pub static mut buf1: [u8; 8] = [0, 0, 0, 0, 0, 0, 0, 0];
pub static mut buf2: [u8; 8] = [8, 7, 6, 5, 4, 3, 2, 1];

impl spi::SpiMasterClient for DummyCB {
    #[allow(unused_variables,dead_code)]
    fn read_write_done(&self,
                       write: &'static mut [u8],
                       read: Option<&'static mut [u8]>,
                       len: usize) {
        unsafe {
            FLOP = !FLOP;
            let len: usize = buf1.len();
            if FLOP {
                sam4l::spi::SPI.read_write_bytes(&mut buf1, Some(&mut buf2), len);
            } else {
                sam4l::spi::SPI.read_write_bytes(&mut buf2, Some(&mut buf1), len);
            }
        }
    }
}

pub static mut SPICB: DummyCB = DummyCB { val: 0x55 as u8 };

// This test first asserts the Firestorm's pin 2, then initiates a continuous
// SPI transfer of 8 bytes.
//
// The first SPI transfer outputs [8, 7, 6, 5, 4, 3, 2, 1] then echoes whatever
// input it recieves from the slave on peripheral 1 continuously.
//
// To test with a logic analyzer, connect probes to pin 2 on the Firestorm, and
// the SPI MOSI and CLK pins (exposed on the Firestorm's 22-pin header). Setup
// the logic analyzer to trigger sampling on assertion of pin 2, then restart
// the board.
pub unsafe fn spi_dummy_test() {
    let pin2: &mut gpio::GPIOPin = &mut sam4l::gpio::PA[16];
    pin2.enable_output();
    pin2.set();


    sam4l::spi::SPI.set_active_peripheral(sam4l::spi::Peripheral::Peripheral1);
    sam4l::spi::SPI.set_client(&SPICB);
    sam4l::spi::SPI.init();
    sam4l::spi::SPI.enable();
    let len = buf2.len();
    sam4l::spi::SPI.read_write_bytes(&mut buf2, Some(&mut buf1), len);

}
