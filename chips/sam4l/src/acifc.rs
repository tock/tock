//! Implementation of the SAM4L ACIFC controller.
//!
//! See datasheet section "37. Analog Comparator Interface (ACIFC)".
//!
//! Overview
//! -----
//! The SAM4L Analog Comparator Interface (ACIFC) controls a number of Analog
//! Comparators (ACs) with identical behavior. Each Analog Comparator compares
//! two voltages and gives an output depending on this comparison. A specific AC
//! is referred to as ACx where x is any number from 0 to n and n is the index
//! of last AC module.
//!
//! The number of analog comparators (ACs) available depends on the board pr
//! microcontroller used.  The SAM4Lmcomes in three different versions: a 48-pin, a
//! 64-pin and a 100-pin version.  On the 48-pin version, one AC is available.
//! On the 64-pin version, two ACs are available.  On the 100-pin version, four
//! ACs are available.
//! The Hail is an example of a board with the 64-pin version of the SAM4L,
//! and therefore supports two ACs.
//! The Imix is an example of a board with the 100-pin version of the SAM4L,
//! and therefore supports four ACs.
//! Currently, no version of the SAM4L exists with all the 8 ACs
//! implemented. Therefore a lot of the defined bitfields remain unused, but
//! are initialized for a possible future scenario.

// Author: Danilo Verhaert <verhaert@cs.stanford.edu>
// Last modified August 8th, 2018

use crate::pm;
use core::cell::Cell;
use kernel::common::registers::{register_bitfields, ReadOnly, ReadWrite, WriteOnly};
use kernel::common::StaticRef;
use kernel::debug;
use kernel::hil::analog_comparator;
use kernel::ReturnCode;

/// Representation of an AC channel on the SAM4L.
pub struct AcChannel {
    chan_num: u32,
}

#[derive(Copy, Clone, Debug)]
#[repr(u8)]
enum Channel {
    AC0 = 0x00,
    AC1 = 0x01,
    AC2 = 0x02,
    AC3 = 0x03,
}

/// Initialization of an AC channel.
impl AcChannel {
    /// Create a new AC channel.
    ///
    /// - `channel`: Channel enum representing the channel number
    const fn new(channel: Channel) -> AcChannel {
        AcChannel {
            chan_num: ((channel as u8) & 0x0F) as u32,
        }
    }
}

// SAM4L has 2 or 4 possible channels. Hail has 2, Imix has 4.
pub static mut CHANNEL_AC0: AcChannel = AcChannel::new(Channel::AC0);
pub static mut CHANNEL_AC1: AcChannel = AcChannel::new(Channel::AC1);
pub static mut CHANNEL_AC2: AcChannel = AcChannel::new(Channel::AC2);
pub static mut CHANNEL_AC3: AcChannel = AcChannel::new(Channel::AC3);

#[repr(C)]
struct AcifcRegisters {
    ctrl: ReadWrite<u32, Control::Register>,
    sr: ReadOnly<u32, Status::Register>,
    _reserved0: [ReadOnly<u32>; 2],
    ier: WriteOnly<u32, Interrupt::Register>,
    idr: WriteOnly<u32, Interrupt::Register>,
    imr: ReadOnly<u32, Interrupt::Register>,
    isr: ReadOnly<u32, Interrupt::Register>,
    icr: WriteOnly<u32, Interrupt::Register>,
    tr: ReadWrite<u32, Test::Register>,
    _reserved1: [ReadOnly<u32>; 2],
    parameter: ReadOnly<u32, Parameter::Register>,
    version: ReadOnly<u32>,
    _reserved2: [ReadOnly<u32>; 18],
    confw: [ReadWrite<u32, WindowConfiguration::Register>; 4],
    _reserved3: [ReadOnly<u32>; 16],
    conf: [ReadWrite<u32, ACConfiguration::Register>; 8],
}

