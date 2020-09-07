//! Analog-Digital Converter (ADC)

use crate::ref_module;
use kernel::common::cells::OptionalCell;
use kernel::common::registers::{
    register_bitfields, register_structs, ReadOnly, ReadWrite, WriteOnly,
};
use kernel::common::StaticRef;
use kernel::hil;
use kernel::ReturnCode;

pub static mut ADC: Adc = Adc {
    registers: ADC_BASE,
    resolution: AdcResolution::Bits14,
    ref_module: OptionalCell::empty(),
    client: OptionalCell::empty(),
    adc_channels: [
        AdcChannel::new(18, ChannelSource::External),
        AdcChannel::new(19, ChannelSource::External),
        AdcChannel::new(20, ChannelSource::External),
        AdcChannel::new(21, ChannelSource::External),
        AdcChannel::new(22, ChannelSource::Internal),
        AdcChannel::new(23, ChannelSource::Internal),
    ],
};

const ADC_BASE: StaticRef<AdcRegisters> =
    unsafe { StaticRef::new(0x4001_2000 as *const AdcRegisters) };

const AVAILABLE_ADC_CHANNELS: usize = 6;

const DEFAULT_ADC_RESOLUTION: AdcResolution = AdcResolution::Bits14;

register_structs! {
    /// ADC14
    AdcRegisters {
        /// ADC control 0 register
        (0x000 => ctl0: ReadWrite<u32, CTL0::Register>),
        /// ADC control 1 register
        (0x004 => ctl1: ReadWrite<u32, CTL1::Register>),
        /// Window comparator low threshold 0 register
        (0x008 => lo0: ReadWrite<u32>),
        /// Window comparator high threshold 1
        (0x00C => hi0: ReadWrite<u32>),
        /// Window comparator low threshold 1 register
        (0x010 => lo1: ReadWrite<u32>),
        /// Window comparator high threshold 1 register
        (0x014 => hi1: ReadWrite<u32>),
        /// Memory control register 0-31
        (0x018 => mctl: [ReadWrite<u32, MCTLx::Register>; 32]),
        /// Memory register 0-31
        (0x098 => mem: [ReadWrite<u32>; 32]),
        (0x118 => _reserved),
        /// Interrupt enable 0 register
        (0x13C => ie0: ReadWrite<u32>),
        /// Interrupt enable 1 register
        (0x140 => ie1: ReadWrite<u32, IER1::Register>),
        /// Interrupt flag 0 register
        (0x144 => ifg0: ReadOnly<u32>),
        /// Interrupt flag 1 register
        (0x148 => ifg1: ReadOnly<u32, IFGR1::Register>),
        /// Clear interrupt flag 0 register
        (0x14C => clrifg0: WriteOnly<u32>),
        /// Clear interrupt flag 1 register
        (0x150 => clrifg1: WriteOnly<u32, CLRIFGR1::Register>),
        /// Interrupt vector register
        (0x154 => iv: ReadOnly<u32, IV::Register>),
        (0x158 => @END),
    }
}

