//! SAM4L USB controller

pub mod data;

#[macro_use]
mod common;
mod registers;

use self::common::register::*;
use self::data::*;
use self::registers::*;

use core::cell::Cell;
use core::fmt;
use core::slice;
use kernel::common::VolatileCell;
use kernel::common::take_cell::MapCell;
use kernel::hil;
use kernel::hil::usb::*;

use pm;
use pm::{Clock, HSBClock, PBBClock, enable_clock, disable_clock};
use scif;

macro_rules! client_err {
    [ $msg:expr ] => {
        debug!($msg)
    };
}

/// State for managing the USB controller
// This ensures the `descriptors` field is laid out first
#[repr(C)]
// This provides the required alignment for the `descriptors` field
#[repr(align(8))]
pub struct Usbc<'a> {
    descriptors: [Endpoint; 8],
    client: Option<&'a hil::usb::Client>,
    state: MapCell<State>,
}

#[derive(Default)]
pub struct EP {
    pub index: u32,
}

impl<'a> UsbController for Usbc<'a> {
    type EndpointState = Cell<Option<EP>>;

    fn attach(&self) {
        self._attach();
    }

    fn enable_device(&self, full_speed: bool) {
        let speed = if full_speed { Speed::Full } else { Speed::Low };
        self._enable(Mode::device_at_speed(speed));
    }

    fn endpoint_configure(&self, e: &'static Self::EndpointState, index: u32) {
        e.set(Some(EP { index: index }));
    }

    fn endpoint_set_buffer<'b>(&'b self, e: u32, buf: &[VolatileCell<u8>]) {
        if buf.len() != 8 {
            panic!("Bad endpoint buffer size");
        }
        self.endpoint_bank_set_buffer(EndpointIndex::new(e), BankIndex::Bank0, buf);
    }

    fn endpoint_ctrl_out_enable(&self, e: u32) {
        let cfg = EndpointConfig::new(BankCount::Single,
                                      EndpointSize::Bytes8,
                                      EndpointDirection::Out,
                                      EndpointType::Control,
                                      EndpointIndex::new(e));
        self.endpoint_enable(e, cfg);
    }

    fn set_address(&self, addr: u16) {
        // The hardware can do only 7-bit addresses
        let addr = (addr as u8) & 0b1111111;

        UDCON_ADDEN.write(false);
        UDCON_UADD.write(addr);

        debug!("Set Address = {}", addr);
    }

    fn enable_address(&self) {
        UDCON_ADDEN.write(true);

        debug!("Enable Address = {}", UDCON.read() & 0b1111111);
    }
}

impl<'a> Usbc<'a> {
    const fn new() -> Self {
        Usbc {
            client: None,
            state: MapCell::new(State::Reset),
            descriptors: [new_endpoint(),
                          new_endpoint(),
                          new_endpoint(),
                          new_endpoint(),
                          new_endpoint(),
                          new_endpoint(),
                          new_endpoint(),
                          new_endpoint()],
        }
    }

    /// Attach to the USB bus after enabling USB clock
    fn _attach(&self) {
        self.state.map(|state| {
            match *state {
                State::Reset => {
                    client_err!("Not enabled");
                }
                State::Active(_) => {
                    client_err!("Already attached");
                }
                State::Idle(mode) => {
                    if pm::get_system_frequency() == 48000000 {
                        // XX: not clear that this always results in a usable USB clock
                        scif::generic_clock_enable(scif::GenericClock::GCLK7,
                                                   scif::ClockSource::CLK_HSB);

                        while !USBSTA_CLKUSABLE.read() {}

                        UDCON_DETACH.write(false);
                        debug!("Attached.");

                        *state = State::Active(mode);
                    } else {
                        debug!("The system clock does not support USB");
                    }
                }
            }
        });
    }

    /// Detach from the USB bus.  Also disable USB clock to save energy.
    fn _detach(&self) {
        self.state.map(|state| match *state {
            State::Reset => {
                client_err!("Not enabled");
            }
            State::Idle(_) => {
                client_err!("Not attached");
            }
            State::Active(mode) => {
                UDCON_DETACH.write(true);

                scif::generic_clock_disable(scif::GenericClock::GCLK7);

                *state = State::Idle(mode);
            }
        });
    }


