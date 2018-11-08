//! Test reception on the virtualized UART: best if multiple Tests are
//! instantiated and tested in parallel.

use kernel::common::cells::TakeCell;
use kernel::hil;
use kernel::hil::uart;
use kernel::ReturnCode;
use virtual_uart::UartDevice;

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

impl uart::ReceiveClient for TestVirtualUartReceive {

    fn received_buffer(
        &self,
        rx_buffer: &'static mut [u8],
        rx_len: usize,
        rcode: ReturnCode,
    ) {
        debug!("Virtual uart read complete: {:?}: ", rcode);
        for i in 0..rx_len {
            debug!("{:02x} ", rx_buffer[i]);
        }
        debug!("Starting receive of length {}", rx_len);
        self.device.receive(rx_buffer, rx_len);
    }

    fn received_word(&self, word: u32) {}
}
