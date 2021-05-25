// use kernel::common::cells::TakeCell;
use kernel::hil::{lmic, spi};
use kernel::ErrorCode;

pub struct LMICSpi<'a, Spi: spi::SpiMasterDevice> {
    pub spi: &'a Spi,
    // txbuffer: TakeCell<'static, [u8]>,
    // rxbuffer: TakeCell<'static, [u8]>,
}

impl<'a, Spi: spi::SpiMasterDevice> LMICSpi<'a, Spi> {
    pub fn new(
        spi: &'a Spi,
        // txbuffer: &'static mut [u8],
        // rxbuffer: &'static mut [u8],
    ) -> LMICSpi<'a, Spi> {
        LMICSpi {
            spi: spi,
            // txbuffer: TakeCell::new(txbuffer),
            // rxbuffer: TakeCell::new(rxbuffer),
        }
    }
}

impl<Spi: spi::SpiMasterDevice> lmic::LMIC for LMICSpi<'_, Spi> {
    fn set_tx_data(&self, tx_data: &'static mut [u8]) -> Result<(), ErrorCode> {
        // let wbuf = self.txbuffer.take().unwrap();
        // let rbuf = self.rxbuffer.take().unwrap();
        let _ = self.spi.read_write_bytes(tx_data, None, tx_data.len());

        Ok(())
    }
}
