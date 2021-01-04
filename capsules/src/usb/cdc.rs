//! Communications Class Device for USB
//!
//! This capsule allows Tock to support a serial port over USB.

use core::cell::Cell;
use core::cmp;

use super::descriptors;
use super::descriptors::Buffer64;
use super::descriptors::CdcInterfaceDescriptor;
use super::descriptors::EndpointAddress;
use super::descriptors::EndpointDescriptor;
use super::descriptors::InterfaceDescriptor;
use super::descriptors::TransferDirection;
use super::usbc_client_ctrl::ClientCtrl;

use kernel::common::cells::OptionalCell;
use kernel::common::cells::TakeCell;
use kernel::common::cells::VolatileCell;
use kernel::common::dynamic_deferred_call::{
    DeferredCallHandle, DynamicDeferredCall, DynamicDeferredCallClient,
};
use kernel::hil;
use kernel::hil::time::{Alarm, AlarmClient};
use kernel::hil::uart;
use kernel::hil::usb::TransferType;
use kernel::ReturnCode;

/// Identifying number for the endpoint when transferring data from us to the
/// host.
const ENDPOINT_IN_NUM: usize = 2;
/// Identifying number for the endpoint when transferring data from the host to
/// us.
const ENDPOINT_OUT_NUM: usize = 3;

static LANGUAGES: &'static [u16; 1] = &[
    0x0409, // English (United States)
];
/// Platform-specific packet length for the `SAM4L` USB hardware.
pub const MAX_CTRL_PACKET_SIZE_SAM4L: u8 = 8;
/// Platform-specific packet length for the `nRF52` USB hardware.
pub const MAX_CTRL_PACKET_SIZE_NRF52840: u8 = 64;
/// Platform-specific packet length for the `earlgrey` USB hardware.
pub const MAX_CTRL_PACKET_SIZE_EARLGREY: u8 = 64;
/// Number of ms to buffer uart transmissions before beginning to drop them.
/// This is useful in that it allows users time to connect over CDC without losing message,
/// while still guaranteeing that blocking uart transmissions eventually get a callback even
/// if a debug output is not connected.
pub const CDC_BUFFER_TIMEOUT_MS: u32 = 10000;

const N_ENDPOINTS: usize = 3;

/// States of the CDC driver.
#[derive(Debug, Copy, Clone, PartialEq)]
enum State {
    /// Default state. User must call `enable()`.
    Disabled,
    /// `enable()` has been called. The descriptor format has been passed to the
    /// hardware.
    Enabled,
    /// `attach()` has been called. The hardware should be ready for a host to
    /// connect.
    Attached,
    /// The host has enumerated this USB device. Things should be functional at
    /// this point.
    Enumerated,
    /// We have seen the CDC messages that we expect to signal that a CDC client
    /// has connected. We stay in the "connecting" state until the USB transfer
    /// has completed.
    Connecting,
    /// A CDC client is connected. We can safely send data.
    Connected,
}

/// States of the Control Endpoint related to CDC-ACM.
#[derive(Debug, Copy, Clone, PartialEq)]
enum CtrlState {
    /// No ongoing ctrl transcation.
    Idle,
    /// Host has sent a SET_LINE_CODING configuration request.
    SetLineCoding,
}

#[derive(PartialEq)]
enum CDCCntrlMessage {
    NotSupported,
    SetLineCoding = 0x20,
    SetControlLineState = 0x22,
    SendBreak = 0x23,
}

impl From<u8> for CDCCntrlMessage {
    fn from(num: u8) -> Self {
        match num {
            0x20 => CDCCntrlMessage::SetLineCoding,
            0x22 => CDCCntrlMessage::SetControlLineState,
            0x23 => CDCCntrlMessage::SendBreak,
            _ => CDCCntrlMessage::NotSupported,
        }
    }
}

/// Implementation of the Abstract Control Model (ACM) for the Communications
/// Class Device (CDC) over USB.
pub struct CdcAcm<'a, U: 'a, A: 'a + Alarm<'a>> {
    /// Helper USB client library for handling many USB operations.
    client_ctrl: ClientCtrl<'a, 'static, U>,