    /// Enable the controller's clocks and interrupt and transition to Idle state
    /// (No effect if current state is not Reset)
    pub fn _enable(&self, mode: Mode) {
        self.state.map(|state| {
            match *state {
                State::Reset => {
                    unsafe {
                        // Are the USBC clocks enabled at reset?
                        //   10.7.4 says no, but 17.5.3 says yes
                        // Also, "Being in Idle state does not require the USB clocks to
                        //   be activated" (17.6.2)
                        enable_clock(Clock::HSB(HSBClock::USBC));
                        enable_clock(Clock::PBB(PBBClock::USBC));

                        // If we got to this state via disable() instead of chip reset,
                        // the values USBCON.FRZCLK, USBCON.UIMOD, UDCON.LS have *not* been
                        // reset to their default values.

                        if let Mode::Device { speed, .. } = mode {
                            UDCON_LS.write(speed)
                        }

                        USBCON_UIMOD.write(mode); // see registers.rs: maybe wrong bit?
                        USBCON_FRZCLK.write(false);
                        USBCON_USBE.write(true);

                        UDESC.write(&self.descriptors as *const _ as u32);

                        // Device interrupts
                        let udints = // UDINT_SUSP |
                                     // UDINT_SOF |
                                     UDINT_EORST |
                                     UDINT_EORSM |
                                     UDINT_UPRSM;

                        // Clear pending device global interrupts
                        UDINTCLR.write(udints);

                        // Enable device global interrupts
                        UDINTESET.write(udints);

                        debug!("Enabled.");
                    }
                    *state = State::Idle(mode);
                }
                _ => client_err!("Already enabled"),
            }
        });
    }

    fn _active(&self) -> bool {
        self.state.map_or(false, |state| match *state {
            State::Active(_) => true,
            _ => false,
        })
    }

    /// Disable the controller, its interrupt, and its clocks
    fn _disable(&self) {
        if self._active() {
            self._detach();
        }

        self.state.map(|state| if *state != State::Reset {
            unsafe {
                USBCON_USBE.write(false);

                disable_clock(Clock::PBB(PBBClock::USBC));
                disable_clock(Clock::HSB(HSBClock::USBC));
            }
            *state = State::Reset;
        });
    }

    /// Provide a buffer for transfers in and out of the given endpoint
    pub fn endpoint_bank_set_buffer(&self,
                                    endpoint: EndpointIndex,
                                    bank: BankIndex,
                                    buf: &[VolatileCell<u8>]) {
        let e: usize = From::from(endpoint);
        let b: usize = From::from(bank);
        let p = buf.as_ptr() as *mut u8;

        debug!("Set Endpoint{}/Bank{} addr={:8?}", e, b, p);
        self.descriptors[e][b].set_addr(p);
        self.descriptors[e][b].set_packet_size(PacketSize::default());
    }

    /// Configure and enable an endpoint
    /// (XX: include addr and packetsize?)
    pub fn endpoint_enable(&self, endpoint: u32, cfg: EndpointConfig) {
        self.state.map(|state| {
            // Record config in case of later reset
            match *state {
                State::Reset => {
                    client_err!("Not enabled");
                }
                State::Idle(Mode::Device { ref mut config, .. }) => {
                    *config = Some(cfg);
                }
                State::Active(Mode::Device { ref mut config, .. }) => {
                    *config = Some(cfg);
                }
                _ => {
                    client_err!("Not in Device mode");
                }
            }
        });

        // Enable the endpoint (meaning the controller will respond to requests)
        UERST.set_bit(endpoint);

        self.endpoint_configure(endpoint as usize, cfg);

        // Set EPnINTE, enabling interrupts for this endpoint
        UDINTESET.set_bit(12 + endpoint);

        debug!("Enabled endpoint {}", endpoint);
    }

    fn endpoint_configure(&self, endpoint: usize, cfg: EndpointConfig) {
        // Configure the endpoint
        UECFGn[endpoint].write(From::from(cfg));

        // Specify which endpoint interrupts we want, among:
        //      TXIN | RXOUT | RXSTP | NAKOUT | NAKIN |
        //      ERRORF | STALLED | CRCERR | RAMACERR
        endpoint_enable_only_interrupts(endpoint, RXSTP | RAMACERR);

        // XX: Set endpoint state to Init
    }

    /// Set a client to receive data from the USBC
    pub fn set_client(&mut self, client: &'a hil::usb::Client) {
        self.client = Some(client);
    }

