use std::cell::RefCell;
use std::io::{self, Read, Write};
use std::ops::DerefMut;
use std::os::unix::net::UnixListener;
use std::path::Path;
use std::sync::mpsc::{channel, Receiver, Sender, TryRecvError};
use std::sync::{Arc, Mutex};
use std::thread;

use kernel::common::cells::OptionalCell;
use kernel::hil::uart;
use kernel::hil::uart::{Configure, Receive, Transmit, Uart, UartData};
use kernel::{static_init, ReturnCode};

type WriteMutex = Mutex<Option<Box<(dyn Write + Send + Sync)>>>;
type ReadMutex = Mutex<Option<Box<(dyn Read + Send + Sync)>>>;

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
    tx_stream: Option<Arc<WriteMutex>>,
    rx_stream: Option<Arc<ReadMutex>>,
    tx_client: OptionalCell<&'a dyn uart::TransmitClient>,
    rx_client: OptionalCell<&'a dyn uart::ReceiveClient>,
    rx_request: RefCell<Option<RxRequest>>,
    tx_callback: RefCell<Option<TxCallback>>,
    rx_receiver: Option<Receiver<u8>>,
}

impl<'a> UartIO<'a> {
    pub const fn create() -> UartIO<'a> {
        UartIO {
            tx_stream: None,
            rx_stream: None,
            tx_client: OptionalCell::empty(),
            rx_client: OptionalCell::empty(),
            rx_request: RefCell::new(None),
            tx_callback: RefCell::new(None),
            rx_receiver: None,
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
            let rx_stream = Arc::new(ReadMutex::new(Some(Box::new(stdin))));
            let (sender, receiver) = channel();

            Self::spawn_client_listener(
                listener,
                tx_stream.clone(),
                rx_stream.clone(),
                sender.clone(),
            );
            //spawn worker for stdin
            Self::spawn_receive_worker(rx_stream.clone(), sender);

            self.tx_stream = Some(tx_stream);
            self.rx_stream = Some(rx_stream);
            self.rx_receiver = Some(receiver);
        }
    }

    fn spawn_client_listener(
        listener: &'static UnixListener,
        tx_stream: Arc<WriteMutex>,
        rx_stream: Arc<ReadMutex>,
        sender: Sender<u8>,
    ) {
        thread::spawn(move || {
            for client in listener.incoming() {
                let stream = match client {
                    Ok(stream) => stream,
                    Err(e) => {
                        panic!("Couldn't connect: {:?}", e);
                    }
                };

                *tx_stream.lock().unwrap() = Some(Box::new(
                    stream.try_clone().expect("Error cloning socket stream"),
                ));
                *rx_stream.lock().unwrap() = Some(Box::new(stream));

                Self::spawn_receive_worker(rx_stream.clone(), sender.clone());
            }
        });
    }

    // Receiver worker passes bytes from the Read object in rx_stream to the sender.
    // It is needed to achieve non-blocking read in handle_pending_rx_request,
    // which Sender provides and Read does not.
    fn spawn_receive_worker(rx_stream: Arc<ReadMutex>, sender: Sender<u8>) {
        thread::spawn(move || loop {
            let mut stream = (*rx_stream.lock().unwrap()).take();
            if let Some(stream) = &mut stream {
                let mut buf: [u8; 1] = [0; 1];
                match stream.read(&mut buf) {
                    Ok(1) => {
                        sender
                            .send(buf[0])
                            .expect("Error sending a byte to channel");
                    }
                    Ok(0) => break, // EOF
                    Ok(_) => assert!(false),
                    Err(err) => {
                        println!("Read error, exiting thread: {}", err);
                        break;
                    }
                }
            }
            // Yield to let the client_listener thread set a new rx stream if it needs so
            // and then exit this receiver thread (a new receiver thread will take over).
            // Otherwise set the current stream back and continue receiving.
            // The functional consequence is that any new connection invalidates the current one
            // also in the middle of an ongoing transfer.
            // i.e. only one simultanous connection is supported (subject to change if needed).
            thread::yield_now();
            let stream_new = &mut *rx_stream.lock().unwrap();
            if stream_new.is_none() {
                *stream_new = stream;
            } else {
                break;
            }
        });
    }

    #[allow(dead_code)]
    pub fn transmit_sync(&self, bytes: &[u8]) {
        let stream_mutex = self.tx_stream.as_ref().expect("Missing tx stream").clone();
        let mut stream_guard = stream_mutex.lock().unwrap();
        if let Some(stream) = stream_guard.deref_mut() {
            let _ = stream.write_all(bytes);
            let _ = stream.flush();
        }
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
            let receiver = self.rx_receiver.as_ref().expect("Missing rx receiver");

            match receiver.try_recv() {
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

        let stream_mutex = self.tx_stream.as_ref().expect("Missing tx stream").clone();
        let mut stream_guard = stream_mutex.lock().unwrap();
        if let Some(stream) = stream_guard.deref_mut() {
            let (tx_buf, _): (&mut [u8], _) = tx_data.split_at_mut(tx_len);
            match stream.write_all(tx_buf) {
                Ok(()) => {}
                Err(_) => {
                    return (ReturnCode::FAIL, Some(tx_data));
                }
            };

            match stream.flush() {
                Ok(()) => {
                    *self.tx_callback.borrow_mut() = Some(TxCallback {
                        buffer: tx_data,
                        len: tx_len,
                    });
                }
                Err(_) => {
                    return (ReturnCode::FAIL, Some(tx_data));
                }
            }
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