    /// 64 byte buffers for each endpoint.
    buffers: [Buffer64; N_ENDPOINTS],

    /// Current state of the CDC driver. This helps us track if a CDC client is
    /// connected and listening or not.
    state: Cell<State>,

    /// Current state of the Control Endpoint. This tracks which configuration
    /// request the host is currently sending us.
    ctrl_state: Cell<CtrlState>,

    /// A holder reference for the TX buffer we are transmitting from.
    tx_buffer: TakeCell<'static, [u8]>,
    /// The number of bytes the client has asked us to send. We track this so we
    /// can pass it back to the client when the transmission has finished.
    tx_len: Cell<usize>,
    /// Where in the `tx_buffer` we need to start sending from when we continue.
    tx_offset: Cell<usize>,
    /// The TX client to use when transmissions finish.
    tx_client: OptionalCell<&'a dyn uart::TransmitClient>,

    /// A holder for the buffer to receive bytes into. We use this as a flag as
    /// well, if we have a buffer then we are actively doing a receive.
    rx_buffer: TakeCell<'static, [u8]>,
    /// How many bytes the client wants us to receive.
    rx_len: Cell<usize>,
    /// How many bytes we have received so far.
    rx_offset: Cell<usize>,
    /// The RX client to use when RX data is received.
    rx_client: OptionalCell<&'a dyn uart::ReceiveClient>,

    /// Alarm used to indicate that data should be dropped and callbacks
    /// returned.
    timeout_alarm: &'a A,
    /// Used to track whether we are in the initial boot up period during which
    /// messages can be queued despite a CDC host not being connected (which is
    /// useful for ensuring debug messages early in the boot process can be
    /// delivered over the console).
    boot_period: Cell<bool>,

    /// Deferred Caller
    deferred_caller: &'a DynamicDeferredCall,
    /// Deferred Call Handle
    handle: OptionalCell<DeferredCallHandle>,
    /// Flag to mark we are waiting on a deferred call for dropping a TX. This
    /// can happen if an upper layer told us to transmit a buffer, but there is
    /// no host connected and therefore we cannot actually transmit. However,
    /// normal UART semantics are that we can always send (perhaps with a
    /// delay), even if nothing is actually listening. To keep the upper layers
    /// happy and to allow this CDC layer to just drop messages, we always
    /// return SUCCESS for TX, and then use a deferred call to signal the
    /// transmit done callback.
    deferred_call_pending_droptx: Cell<bool>,
    /// Flag to mark we need a deferred call to signal a callback after an RX
    /// abort occurs.
    deferred_call_pending_abortrx: Cell<bool>,

    /// Optional host-initiated function. This function (if supplied) is called
    /// when the host sends a special message to the device. The normal signal
    /// for calling this function is the host configuring the baud rate to be
    /// 1200 baud.
    ///
    /// This was originally added for the bootloader to allow the host to tell
    /// the device to enter bootloader mode.
    host_initiated_function: Option<&'a (dyn Fn() + 'a)>,
}