register_bitfields![u32,
    /// ADC Control 0 register
    CTL0 [
        /// ADC Start conversion
        SC OFFSET(0) NUMBITS(1) [],
        /// ADC Enable conversion
        ENC OFFSET(1) NUMBITS(1) [],
        // ADC on
        ON OFFSET(4) NUMBITS(1) [],
        /// ADC multiple sample an conversion
        MSC OFFSET(7) NUMBITS(1) [],
        /// ADC sample-and-hold time for pulse sample mode.
        /// Valid for ADCMEM0 to ADCMEM7 and ADCMEM24 to ADCMEM31.
        SHTOx OFFSET(8) NUMBITS(4) [
            /// 4 clock cycles sample-and-hold time
            Cycles4 = 0,
            /// 8 clock cycles sample-and-hold time
            Cycles8 = 1,
            /// 16 clock cycles sample-and-hold time
            Cycles16 = 2,
            /// 32 clock cycles sample-and-hold time
            Cycles32 = 3,
            /// 64 clock cycles sample-and-hold time
            Cycles64 = 4,
            /// 96 clock cycles sample-and-hold time
            Cycles96 = 5,
            /// 128 clock cycles sample-and-hold time
            Cycles128 = 6,
            /// 192 clock cycles sample-and-hold time
            Cycles192 = 7
        ],
        /// ADC sample-and-hold time for pulse sample mode.
        /// Valid for ADCMEM8 to ADCMEM23.
        SHT1x OFFSET(12) NUMBITS(4) [
            /// 4 clock cycles sample-and-hold time
            Cycles4 = 0,
            /// 8 clock cycles sample-and-hold time
            Cycles8 = 1,
            /// 16 clock cycles sample-and-hold time
            Cycles16 = 2,
            /// 32 clock cycles sample-and-hold time
            Cycles32 = 3,
            /// 64 clock cycles sample-and-hold time
            Cycles64 = 4,
            /// 96 clock cycles sample-and-hold time
            Cycles96 = 5,
            /// 128 clock cycles sample-and-hold time
            Cycles128 = 6,
            /// 192 clock cycles sample-and-hold time
            Cycles192 = 7
        ],
        /// ADC Busy
        BUSY OFFSET(16) NUMBITS(1) [],
        /// ADC conversion sequence mode select
        CONSEQx OFFSET(17) NUMBITS(2) [
            /// Single channel, single conversion
            SingleChannelSingleConversion = 0,
            /// Sequence of channels
            ChannelSequence = 1,
            /// Repeat single channel
            RepeatSingleChannel = 2,
            /// Repeat sequence of channels
            RepeatChannelSequence = 3
        ],
        /// ADC clock source select
        SSELx OFFSET(19) NUMBITS(3) [
            /// MODCLK
            MODCLK = 0,
            /// SYSCLK
            SYSCLK = 1,
            /// ACLK
            ACLK = 2,
            /// MCLK
            MCLK = 3,
            /// SMCLK
            SMCLK = 4,
            /// HSMCLK
            HSMCLK = 5
        ],
        /// ADC clock divider
        DIVx OFFSET(22) NUMBITS(3) [
            /// Divide clock by 1
            DivideBy1 = 0,
            /// Divide clock by 2
            DivideBy2 = 1,
            /// Divide clock by 3
            DivideBy3 = 2,
            /// Divide clock by 4
            DivideBy4 = 3,
            /// Divide clock by 5
            DivideBy5 = 4,
            /// Divide clock by 6
            DivideBy6 = 5,
            /// Divide clock by 7
            DivideBy7 = 6,
            /// Divide clock by 8
            DivideBy8 = 7
        ],
        /// ADC invert signal sample-and-hold
        ISSH OFFSET(25) NUMBITS(1) [],
        /// ADC sample-and-hold pulse-mode select
        SHP OFFSET(26) NUMBITS(1) [],
        /// ADC sample-and-hold souce select
        SHSx OFFSET(27) NUMBITS(3) [
            /// ADC14SC bit
            SCBit = 0,
            /// Source 1, see device-specific datasheet
            Source1 = 1,
            /// Source 2, see device-specific datasheet
            Source2 = 2,
            /// Source 3, see device-specific datasheet
            Source3 = 3,
            /// Source 4, see device-specific datasheet
            Source4 = 4,
            /// Source 5, see device-specific datasheet
            Source5 = 5,
            /// Source 6, see device-specific datasheet
            Source6 = 6,
            /// Source 7, see device-specific datasheet
            Source7 = 7
        ],
        /// ADC pre-divider
        PDIV OFFSET(30) NUMBITS(2) [
            /// Pre-divde by 1
            PreDivideBy1 = 0,
            /// Pre-divde by 4
            PreDivideBy4 = 1,
            /// Pre-divde by 32
            PreDivideBy32 = 2,
            /// Pre-divde by 64
            PreDivideBy64 = 3
        ]
    ],
    /// ADC control 1 register
    CTL1 [
        /// ADC power modes
        PWRMD OFFSET(0) NUMBITS(2) [
            /// Regular power mode with any resolution setting. Sample-rate up to 1Msps.
            Regular = 0,
            /// Low-power mode for 12-, 10-, and 8-bit resolutions. Sample-rate up to 200ksps.
            LowPower = 1
        ],
        /// ADC reference buffer burst
        REFBURST OFFSET(2) NUMBITS(1) [
            /// ADC reference buffer on continuously
            Continuously = 0,
            /// ADC reference buffer on only during sample-and-conversion
            DuringSampleAndConversion = 1
        ],
        /// ADC data read-back format. Data is always stored in the binary unsigned format.
        DF OFFSET(3) NUMBITS(1) [
            /// Binary unsigned, at 14bit: -Vref = 0, +Vref = 0x3FFF
            Unsigned = 0,
            /// Binary signed, at 14bit: -Vref = 0x8000, +Vref = 0x7FFC
            Signed = 1
        ],
        /// ADC resolution
        RES OFFSET(4) NUMBITS(2) [
            /// 8bit (9 clock cycles conversion time)
            Resolution8Bit = 0,
            /// 10bit (11 clock cycles conversion time)
            Resolution11Bit = 1,
            /// 12bit (14 clock cycles conversion time)
            Resolution14Bit = 2,
            /// 14bit (16 clock cycles conversion time)
            Resolution16Bit = 3
        ],
        /// ADC conversion start address, select ADC14MEM0 to ADC14MEM31
        STARTADDx OFFSET(16) NUMBITS(5) [],
        /// Controls 1/2 AVCC ADC input channel selection
        BATMAP OFFSET(22) NUMBITS(1) [
            /// ADC internal 1/2 x AVCC channel is not selected for ADC
            NotSelected = 0,
            /// ADC internal 1/2 x AVCC channel is selected for ADC input channel MAX
            Selected = 1
        ],
        /// Controls temperature sensor ADC input channel selection
        TCMAP OFFSET(23) NUMBITS(1) [],
        /// Controls internal channel 0 selection to ADC input channel MAX - 2
        CH0MAP OFFSET(24) NUMBITS(1) [],
        /// Controls internal channel 1 selection to ADC input channel MAX - 3
        CH1MAP OFFSET(25) NUMBITS(1) [],
        /// Controls internal channel 2 selection to ADC input channel MAX - 4
        CH2MAP OFFSET(26) NUMBITS(1) [],
        /// Controls internal channel 3 selection to ADC input channel MAX - 5
        CH3MAP OFFSET(27) NUMBITS(1) []
    ],
    /// ADC conversion memory control x register
    MCTLx [
        /// Input channel select. If even channels are set as differential then odd channel configuration is ignored.
        INCHx OFFSET(0) NUMBITS(5) [
            ///  If ADC14DIF = 0: A0; If ADC14DIF = 1: Ain+ = A0, Ain- = A1
            A0A1Even = 0,
            /// If ADC14DIF = 0: A1; If ADC14DIF = 1: Ain+ = A0, Ain- = A1
            A0A1Odd = 1,
            /// If ADC14DIF = 0: A2; If ADC14DIF = 1: Ain+ = A2, Ain- = A3
            A2A3Even = 2,
            /// If ADC14DIF = 0: A3; If ADC14DIF = 1: Ain+ = A2, Ain- = A3
            A2A3Odd = 3,
            /// If ADC14DIF = 0: A4; If ADC14DIF = 1: Ain+ = A4, Ain- = A5
            A4A5Even = 4,
            /// If ADC14DIF = 0: A5; If ADC14DIF = 1: Ain+ = A4, Ain- = A5
            A4A5Odd = 5,
            /// If ADC14DIF = 0: A6; If ADC14DIF = 1: Ain+ = A6, Ain- = A7
            A6A7Even = 6,
            /// If ADC14DIF = 0: A7; If ADC14DIF = 1: Ain+ = A6, Ain- = A7
            A6A7Odd = 7,
            /// If ADC14DIF = 0: A8; If ADC14DIF = 1: Ain+ = A8, Ain- = A9
            A8A9Even = 8,
            /// If ADC14DIF = 0: A9; If ADC14DIF = 1: Ain+ = A8, Ain- = A9
            A8A9Odd = 9,
            /// If ADC14DIF = 0: A10; If ADC14DIF = 1: Ain+ = A10, Ain- = A11
            A10A11Even = 10,
            /// If ADC14DIF = 0: A11; If ADC14DIF = 1: Ain+ = A10, Ain- = A11
            A10A11Odd = 11,
            /// If ADC14DIF = 0: A12; If ADC14DIF = 1: Ain+ = A12, Ain- = A13
            A12A13Even = 12,
            /// If ADC14DIF = 0: A13; If ADC14DIF = 1: Ain+ = A12, Ain- = A13
            A12A13Odd = 13,
            /// If ADC14DIF = 0: A14; If ADC14DIF = 1: Ain+ = A14, Ain- = A15
            A14A15Even = 14,
            /// If ADC14DIF = 0: A15; If ADC14DIF = 1: Ain+ = A14, Ain- = A15
            A14A15Odd = 15,
            /// If ADC14DIF = 0: A16; If ADC14DIF = 1: Ain+ = A16, Ain- = A17
            A16A17Even = 16,
            /// If ADC14DIF = 0: A17; If ADC14DIF = 1: Ain+ = A16, Ain- = A17
            A16A17Odd = 17,
            /// If ADC14DIF = 0: A18; If ADC14DIF = 1: Ain+ = A18, Ain- = A19
            A18A19Even = 18,
            /// If ADC14DIF = 0: A19; If ADC14DIF = 1: Ain+ = A18, Ain- = A19
            A18A19Odd = 19,
            /// If ADC14DIF = 0: A20; If ADC14DIF = 1: Ain+ = A20, Ain- = A21
            A20A21Even = 20,
            /// If ADC14DIF = 0: A21; If ADC14DIF = 1: Ain+ = A20, Ain- = A21
            A20A21Odd = 21,
            /// If ADC14DIF = 0: A22; If ADC14DIF = 1: Ain+ = A22, Ain- = A23
            A22A23Even = 22,
            /// If ADC14DIF = 0: A23; If ADC14DIF = 1: Ain+ = A22, Ain- = A23
            A22A23Odd = 23,
            /// If ADC14DIF = 0: A24; If ADC14DIF = 1: Ain+ = A24, Ain- = A25
            A24A25Even = 24,
            /// If ADC14DIF = 0: A25; If ADC14DIF = 1: Ain+ = A24, Ain- = A25
            A24A25Odd = 25,
            /// If ADC14DIF = 0: A26; If ADC14DIF = 1: Ain+ = A26, Ain- = A27
            A26A27Even = 26,
            /// If ADC14DIF = 0: A27; If ADC14DIF = 1: Ain+ = A26, Ain- = A27
            A26A27Odd = 27,
            /// If ADC14DIF = 0: A28; If ADC14DIF = 1: Ain+ = A28, Ain- = A29
            A28A29Even = 28,
            /// If ADC14DIF = 0: A29; If ADC14DIF = 1: Ain+ = A28, Ain- = A29
            A28A29Odd = 29,
            /// If ADC14DIF = 0: A30; If ADC14DIF = 1: Ain+ = A30, Ain- = A31
            A30A31Even = 30,
            /// If ADC14DIF = 0: A31; If ADC14DIF = 1: Ain+ = A30, Ain- = A31
            A30A31Odd = 31
        ],
        /// End of sequence. Indicates the last conversion in a sequence.
        EOS OFFSET(7) NUMBITS(1) [],
        /// Selects combinations of +Vref and -Vref sources as well as the buffer selection and buffer on or off.
        VRSEL OFFSET(8) NUMBITS(4) [
            /// +Vref = AVCC, -Vref = AVSS
            AvccAvss = 0,
            /// +Vref = VREF buffered, -Vref = AVSS
            VRefBufferedAvss = 1,
            /// +Vref = VeREF+, -Vref = VeRE-
            VeRef = 14,
            /// +Vref = VeREF+ buffered, -Vref = VeREF-
            VeRefBuffered = 15
        ],
        /// Differential mode
        DIF OFFSET(13) NUMBITS(1) [
            /// Single-ended mode enabled
            SingleEnded = 0,
            /// Differential mode enabled
            Differential = 1
        ],
        /// Comparator window enable
        WINC OFFSET(14) NUMBITS(1) [],
        /// Window comparator threshold register selection
        WINCTH OFFSET(15) NUMBITS(1) [
            /// Use window comparator thresholds 0, ADC14LO0 and ADC14HI0
            Threshold0 = 0,
            /// Use window comparator thresholds 1, ADC14LO1 and ADC14HI1
            Threshold1 = 1
        ]
    ],
    /// ADC interrupt enable 1 register
    IER1 [
        /// Interrupt enable for the ADC14MEMx result register being greater than the ADC14LO
        /// threshold and below the ADC14HI threshold
        INIE OFFSET(1) NUMBITS(1) [],
        /// Interrupt enable for the falling short of the lower limit interrupt of the window
        /// comparator for the ADC14MEMx result registers.
        LOIE OFFSET(2) NUMBITS(1) [],
        /// Interrupt enable for the exceeding the upper limit interrupt of the window
        /// comparator for ADC14MEMx result register.
        HIIE OFFSET(3) NUMBITS(1) [],
        /// ADC14MEMx overflow interrupt enable
        OVIE OFFSET(4) NUMBITS(1) [],
        /// ADC14 conversion-time-overflow interrupt enable
        TOVIE OFFSET(5) NUMBITS(1) [],
        /// ADC14 local buffered reference ready interrupt enable
        RDYIE OFFSET(6) NUMBITS(1) []
    ],
    /// ADC interrupt flag 1 register
    IFGR1 [
        /// Interrupt flag for the ADC14MEMx result register being greater than the ADC14LO
        /// threshold and below the ADC14HI threshold
        INIFG OFFSET(1) NUMBITS(1) [],
        /// Interrupt flag for the falling short of the lower limit interrupt of the window
        /// comparator for the ADC14MEMx result registers.
        LOIFG OFFSET(2) NUMBITS(1) [],
        /// Interrupt flag for the exceeding the upper limit interrupt of the window
        /// comparator for ADC14MEMx result register.
        HIIFG OFFSET(3) NUMBITS(1) [],
        /// ADC14MEMx overflow interrupt flag
        OVIFG OFFSET(4) NUMBITS(1) [],
        /// ADC14 conversion-time-overflow interrupt flag
        TOVIFG OFFSET(5) NUMBITS(1) [],
        /// ADC14 local buffered reference ready interrupt flag
        RDYIFG OFFSET(6) NUMBITS(1) []
    ],
    /// ADC clear interrupt flag 1 register
    CLRIFGR1 [
        /// Clear INIFG
        CLRINIFG OFFSET(1) NUMBITS(1) [],
        /// Clear LOIFG
        CLRLOIFG OFFSET(2) NUMBITS(1) [],
        /// Clear HIIFG
        CLRHIIFG OFFSET(3) NUMBITS(1) [],
        /// Clear OVIFG
        CLROVIFG OFFSET(4) NUMBITS(1) [],
        /// Clear TOIFG
        CLRTOVIFG OFFSET(5) NUMBITS(1) [],
        /// Clear RDYIFG
        CLRRDYIFG OFFSET(6) NUMBITS(1) []
    ],
    /// ADC interrupt vector register
    IV [
        /// ADC interrupt vector value
        IVx OFFSET(0) NUMBITS(32) [
            /// No interrupt pending
            NoInterrupt = 0x00,
            /// ADC14MEMx overflow, highest priority
            MemOverflow = 0x02,
            /// Conversion time overflow
            ConversionTimeOverflow = 0x04,
            /// ADC window high interrupt flag
            WindowHigh = 0x06,
            /// ADC window low interrupt flag
            WindowLow = 0x08,
            /// ADC in-window interrupt flag
            WindowIn = 0x0A,
            /// MEM0 interrupt flag
            Mem0 = 0x0C,
            /// MEM1 interrupt flag
            Mem1 = 0x0E,
            /// MEM2 interrupt flag
            Mem2 = 0x10,
            /// MEM3 interrupt flag
            Mem3 = 0x12,
            /// MEM4 interrupt flag
            Mem4 = 0x14,
            /// MEM5 interrupt flag
            Mem5 = 0x16,
            /// MEM6 interrupt flag
            Mem6 = 0x18,
            /// MEM7 interrupt flag
            Mem7 = 0x1A,
            /// MEM8 interrupt flag
            Mem8 = 0x1C,
            /// MEM9 interrupt flag
            Mem9 = 0x1E,
            /// MEM10 interrupt flag
            Mem10 = 0x20,
            /// MEM11 interrupt flag
            Mem11 = 0x22,
            /// MEM12 interrupt flag
            Mem12 = 0x24,
            /// MEM13 interrupt flag
            Mem13 = 0x26,
            /// MEM14 interrupt flag
            Mem14 = 0x28,
            /// MEM15 interrupt flag
            Mem15 = 0x2A,
            /// MEM16 interrupt flag
            Mem16 = 0x2C,
            /// MEM17 interrupt flag
            Mem17 = 0x2E,
            /// MEM18 interrupt flag
            Mem18 = 0x30,
            /// MEM19 interrupt flag
            Mem19 = 0x32,
            /// MEM20 interrupt flag
            Mem20 = 0x34,
            /// MEM21 interrupt flag
            Mem21 = 0x36,
            /// MEM22 interrupt flag
            Mem22 = 0x38,
            /// MEM23 interrupt flag
            Mem23 = 0x3A,
            /// MEM24 interrupt flag
            Mem24 = 0x3C,
            /// MEM25 interrupt flag
            Mem25 = 0x3E,
            /// MEM26 interrupt flag
            Mem26 = 0x40,
            /// MEM27 interrupt flag
            Mem27 = 0x42,
            /// MEM28 interrupt flag
            Mem28 = 0x44,
            /// MEM29 interrupt flag
            Mem29 = 0x46,
            /// MEM30 interrupt flag
            Mem30 = 0x48,
            /// MEM31 interrupt flag
            Mem31 = 0x4A,
            /// RDYIFG interrupt flag
            Ready = 0x4C
        ]
    ]
];