    /// Handle an interrupt from the USBC
    pub fn handle_interrupt(&mut self) {
        // TODO: Use a cell type with get_mut() so we don't have to copy the state value around
        let mut state = self.state.take().unwrap_or(State::Reset);

        match state {
            State::Reset => panic!("Not reached"),
            State::Idle(_) => panic!("Not reached"),
            State::Active(ref mut mode) => {
                match *mode {
                    Mode::Device { speed, ref config, ref mut state } => {
                        self.handle_device_interrupt(speed, config, state)
                    }
                    Mode::Host => panic!("Unimplemented"),
                }
            }
        }

        self.state.replace(state);
    }

    fn handle_device_interrupt(&mut self,
                               speed: Speed,
                               config: &Option<EndpointConfig>,
                               dstate: &mut DeviceState) {

        let udint: u32 = UDINT.read();

        // debug!("--> UDINT={:?} {:?}", UdintFlags(udint), *dstate);

        if udint & UDINT_EORST != 0 {
            // Bus reset

            // Reconfigure what has been reset in the USBC
            UDCON_LS.write(speed);
            if let Some(ref config) = *config {
                self.endpoint_configure(0, *config);
            }

            // Re-initialize our record of the controller state
            *dstate = DeviceState::Init;

            // Alert the client
            self.client.map(|client| { client.bus_reset(); });
            debug!("USB Bus Reset");
            // debug_regs();

            // Acknowledge the interrupt
            UDINTCLR.write(UDINT_EORST);
        }

        if udint & UDINT_SUSP != 0 {
            // The transceiver has been suspended due to the bus being idle for 3ms.
            // This condition is over when WAKEUP is set.

            // "To further reduce power consumption it is recommended to freeze the USB
            // clock by writing a one to the Freeze USB Clock (FRZCLK) bit in USBCON when
            // the USB bus is in suspend mode.
            //
            // To recover from the suspend mode, the user shall wait for the Wakeup
            // (WAKEUP) interrupt bit, which is set when a non-idle event is detected, and
            // then write a zero to FRZCLK.
            //
            // As the WAKEUP interrupt bit in UDINT is set when a non-idle event is
            // detected, it can occur regardless of whether the controller is in the
            // suspend mode or not."

            // Subscribe to WAKEUP
            UDINTESET.write(UDINT_WAKEUP);

            // Acknowledge the "suspend" event
            UDINTCLR.write(UDINT_SUSP);
        }

        if udint & UDINT_WAKEUP != 0 {
            // If we were suspended: Unfreeze the clock (and unsleep the MCU)

            // Unsubscribe from WAKEUP
            UDINTECLR.write(UDINT_WAKEUP);

            // Acknowledge the interrupt
            UDINTCLR.write(UDINT_WAKEUP);

            // Continue processing, as WAKEUP is usually set
        }

        if udint & UDINT_SOF != 0 {
            // Acknowledge Start of frame
            UDINTCLR.write(UDINT_SOF);
        }

        if udint & UDINT_EORSM != 0 {
            // Controller received End of Resume
            debug!("UDINT EORSM");
        }

        if udint & UDINT_UPRSM != 0 {
            // Controller sent Upstream Resume
            debug!("UDINT UPRSM");
        }

        // Process per-endpoint interrupt flags
        for endpoint in 0..1 {
            if udint & (1 << (12 + endpoint)) == 0 {
                // No interrupts for this endpoint
                continue;
            }

            // Set to true to process more flags without waiting for another interrupt
            // (Using this with debugging messages tends to fill up the console buffer too fast.)
            // (Ignoring `again` should not cause incorrect behavior.)
            //
            // let mut again = true;
            // while again {
            //    again = false;
            {
                let status = UESTAn[endpoint].read();
                // debug!("UESTA{}={:?}", endpoint, UestaFlags(status));

                if status & STALLED != 0 {
                    debug!("D({}) STALLED/CRCERR", endpoint);

                    // Acknowledge
                    UESTAnCLR[endpoint].write(STALLED);
                }

                if status & RAMACERR != 0 {
                    debug!("D({}) RAMACERR", endpoint);

                    // Acknowledge
                    UESTAnCLR[endpoint].write(RAMACERR);
                }

                match *dstate {
                    DeviceState::Init => {
                        if status & RXSTP != 0 {
                            // We received a SETUP transaction

                            // debug!("D({}) RXSTP", endpoint);
                            // self.debug_show_d0();

                            let packet_bytes =
                                self.descriptors[0][0].packet_size.get().byte_count();
                            let result = if packet_bytes == 8 {
                                self.client.map(|c| c.ctrl_setup())
                            } else {
                                Some(CtrlSetupResult::ErrBadLength)
                            };

                            match result {
                                Some(CtrlSetupResult::Ok) => {
                                    if status & CTRLDIR != 0 {
                                        // The following Data stage will be IN

                                        *dstate = DeviceState::CtrlReadIn;

                                        // Wait until bank is clear to send
                                        // Also, wait for NAKOUT to signal end of IN stage
                                        // (The datasheet incorrectly says NAKIN)
                                        UESTAnCLR[endpoint].write(NAKOUT);
                                        endpoint_enable_only_interrupts(endpoint,
                                                                        RAMACERR | TXIN | NAKOUT);
                                    } else {
                                        // The following Data stage will be OUT

                                        *dstate = DeviceState::CtrlWriteOut;

                                        // Wait for OUT packets
                                        // Also, wait for NAKIN to signal end of OUT stage
                                        UESTAnCLR[endpoint].write(RXOUT);
                                        UESTAnCLR[endpoint].write(NAKIN);
                                        endpoint_enable_only_interrupts(endpoint,
                                                                        RAMACERR | RXOUT | NAKIN);
                                    }
                                }
                                failure => {
                                    // Respond with STALL to any following transactions
                                    // in this request
                                    UECONnSET[endpoint].write(STALLRQ);

                                    match failure {
                                        None => debug!("D({}) No client to handle Setup", endpoint),
                                        Some(err) => {
                                            debug!("D({}) Client err on Setup: {:?}", endpoint, err)
                                        }
                                    }

                                    endpoint_enable_only_interrupts(endpoint, RXSTP | RAMACERR);

                                    // Remain in DeviceState::Init for next SETUP
                                }
                            }

                            // Acknowledge
                            UESTAnCLR[endpoint].write(RXSTP);
                        }
                    }
                    DeviceState::CtrlReadIn => {
                        if status & NAKOUT != 0 {
                            // The host has completed the IN stage by sending an OUT token

                            endpoint_disable_interrupts(endpoint, TXIN | NAKOUT);

                            // debug!("D({}) NAKOUT");
                            self.client.map(|c| c.ctrl_status());

                            *dstate = DeviceState::CtrlReadStatus;

                            // Await end of Status stage
                            endpoint_enable_interrupts(endpoint, RXOUT);

                            // Acknowledge
                            UESTAnCLR[endpoint].write(NAKOUT);

                            // Run handler again in case the RXOUT has already arrived
                            // again = true;
                        } else if status & TXIN != 0 {
                            // The data bank is ready to receive another IN payload
                            // debug!("D({}) TXIN", endpoint);

                            let result = self.client.map(|c| {
                                // Allow client to write a packet payload to buffer
                                c.ctrl_in()
                            });
                            match result {
                                Some(CtrlInResult::Packet(packet_bytes, transfer_complete)) => {
                                    self.descriptors[0][0]
                                        .packet_size
                                        .set(if packet_bytes == 8 && transfer_complete {
                                            // Send a complete final packet, and request
                                            // that the controller also send a zero-length
                                            // packet to signal the end of transfer
                                            PacketSize::single_with_zlp(8)
                                        } else {
                                            // Send either a complete but not-final
                                            // packet, or a short and final packet (which
                                            // itself signals end of transfer)
                                            PacketSize::single(packet_bytes as u32)
                                        });

                                    // debug!("D({}) Send CTRL IN packet ({} bytes)",
                                    //        endpoint,
                                    //        packet_bytes);
                                    // self.debug_show_d0();

                                    if transfer_complete {
                                        // IN data completely sent.  Unsubscribe from TXIN.
                                        // (Continue awaiting NAKOUT to indicate end of Data stage)
                                        endpoint_disable_interrupts(endpoint, TXIN);
                                    } else {
                                        // Continue waiting for next TXIN
                                    }

                                    // Signal to the controller that the IN payload is
                                    // ready to send
                                    UESTAnCLR[endpoint].write(TXIN);
                                }
                                Some(CtrlInResult::Delay) => {
                                    endpoint_disable_interrupts(endpoint, TXIN);
                                    debug!("*** Client NAK");
                                    // XXX set busy bits?
                                    *dstate = DeviceState::CtrlInDelay;
                                }
                                _ => {
                                    // Respond with STALL to any following IN/OUT transactions
                                    UECONnSET[endpoint].write(STALLRQ);

                                    debug!("D({}) Client IN err => STALL", endpoint);

                                    *dstate = DeviceState::Init;

                                    // Wait for next SETUP
                                    endpoint_enable_interrupts(endpoint, RXSTP);
                                }
                            }
                        }
                    }
                    DeviceState::CtrlReadStatus => {
                        if status & RXOUT != 0 {
                            // Host has completed Status stage by sending an OUT packet

                            endpoint_disable_interrupts(endpoint, RXOUT);

                            // debug!("D({}) RXOUT: End of Control Read transaction", endpoint);
                            self.client.map(|c| c.ctrl_status_complete());

                            *dstate = DeviceState::Init;

                            // Wait for next SETUP
                            endpoint_enable_interrupts(endpoint, RXSTP);

                            // Acknowledge
                            UESTAnCLR[endpoint].write(RXOUT);
                        }
                    }
                    DeviceState::CtrlWriteOut => {
                        if status & RXOUT != 0 {
                            // Received data

                            // debug!("D({}) RXOUT: Received Control Write data", endpoint);
                            // self.debug_show_d0();
                            let result = self.client.map(|c| {
                                c.ctrl_out(self.descriptors[0][0].packet_size.get().byte_count())
                            });
                            match result {
                                Some(CtrlOutResult::Ok) => {
                                    // Acknowledge
                                    UESTAnCLR[endpoint].write(RXOUT);
                                }
                                Some(CtrlOutResult::Delay) => {
                                    // Don't acknowledge; hardware will have to send NAK

                                    // Unsubscribe from RXOUT until client says it is ready
                                    // (But there is not yet any interface for that)
                                    endpoint_disable_interrupts(endpoint, RXOUT);
                                }
                                _ => {
                                    // Respond with STALL to any following transactions
                                    // in this request
                                    UECONnSET[endpoint].write(STALLRQ);

                                    debug!("D({}) Client OUT err => STALL", endpoint);

                                    *dstate = DeviceState::Init;

                                    // Wait for next SETUP
                                    endpoint_enable_interrupts(endpoint, RXSTP);
                                }
                            }

                            // Continue awaiting RXOUT and NAKIN
                        }
                        if status & NAKIN != 0 {
                            // The host has completed the Data stage by sending an IN token
                            // debug!("D({}) NAKIN: Control Write -> Status stage", endpoint);

                            endpoint_disable_interrupts(endpoint, RXOUT | NAKIN);

                            *dstate = DeviceState::CtrlWriteStatus;

                            // Wait for bank to be free so we can write ZLP to acknowledge transfer
                            endpoint_enable_interrupts(endpoint, TXIN);

                            // Acknowledge
                            UESTAnCLR[endpoint].write(NAKIN);

                            // Can probably send the ZLP immediately
                            // again = true;
                        }
                    }
                    DeviceState::CtrlWriteStatus => {
                        if status & TXIN != 0 {
                            // debug!("D({}) TXIN for Control Write Status (will send ZLP)",
                            //        endpoint);

                            self.client.map(|c| c.ctrl_status());

                            // Send zero-length packet to acknowledge transaction
                            self.descriptors[0][0].packet_size.set(PacketSize::single(0));

                            *dstate = DeviceState::CtrlWriteStatusWait;

                            // Signal to the controller that the IN payload is ready to send
                            UESTAnCLR[endpoint].write(TXIN);

                            // Wait for TXIN again to confirm that IN payload has been sent
                        }
                    }
                    DeviceState::CtrlWriteStatusWait => {
                        if status & TXIN != 0 {
                            // debug!("D({}) TXIN: Control Write Status Complete", endpoint);

                            endpoint_disable_interrupts(endpoint, TXIN);

                            *dstate = DeviceState::Init;

                            // Wait for next SETUP
                            endpoint_enable_interrupts(endpoint, RXSTP);

                            // for SetAddress, client must enable address after STATUS stage
                            self.client.map(|c| c.ctrl_status_complete());
                        }
                    }
                    DeviceState::CtrlInDelay => { /* XX: Spin fruitlessly */ }
                } // match dstate

                // again = false; // XX
            } // while again
        } // for endpoint
    } // handle_device_interrupt