register_bitfields![u32,
    Control [
        /// Analog comparator test mode. Equal to 1 means AC outputs will be
        /// bypassed with values in AC test register.
        ACTEST 7,
        /// This bit is set when an enabled peripheral event is received (called
        /// by EVENTEN), and starts a single comparison.
        ESTART 5,
        /// This bit can be set by the user and starts a single comparison.
        USTART 4,
        /// This bit sets ESTART to 1 on receiving a peripheral event from
        /// another hardware module.
        EVENTEN 1,
        /// Enables or disables the ACIFC.
        EN 0
    ],

    Status [
        /// This bit represents an output for the window mode, and reads one
        /// when the common input voltage is inside the window of the two
        /// non-common inputs.
        WFCS3 27,
        WFCS2 26,
        WFCS1 25,
        WFCS0 24,
        /// ACRDY is set when the AC output is ready. ACCS is set when the
        /// positive input voltage V_{INP} is greater than the negative input
        /// voltage V_{INN}.
        ACRDY7 15,
        ACCS7 14,
        ACRDY6 13,
        ACCS6 12,
        ACRDY5 11,
        ACCS5 10,
        ACRDY4 9,
        ACCS4 8,
        ACRDY3 7,
        ACCS3 6,
        ACRDY2 5,
        ACCS2 4,
        ACRDY1 3,
        ACCS1 2,
        ACRDY0 1,
        ACCS0 0
    ],

    /// - IER: Writing a one to a bit in this register will set the
    ///   corresponding bit in IMR.
    /// - IDR: Writing a one to a bit in this register will clear the
    ///   corresponding bit in IMR.
    /// - IMR: Writing a one in any of these bits will enable the corresponding
    ///   interrupt.
    /// - ISR: WFINTx shows if a window mode interrupt is pending. SUTINTx shows
    ///   if a startup time interrupt is pending. ACINTx shows if a normal mode
    ///   interrupt is pending.
    /// - ICR: Writing a one to a bit in this register will clear the
    ///   corresponding bit in ISR and the corresponding interrupt request.
    Interrupt [
        WFINT3 27,
        WFINT2 26,
        WFINT1 25,
        WFINT0 24,
        SUTINT7 15,
        ACINT7 14,
        SUTINT6 13,
        ACINT6 12,
        SUTINT5 11,
        ACINT5 10,
        SUTINT4 9,
        ACINT4 8,
        SUTINT3 7,
        ACINT3 6,
        SUTINT2 5,
        ACINT2 4,
        SUTINT1 3,
        ACINT1 2,
        SUTINT0 1,
        ACINT0 0
    ],

    Test [
        /// If equal to one, overrides ACx output with the value of ACTESTx.
        ACTEST7 7,
        ACTEST6 6,
        ACTEST5 5,
        ACTEST4 4,
        ACTEST3 3,
        ACTEST2 2,
        ACTEST1 1,
        ACTEST0 0
    ],

    Parameter [
        /// If equal to one, window mode x is implemented.
        WIMPL3 19,
        WIMPL2 18,
        WIMPL1 17,
        WIMPL0 16,
        /// If equal to one, analog comparator x is implemented.
        ACIMPL7 7,
        ACIMPL6 6,
        ACIMPL5 5,
        ACIMPL4 4,
        ACIMPL3 3,
        ACIMPL2 2,
        ACIMPL1 1,
        ACIMPL0 0
        ],

    WindowConfiguration [
        /// If equal to one, window mode is enabled.
        WFEN OFFSET(16) NUMBITS(1) [],
        /// If equal to one, peripheral event from ACWOUT is enabled.
        WEVEN OFFSET(11) NUMBITS(1) [],
        /// Peripheral Event Source Selection for Window Mode
        WEVSRC OFFSET (8) NUMBITS(3) [
            AcwoutRisingEdge = 0,
            AcwoutFallingEdge = 1,
            AcwoutRisingOrFallingEdge = 2,
            InsideWindow = 3,
            OutsideWindow = 4,
            MeasureDone = 5
        ],
            /// Window Mode Interrupt Settings
        WIS OFFSET(0) NUMBITS (3)[
            /// Window interrupt as soon as the common input voltage is inside
            /// the window
            InterruptInsideWindow = 0,
            /// Window interrupt as soon as the common input voltage is outside
            /// the window
            InterruptOutsideWindow = 1,
            /// Window interrupt on toggle of ACWOUT
            InterruptToggleAcwout = 2,
            /// Window interrupt when evaluation of common input voltage is done
            InterruptAfterEvaluation = 3,
            /// Window interrupt when the common input voltage enters the window
            /// (i.e., rising-edge of ACWOUT)
            InterruptEnterWindow = 4,
            /// Window interrupt when the common input voltage leaves the window
            /// (i.e., falling-edge of ACWOUT)
            InterruptLeaveWindow = 5
    ]
    ],

    ACConfiguration [
        /// If equal to one, AC is always enabled.
        ALWAYSON OFFSET(27) NUMBITS(1) [],
        /// 0: Low-power mode. 1: Fastm ode.
        FAST OFFSET(26) NUMBITS(1) [],
        /// Hysteresis voltage value: 0/25/50/75 mV
        HYS OFFSET(24) NUMBITS(2) [
            HysteresisVoltage0mV = 0,
            HysteresisVoltage25mV = 1,
            HysteresisVoltage50mV = 2,
            HysteresisVoltage75mV = 3
        ],
        /// Setting this to one will output peripheral event when ACOUT is zero.
        EVENP OFFSET(17) NUMBITS(1) [],
        /// Setting this to one will output peripheral event when ACOUT is one.
        EVENN OFFSET(16) NUMBITS(1) [],
        /// Negative input select. 00: ACANx pint selected, others reserved.
        INSELN OFFSET(8) NUMBITS(2) [],
        /// Choose between analog comparator mode.
        MODE OFFSET(4) NUMBITS(2) [
            Off = 0,
            ContinuousMeasurementMode = 1,
            /// User Triggered Single Measurement Mode
            UserMode = 2,
            /// Peripheral Event Single Measurement Mode
            PeripheralMode = 3
        ],
        /// Interrupt settings
        IS OFFSET(0) NUMBITS(2) [
            /// When Vinp > Vinn
            WhenVinpGtVinn = 0,
            /// When Vinp < Vinn
            WhenVinpLtVinn = 1,
            /// On toggle of ACOUT
            OnToggleOfACOUT = 2,
            /// When comparison of Vinp and Vinn is done
            WhenComparisonDone = 3
        ]
    ]
];

