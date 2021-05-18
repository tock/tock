// use kernel::common::cells::TakeCell;
use kernel::hil::spi;

// TODO: intialize buffers to 0
pub static mut TXBUFFER: [u8; 1] = [21];
pub static mut RXBUFFER: [u8; 1] = [21];
pub static mut A5: [u8; 16] = [0xA5; 16];

pub struct LMICSpi<'a, Spi: spi::SpiMaster> {
    pub spi: &'a Spi,
    // txbuffer: TakeCell<'static, [u8]>,
    // rxbuffer: TakeCell<'static, [u8]>,
}

impl<'a, Spi: spi::SpiMaster> LMICSpi<'a, Spi> {
    pub fn new(
        spi: &'a Spi,
        // txbuffer: &'static mut [u8],
        // rxbuffer: &'static mut [u8],
    ) -> LMICSpi<Spi> {
        LMICSpi {
            spi: spi,
            // txbuffer: TakeCell::new(txbuffer),
            // rxbuffer: TakeCell::new(rxbuffer),
        }
    }

    // pub fn send_bytes(&self) {
    //     let _ = self.spi.read_write_bytes(&mut A5, None, A5.len());
    // }
}
