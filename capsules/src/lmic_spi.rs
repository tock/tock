use kernel::hil::spi;
use kernel::common::cells::TakeCell;

// TODO: intialize buffers to 0
pub static mut TXBUFFER: [u8; 1] = [21];
pub static mut RXBUFFER: [u8; 1] = [21];

pub struct LoRaMACSpi<'a, S: spi::SpiMasterDevice + 'a> {
    spi: &'a S,
    txbuffer: TakeCell<'static, [u8]>,
    rxbuffer: TakeCell<'static, [u8]>,
}

impl<'a, S: spi::SpiMasterDevice + 'a> LoRaMACSpi<'a, S> {
    pub fn new(
        spi: &'a S,
        txbuffer: &'static mut [u8],
        rxbuffer: &'static mut [u8],
    ) -> LoRaMACSpi<'a, S> {
        LoRaMACSpi {
            spi: spi,
            txbuffer: TakeCell::new(txbuffer),
            rxbuffer: TakeCell::new(rxbuffer),
        }
    }
}