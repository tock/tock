/// Serial interface for native processes.
use core::cell::Cell;
use std::str;

use kernel::hil;
//use kernel::ReturnCode;

pub struct NativeSerial<'a> {
    client: Cell<Option<&'a hil::uart::Client>>,
}

pub static mut NATIVE_SERIAL_0: NativeSerial = NativeSerial::new();

/// Local detials of the NativeSerial implementation
impl NativeSerial<'a> {
    const fn new() -> NativeSerial<'a> {
        NativeSerial {
            client: Cell::new(None),
        }
    }
}

/// Implementation of kernel::hil::UART
impl hil::uart::UART for NativeSerial<'a> {
    fn set_client(&self, client: &'static hil::uart::Client) {
        self.client.set(Some(client));
    }

    fn init(&self, _params: hil::uart::UARTParams) {}

    fn transmit(&self, tx_data: &'static mut [u8], tx_len: usize) {
        // TODO: Need to use len and not assume this is a string
        let s = str::from_utf8(tx_data).unwrap();
        print!("{}", s);

        // TODO: Trigger async done somehow
    }

    fn receive(&self, _rx_buffer: &'static mut [u8], _rx_len: usize) {
        unimplemented!("NativeSerial receive");
    }

    fn abort_receive(&self) {
        unimplemented!("NativeSerial abort_receive");
    }
}