    #[allow(dead_code)]
    fn debug_show_d0(&self) {
        for bi in 0..1 {
            let b = &self.descriptors[0][bi];
            let addr = b.addr.get();
            let buf = if addr.is_null() {
                None
            } else {
                unsafe {
                    Some(slice::from_raw_parts(addr, b.packet_size.get().byte_count() as usize))
                }
            };

            debug!("B_0_{} \
                   \n     {:?}\
                   \n     {:?}\
                   \n     {:?}",
                   bi, // (&b.addr as *const _), b.addr.get(),
                   b.packet_size.get(),
                   b.ctrl_status.get(),
                   buf.map(HexBuf));
        }
    }

    pub fn mode(&self) -> Option<Mode> {
        self.state.map_or(None, |state| match *state {
            State::Idle(mode) => Some(mode),
            State::Active(mode) => Some(mode),
            _ => None,
        })
    }

    pub fn speed(&self) -> Option<Speed> {
        match self.mode() {
            Some(mode) => {
                match mode {
                    Mode::Device { speed, .. } => Some(speed),
                    Mode::Host => {
                        None // XX USBSTA.SPEED
                    }
                }
            }
            _ => None,
        }
    }

    // Remote wakeup (Device -> Host, after receiving DEVICE_REMOTE_WAKEUP)
}

