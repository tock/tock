#![allow(unused)]

use mk20::{clock, spi};
use kernel::hil::spi::*;
use tests::{blink, alarm};

static mut WBUF: [u8; 8] = ['H' as u8,
                            'i' as u8,
                            ' ' as u8,
                            't' as u8,
                            'h' as u8,
                            'e' as u8,
                            'r' as u8,
                            'e' as u8];


struct SpiClient;
impl SpiMasterClient for SpiClient {
    fn read_write_done(&self, write_buf: &'static mut [u8],
                              read_buf: Option<&'static mut [u8]>,
                              len: usize) {
        println!("SPI transfer complete");
        blink::led_toggle();
    }
}

static SPI: SpiClient = SpiClient;

pub fn spi_test() {
    unsafe {
        spi::SPI1.set_client(&SPI);

        let rate = spi::SPI1.set_rate(20_000_000);
        println!("Baud rate: {}", rate);
        println!("Bus clock: {}", clock::bus_clock_hz());
    }

    alarm::loop_500ms(|| {
        unsafe {
            spi::SPI1.read_write_bytes(&mut WBUF, None, 8);
        }
    });
}
