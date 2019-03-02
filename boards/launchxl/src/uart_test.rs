use kernel::common::cells::{MapCell, TakeCell};

const MSG1: &'static [u8; 15] = b"Hello, World!\r\n";
const MSG2: &'static [u8; 22] = b"You can start typing\r\n";

enum State {
    FirstMsg,
    SecondMsg,
    Echo,
}

pub struct TestClient<'a> {
    state: MapCell<State>,

    tx_request_buffer: TakeCell<'a, [u8]>,
    tx_request: TakeCell<'a, hil::uart::TxRequest<'a>>,
    rx_request_buffer: TakeCell<'a, [u8]>,
    rx_request: TakeCell<'a, hil::uart::RxRequest<'a>>,
}

impl<'a> TestClient<'a> {
    pub fn space() -> (
        [u8; 2],
        hil::uart::TxRequest<'a>,
        [u8; 1],
        hil::uart::RxRequest<'a>,
    ) {
        (
            [0; 2],
            hil::uart::TxRequest::new(),
            [0],
            hil::uart::RxRequest::new(),
        )
    }

    pub fn new_with_default_space(
        space: &'a mut (
            [u8; 2],
            hil::uart::TxRequest<'a>,
            [u8; 1],
            hil::uart::RxRequest<'a>,
        ),
    ) -> TestClient<'a> {
        let (tx_request_buffer, tx_request, rx_request_buffer, rx_request) = space;

        Self::new(tx_request_buffer, tx_request, rx_request_buffer, rx_request)
    }

    pub fn new(
        tx_request_buffer: &'a mut [u8],
        tx_request: &'a mut kernel::ikc::TxRequest<'a, u8>,
        rx_request_buffer: &'a mut [u8],
        rx_request: &'a mut kernel::ikc::RxRequest<'a, u8>,
    ) -> TestClient<'a> {
        tx_request.set_with_const_ref(MSG1);
        rx_request.set_buf(rx_request_buffer);

        TestClient {
            state: MapCell::new(State::FirstMsg),
            tx_request_buffer: TakeCell::new(tx_request_buffer),
            tx_request: TakeCell::new(tx_request),
            rx_request_buffer: TakeCell::empty(),
            rx_request: TakeCell::new(rx_request),
        }
    }
}

use kernel::hil;
impl<'a> hil::uart::Client<'a> for TestClient<'a> {
    fn has_tx_request(&self) -> bool {
        let mut ret = false;
        self.tx_request.take().map(|tx| {
            ret = tx.has_some();
            self.tx_request.put(Some(tx));
        });
        ret
    }

    fn get_tx_request(&self) -> Option<&mut hil::uart::TxRequest<'a>> {
        self.tx_request.take()
    }

    fn tx_request_complete(&self, _uart_num: usize, returned_request: &'a mut hil::uart::TxRequest<'a>) {
        self.state.take().map(|mut state| {
            match state {
                State::FirstMsg => {
                    // update tx_requested with new const string
                    returned_request.set_with_const_ref(MSG2);
                    state = State::SecondMsg;
                }
                State::SecondMsg => {
                    // switch tx_request to mutable buffer
                    if let Some(buf) = self.tx_request_buffer.take() {
                        returned_request.set_with_mut_ref(buf);
                    };
                    state = State::Echo;
                }
                State::Echo => {
                    returned_request.reset();
                }
            }
            self.state.put(state);
        });

        self.tx_request.put(Some(returned_request));
    }

    fn has_rx_request(&self) -> bool {
        self.rx_request.is_some()
    }

    fn get_rx_request(&self) -> Option<&mut hil::uart::RxRequest<'a>> {
        self.rx_request.take()
    }

    fn rx_request_complete(&self, _uart_num: usize,returned_request: &'a mut hil::uart::RxRequest<'a>) {
        self.state.take().map(|state| {
            match state {
                State::Echo => {
                    //copy it into the tx_request to echo
                    self.tx_request.take().map(|tx| {
                        tx.reset();
                        // if there is data
                        for _i in 0..returned_request.items_pushed() {
                            if let Some(data) = returned_request.pop() {
                                tx.push(data);
                                if data == b'\r' {
                                    tx.push(b'\n')
                                }
                            }
                        }
                        self.tx_request.put(Some(tx));
                    });
                }
                // no behavior in other states
                _ => {}
            }
            self.state.put(state);
        });

        // reset the request so it is ready to be used again
        returned_request.reset();
        self.rx_request.put(Some(returned_request));
    }
}