#[inline]
fn endpoint_disable_interrupts(endpoint: usize, mask: u32) {
    UECONnCLR[endpoint].write(mask);
}

#[inline]
fn endpoint_enable_interrupts(endpoint: usize, mask: u32) {
    UECONnSET[endpoint].write(mask);
}

#[inline]
fn endpoint_enable_only_interrupts(endpoint: usize, mask: u32) {
    endpoint_disable_interrupts(endpoint, !0);
    endpoint_enable_interrupts(endpoint, mask);
}

#[allow(dead_code)]
fn debug_regs() {
    debug!("    registers:\
            \n    USBFSM={:08x}\
            \n    USBCON={:08x}\
            \n    USBSTA={:08x}\
            \n     UDESC={:08x}\
            \n     UDCON={:08x}\
            \n    UDINTE={:08x}\
            \n     UDINT={:08x}\
            \n     UERST={:08x}\
            \n    UECFG0={:08x}\
            \n    UECON0={:08x}",

           USBFSM.read(),
           USBCON.read(),
           USBSTA.read(),
           UDESC.read(),
           UDCON.read(),
           UDINTE.read(),
           UDINT.read(),
           UERST.read(),
           UECFG0.read(),
           UECON0.read());
}

#[allow(dead_code)]
struct UdintFlags(u32);