impl<'a, U: hil::usb::UsbController<'a>, A: 'a + Alarm<'a>> CdcAcm<'a, U, A> {
    pub fn new(
        controller: &'a U,
        max_ctrl_packet_size: u8,
        vendor_id: u16,
        product_id: u16,
        strings: &'static [&'static str; 3],
        timeout_alarm: &'a A,
        deferred_caller: &'a DynamicDeferredCall,
        host_initiated_function: Option<&'a (dyn Fn() + 'a)>,
    ) -> Self {
        let interfaces: &mut [InterfaceDescriptor] = &mut [
            InterfaceDescriptor {
                interface_number: 0,
                interface_class: 0x02,    // CDC communication
                interface_subclass: 0x02, // abstract control model (ACM)
                interface_protocol: 0x01, // V.25ter (AT commands)
                ..InterfaceDescriptor::default()
            },
            InterfaceDescriptor {
                interface_number: 1,
                interface_class: 0x0a,    // CDC data
                interface_subclass: 0x00, // none
                interface_protocol: 0x00, // none
                ..InterfaceDescriptor::default()
            },
        ];

        let cdc_descriptors: &mut [CdcInterfaceDescriptor] = &mut [
            CdcInterfaceDescriptor {
                subtype: descriptors::CdcInterfaceDescriptorSubType::Header,
                field1: 0x10, // CDC
                field2: 0x11, // CDC
            },
            CdcInterfaceDescriptor {
                subtype: descriptors::CdcInterfaceDescriptorSubType::CallManagement,
                field1: 0x00, // Capabilities
                field2: 0x01, // Data interface 1
            },
            CdcInterfaceDescriptor {
                subtype: descriptors::CdcInterfaceDescriptorSubType::AbstractControlManagement,
                field1: 0x06, // Capabilities
                field2: 0x00, // unused
            },
            CdcInterfaceDescriptor {
                subtype: descriptors::CdcInterfaceDescriptorSubType::Union,
                field1: 0x00, // Interface 0
                field2: 0x01, // Interface 1
            },
        ];

        let endpoints: &[&[EndpointDescriptor]] = &[
            &[EndpointDescriptor {
                endpoint_address: EndpointAddress::new_const(4, TransferDirection::DeviceToHost),
                transfer_type: TransferType::Interrupt,
                max_packet_size: 8,
                interval: 16,
            }],
            &[
                EndpointDescriptor {
                    endpoint_address: EndpointAddress::new_const(
                        2,
                        TransferDirection::DeviceToHost,
                    ),
                    transfer_type: TransferType::Bulk,
                    max_packet_size: 64,
                    interval: 0,
                },
                EndpointDescriptor {
                    endpoint_address: EndpointAddress::new_const(
                        3,
                        TransferDirection::HostToDevice,
                    ),
                    transfer_type: TransferType::Bulk,
                    max_packet_size: 64,
                    interval: 0,
                },
            ],
        ];

        let (device_descriptor_buffer, other_descriptor_buffer) =
            descriptors::create_descriptor_buffers(
                descriptors::DeviceDescriptor {
                    vendor_id: vendor_id,
                    product_id: product_id,
                    manufacturer_string: 1,
                    product_string: 2,
                    serial_number_string: 3,
                    class: 0x2, // Class: CDC
                    max_packet_size_ep0: max_ctrl_packet_size,
                    ..descriptors::DeviceDescriptor::default()
                },
                descriptors::ConfigurationDescriptor {
                    ..descriptors::ConfigurationDescriptor::default()
                },
                interfaces,
                endpoints,
                None, // No HID descriptor
                Some(cdc_descriptors),
            );

        Self {
            client_ctrl: ClientCtrl::new(
                controller,
                device_descriptor_buffer,
                other_descriptor_buffer,
                None, // No HID descriptor
                None, // No report descriptor
                LANGUAGES,
                strings,
            ),
            buffers: [
                Buffer64::default(),
                Buffer64::default(),
                Buffer64::default(),
            ],
            state: Cell::new(State::Disabled),
            ctrl_state: Cell::new(CtrlState::Idle),
            tx_buffer: TakeCell::empty(),
            tx_len: Cell::new(0),
            tx_offset: Cell::new(0),
            tx_client: OptionalCell::empty(),
            rx_buffer: TakeCell::empty(),
            rx_len: Cell::new(0),
            rx_offset: Cell::new(0),
            rx_client: OptionalCell::empty(),
            timeout_alarm,
            boot_period: Cell::new(true),
            deferred_caller,
            handle: OptionalCell::empty(),
            deferred_call_pending_droptx: Cell::new(false),
            deferred_call_pending_abortrx: Cell::new(false),
            host_initiated_function,
        }
    }

    pub fn initialize_callback_handle(&self, handle: DeferredCallHandle) {
        self.handle.replace(handle);
    }

    #[inline]
    pub fn controller(&self) -> &'a U {
        self.client_ctrl.controller()
    }

    #[inline]
    fn buffer(&'a self, i: usize) -> &'a [VolatileCell<u8>; 64] {
        &self.buffers[i - 1].buf
    }

    /// This is a helper function used to indicate successful uart transmission to
    /// a higher layer client despite not actually being connected to a host. Allows
    /// blocking debug interfaces to function in the same way they do when an actual UART
    /// interface is in use. This should only be called in an upcall.
    fn indicate_tx_success(&self) {
        self.tx_len.set(0);
        self.tx_offset.set(0);
        self.tx_client.map(|client| {
            self.tx_buffer.take().map(|buf| {
                client.transmitted_buffer(buf, 0, ReturnCode::FAIL);
            });
        });
    }
}

