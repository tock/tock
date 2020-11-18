//! A dummy SPI client to test the SPI implementation

use kernel::hil::gpio::Configure;
use kernel::hil::spi::{self, SpiMaster};
use kernel::ReturnCode;

#[allow(unused_variables, dead_code)]
pub struct DummyCB {
    val: u8,
    spi: &'static sam4l::spi::SpiHw,
}

impl DummyCB {
    pub fn new(spi: &'static sam4l::spi::SpiHw) -> Self {
        Self {
            val: 0x55 as u8,
            spi,
        }
    }
}

pub static mut FLOP: bool = false;
pub static mut BUF1: [u8; 8] = [0, 0, 0, 0, 0, 0, 0, 0];
pub static mut BUF2: [u8; 8] = [8, 7, 6, 5, 4, 3, 2, 1];
pub static mut A5: [u8; 16] = [0xA5; 16];

impl spi::SpiMasterClient for DummyCB {
    #[allow(unused_variables, dead_code)]
    fn read_write_done(
        &self,
        write: &'static mut [u8],
        read: Option<&'static mut [u8]>,
        len: usize,
    ) {
        unsafe {
            // do actual stuff
            self.spi.read_write_bytes(&mut A5, None, A5.len());

            // FLOP = !FLOP;
            // let len: usize = BUF1.len();
            // if FLOP {
            //     sam4l::spi::SPI.read_write_bytes(&mut BUF1, Some(&mut BUF2), len);
            // } else {
            //     sam4l::spi::SPI.read_write_bytes(&mut BUF2, Some(&mut BUF1), len);
            // }
        }
    }
}

// This test first turns on the Imix's User led, asserts pin D2 and then
// initiates a continuous SPI transfer of 8 bytes.
//
// If the SPI transfer of multiple bytes fail, then the test will loop writing
// 0xA5.
//
// The first SPI transfer outputs [8, 7, 6, 5, 4, 3, 2, 1] then echoes whatever
// input it recieves from the slave on peripheral 1 continuously.
//
// To test with a logic analyzer, connect probes to pin D2 on the Imix, and
// the SPI MOSI and CLK pins (exposed on the Imix's 20-pin header). Setup
// the logic analyzer to trigger sampling on assertion of pin 2, then restart
// the board.
#[inline(never)]
#[allow(unused_variables, dead_code)]
pub unsafe fn spi_dummy_test(spi: &'static sam4l::spi::SpiHw) {
    // set the LED to mark that we've programmed.
    let pin = sam4l::gpio::GPIOPin::new(sam4l::gpio::Pin::PC10);
    pin.make_output();
    pin.set();

    let pin2 = sam4l::gpio::GPIOPin::new(sam4l::gpio::Pin::PC31); // It's on D2 of the IMIX
    pin2.make_output();
    pin2.set();

    let spicb = kernel::static_init!(DummyCB, DummyCB::new(spi));
    spi.set_active_peripheral(sam4l::spi::Peripheral::Peripheral0);
    spi.set_client(spicb);
    spi.init();
    spi.set_baud_rate(200000);

    let len = BUF2.len();
    if spi.read_write_bytes(&mut BUF2, Some(&mut BUF1), len) != ReturnCode::SUCCESS {
        loop {
            spi.write_byte(0xA5);
        }
    }

    pin2.clear();
}
