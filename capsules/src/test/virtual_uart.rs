//! Test reception on the virtualized UART: best if multiple Tests are
//! instantiated and tested in parallel.
use crate::virtual_uart::UartDevice;
use kernel::common::cells::TakeCell;
use kernel::debug;
use kernel::hil;
use kernel::hil::uart::Client;
use kernel::hil::uart::UART;

pub struct TestVirtualUartReceive {
    device: &'static UartDevice<'static>,
    buffer: TakeCell<'static, [u8]>,
}

impl TestVirtualUartReceive {
    pub fn new(device: &'static UartDevice<'static>, buffer: &'static mut [u8]) -> Self {
        TestVirtualUartReceive {
            device: device,
            buffer: TakeCell::new(buffer),
        }
    }

    pub fn run(&self) {
        let buf = self.buffer.take().unwrap();
        let len = buf.len();
        debug!("Starting receive of length {}", len);
        self.device.receive(buf, len);
    }
}

impl Client for TestVirtualUartReceive {
    fn transmit_complete(&self, _tx_buffer: &'static mut [u8], _error: hil::uart::Error) {}

    fn receive_complete(
        &self,
        rx_buffer: &'static mut [u8],
        rx_len: usize,
        error: hil::uart::Error,
    ) {
        debug!("Virtual uart read complete: {:?}: ", error);
        for i in 0..rx_len {
            debug!("{:02x} ", rx_buffer[i]);
        }
        debug!("Starting receive of length {}", rx_len);
        self.device.receive(rx_buffer, rx_len);
    }
}
