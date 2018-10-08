use kernel::common::cells::OptionalCell;
use kernel::hil::radio_client;
use kernel::ReturnCode;

pub trait RFCore {
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
    fn get_radio_status(&self) -> bool;
    
    fn send_stop_command(&self) -> ReturnCode;
    
    fn send_kill_command(&self) -> ReturnCode;
    
    fn get_command_status(&self) -> (ReturnCode, Option<u32>);

    fn set_tx_power(&self, power: u16) -> ReturnCode;

    /// Transmits complete MAC frames, which must be prepared by an ieee802154::device::MacDevice
    /// before being passed to the Mac layer. Returns the frame buffer in case of an error.
    fn transmit(
        &self,
        full_mac_frame: &'static mut [u8],
        frame_len: usize,
    ) -> (ReturnCode, Option<&'static mut [u8]>);

}

pub struct VirtualRadio<'a, R>
where
    R: radio_client::Radio, 
{
    radio: &'a R,
    tx_client: OptionalCell<&'static radio_client::TxClient>,
    rx_client: OptionalCell<&'static radio_client::RxClient>,
}

impl<R> VirtualRadio<'a, R>
where
    R: radio_client::Radio,
{
    pub fn new(
        radio: &'a R,
    ) -> VirtualRadio<'a, R> {
        VirtualRadio {
            radio: radio,
            tx_client: OptionalCell::empty(),
            rx_client: OptionalCell::empty(),
        }
    }
}

impl<R> RFCore for VirtualRadio<'a, R> 
where
    R: radio_client::Radio
{
    fn initialize(&self, _setup_buf: &'static mut [u8]) -> ReturnCode {
        // Maybe use this buf later for firmware patches on load but for now, do nothing
        ReturnCode::SUCCESS
    }

    fn set_config_client(&self, client: &'static radio_client::ConfigClient) {
        self.radio.set_config_client(client)
    }

    fn set_transmit_client(&self, client: &'static radio_client::TxClient) {
        self.tx_client.set(client);
    }

    fn config_commit(&self) {
        self.radio.config_commit();
    }

    fn set_receive_client(&self, client: &'static radio_client::RxClient) {
        self.rx_client.set(client);
    }

    fn set_receive_buffer(&self, buffer: &'static mut [u8]) {
        self.radio.set_receive_buffer(buffer);
    }
       
    fn get_radio_status(&self) -> bool {
        self.radio.is_on()
    }
    
    fn send_stop_command(&self) -> ReturnCode {
        let status = self.radio.send_stop_command();
        match status {
            ReturnCode::SUCCESS => ReturnCode::SUCCESS,
            _ => ReturnCode::FAIL,
        }
    }
    
    fn send_kill_command(&self) -> ReturnCode {
        self.radio.send_kill_command()
    }
    
    fn get_command_status(&self) -> (ReturnCode, Option<u32>) {
        // TODO Parsing with the returned Option<retval> which is some u32 hex code the
        // radio responds with during radio operation command processing
        let (status, _retval) = self.radio.get_command_status();
        (status, None)
    }

    fn set_tx_power(&self, power: u16) -> ReturnCode {
        self.radio.set_tx_power(power)
    }

    fn transmit(
        &self,
        frame: &'static mut [u8],
        frame_len: usize,
    ) -> (ReturnCode, Option<&'static mut [u8]>) {
        self.radio.transmit(frame, frame_len)
    }
}

impl<R: radio_client::Radio> radio_client::TxClient for VirtualRadio<'a, R> {
    fn transmit_event(&self, buf: &'static mut [u8], result: ReturnCode) {
        self.tx_client.map(move |c| {
            c.transmit_event(buf, result);
        });
    }
}

impl<R: radio_client::Radio> radio_client::RxClient for VirtualRadio<'a, R> {
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