const ACIFC_BASE: StaticRef<AcifcRegisters> =
    unsafe { StaticRef::new(0x40040000 as *const AcifcRegisters) };

pub struct Acifc<'a> {
    client: Cell<Option<&'a dyn analog_comparator::Client>>,
}

/// Implement constructor for struct Acifc
impl<'a> Acifc<'a> {
    const fn new() -> Acifc<'a> {
        Acifc {
            client: Cell::new(None),
        }
    }

    fn enable_clock(&self) {
        pm::enable_clock(pm::Clock::PBA(pm::PBAClock::ACIFC));
    }

    fn disable_clock(&self) {
        pm::disable_clock(pm::Clock::PBA(pm::PBAClock::ACIFC));
    }

    pub fn set_client(&self, client: &'a dyn analog_comparator::Client) {
        self.client.set(Some(client));
    }

    /// Enabling the ACIFC by activating the clock and the ACs (Analog
    /// Comparators). Currently always-on mode is enabled, allowing a
    /// measurement on an AC to be made quickly after a measurement is
    /// triggered, without waiting for the AC startup time. The drawback is
    /// that the AC is always on, leading to a higher power dissipation.
    fn enable(&self) {
        let regs = ACIFC_BASE;
        self.enable_clock();
        regs.ctrl.write(Control::EN::SET);

        // Enable continuous measurement mode and always-on mode for all the analog comparators
        regs.conf[0].write(
            ACConfiguration::MODE::ContinuousMeasurementMode + ACConfiguration::ALWAYSON::SET,
        );
        regs.conf[1].write(
            ACConfiguration::MODE::ContinuousMeasurementMode + ACConfiguration::ALWAYSON::SET,
        );
        regs.conf[2].write(
            ACConfiguration::MODE::ContinuousMeasurementMode + ACConfiguration::ALWAYSON::SET,
        );
        regs.conf[3].write(
            ACConfiguration::MODE::ContinuousMeasurementMode + ACConfiguration::ALWAYSON::SET,
        );

        // Make sure enabling was succesful
        let result = regs.ctrl.is_set(Control::EN);
        if result == false {
            debug!("Failed enabling analog comparator, are you sure the clock is enabled?");
        }
    }

    /// Disable the entire ACIFC
    fn disable(&self) {
        let regs = ACIFC_BASE;
        self.disable_clock();
        regs.ctrl.write(Control::EN::CLEAR);
    }

    /// Handling of interrupts. Currently set up so that an interrupt fires
    /// only once when the condition is true (e.g. Vinp > Vinn), and then
    /// doesn't fire anymore until the condition is false (e.g. Vinp < Vinn).
    /// This way we won't get a barrage of interrupts as soon as Vinp > Vinn:
    /// we'll get just one.
    pub fn handle_interrupt(&mut self) {
        let regs = ACIFC_BASE;

        // We check which AC generated the interrupt, and callback to the client accordingly
        if regs.isr.is_set(Interrupt::ACINT0) {
            // Return if we had a pending interrupt while we already set IMR to 0 (edge case)
            if !regs.imr.is_set(Interrupt::ACINT0) {
                return;
            }

            // Disable IMR, making sure no more interrupts can occur until we write
            // to IER
            regs.idr.write(Interrupt::ACINT0::SET);

            // If Vinp > Vinn, throw an interrupt to the client and set the AC so
            // that it will throw an interrupt when Vinn < Vinp instead.
            if !regs.conf[0].is_set(ACConfiguration::IS) {
                self.client.get().map(|client| {
                    client.fired(0);
                });
                regs.conf[0].modify(ACConfiguration::IS::WhenVinpLtVinn);
            }
            // If Vinp < Vinn, set the AC so that it will throw an interrupt when
            // Vinp > Vinn instead.
            else {
                regs.conf[0].modify(ACConfiguration::IS::WhenVinpGtVinn);
            }

            // Clear the interrupt request
            regs.icr.write(Interrupt::ACINT0::SET);
            regs.ier.write(Interrupt::ACINT0::SET);
        } else if regs.isr.is_set(Interrupt::ACINT1) {
            // Return if we had a pending interrupt while we already set IMR to 0 (edge case)
            if !regs.imr.is_set(Interrupt::ACINT1) {
                return;
            }

            // Disable IMR, making sure no more interrupts can occur until we write
            // to IER
            regs.idr.write(Interrupt::ACINT1::SET);

            // If Vinp > Vinn, throw an interrupt to the client and set the AC so
            // that it will throw an interrupt when Vinn < Vinp instead.
            if !regs.conf[1].is_set(ACConfiguration::IS) {
                self.client.get().map(|client| {
                    client.fired(1);
                });
                regs.conf[1].modify(ACConfiguration::IS::WhenVinpLtVinn);
            }
            // If Vinp < Vinn, set the AC so that it will throw an interrupt when
            // Vinp > Vinn instead.
            else {
                regs.conf[1].modify(ACConfiguration::IS::WhenVinpGtVinn);
            }

            // Clear the interrupt request
            regs.icr.write(Interrupt::ACINT1::SET);
            regs.ier.write(Interrupt::ACINT1::SET);
        } else if regs.isr.is_set(Interrupt::ACINT2) {
            // Return if we had a pending interrupt while we already set IMR to 0 (edge case)
            if !regs.imr.is_set(Interrupt::ACINT2) {
                return;
            }

            // Disable IMR, making sure no more interrupts can occur until we write
            // to IER
            regs.idr.write(Interrupt::ACINT2::SET);

            // If Vinp > Vinn, throw an interrupt to the client and set the AC so
            // that it will throw an interrupt when Vinn < Vinp instead.
            if !regs.conf[2].is_set(ACConfiguration::IS) {
                self.client.get().map(|client| {
                    client.fired(2);
                });
                regs.conf[2].modify(ACConfiguration::IS::WhenVinpLtVinn);
            }
            // If Vinp < Vinn, set the AC so that it will throw an interrupt when
            // Vinp > Vinn instead.
            else {
                regs.conf[2].modify(ACConfiguration::IS::WhenVinpGtVinn);
            }

            // Clear the interrupt request
            regs.icr.write(Interrupt::ACINT2::SET);
            regs.ier.write(Interrupt::ACINT2::SET);
        } else if regs.isr.is_set(Interrupt::ACINT3) {
            // Return if we had a pending interrupt while we already set IMR to 0 (edge case)
            if !regs.imr.is_set(Interrupt::ACINT3) {
                return;
            }

            // Disable IMR, making sure no more interrupts can occur until we write
            // to IER
            regs.idr.write(Interrupt::ACINT3::SET);

            // If Vinp > Vinn, throw an interrupt to the client and set the AC so
            // that it will throw an interrupt when Vinn < Vinp instead.
            if !regs.conf[3].is_set(ACConfiguration::IS) {
                self.client.get().map(|client| {
                    client.fired(3);
                });
                regs.conf[3].modify(ACConfiguration::IS::WhenVinpLtVinn);
            }
            // If Vinp < Vinn, set the AC so that it will throw an interrupt when
            // Vinp > Vinn instead.
            else {
                regs.conf[3].modify(ACConfiguration::IS::WhenVinpGtVinn);
            }

            // Clear the interrupt request
            regs.icr.write(Interrupt::ACINT3::SET);
            regs.ier.write(Interrupt::ACINT3::SET);
        }
    }
}

