//! A dummy SPI client to test the SPI implementation

extern crate kernel;
use kernel::hil::gpio;
use kernel::hil::gpio::Pin;
use kernel::hil::spi::{self, SpiSlave};
use sam4l::spi::SPI as SPI_SLAVE;

#[allow(unused_variables,dead_code)]
pub struct SlaveCB {
    val: u8,
}

pub static mut COUNTER: usize = 0;
pub static mut FLOP: bool = false;
pub static mut BUF1: [u8; 8] = [0, 0, 0, 0, 0, 0, 0, 0];
pub static mut BUF2: [u8; 8] = [8, 7, 6, 5, 4, 3, 2, 1];

impl spi::SpiSlaveClient for SlaveCB {
    #[allow(unused_variables,dead_code)]
    fn read_write_done(&self,
                       write_buffer: Option<&'static mut [u8]>,
                       read_buffer: Option<&'static mut [u8]>,
                       len: usize) {
        unsafe {
            SPI_SLAVE.read_write_bytes(Some(&mut BUF2), None, 8);
        }
    }

    #[allow(unused_variables, dead_code)]
    fn chip_selected(&self) {
        unsafe {
            // This should be 0 at the start of every transfer
            // if COUNTER != 0 {
            //     loop {
            //         SPI_SLAVE.set_write_byte(0xA5);
            //     }
            // }

            SPI_SLAVE.set_write_byte(0x05);
            // Send initial byte
            /*
            if FLOP {
                SPI_SLAVE.set_write_byte(BUF1[COUNTER]);
            } else {
                SPI_SLAVE.set_write_byte(BUF2[COUNTER]);
            }
            */



        }
    }
}

pub static mut SPISLAVECB: SlaveCB = SlaveCB { val: 0x55 as u8 };

#[inline(never)]
#[allow(unused_variables,dead_code)]
pub unsafe fn spi_slave_dummy_test() {

    // set the LED to mark that we've programmed.
    // TODO: This doesn't do anything? We always blink...
    sam4l::gpio::PC[10].make_output();
    &sam4l::gpio::PC[10].set();

    let pin2: &mut gpio::Pin = &mut sam4l::gpio::PC[31]; // It's on D2 of the IMIX
    pin2.make_output();
    pin2.set();

    //sam4l::spi::SPI_SLAVE.set_active_peripheral(sam4l::spi::Peripheral::Peripheral0);
    SPI_SLAVE.set_client(Some(&SPISLAVECB));
    SPI_SLAVE.init(); // SpiSlave::init
    SPI_SLAVE.read_write_bytes(Some(&mut BUF2), Some(&mut BUF1), 8);
    SPI_SLAVE.enable();

    // Hint: Temporarily, when switching between master and slave dummy code,
    // - uncomment the right line at the end of reset_handler in main.rs
    // - uncomment the right client at the end of transfer_done in spi.rs
    // - uncomment 240-242 in main.rs for slave and comment it for master

    // YES interrupts are up, prints 0x07 all the way
    // SPI_SLAVE.set_write_byte(SPI_SLAVE.are_interrupts_up());
    //sam4l::spi::SPI_SLAVE.set_baud_rate(1000000);

    // pin2.clear();

    // TODO: We clear this for the trigger, set it perminantly to behave as NSS
}