impl<'a, U: hil::usb::UsbController<'a>, A: 'a + Alarm<'a>> hil::usb::Client<'a>
    for CdcAcm<'a, U, A>
{
    fn enable(&'a self) {
        // Set up the default control endpoint
        self.client_ctrl.enable();

        // Setup buffers for IN and OUT data transfer.
        self.controller()
            .endpoint_set_in_buffer(ENDPOINT_IN_NUM, self.buffer(ENDPOINT_IN_NUM));
        self.controller()
            .endpoint_in_enable(TransferType::Bulk, ENDPOINT_IN_NUM);

        self.controller()
            .endpoint_set_out_buffer(ENDPOINT_OUT_NUM, self.buffer(ENDPOINT_OUT_NUM));
        self.controller()
            .endpoint_out_enable(TransferType::Bulk, ENDPOINT_OUT_NUM);

        self.state.set(State::Enabled);

        self.timeout_alarm.set_alarm(
            self.timeout_alarm.now(),
            A::ticks_from_ms(CDC_BUFFER_TIMEOUT_MS),
        );
    }

    fn attach(&'a self) {
        self.client_ctrl.attach();
        self.state.set(State::Attached);
    }

    fn bus_reset(&'a self) {
        // We take a bus reset to mean the enumeration has finished.
        self.state.set(State::Enumerated);
    }

    /// Handle a Control Setup transaction.
    ///
    /// CDC uses special values here, and we can use these to know when a CDC
    /// client is connected or not.
    fn ctrl_setup(&'a self, endpoint: usize) -> hil::usb::CtrlSetupResult {
        descriptors::SetupData::get(&self.client_ctrl.ctrl_buffer.buf).map(|setup_data| {
            let b_request = setup_data.request_code;

            match CDCCntrlMessage::from(b_request) {
                CDCCntrlMessage::SetLineCoding => {
                    self.ctrl_state.set(CtrlState::SetLineCoding);
                }
                CDCCntrlMessage::SetControlLineState => {
                    // Bit 0 and 1 of the value (setup_data.value) can be set
                    // D0: Indicates to DCE if DTE is present or not.
                    //     - 0 -> Not present
                    //     - 1 -> Present
                    // D1: Carrier control for half duplex modems.
                    //     - 0 -> Deactivate carrier
                    //     - 1 -> Activate carrier
                    // Currently we don't care about the value
                }
                CDCCntrlMessage::SendBreak => {
                    // On Mac, we seem to get the SEND_BREAK to signal that a
                    // client disconnects.
                    self.state.set(State::Enumerated)
                }
                _ => {}
            }
        });

        self.client_ctrl.ctrl_setup(endpoint)
    }

    /// Handle a Control In transaction
    fn ctrl_in(&'a self, endpoint: usize) -> hil::usb::CtrlInResult {
        self.client_ctrl.ctrl_in(endpoint)
    }

    /// Handle a Control Out transaction
    fn ctrl_out(&'a self, endpoint: usize, packet_bytes: u32) -> hil::usb::CtrlOutResult {
        // Check what state our Ctrl endpoint is in.
        if self.ctrl_state.get() == CtrlState::SetLineCoding {
            // We got a Ctrl SET_LINE_CODING setup, now we are getting the data.
            // We can parse the data we got.
            descriptors::CdcAcmSetLineCodingData::get(&self.client_ctrl.ctrl_buffer.buf).map(
                |line_coding| {
                    // Check if we should switch our main state machine to
                    // connecting meaning that the host is connecting to the virtual
                    // serial port. We decide this based on if the host is
                    // configuring the baud rate to what we expect.
                    if self.state.get() == State::Enumerated && line_coding.baud_rate == 115200 {
                        self.state.set(State::Connecting);
                    }

                    // Check if the baud rate we got matches the special flag
                    // value (1200 baud). If so, we run an optional function
                    // provided when the CDC stack was configured.
                    if line_coding.baud_rate == 1200 {
                        self.host_initiated_function.map(|f| {
                            f();
                        });
                    }
                },
            );
        }

        self.client_ctrl.ctrl_out(endpoint, packet_bytes)
    }

    fn ctrl_status(&'a self, endpoint: usize) {
        self.client_ctrl.ctrl_status(endpoint)
    }

    /// Handle the completion of a Control transfer
    fn ctrl_status_complete(&'a self, endpoint: usize) {
        self.ctrl_state.set(CtrlState::Idle);

        // Here we check to see if we just got connected to a CDC client. If so,
        // we can begin transmitting if needed.
        if self.state.get() == State::Connecting {
            self.state.set(State::Connected);
            if self.tx_buffer.is_some() {
                self.controller().endpoint_resume_in(ENDPOINT_IN_NUM);
            }
        }

        self.client_ctrl.ctrl_status_complete(endpoint)
    }

    /// Handle a Bulk/Interrupt IN transaction.
    ///
    /// This is called when we can send data to the host. It should get called
    /// when we tell the controller we want to resume the IN endpoint (meaning
    /// we know we have data to send) and afterwards until we return
    /// `hil::usb::InResult::Delay` from this function. That means we can use
    /// this as a callback to mean that the transmission finished by waiting
    /// until this function is called when we don't have anything left to send.
    fn packet_in(&'a self, transfer_type: TransferType, endpoint: usize) -> hil::usb::InResult {
        match transfer_type {
            TransferType::Bulk => {
                self.tx_buffer
                    .take()
                    .map_or(hil::usb::InResult::Delay, |tx_buf| {
                        // Check if we have any bytes to send.
                        let offset = self.tx_offset.get();
                        let remaining = self.tx_len.get() - offset;
                        if remaining > 0 {
                            // We do, so we go ahead and send those.

                            // Get packet that we have shared with the underlying
                            // USB stack to copy the tx into.
                            let packet = self.buffer(endpoint);

                            // Calculate how much more we can send.
                            let to_send = cmp::min(packet.len(), remaining);

                            // Copy from the TX buffer to the outgoing USB packet.
                            for i in 0..to_send {
                                packet[i].set(tx_buf[offset + i]);
                            }

                            // Update our state on how much more there is to send.
                            self.tx_offset.set(offset + to_send);

                            // Put the TX buffer back so we can keep sending from it.
                            self.tx_buffer.replace(tx_buf);

                            // Return that we have data to send.
                            hil::usb::InResult::Packet(to_send)
                        } else {
                            // We don't have anything to send, so that means we are
                            // ok to signal the callback.

                            // Signal the callback and pass back the TX buffer.
                            self.tx_client.map(move |tx_client| {
                                tx_client.transmitted_buffer(
                                    tx_buf,
                                    self.tx_len.get(),
                                    ReturnCode::SUCCESS,
                                )
                            });

                            // Return that we have nothing else to do to the USB
                            // driver.
                            hil::usb::InResult::Delay
                        }
                    })
            }
            TransferType::Control | TransferType::Isochronous | TransferType::Interrupt => {
                // Nothing to do for CDC ACM.
                hil::usb::InResult::Delay
            }
        }
    }

    /// Handle a Bulk/Interrupt OUT transaction
    fn packet_out(
        &'a self,
        transfer_type: TransferType,
        endpoint: usize,
        packet_bytes: u32,
    ) -> hil::usb::OutResult {
        match transfer_type {
            TransferType::Bulk => {
                // Start by checking to see if we even care about this RX or
                // not.
                self.rx_buffer.take().map(|rx_buf| {
                    let rx_offset = self.rx_offset.get();

                    // How many more bytes can we store in our RX buffer?
                    let available_bytes = rx_buf.len() - rx_offset;
                    let copy_length = cmp::min(packet_bytes as usize, available_bytes);

                    // Do the copy into the RX buffer.
                    let packet = self.buffer(endpoint);
                    for i in 0..copy_length {
                        rx_buf[rx_offset + i] = packet[i].get();
                    }

                    // Keep track of how many bytes we have received so far.
                    let total_received_bytes = rx_offset + copy_length;

                    // Update how many bytes we have gotten.
                    self.rx_offset.set(total_received_bytes);

                    // Check if we have received at least as many bytes as the
                    // client asked for.
                    if total_received_bytes >= self.rx_len.get() {
                        self.rx_client.map(move |client| {
                            client.received_buffer(
                                rx_buf,
                                total_received_bytes,
                                ReturnCode::SUCCESS,
                                uart::Error::None,
                            );
                        });
                    } else {
                        // Make sure to put the RX buffer back.
                        self.rx_buffer.replace(rx_buf);
                    }
                });

                // No error cases to report to the USB.
                hil::usb::OutResult::Ok
            }
            TransferType::Control | TransferType::Isochronous | TransferType::Interrupt => {
                // Nothing to do for CDC ACM.
                hil::usb::OutResult::Ok
            }
        }
    }

    fn packet_transmitted(&'a self, _endpoint: usize) {
        // Check if more to send.
        self.tx_buffer.take().map(|tx_buf| {
            // Check if we have any bytes to send.
            let remaining = self.tx_len.get() - self.tx_offset.get();
            if remaining > 0 {
                // We do, so ask to send again.
                self.tx_buffer.replace(tx_buf);
                self.controller().endpoint_resume_in(ENDPOINT_IN_NUM);
            } else {
                // We don't have anything to send, so that means we are
                // ok to signal the callback.

                // Signal the callback and pass back the TX buffer.
                self.tx_client.map(move |tx_client| {
                    tx_client.transmitted_buffer(tx_buf, self.tx_len.get(), ReturnCode::SUCCESS)
                });
            }
        });
    }
}

