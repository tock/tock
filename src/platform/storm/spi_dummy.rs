///! A dummy SPI client to test the SPI implementation

use sam4l;
use hil::spi_master::{self, SpiMaster};

#[allow(unused_variables,dead_code)]
pub struct DummyCB {
  val: u8
}

pub static mut FLOP: bool = false;
pub static mut buf1: [u8; 8] = [0, 0, 0, 0, 0, 0, 0, 0];
pub static mut buf2: [u8; 8] = [7, 6, 5, 4, 3, 2, 1, 0];

impl spi_master::SpiCallback for DummyCB {
#[allow(unused_variables,dead_code)]
    fn read_write_done(&'static self) {
        unsafe {
            FLOP = !FLOP;
            let len: usize = buf1.len();
            if FLOP {
                sam4l::spi::SPI.read_write_bytes(Some(&mut buf1), Some(&mut buf2), len);
            } else {
                sam4l::spi::SPI.read_write_bytes(Some(&mut buf2), Some(&mut buf1), len);
            }
        }
    }
}

pub static mut SPICB: DummyCB = DummyCB{val: 0x55 as u8};

pub unsafe fn spi_dummy_test() {
    sam4l::spi::SPI.set_active_peripheral(sam4l::spi::Peripheral::Peripheral1);
    sam4l::spi::SPI.init(&SPICB);
    sam4l::spi::SPI.enable();
    let len = buf2.len();
    sam4l::spi::SPI.read_write_bytes(Some(&mut buf2), Some(&mut buf1), len);

}
