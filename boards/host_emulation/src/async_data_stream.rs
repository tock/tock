use std::io::{self, Read, Write};
use std::ops::DerefMut;
use std::os::unix::net::UnixListener;
use std::path::Path;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    mpsc::{channel, Receiver, Sender, TryRecvError},
    Arc, Mutex,
};
use std::thread;

type WriteMutex = Mutex<Option<Box<(dyn Write + Send + Sync)>>>;
type ReadMutex = Mutex<Option<Box<(dyn Read + Send + Sync)>>>;

pub struct AsyncDataStream {
    tx_stream: Arc<WriteMutex>,
    rx_receiver: Receiver<u8>,
}

impl AsyncDataStream {
    pub fn new_socket_stream(path: &Path, use_stdio: bool) -> Self {
        if path.exists() {
            std::fs::remove_file(path).unwrap();
        }

        let listener = match UnixListener::bind(path) {
            Ok(listener) => listener,
            Err(e) => {
                panic!("Couldn't bind: {:?}", e);
            }
        };

        let tx_stream = Arc::new(WriteMutex::new(if use_stdio {
            Some(Box::new(io::stdout()))
        } else {
            None
        }));
        let rx_stream = Arc::new(ReadMutex::new(if use_stdio {
            Some(Box::new(io::stdin()))
        } else {
            None
        }));
        let (rx_sender, rx_receiver) = channel();

        let stop_worker: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
        Self::spawn_client_listener(
            listener,
            tx_stream.clone(),
            rx_stream.clone(),
            rx_sender.clone(),
            stop_worker.clone(),
        );
        //spawn worker for stdin
        if use_stdio {
            Self::spawn_receive_worker(rx_stream, tx_stream.clone(), rx_sender, stop_worker);
        }

        AsyncDataStream {
            tx_stream,
            rx_receiver,
        }
    }

    #[allow(dead_code)]
    pub fn new_socket_stream_str(path: &str, use_stdio: bool) -> Self {
        Self::new_socket_stream(Path::new(path), use_stdio)
    }

    fn spawn_client_listener(
        listener: UnixListener,
        tx_stream: Arc<WriteMutex>,
        rx_stream: Arc<ReadMutex>,
        sender: Sender<u8>,
        mut stop_last_worker: Arc<AtomicBool>,
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

                stop_last_worker.store(true, Ordering::Relaxed);
                stop_last_worker = Arc::new(AtomicBool::new(false));
                Self::spawn_receive_worker(
                    rx_stream.clone(),
                    tx_stream.clone(),
                    sender.clone(),
                    stop_last_worker.clone(),
                );
            }
        });
    }

    // Receiver worker passes bytes from the Read object in rx_stream to the sender.
    // It is needed to achieve non-blocking read in try_recv,
    // which Sender provides and Read does not.
    // At EOF or stop signal, tx stream associated with the same connection is cleared
    // so that it's not used in any further write operations.
    fn spawn_receive_worker(
        rx_stream: Arc<ReadMutex>,
        tx_stream: Arc<WriteMutex>,
        sender: Sender<u8>,
        stop: Arc<AtomicBool>,
    ) {
        thread::spawn(move || {
            loop {
                let mut stream = (*rx_stream.lock().unwrap()).take();
                if let Some(stream) = &mut stream {
                    let mut buf: [u8; 1] = [0; 1];
                    let result = stream.read(&mut buf);
                    if stop.load(Ordering::Acquire) {
                        // Another connection is taking over, exit the thread.
                        // Note: in unfortunate timing a race can happen and some
                        // data from current stream will be lost/skipped.
                        break;
                    }
                    match result {
                        Ok(1) => {
                            sender
                                .send(buf[0])
                                .expect("Error sending a byte to channel");
                        }
                        Ok(0) => {
                            break; // EOF, exit the thread
                        }
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
                // i.e. only one simultanous connection is supported.
                thread::yield_now();
                let stream_new = &mut *rx_stream.lock().unwrap();
                if stream_new.is_none() {
                    *stream_new = stream;
                } else {
                    break;
                }
            }
            // Clear associated tx stream
            *tx_stream.lock().unwrap() = None;
        });
    }

    pub fn try_recv(&mut self) -> Result<u8, TryRecvError> {
        self.rx_receiver.try_recv()
    }

    #[allow(dead_code)]
    pub fn try_recv_unwrapped(&mut self) -> Option<u8> {
        match self.try_recv() {
            Ok(byte) => Some(byte),
            Err(TryRecvError::Empty) => None,
            Err(err) => {
                panic!("Receive error: {:?}", err);
            }
        }
    }

    pub fn write_all(&mut self, buf: &[u8]) -> Result<(), std::io::Error> {
        let mut stream_guard = self.tx_stream.lock().unwrap();
        if let Some(stream) = stream_guard.deref_mut() {
            stream.write_all(buf)?;
            stream.flush()?;
        }
        Ok(())
    }
}