impl<'a, U: hil::usb::UsbController<'a>, A: 'a + Alarm<'a>> uart::Configure for CdcAcm<'a, U, A> {
    fn configure(&self, _parameters: uart::Parameters) -> ReturnCode {
        // Since this is not a real UART, we don't need to consider these
        // parameters.
        ReturnCode::SUCCESS
    }
}

impl<'a, U: hil::usb::UsbController<'a>, A: 'a + Alarm<'a>> uart::Transmit<'a>
    for CdcAcm<'a, U, A>
{
    fn set_transmit_client(&self, client: &'a dyn uart::TransmitClient) {
        self.tx_client.set(client);
    }

    fn transmit_buffer(
        &self,
        tx_buffer: &'static mut [u8],
        tx_len: usize,
    ) -> (ReturnCode, Option<&'static mut [u8]>) {
        if self.tx_buffer.is_some() {
            // We are already handling a transmission, we cannot queue another
            // request.
            (ReturnCode::EBUSY, Some(tx_buffer))
        } else if tx_len > tx_buffer.len() {
            // Can't send more bytes than will fit in the buffer.
            (ReturnCode::ESIZE, Some(tx_buffer))
        } else {
            // Ok, we can handle this transmission. Initialize all of our state
            // for our TX state machine.
            self.tx_len.set(tx_len);
            self.tx_offset.set(0);
            self.tx_buffer.replace(tx_buffer);

            // Don't try to send if there is no CDC client connected.
            if self.state.get() == State::Connected {
                // Then signal to the lower layer that we are ready to do a TX
                // by putting data in the IN endpoint.
                self.controller().endpoint_resume_in(ENDPOINT_IN_NUM);
                (ReturnCode::SUCCESS, None)
            } else if self.boot_period.get() {
                // indicate success because we will try to send it once a host connects
                (ReturnCode::SUCCESS, None)
            } else {
                // indicate success, but we will not actually queue this message -- just schedule
                // a deferred callback to return the buffer immediately.
                self.deferred_call_pending_droptx.set(true);
                self.handle.map(|handle| self.deferred_caller.set(*handle));
                (ReturnCode::SUCCESS, None)
            }
        }
    }

    fn transmit_abort(&self) -> ReturnCode {
        ReturnCode::FAIL
    }

    fn transmit_word(&self, _word: u32) -> ReturnCode {
        ReturnCode::FAIL
    }
}

