// use kernel::common::cells::TakeCell;
use kernel::ErrorCode;
use kernel::{
    debug,
    hil::{lmic, spi},
};

pub struct LMICSpi<'a, Spi: spi::SpiMasterDevice> {
    pub spi: &'a Spi,
    // txbuffer: TakeCell<'static, [u8]>,
    // rxbuffer: TakeCell<'static, [u8]>,
    // TODO: Probably need some flag to check if spi is busy...
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

impl<'a, Spi: spi::SpiMasterDevice> lmic::LMIC for LMICSpi<'a, Spi> {
    fn set_tx_data(&self, tx_data: &'static mut [u8], len: u8) -> Result<(), ErrorCode> {
        // let wbuf = self.txbuffer.take().unwrap();
        // let rbuf = self.rxbuffer.take().unwrap();
        // read_write_bytes always returns Ok(())
        debug!("lmic_spi call to spi read_write_bytes");
        let _ = self.spi.read_write_bytes(tx_data, None, usize::from(len));

        Ok(())
    }
}

// Unclear if this should be here or in lora_controller.rs
impl<Spi: spi::SpiMasterDevice> spi::SpiMasterClient for LMICSpi<'_, Spi> {
    // Callback for when SPI read_write_bytes() finishes
    fn read_write_done(
        &self,
        write_buffer: &'static mut [u8],
        read_buffer: Option<&'static mut [u8]>,
        len: usize,
    ) {
        debug!("Spi read_write_done callback!");
        // TODO: probably want to read out of spi and manage spi state
        // self.spi_busy.set(false);
        // let rbuf = read_buffer.take().unwrap();
        // self.rxbuffer.replace(rbuf);
        // Maybe this also triggers lmic_spi set_tx_data() callback??

        // TODO: Somehow bubble up these buffers from this callback up to LoraSyscall object
        // to call replace on LoraSyscall object's kernel_write and read buffers
    }
}