pub struct Adc<'a> {
    registers: StaticRef<AdcRegisters>,
    resolution: AdcResolution,
    ref_module: OptionalCell<&'a dyn ref_module::AnalogReference>,
    client: OptionalCell<&'a dyn hil::adc::Client>,
    adc_channels: [AdcChannel; AVAILABLE_ADC_CHANNELS],
}

#[derive(Copy, Clone, PartialEq)]
pub enum Channel {
    Channel18 = 0,
    Channel19 = 1,
    Channel20 = 2,
    Channel21 = 3,
    Channel22 = 4,
    Channel23 = 5,
}

struct AdcChannel {
    registers: StaticRef<AdcRegisters>,
    chan_nr: usize,
    chan_src: ChannelSource,
}

#[repr(u32)]
#[derive(Copy, Clone, PartialEq)]
enum AdcResolution {
    Bits8 = 0,
    Bits10 = 1,
    Bits12 = 2,
    Bits14 = 3,
}

#[repr(u32)]
#[derive(Copy, Clone, PartialEq)]
enum ChannelSource {
    External = 0,
    Internal = 1,
}

impl<'a> Adc<'a> {
    fn is_enabled(&self) -> bool {
        self.registers.ctl0.is_set(CTL0::ON)
    }

    fn setup(&self) {}

