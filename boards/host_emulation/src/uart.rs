use std::io::{self, Read, Write};

use kernel::common::cells::OptionalCell;
use kernel::common::cells::TakeCell;
use kernel::common::dynamic_deferred_call::DynamicDeferredCall;
use kernel::hil::uart;
use kernel::hil::uart::{Configure, Receive, Transmit, Uart, UartData};
use kernel::{static_init, ReturnCode};

pub struct UartIO<'a> {
    tx_stream: TakeCell<'a, dyn Write>,
    rx_stream: TakeCell<'a, dyn Read>,
    tx_client: OptionalCell<&'a dyn uart::TransmitClient>,
    rx_client: OptionalCell<&'a dyn uart::ReceiveClient>,
}

impl<'a> UartIO<'a> {
    pub const fn create() -> UartIO<'a> {
        UartIO {
            tx_stream: TakeCell::empty(),
            rx_stream: TakeCell::empty(),
            tx_client: OptionalCell::empty(),
            rx_client: OptionalCell::empty(),
        }
    }

    pub fn initialize(&self) {
        unsafe {
            let stdin = static_init!(io::Stdin, io::stdin());
            let stdout = static_init!(io::Stdout, io::stdout());

            self.tx_stream.replace(stdout);
            self.rx_stream.replace(stdin);
        }
    }

    #[allow(dead_code)]
    pub fn new(
        rx_stream: &'a mut dyn Read,
        tx_stream: &'a mut dyn Write,
        _dynamic_caller: &'static DynamicDeferredCall,
    ) -> UartIO<'a> {
        UartIO {
            tx_stream: TakeCell::new(tx_stream),
            rx_stream: TakeCell::new(rx_stream),
            tx_client: OptionalCell::empty(),
            rx_client: OptionalCell::empty(),
        }
    }

    #[allow(dead_code)]
    pub fn transmit_sync(&self, bytes: &[u8]) {
        let stream = match self.tx_stream.take() {
            Some(stream) => stream,
            None => return,
        };
        let _ = stream.write_all(bytes);
        self.tx_stream.replace(stream);
    }
}

impl<'a> UartData<'a> for UartIO<'a> {}
impl<'a> Uart<'a> for UartIO<'a> {}

impl<'a> Configure for UartIO<'a> {
    fn configure(&self, _: uart::Parameters) -> kernel::ReturnCode {
        ReturnCode::SUCCESS
    }
}

impl<'a> Transmit<'a> for UartIO<'a> {
    fn set_transmit_client(&self, client: &'a dyn uart::TransmitClient) {
        self.tx_client.replace(client);
    }

    fn transmit_buffer(
        &self,
        tx_data: &'static mut [u8],
        tx_len: usize,
    ) -> (ReturnCode, Option<&'static mut [u8]>) {
        if tx_len == 0 {
            return (ReturnCode::ESIZE, Some(tx_data));
        }

        let stream = match self.tx_stream.take() {
            Some(stream) => stream,
            None => return (ReturnCode::EOFF, Some(tx_data)),
        };

        let client = match self.tx_client.take() {
            Some(client) => client,
            None => {
                self.tx_stream.replace(stream);
                return (ReturnCode::FAIL, Some(tx_data));
            }
        };

        let (tx_buf, _): (&mut [u8], _) = tx_data.split_at_mut(tx_len);
        let ret = match stream.write(tx_buf) {
            Ok(written) => {
                if written == tx_len {
                    client.transmitted_buffer(tx_data, tx_len, ReturnCode::SUCCESS);
                    (ReturnCode::SUCCESS, None)
                } else if written == 0 {
                    (ReturnCode::EOFF, Some(tx_data))
                } else {
                    client.transmitted_buffer(tx_data, written, ReturnCode::ESIZE);
                    (ReturnCode::SUCCESS, None)
                }
            }
            Err(_) => (ReturnCode::FAIL, Some(tx_data)),
        };

        self.tx_stream.replace(stream);
        self.tx_client.replace(client);

        ret
    }

    fn transmit_abort(&self) -> ReturnCode {
        ReturnCode::FAIL
    }

    fn transmit_word(&self, _: u32) -> ReturnCode {
        ReturnCode::FAIL
    }
}

impl<'a> Receive<'a> for UartIO<'a> {
    fn set_receive_client(&self, client: &'a dyn uart::ReceiveClient) {
        self.rx_client.replace(client);
    }

    fn receive_buffer(
        &self,
        rx_data: &'static mut [u8],
        rx_len: usize,
    ) -> (ReturnCode, Option<&'static mut [u8]>) {
        if rx_len == 0 {
            return (ReturnCode::ESIZE, Some(rx_data));
        }

        let stream = match self.rx_stream.take() {
            Some(stream) => stream,
            None => return (ReturnCode::EOFF, Some(rx_data)),
        };

        let client = match self.rx_client.take() {
            Some(client) => client,
            None => {
                self.rx_stream.replace(stream);
                return (ReturnCode::FAIL, Some(rx_data));
            }
        };

        let ret = match stream.read(rx_data) {
            Ok(read) => {
                if read == rx_len {
                    client.received_buffer(rx_data, rx_len, ReturnCode::SUCCESS, uart::Error::None);
                    (ReturnCode::SUCCESS, None)
                } else if read == 0 {
                    (ReturnCode::EOFF, Some(rx_data))
                } else {
                    client.received_buffer(rx_data, read, ReturnCode::ESIZE, uart::Error::None);
                    (ReturnCode::SUCCESS, None)
                }
            }
            Err(_) => (ReturnCode::FAIL, Some(rx_data)),
        };

        self.rx_stream.replace(stream);
        self.rx_client.replace(client);

        ret
    }

    fn receive_word(&self) -> ReturnCode {
        ReturnCode::FAIL
    }

    fn receive_abort(&self) -> ReturnCode {
        ReturnCode::FAIL
    }
}