impl<'a, U: hil::usb::UsbController<'a>, A: 'a + Alarm<'a>> uart::Receive<'a> for CdcAcm<'a, U, A> {
    fn set_receive_client(&self, client: &'a dyn uart::ReceiveClient) {
        self.rx_client.set(client);
    }

    fn receive_buffer(
        &self,
        rx_buffer: &'static mut [u8],
        rx_len: usize,
    ) -> (ReturnCode, Option<&'static mut [u8]>) {
        if self.rx_buffer.is_some() {
            (ReturnCode::EBUSY, Some(rx_buffer))
        } else if rx_len > rx_buffer.len() {
            (ReturnCode::ESIZE, Some(rx_buffer))
        } else {
            self.rx_buffer.replace(rx_buffer);
            self.rx_offset.set(0);
            self.rx_len.set(rx_len);

            (ReturnCode::SUCCESS, None)
        }
    }

    fn receive_abort(&self) -> ReturnCode {
        if self.rx_buffer.is_none() {
            // If we have nothing pending then aborting is very easy.
            ReturnCode::SUCCESS
        } else {
            // If we do have a receive pending then we need to start a deferred
            // call to set the callback and return `EBUSY`.
            self.deferred_call_pending_abortrx.set(true);
            self.handle.map(|handle| self.deferred_caller.set(*handle));
            ReturnCode::EBUSY
        }
    }