impl<'a> analog_comparator::AnalogComparator for Acifc<'a> {
    type Channel = AcChannel;

    /// Do a single comparison
    fn comparison(&self, channel: &Self::Channel) -> bool {
        self.enable();
        let regs = ACIFC_BASE;
        let result;
        if channel.chan_num == 0 {
            result = regs.sr.is_set(Status::ACCS0);
        } else if channel.chan_num == 1 {
            result = regs.sr.is_set(Status::ACCS1);
        } else if channel.chan_num == 2 {
            result = regs.sr.is_set(Status::ACCS2);
        } else if channel.chan_num == 3 {
            result = regs.sr.is_set(Status::ACCS3);
        } else {
            // Should never get here, just making sure
            self.disable();
            panic!("PANIC! Please choose a comparator (value of ac) that this chip supports");
        }
        result
    }

    /// Start interrupt-based comparisons
    fn start_comparing(&self, channel: &Self::Channel) -> ReturnCode {
        self.enable();
        let regs = ACIFC_BASE;

        if channel.chan_num == 0 {
            // Enable interrupts.
            regs.ier.write(Interrupt::ACINT0::SET);
            ReturnCode::SUCCESS
        } else if channel.chan_num == 1 {
            // Repeat the same for ac == 1
            regs.ier.write(Interrupt::ACINT1::SET);
            ReturnCode::SUCCESS
        } else if channel.chan_num == 2 {
            // Repeat the same for ac == 2
            regs.ier.write(Interrupt::ACINT2::SET);
            ReturnCode::SUCCESS
        } else if channel.chan_num == 3 {
            // Repeat the same for ac == 3
            regs.ier.write(Interrupt::ACINT3::SET);
            ReturnCode::SUCCESS
        } else {
            // Should never get here, just making sure
            self.disable();
            debug!("Please choose a comparator (value of ac) that this chip supports");
            ReturnCode::EINVAL
        }
    }

    /// Stop interrupt-based comparisons
    fn stop_comparing(&self, channel: &Self::Channel) -> ReturnCode {
        let regs = ACIFC_BASE;

        if channel.chan_num == 0 {
            // Disable interrupts.
            regs.ier.write(Interrupt::ACINT0::CLEAR);
            ReturnCode::SUCCESS
        } else if channel.chan_num == 1 {
            // Repeat the same for ac == 1
            regs.ier.write(Interrupt::ACINT1::CLEAR);
            ReturnCode::SUCCESS
        } else if channel.chan_num == 2 {
            // Repeat the same for ac == 2
            regs.ier.write(Interrupt::ACINT2::CLEAR);
            ReturnCode::SUCCESS
        } else if channel.chan_num == 3 {
            // Repeat the same for ac == 3
            regs.ier.write(Interrupt::ACINT3::CLEAR);
            ReturnCode::SUCCESS
        } else {
            // Should never get here, just making sure
            self.disable();
            debug!("Please choose a comparator (value of ac) that this chip supports");
            ReturnCode::EINVAL
        }
    }
}

/// Static state to manage the ACIFC
pub static mut ACIFC: Acifc = Acifc::new();
