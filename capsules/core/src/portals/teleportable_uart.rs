use kernel::utilities::cells::{OptionalCell, TakeCell};
use kernel::ErrorCode;

use kernel::hil::uart;
use kernel::hil::portal::{Portal, PortalClient};


#[derive(Copy, Clone, Debug)]
pub enum UartOperation {
    TransmitBuffer { len: usize },
    TransmittedBuffer { len: usize, rcode: Result<(), ErrorCode> },
}

pub struct UartTraveler {
    op: UartOperation,
    buffer: TakeCell<'static, [u8]>,
}

impl UartTraveler {
    pub fn empty() -> UartTraveler {
        UartTraveler {
            op: UartOperation::TransmitBuffer { len: 0 },
            buffer: TakeCell::empty(),
        }
    }
}


pub struct UartPortalClient<'a> {
    traveler: TakeCell<'static, UartTraveler>,
    portal: &'a dyn Portal<'a, UartTraveler>,
    tx_client: OptionalCell<&'a dyn uart::TransmitClient>,
    rx_client: OptionalCell<&'a dyn uart::ReceiveClient>,
}

impl<'a> UartPortalClient<'a> {
    pub fn new(
        traveler: &'static mut UartTraveler,
        portal: &'a dyn Portal<'a, UartTraveler>,
    ) -> UartPortalClient<'a> {
        UartPortalClient {
            portal,
            traveler: TakeCell::new(traveler),
            tx_client: OptionalCell::empty(),
            rx_client: OptionalCell::empty(),
        }
    }
}

impl PortalClient<UartTraveler> for UartPortalClient<'_> {
    fn teleported(
        &self,
        traveler: &'static mut UartTraveler,
        _rcode: Result<(), ErrorCode>,
    ) {
        match traveler.op {
            UartOperation::TransmittedBuffer { len, rcode } => {
                traveler.buffer.take().map(|buffer| {
                    self.tx_client.map(|client| {
                        client.transmitted_buffer(buffer, len, rcode);
                    });
                });
                self.traveler.replace(traveler);
            }
            _ => (),
        }
    }
}

impl<'a> uart::Configure for UartPortalClient<'a> {
    fn configure(&self, _params: uart::Parameters) -> Result<(), ErrorCode> {
        Ok(()) // Portal server is responsible for configuring the hardware UART
    }
}

impl<'a> uart::Transmit<'a> for UartPortalClient<'a> {
    fn set_transmit_client(&self, client: &'a dyn uart::TransmitClient) {
        self.tx_client.set(client);
    }

    fn transmit_abort(&self) -> Result<(), ErrorCode> {
        Err(ErrorCode::FAIL)
    }

    fn transmit_buffer(
        &self,
        tx_data: &'static mut [u8],
        tx_len: usize,
    ) -> Result<(), (ErrorCode, &'static mut [u8])> {
        match self.traveler.take() {
            Some(traveler) => {
                traveler.op = UartOperation::TransmitBuffer { len: tx_len };
                traveler.buffer.replace(tx_data);
                self.portal.teleport(traveler).map_err(|(ecode, rtraveler)| {
                    let buf = rtraveler.buffer.take().unwrap_or_else(|| unreachable!());
                    self.traveler.replace(rtraveler);
                    (ecode, buf)
                })
            }
            None => Err((ErrorCode::BUSY, tx_data))
        }
    }

    fn transmit_word(&self, word: u32) -> Result<(), ErrorCode> {
        Err(ErrorCode::FAIL)
    }
}

impl<'a> uart::Receive<'a> for UartPortalClient<'a> {
    fn set_receive_client(&self, client: &'a dyn uart::ReceiveClient) {
        self.rx_client.set(client);
    }

    fn receive_buffer(
        &self,
        rx_buffer: &'static mut [u8],
        rx_len: usize,
    ) -> Result<(), (ErrorCode, &'static mut [u8])> { todo!() }

    fn receive_word(&self) -> Result<(), ErrorCode> { todo!() }

    fn receive_abort(&self) -> Result<(), ErrorCode> { todo!() }
}


pub struct UartPortal<'a> {
    uart: &'a dyn uart::UartData<'a>,
    traveler: TakeCell<'static, UartTraveler>,
    portal_client: OptionalCell<&'a dyn PortalClient<UartTraveler>>,
}

impl<'a> UartPortal<'a> {
    pub fn new(
        uart: &'a dyn uart::UartData<'a>,
    ) -> UartPortal<'a> {
        UartPortal {
            uart,
            traveler: TakeCell::empty(),
            portal_client: OptionalCell::empty(),
        }
    }
}

impl<'a> uart::TransmitClient for UartPortal<'a> {
    fn transmitted_buffer(
        &self,
        tx_buffer: &'static mut [u8],
        tx_len: usize,
        rcode: Result<(), ErrorCode>,
    ) {
        self.traveler.take().map(|traveler| {
            traveler.op = UartOperation::TransmittedBuffer { len: tx_len, rcode };
            traveler.buffer.replace(tx_buffer);
            self.portal_client.map(|client| client.teleported(traveler, Ok(())));
        });
    }

    fn transmitted_word(&self, rcode: Result<(), ErrorCode>) {
        todo!()
    }
}

impl<'a> Portal<'a, UartTraveler> for UartPortal<'a> {
    fn set_portal_client(&self, client: &'a dyn PortalClient<UartTraveler>) {
        self.portal_client.set(client);
    }

    fn teleport(
        &self,
        traveler: &'static mut UartTraveler,
    ) -> Result<(), (ErrorCode, &'static mut UartTraveler)> {
        match traveler.op {
            UartOperation::TransmitBuffer { len } => {
                traveler.buffer.take().map(|buf| {
                    match self.uart.transmit_buffer(buf, len) {
                        Ok(()) => {
                            self.traveler.replace(traveler);
                        }
                        Err((ecode, buf)) => {
                            self.portal_client.map(|client| {
                                traveler.op = UartOperation::TransmittedBuffer { len, rcode: Err(ecode) };
                                traveler.buffer.replace(buf);
                                let _ = client.teleported(traveler, Ok(()));
                            });
                        }
                    }
                });
                Ok(())
            }
            _ => Err((ErrorCode::FAIL, traveler))
        }
    }
}
