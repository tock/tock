use std::io::{self, Read, Write};
use std::ops::DerefMut;
use std::os::unix::net::UnixListener;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::thread;

use kernel::common::cells::OptionalCell;
use kernel::hil::uart;
use kernel::hil::uart::{Configure, Receive, Transmit, Uart, UartData};
use kernel::{static_init, ReturnCode};

type WriteMutex = Mutex<Option<Box<(dyn Write + Send + Sync)>>>;
type ReadMutex = Mutex<Option<Box<(dyn Read + Send + Sync)>>>;

pub struct UartIO<'a> {
    tx_stream: Option<Arc<WriteMutex>>,
    rx_stream: Option<Arc<ReadMutex>>,
    tx_client: OptionalCell<&'a dyn uart::TransmitClient>,
    rx_client: OptionalCell<&'a dyn uart::ReceiveClient>,
}

impl<'a> UartIO<'a> {
    pub const fn create() -> UartIO<'a> {
        UartIO {
            tx_stream: None,
            rx_stream: None,
            tx_client: OptionalCell::empty(),
            rx_client: OptionalCell::empty(),
        }
    }

    pub fn initialize(&mut self) {
        unsafe {
            let stdin = static_init!(io::Stdin, io::stdin());
            let stdout = static_init!(io::Stdout, io::stdout());
            let socket_path = Path::new("/tmp/he_uart"); //TODO add UART idx

            if socket_path.exists() {
                std::fs::remove_file(&socket_path).unwrap();
            }

            let listener = match UnixListener::bind(socket_path) {
                Ok(listener) => listener,
                Err(e) => {
                    panic!("Couldn't bind: {:?}", e);
                }
            };
            let listener = static_init!(UnixListener, listener);

            let tx_stream = Arc::new(WriteMutex::new(Some(Box::new(stdout))));
            let tx_stream_clone = tx_stream.clone();
            let rx_stream = Arc::new(ReadMutex::new(Some(Box::new(stdin))));
            let rx_stream_clone = rx_stream.clone();

            thread::spawn(move || {
                for client in listener.incoming() {
                    let stream = match client {
                        Ok(stream) => stream,
                        Err(e) => {
                            panic!("Couldn't connect: {:?}", e);
                        }
                    };

                    *tx_stream_clone.lock().unwrap() = Some(Box::new(stream.try_clone().unwrap()));
                    *rx_stream_clone.lock().unwrap() = Some(Box::new(stream));
                }
            });

            self.tx_stream = Some(tx_stream);
            self.rx_stream = Some(rx_stream);
        }
    }

    #[allow(dead_code)]
    pub fn transmit_sync(&self, bytes: &[u8]) {
        let stream_mutex = self.tx_stream.as_ref().unwrap().clone();
        let mut stream_guard = stream_mutex.lock().unwrap();
        if let Some(stream) = stream_guard.deref_mut() {
            let _ = stream.write_all(bytes);
            let _ = stream.flush();
        }
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

        let stream_mutex = self.tx_stream.as_ref().unwrap().clone();
        let mut stream_guard = stream_mutex.lock().unwrap();
        if let Some(stream) = stream_guard.deref_mut() {
            let client = match self.tx_client.take() {
                Some(client) => client,
                None => {
                    return (ReturnCode::FAIL, Some(tx_data));
                }
            };
            let (tx_buf, _): (&mut [u8], _) = tx_data.split_at_mut(tx_len);
            match stream.write_all(tx_buf) {
                Ok(()) => {}
                Err(_) => {
                    self.tx_client.replace(client);
                    return (ReturnCode::FAIL, Some(tx_data));
                }
            };

            match stream.flush() {
                Ok(()) => {
                    client.transmitted_buffer(tx_data, tx_len, ReturnCode::SUCCESS);
                }
                Err(_) => {
                    self.tx_client.replace(client);
                    return (ReturnCode::FAIL, Some(tx_data));
                }
            }
            self.tx_client.replace(client);
        }

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

        let client = match self.rx_client.take() {
            Some(client) => client,
            None => {
                return (ReturnCode::FAIL, Some(rx_data));
            }
        };

        let stream_mutex = self.rx_stream.as_ref().unwrap().clone();
        let mut stream_guard = stream_mutex.lock().unwrap(); //handle unwraps with busy error
        let mut read_result = None;
        if let Some(stream) = stream_guard.deref_mut() {
            read_result = Some(stream.read(rx_data));
        }

        if let Some(result) = read_result {
            let ret = match result {
                Ok(read) => {
                    if read == rx_len {
                        client.received_buffer(
                            rx_data,
                            rx_len,
                            ReturnCode::SUCCESS,
                            uart::Error::None,
                        );
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

            self.rx_client.replace(client);
            return ret;
        };

        (ReturnCode::EUNINSTALLED, None)
    }

    fn receive_word(&self) -> ReturnCode {
        ReturnCode::FAIL
    }

    fn receive_abort(&self) -> ReturnCode {
        ReturnCode::FAIL
    }
}
