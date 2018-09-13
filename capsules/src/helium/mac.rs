use kernel::common::cells::OptionalCell;
use kernel::hil::radio_client;
use kernel::ReturnCode;

pub trait Mac {
    /// Initializes the layer; may require a buffer to temporarily retaining frames to be
    /// transmitted
    fn initialize(&self, mac_buf: &'static mut [u8]) -> ReturnCode;
    /// Sets the notified client for configuration changes
    fn set_config_client(&self, client: &'static radio_client::ConfigClient);
    /// Sets the notified client for transmission completions
    fn set_transmit_client(&self, client: &'static radio_client::TxClient);
    /// Sets the notified client for frame receptions
    fn set_receive_client(&self, client: &'static radio_client::RxClient);
    /// Sets the buffer for packet reception
    fn set_receive_buffer(&self, buffer: &'static mut [u8]);

    /// Must be called after one or more calls to `set_*`. If
    /// `set_*` is called without calling `config_commit`, there is no guarantee
    /// that the underlying hardware configuration (addresses, pan ID) is in
    /// line with this MAC protocol implementation. The specificed config_client is
    /// notified on completed reconfiguration.
    fn config_commit(&self);

    /// Indicates whether or not the MAC protocol is active and can send frames
    fn is_on(&self) -> bool;

    /// Transmits complete MAC frames, which must be prepared by an ieee802154::device::MacDevice
    /// before being passed to the Mac layer. Returns the frame buffer in case of an error.
    fn transmit(
        &self,
        full_mac_frame: &'static mut [u8],
        frame_len: usize,
    ) -> (ReturnCode, Option<&'static mut [u8]>);
}

///
/// Default implementation of a Mac layer. Acts as a pass-through between a MacDevice
/// implementation and the underlying radio::Radio device. Does not change the power
/// state of the radio during operation.
///
pub struct VirtualMac<'a, R: radio_client::Radio> {
    radio: &'a R,

    tx_client: OptionalCell<&'static radio_client::TxClient>,
    rx_client: OptionalCell<&'static radio_client::RxClient>,
}

impl<R: radio_client::Radio> VirtualMac<'a, R> {
    pub fn new(radio: &'a R) -> VirtualMac<'a, R> {
        VirtualMac {
            radio: radio,
            tx_client: OptionalCell::empty(),
            rx_client: OptionalCell::empty(),
        }
    }
}

impl<R: radio_client::Radio> Mac for VirtualMac<'a, R> {
    fn initialize(&self, _mac_buf: &'static mut [u8]) -> ReturnCode {
        // do nothing, extra buffer unnecessary
        ReturnCode::SUCCESS
    }

    fn is_on(&self) -> bool {
        self.radio.is_on()
    }
    
    fn set_config_client(&self, client: &'static radio_client::ConfigClient) {
        self.radio.set_config_client(client)
    }

    fn set_transmit_client(&self, client: &'static radio_client::TxClient) {
        self.tx_client.set(client);
    }
    
    fn config_commit(&self) {
        self.radio.config_commit()
    }
    
    fn set_receive_client(&self, client: &'static radio_client::RxClient) {
        self.rx_client.set(client);
    }

    fn set_receive_buffer(&self, buffer: &'static mut [u8]) {
        self.radio.set_receive_buffer(buffer);
    }

    fn transmit(
        &self,
        frame: &'static mut [u8],
        frame_len: usize,
    ) -> (ReturnCode, Option<&'static mut [u8]>) {
        self.radio.transmit(frame, frame_len)
    }
}

impl<R: radio_client::Radio> radio_client::TxClient for VirtualMac<'a, R> {
    fn transmit_event(&self, buf: &'static mut [u8], result: ReturnCode) {
        self.tx_client.map(move |c| {
            c.transmit_event(buf, result);
        });
    }
}

impl<R: radio_client::Radio> radio_client::RxClient for VirtualMac<'a, R> {
    fn receive_event(
        &self,
        buf: &'static mut [u8],
        frame_len: usize,
        crc_valid: bool,
        result: ReturnCode,
    ) {
        // Filter packets by destination because radio is in promiscuous mode
        let addr_match = false;
        // CHECK IF THE RECEIVE PACKET DECAUT AND DECODE IS OK HERE 

        if addr_match {
            self.rx_client.map(move |c| {
                c.receive_event(buf, frame_len, crc_valid, result);
            });
        } else {
            self.radio.set_receive_buffer(buf);
        }
    }
}
