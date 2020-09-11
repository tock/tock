//! Reference Module (REF)

use core::cell::Cell;
use kernel::common::registers::{register_bitfields, register_structs, ReadWrite};
use kernel::common::StaticRef;

pub static mut REF: Ref = Ref::new();

register_structs! {
    /// REF
    RefRegisters {
        /// REF Control Register 0
        (0x000 => ctl0: ReadWrite<u16, CTL0::Register>),
        (0x002 => @END),
    }
}
register_bitfields![u16,
    CTL0 [
        /// Reference enable
        REFON OFFSET(0) NUMBITS(1) [
            /// Disables reference if no other reference requests are pending
            DisablesReferenceIfNoOtherReferenceRequestsArePending = 0,
            /// Enables reference in static mode
            EnablesReferenceInStaticMode = 1
        ],
        /// Reference output buffer
        REFOUT OFFSET(1) NUMBITS(1) [
            /// Reference output not available externally
            ReferenceOutputNotAvailableExternally = 0,
            /// Reference output available externally. If ADC14REFBURST = 0, output is available
            REFOUT_1 = 1
        ],
        /// Temperature sensor disabled
        REFTCOFF OFFSET(3) NUMBITS(1) [
            /// Temperature sensor enabled
            TemperatureSensorEnabled = 0,
            /// Temperature sensor disabled to save power
            TemperatureSensorDisabledToSavePower = 1
        ],
        /// Reference voltage level select
        REFVSEL OFFSET(4) NUMBITS(2) [
            /// 1.2 V available when reference requested or REFON = 1
            _12VAvailableWhenReferenceRequestedOrREFON1 = 0,
            /// 1.45 V available when reference requested or REFON = 1
            _145VAvailableWhenReferenceRequestedOrREFON1 = 1,
            /// 2.5 V available when reference requested or REFON = 1
            _25VAvailableWhenReferenceRequestedOrREFON1 = 3
        ],
        /// Reference generator one-time trigger
        REFGENOT OFFSET(6) NUMBITS(1) [
            /// No trigger
            NoTrigger = 0,
            /// Generation of the reference voltage is started by writing 1 or by a hardware tri
            GenerationOfTheReferenceVoltageIsStartedByWriting1OrByAHardwareTrigger = 1
        ],
        /// Bandgap and bandgap buffer one-time trigger
        REFBGOT OFFSET(7) NUMBITS(1) [
            /// No trigger
            NoTrigger = 0,
            /// Generation of the bandgap voltage is started by writing 1 or by a hardware trigg
            GenerationOfTheBandgapVoltageIsStartedByWriting1OrByAHardwareTrigger = 1
        ],
        /// Reference generator active
        REFGENACT OFFSET(8) NUMBITS(1) [
            /// Reference generator not active
            ReferenceGeneratorNotActive = 0,
            /// Reference generator active
            ReferenceGeneratorActive = 1
        ],
        /// Reference bandgap active
        REFBGACT OFFSET(9) NUMBITS(1) [
            /// Reference bandgap buffer not active
            ReferenceBandgapBufferNotActive = 0,
            /// Reference bandgap buffer active
            ReferenceBandgapBufferActive = 1
        ],
        /// Reference generator busy
        REFGENBUSY OFFSET(10) NUMBITS(1) [
            /// Reference generator not busy
            ReferenceGeneratorNotBusy = 0,
            /// Reference generator busy
            ReferenceGeneratorBusy = 1
        ],
        /// Bandgap mode
        BGMODE OFFSET(11) NUMBITS(1) [
            /// Static mode
            StaticMode = 0,
            /// Sampled mode
            SampledMode = 1
        ],
        /// Variable reference voltage ready status
        REFGENRDY OFFSET(12) NUMBITS(1) [
            /// Reference voltage output is not ready to be used
            ReferenceVoltageOutputIsNotReadyToBeUsed = 0,
            /// Reference voltage output is ready to be used
            ReferenceVoltageOutputIsReadyToBeUsed = 1
        ],
        /// Buffered bandgap voltage ready status
        REFBGRDY OFFSET(13) NUMBITS(1) [
            /// Buffered bandgap voltage is not ready to be used
            BufferedBandgapVoltageIsNotReadyToBeUsed = 0,
            /// Buffered bandgap voltage is ready to be used
            BufferedBandgapVoltageIsReadyToBeUsed = 1
        ]
    ]
];

const REF_BASE: StaticRef<RefRegisters> =
    unsafe { StaticRef::new(0x4000_3000 as *const RefRegisters) };

pub struct Ref {
    registers: StaticRef<RefRegisters>,
    ref_voltage: Cell<ReferenceVoltage>,
}

#[repr(u16)]
#[derive(Copy, Clone, PartialEq)]
pub enum ReferenceVoltage {
    Volt1_2 = 0,
    Volt1_45 = 1,
    Volt2_5 = 3,
}

pub trait AnalogReference {
    /// Return the configured reference voltage in mV
    fn ref_voltage_mv(&self) -> usize;
}

impl Ref {
    const fn new() -> Ref {
        Ref {
            registers: REF_BASE,
            ref_voltage: Cell::new(ReferenceVoltage::Volt1_2),
        }
    }

    /// Set the reference voltage of this module which will be used in the ADC and DAC modules.
    /// The default voltage is 1.2V.
    pub fn select_ref_voltage(&self, ref_voltage: ReferenceVoltage) {
        self.ref_voltage.set(ref_voltage);
        while self.registers.ctl0.is_set(CTL0::REFGENBUSY) {}
        self.registers
            .ctl0
            .modify(CTL0::REFVSEL.val(ref_voltage as u16));
    }

    /// Enable or disable the internal temperature sensor.
    /// The default-setting is enabled.
    pub fn enable_temp_sensor(&self, enable: bool) {
        while self.registers.ctl0.is_set(CTL0::REFGENBUSY) {}
        self.registers.ctl0.modify(
            // Enable the temperature sensor
            CTL0::REFTCOFF.val((!enable) as u16)
            // Enable the reference module, otherwise the temperature sensor doesn't work
            + CTL0::REFON::SET,
        );
    }
}

impl AnalogReference for Ref {
    fn ref_voltage_mv(&self) -> usize {
        match self.ref_voltage.get() {
            ReferenceVoltage::Volt1_2 => 1200,
            ReferenceVoltage::Volt1_45 => 1450,
            ReferenceVoltage::Volt2_5 => 2500,
        }
    }
}
