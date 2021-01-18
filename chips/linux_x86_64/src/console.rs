use core::cell::Cell;
use kernel::common::cells::OptionalCell;
use kernel::common::cells::TakeCell;
use kernel::common::deferred_call::DeferredCall;
use kernel::hil;
use kernel::ReturnCode;

static mut OUT_BUFFER: [u8; 4] = [0; 4];
static mut OUT_BUFFER_INDEX: usize = 0;

use crate::deferred_call_tasks::DeferredCallTask;

static DEFERRED_CALL: DeferredCall<DeferredCallTask> =
    unsafe { DeferredCall::new(DeferredCallTask::Console) };

pub struct Console<'a> {
    tx_client: OptionalCell<&'a dyn hil::uart::TransmitClient>,
    rx_client: OptionalCell<&'a dyn hil::uart::ReceiveClient>,

    tx_buffer: TakeCell<'static, [u8]>,
    _rx_buffer: TakeCell<'static, [u8]>,

    tx_len: Cell<usize>,
    _rx_len: Cell<usize>,
}

pub static mut CONSOLE: Console = Console::new();

impl<'a> Console<'a> {
    const fn new() -> Console<'a> {
        Console {
            tx_client: OptionalCell::empty(),
            rx_client: OptionalCell::empty(),

            tx_buffer: TakeCell::empty(),
            _rx_buffer: TakeCell::empty(),

            tx_len: Cell::new(0),
            _rx_len: Cell::new(0),
        }
    }

    pub fn handle_interrupt(&self) {
        if self.tx_len.get() > 0 {
            let tx_len = self.tx_len.get();
            self.tx_len.set(0);
            self.tx_client.map(move |client| {
                self.tx_buffer.take().map(|buf| {
                    client.transmitted_buffer(buf, tx_len, ReturnCode::SUCCESS);
                });
            });
        }
    }

    // for use by panic in io.rs
    pub fn send_byte(&self, byte: u8) {
        unsafe {
            OUT_BUFFER[OUT_BUFFER_INDEX] = byte;
            // println!("{:?}", &OUT_BUFFER[0..=OUT_BUFFER_INDEX]);
            if let Ok(s) = std::str::from_utf8(&OUT_BUFFER[0..=OUT_BUFFER_INDEX]) {
                print!("{}", s);
                OUT_BUFFER_INDEX = 0;
                for index in 0..4 {
                    OUT_BUFFER[index] = 0;
                }
            } else {
                OUT_BUFFER_INDEX = OUT_BUFFER_INDEX + 1;
                if OUT_BUFFER_INDEX > 3 {
                    OUT_BUFFER_INDEX = 0;
                    // println!("wrong utf8");
                }
            }
        }
    }
}

impl<'a> hil::uart::Configure for Console<'a> {
    fn configure(&self, _params: hil::uart::Parameters) -> ReturnCode {
        ReturnCode::SUCCESS
    }
}

impl<'a> hil::uart::Transmit<'a> for Console<'a> {
    fn set_transmit_client(&self, client: &'a dyn hil::uart::TransmitClient) {
        self.tx_client.set(client);
    }

    fn transmit_buffer(
        &self,
        tx_data: &'static mut [u8],
        tx_len: usize,
    ) -> (ReturnCode, Option<&'static mut [u8]>) {
        // TODO use async
        if tx_len <= tx_data.len() {
            for index in 0..tx_len {
                // println! ("transmit buffer {}", *byte);
                self.send_byte(tx_data[index]);
            }

            self.tx_buffer.replace(tx_data);

            self.tx_len.set(tx_len);

            DEFERRED_CALL.set();
            (ReturnCode::SUCCESS, None)
        } else {
            (ReturnCode::EINVAL, Some(tx_data))
        }
    }

    fn transmit_word(&self, _word: u32) -> ReturnCode {
        ReturnCode::FAIL
    }

    fn transmit_abort(&self) -> ReturnCode {
        ReturnCode::SUCCESS
    }
}

impl<'a> hil::uart::Receive<'a> for Console<'a> {
    fn set_receive_client(&self, client: &'a dyn hil::uart::ReceiveClient) {
        self.rx_client.set(client);
    }

    fn receive_buffer(
        &self,
        rx_buffer: &'static mut [u8],
        _rx_len: usize,
    ) -> (ReturnCode, Option<&'static mut [u8]>) {
        (ReturnCode::EBUSY, Some(rx_buffer))
    }

    fn receive_word(&self) -> ReturnCode {
        ReturnCode::FAIL
    }

    fn receive_abort(&self) -> ReturnCode {
        ReturnCode::EBUSY
    }
}

impl<'a> hil::uart::UartData<'a> for Console<'a> {}
impl<'a> hil::uart::Uart<'a> for Console<'a> {}
