// Capsule to test signbus intialization functions in tock
// By: Justin Hsieh

use core::cell::Cell;
use kernel::ReturnCode;
use kernel::common::take_cell::TakeCell;
use signbus::{io_layer, support, app_layer, port_layer};

pub static mut BUFFER0: [u8; 256] = [0; 256];
pub static mut BUFFER1: [u8; 256] = [0; 256];

// Signpost specific enums used only for testing.
#[derive(Clone,Copy,PartialEq)]
pub enum ModuleAddress {
    Controller = 0x20,
    Storage = 0x21,
    Radio = 0x22,
}

#[derive(Clone,Copy,PartialEq)]
pub enum SignbusFrameType {
    NotificationFrame = 0,
    CommandFrame = 1,
    ResponseFrame = 2,
    ErrorFrame = 3,
}

#[derive(Clone,Copy,PartialEq)]
pub enum SignbusApiType {
    InitializationApiType = 1,
    StorageApiType = 2,
    NetworkingApiType = 3,
    ProcessingApiType = 4,
    EnergyApiType = 5,
    TimeLocationApiType = 6,
    EdisonApiType = 7,
    JsonApiType = 8,
    WatchdogApiType = 9,
    HighestApiType = 10,
}

#[derive(Clone,Copy,PartialEq)]
pub enum InitMessageType {
    Declare = 0,
    KeyExchange = 1,
    GetMods = 2,
}

#[derive(Clone,Copy,PartialEq)]
pub enum DelayState {
    Idle,
    RequestIsolation,
}

pub struct SignbusInitialization<'a> {
    // app_layer used to send/ recv messages
    app_layer: &'a app_layer::SignbusAppLayer<'a>,
    // io_layer used to init
    io_layer: &'a io_layer::SignbusIOLayer<'a>,
    // port_layer used for gpio and timer
    port_layer: &'a port_layer::PortLayer,

    device_address: Cell<u8>,
    delay_state: Cell<DelayState>,
    send_buf: TakeCell<'static, [u8]>,
    recv_buf: TakeCell<'static, [u8]>,
}

impl<'a> SignbusInitialization<'a> {
    pub fn new(app_layer: &'a app_layer::SignbusAppLayer,
               io_layer: &'a io_layer::SignbusIOLayer,
               port_layer: &'a port_layer::PortLayer,
               send_buf: &'static mut [u8],
               recv_buf: &'static mut [u8])
               -> SignbusInitialization<'a> {

        SignbusInitialization {
            app_layer: app_layer,
            io_layer: io_layer,
            port_layer: port_layer,

            device_address: Cell::new(0),
            delay_state: Cell::new(DelayState::Idle),
            send_buf: TakeCell::new(send_buf),
            recv_buf: TakeCell::new(recv_buf),
        }
    }

    // Send declaration I2C message.
    pub fn signpost_initialization_declare_controller(&self) {
        debug!("Declare controller...");

        self.send_buf.take().map(|buf| {
            // Will only work for 0x32 because of concatenated HMAC
            buf[0] = self.device_address.get();

            self.app_layer.signbus_app_send(ModuleAddress::Controller as u8,
                                            SignbusFrameType::CommandFrame as u8,
                                            SignbusApiType::InitializationApiType as u8,
                                            InitMessageType::Declare as u8,
                                            1,
                                            buf);
        });
    }

    // Use mod out/in gpio to request isolation.
    pub fn signpost_initialization_request_isolation(&self) {
        debug!("Request I2C isolation");
        // intialize mod out/in gpio
        self.port_layer.mod_out_set();
        self.port_layer.debug_led_off();
        self.port_layer.mod_in_enable_interrupt();

        // pull mod out low to signal controller
        // wait on controller interrupt on mod_in
        self.port_layer.mod_out_clear();
        self.port_layer.debug_led_on();
    }

    // Intialize HAIL
    pub fn signpost_initialization_module_init(&self, i2c_address: u8) {
        debug!("Start Initialization");
        // intialize lower layers
        self.io_layer.signbus_io_init(i2c_address);
        self.device_address.set(i2c_address);

        // listen for messages
        self.recv_buf.take().map(|buf| { self.app_layer.signbus_app_recv(buf); });

        // communicate with controller and request 1:1 talk (isolation)
        self.signpost_initialization_request_isolation();
    }
}

impl<'a> port_layer::PortLayerClientGPIOTimer for SignbusInitialization<'a> {
    // Called when the mod_in GPIO goes low.
    fn mod_in_interrupt(&self) {
        self.delay_state.set(DelayState::RequestIsolation);
        self.port_layer.delay_ms(50);
    }

    // Called when a delay_ms has completed.
    fn delay_complete(&self) {
        match self.delay_state.get() {

            DelayState::Idle => {}

            DelayState::RequestIsolation => {
                match self.port_layer.mod_in_read() {

                    ReturnCode::SuccessWithValue { value } => {
                        if value != 0 {
                            debug!("Spurrious interrupt");
                        } else {
                            self.signpost_initialization_declare_controller();
                        }
                    }

                    _ => {}
                }
            }
        }
    }
}


impl<'a> app_layer::AppLayerClient for SignbusInitialization<'a> {
    // Called when a new packet is received over I2C.
    fn packet_received(&self, data: &'static mut [u8], length: usize, error: support::Error) {
        match error {
            support::Error::AddressNak => debug!("Error: AddressNak"),
            support::Error::DataNak => debug!("Error: DataNak"),
            support::Error::ArbitrationLost => debug!("Error: ArbitrationNak"),
            support::Error::CommandComplete => debug!("Command Complete!"),
        };

        // signpost_initialization_declared_callback
        if length > 0 {
            // check incoming_api_type and incoming_message_type
            if data[1] == SignbusApiType::InitializationApiType as u8 &&
               data[2] == InitMessageType::Declare as u8 {
                debug!("Correct response for declaration.");
            } else {
                debug!("Incorrect response for declaration.");
            }

        } else {
            debug!("Error: Length = 0");
        }
        self.send_buf.replace(data);
    }
    // Called when an I2C master write command is complete.
    fn packet_sent(&self, data: &'static mut [u8], error: support::Error) {

        if error != support::Error::CommandComplete {
            debug!("Error: Packet sent incomplete");
        }

        self.send_buf.replace(data);
    }


    // Called when an I2C slave read has completed.
    fn packet_read_from_slave(&self) {}
}
