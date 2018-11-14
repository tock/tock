//! Test reception on the virtualized UART: best if multiple Tests are
//! instantiated and tested in parallel.

use kernel::common::cells::TakeCell;
use kernel::hil::uart;
use kernel::hil::uart::Receive;
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
        let (err, _opt) = self.device.receive_buffer(buf, len);
        if err != ReturnCode::SUCCESS {
            panic!("Calling receive_buffer() in virtual_uart test failed: {:?}",  err);
        }
    }
}

impl uart::ReceiveClient<'static> for TestVirtualUartReceive {

    fn received_buffer(
        &self,
        rx_buffer: &'static mut [u8],
        rx_len: usize,
        rcode: ReturnCode,
        _error: uart::Error,
    ) {
        debug!("Virtual uart read complete: {:?}: ", rcode);
        for i in 0..rx_len {
            debug!("{:02x} ", rx_buffer[i]);
        }
        debug!("Starting receive of length {}", rx_len);
        let (err, _opt) = self.device.receive_buffer(rx_buffer, rx_len);
        if err != ReturnCode::SUCCESS {
            panic!("Calling receive_buffer() in virtual_uart test failed: {:?}",  err);
        }
    }
}