impl fmt::Debug for UdintFlags {
    #[allow(unused_must_use)]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let w: u32 = self.0;

        write!(f, "{{");
        if w & UDINT_WAKEUP != 0 {
            write!(f, "w");
        }
        if w & UDINT_SOF != 0 {
            write!(f, "s");
        }

        if w & UDINT_SUSP != 0 {
            write!(f, " SUSP");
        }
        if w & UDINT_EORST != 0 {
            write!(f, " EORST");
        }
        if w & UDINT_EORSM != 0 {
            write!(f, " EORSM");
        }
        if w & UDINT_UPRSM != 0 {
            write!(f, " UPRSM");
        }

        for i in 0..9 {
            if w & (1 << (12 + i)) != 0 {
                write!(f, " EP{}", i);
            }
        }
        write!(f, "}}")
    }
}

#[allow(dead_code)]
struct UestaFlags(u32);

impl fmt::Debug for UestaFlags {
    #[allow(unused_must_use)]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let w: u32 = self.0;

        write!(f, "{{");
        if w & TXIN != 0 {
            write!(f, "TXIN ");
        }
        if w & RXOUT != 0 {
            write!(f, "RXOUT ");
        }
        if w & RXSTP != 0 {
            write!(f, "RXSTP");
        }
        if w & ERRORF != 0 {
            write!(f, "/ERRORF ");
        }
        if w & NAKOUT != 0 {
            write!(f, "NAKOUT ");
        }
        if w & NAKIN != 0 {
            write!(f, "NAKIN ");
        }
        if w & STALLED != 0 {
            write!(f, "STALLED");
        }
        if w & CRCERR != 0 {
            write!(f, "/CRCERR ");
        }
        if w & RAMACERR != 0 {
            write!(f, "RAMACERR ");
        }
        write!(f, "NBUSYBK={} ", (w >> 12) & 0x3);
        write!(f, "CURBK={} ", (w >> 14) & 0x3);
        write!(f, "CTRLDIR={}", if w & CTRLDIR != 0 { "IN" } else { "OUT" });
        write!(f, "}}")
    }
}

/// Static state to manage the USBC
pub static mut USBC: Usbc<'static> = Usbc::new();
