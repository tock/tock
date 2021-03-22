use std::cell::RefCell;
use std::path::Path;
use std::sync::mpsc::TryRecvError;

use super::async_data_stream::AsyncDataStream;

use kernel::common::cells::OptionalCell;
use kernel::hil::uart;
use kernel::hil::uart::{Configure, Receive, Transmit, Uart, UartData};
use kernel::ReturnCode;

const SOCKET_PATH_BASE: &str = "/tmp/he_uart";

struct RxRequest {
    buffer: &'static mut [u8],
    len: usize,
    inbuf: usize,
}

struct TxCallback {
    buffer: &'static mut [u8],
    len: usize,
}

pub struct UartIO<'a> {
    tx_client: OptionalCell<&'a dyn uart::TransmitClient>,
    rx_client: OptionalCell<&'a dyn uart::ReceiveClient>,
    rx_request: RefCell<Option<RxRequest>>,
    tx_callback: RefCell<Option<TxCallback>>,
    stream: RefCell<Option<AsyncDataStream>>,
    id: &'a str,
}

impl<'a> UartIO<'a> {
    pub const fn create(id: &'a str) -> UartIO<'a> {
        UartIO {
            tx_client: OptionalCell::empty(),
            rx_client: OptionalCell::empty(),
            rx_request: RefCell::new(None),
            tx_callback: RefCell::new(None),
            stream: RefCell::new(None),
            id,
        }
    }

    pub fn initialize(&mut self) {
        *self.stream.borrow_mut() = Some(AsyncDataStream::new_socket_stream(
            Path::new(&(SOCKET_PATH_BASE.to_owned() + self.id)),
            true,
        ));
    }

    #[allow(dead_code)]
    pub fn transmit_sync(&self, bytes: &[u8]) {
        self.stream
            .borrow_mut()
            .as_mut()
            .unwrap()
            .write_all(bytes)
            .expect("Test stream write error");
    }

    fn handle_pending_tx_callback(&'static self) {
        if let Some(callback) = self.tx_callback.borrow_mut().take() {
            let client = match self.tx_client.take() {
                Some(client) => client,
                None => {
                    return;
                }
            };
            client.transmitted_buffer(callback.buffer, callback.len, ReturnCode::SUCCESS);

            self.tx_client.replace(client);
        }
    }

    fn handle_pending_rx_request(&'static self) {
        let request = self.rx_request.borrow_mut().take();
        if let Some(mut request) = request {
            let client = self
                .rx_client
                .expect("Missing rx client for a receive request");
            let mut stream_temp = self.stream.borrow_mut();
            let stream = stream_temp.as_mut().expect("missing data stream");

            match stream.try_recv() {
                Ok(byte) => {
                    request.buffer[request.inbuf] = byte;
                    request.inbuf += 1;
                    if request.inbuf >= request.len {
                        client.received_buffer(
                            request.buffer,
                            request.len,
                            ReturnCode::SUCCESS,
                            uart::Error::None,
                        );
                    } else {
                        *self.rx_request.borrow_mut() = Some(request);
                    }
                }
                Err(TryRecvError::Empty) => {
                    *self.rx_request.borrow_mut() = Some(request);
                }
                Err(_) => {
                    client.received_buffer(
                        request.buffer,
                        0,
                        ReturnCode::FAIL,
                        uart::Error::Aborted,
                    );
                }
            }
        }
    }

    pub fn handle_pending_requests(&'static self) {
        self.handle_pending_tx_callback();
        self.handle_pending_rx_request();
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

        if self.tx_callback.borrow().is_some() {
            return (ReturnCode::EBUSY, Some(tx_data));
        }

        let (tx_buf, _): (&mut [u8], _) = tx_data.split_at_mut(tx_len);
        match self
            .stream
            .borrow_mut()
            .as_mut()
            .expect("Missing output stream")
            .write_all(tx_buf)
        {
            Ok(()) => {
                *self.tx_callback.borrow_mut() = Some(TxCallback {
                    buffer: tx_data,
                    len: tx_len,
                });
            }
            Err(_) => {
                return (ReturnCode::FAIL, Some(tx_data));
            }
        };

        (ReturnCode::SUCCESS, None)
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

        if self.rx_request.borrow_mut().is_some() {
            return (ReturnCode::EBUSY, Some(rx_data));
        }

        *self.rx_request.borrow_mut() = Some(RxRequest {
            buffer: rx_data,
            len: rx_len,
            inbuf: 0,
        });

        (ReturnCode::SUCCESS, None)
    }

    fn receive_word(&self) -> ReturnCode {
        ReturnCode::FAIL
    }

    fn receive_abort(&self) -> ReturnCode {
        ReturnCode::FAIL
    }
}