    pub fn set_ref_module(&self, ref_module: &'a dyn ref_module::AnalogReference) {
        self.ref_module.set(ref_module);
    }

    pub fn set_client(&self, client: &'a dyn hil::adc::Client) {
        self.client.set(client);
    }
}

impl AdcChannel {
    const fn new(chan_nr: usize, chan_src: ChannelSource) -> AdcChannel {
        AdcChannel {
            registers: ADC_BASE,
            chan_nr: chan_nr,
            chan_src: chan_src,
        }
    }
}

impl<'a> hil::adc::Adc for Adc<'a> {
    type Channel = Channel;

    fn sample(&self, channel: &Self::Channel) -> ReturnCode {
        if !self.is_enabled() {
            self.setup();
        }
        ReturnCode::ENOSUPPORT
    }

    fn sample_continuous(&self, channel: &Self::Channel, frequency: u32) -> ReturnCode {
        ReturnCode::ENOSUPPORT
    }

    fn stop_sampling(&self) -> ReturnCode {
        ReturnCode::ENOSUPPORT
    }

    fn get_resolution_bits(&self) -> usize {
        match self.resolution {
            AdcResolution::Bits8 => 8,
            AdcResolution::Bits10 => 10,
            AdcResolution::Bits12 => 12,
            AdcResolution::Bits14 => 14,
        }
    }

    fn get_voltage_reference_mv(&self) -> Option<usize> {
        self.ref_module.map(|refmod| refmod.ref_voltage_mv())
    }
}