    fn receive_word(&self) -> ReturnCode {
        ReturnCode::FAIL
    }
}

impl<'a, U: hil::usb::UsbController<'a>, A: 'a + Alarm<'a>> AlarmClient for CdcAcm<'a, U, A> {
    fn alarm(&self) {
        self.boot_period.set(false);
        if self.state.get() == State::Connected {
            // we are already connected, so any queued messages are going to be sent.
            // do nothing.
        } else {
            // no client has connected, but we do not want to block indefinitely, so go ahead
            // and deliver a callback.
            self.indicate_tx_success();
        }
    }
}

impl<'a, U: hil::usb::UsbController<'a>, A: 'a + Alarm<'a>> DynamicDeferredCallClient
    for CdcAcm<'a, U, A>
{
    fn call(&self, _handle: DeferredCallHandle) {
        if self.deferred_call_pending_droptx.replace(false) {
            self.indicate_tx_success()
        }

        if self.deferred_call_pending_abortrx.replace(false) {
            // Signal the RX callback with ECANCEL error.
            self.rx_buffer.take().map(|rx_buf| {
                let rx_offset = self.rx_offset.get();

                // The total number of bytes we have received so far.
                let total_received_bytes = rx_offset;

                self.rx_client.map(move |client| {
                    client.received_buffer(
                        rx_buf,
                        total_received_bytes,
                        ReturnCode::ECANCEL,
                        uart::Error::None,
                    );
                });
            });
        }
    }
}

impl<'a, U: hil::usb::UsbController<'a>, A: 'a + Alarm<'a>> uart::Uart<'a> for CdcAcm<'a, U, A> {}
impl<'a, U: hil::usb::UsbController<'a>, A: 'a + Alarm<'a>> uart::UartData<'a>
    for CdcAcm<'a, U, A>
{
}
