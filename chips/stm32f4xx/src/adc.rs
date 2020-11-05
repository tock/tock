use crate::rcc;
use core::cell::Cell;
use kernel::common::cells::OptionalCell;
use kernel::common::registers::{register_bitfields, ReadOnly, ReadWrite};
use kernel::common::StaticRef;
use kernel::hil;
use kernel::ClockInterface;
use kernel::ReturnCode;

pub trait EverythingClient: hil::adc::Client + hil::adc::HighSpeedClient {}
impl<C: hil::adc::Client + hil::adc::HighSpeedClient> EverythingClient for C {}

#[repr(C)]
struct AdcRegisters {
    sr: ReadWrite<u32, SR::Register>,
    cr1: ReadWrite<u32, CR1::Register>,
    cr2: ReadWrite<u32, CR2::Register>,
    smpr1: ReadWrite<u32, SMPR1::Register>,
    smpr2: ReadWrite<u32, SMPR2::Register>,
    jofr1: ReadWrite<u32, JOFR::Register>,
    jofr2: ReadWrite<u32, JOFR::Register>,
    jofr3: ReadWrite<u32, JOFR::Register>,
    jofr4: ReadWrite<u32, JOFR::Register>,
    htr: ReadWrite<u32, HTR::Register>,
    ltr: ReadWrite<u32, LTR::Register>,
    sqr1: ReadWrite<u32, SQR1::Register>,
    sqr2: ReadWrite<u32, SQR2::Register>,
    sqr3: ReadWrite<u32, SQR3::Register>,
    jsqr: ReadWrite<u32, JSQR::Register>,
    jdr1: ReadOnly<u32, JDR::Register>,
    jdr2: ReadOnly<u32, JDR::Register>,
    jdr3: ReadOnly<u32, JDR::Register>,
    jdr4: ReadOnly<u32, JDR::Register>,
    dr: ReadOnly<u32, DR::Register>,
}

#[repr(C)]
struct AdcCommonRegisters {
    csr: ReadOnly<u32, CSR::Register>,
    ccr: ReadWrite<u32, CCR::Register>,
}

register_bitfields![u32,
    /// Status register
    SR [
        /// Overrun
        OVR OFFSET(5) NUMBITS(1) [],
        /// Regular channel start flag
        STRT OFFSET(4) NUMBITS(1) [],
        /// Injected channel start flag
        JSTRT OFFSET(3) NUMBITS(1) [],
        /// Injected channel end of conversion
        JEOC OFFSET(2) NUMBITS(1) [],
        /// Regular channel end of conversion
        EOC OFFSET(1) NUMBITS(1) [],
        /// Analog watchdog flag
        AWD OFFSET(0) NUMBITS(1) []
    ],
    /// Control register 1
    CR1 [
        /// Overrun interrupt enable
        OVRIE OFFSET(26) NUMBITS(1) [],
        /// Resolution
        RES OFFSET(24) NUMBITS(2) [],
        /// Analog watchdog enable on regular channels
        AWDEN OFFSET(23) NUMBITS(1) [],
        /// Analog watchdog enable on injected channels
        JAWDEN OFFSET(22) NUMBITS(1) [],
        /// Discontinuous mode channel count
        DISCNUM OFFSET(13) NUMBITS(3) [],
        /// Discontinuous mode on injected channels
        JDISCEN OFFSET(12) NUMBITS(1) [],
        /// Discontinuous mode on regular channels
        DISCEN OFFSET(11) NUMBITS(1) [],
        /// Automatic injected group conversion
        JAUTO OFFSET(10) NUMBITS(1) [],
        /// Enable the watchdog on a single channel in scan mode
        AWDSGL OFFSET(9) NUMBITS(1) [],
        /// Scan mode
        SCAN OFFSET(8) NUMBITS(1) [],
        /// Interrupt enable for injected channels
        JEOCIE OFFSET(7) NUMBITS(1) [],
        /// Analog watchdog interrupt enable
        AWDIE OFFSET(6) NUMBITS(1) [],
        /// Interrupt enable for EOC
        EOCIE OFFSET(5) NUMBITS(1) [],
        /// Analog watchdog channel select bits
        AWDCH OFFSET(0) NUMBITS(4) []
    ],
    /// Control register 2
    CR2 [
        /// Start conversion of regular channels
        SWSTART OFFSET(30) NUMBITS(1) [],
        /// External trigger enable for regular channels
        EXTEN OFFSET(28) NUMBITS(2) [],
        /// External event select for regular group
        EXTSEL OFFSET(24) NUMBITS(4) [],
        /// Start conversion of injected channels
        JSWSTART OFFSET(22) NUMBITS(1) [],
        /// External trigger enable for injected channels
        JEXTEN OFFSET(20) NUMBITS(2) [],
        /// External event select for injected group
        JEXTSEL OFFSET(16) NUMBITS(4) [],
        /// Data alignment
        ALIGN OFFSET(11) NUMBITS(1) [],
        /// End of conversion selection
        EOCS OFFSET(10) NUMBITS(1) [],
        /// DMA disable selection (for single ADC mode)
        DDS OFFSET(9) NUMBITS(1) [],
        /// Direct memory access mode (for single ADC mode)
        DMA OFFSET(8) NUMBITS(1) [],
        /// Continuous conversion
        CONT OFFSET(1) NUMBITS(1) [],
        /// A/D Converter ON / OFF
        ADON OFFSET(0) NUMBITS(1) []
    ],
    /// Sample time register 1
    SMPR1 [
        /// Channel x sampling time selection
        SMP18 OFFSET(24) NUMBITS(3) [],
        SMP17 OFFSET(21) NUMBITS(3) [],
        SMP16 OFFSET(18) NUMBITS(3) [],
        SMP15 OFFSET(15) NUMBITS(3) [],
        SMP14 OFFSET(12) NUMBITS(3) [],
        SMP13 OFFSET(9) NUMBITS(3) [],
        SMP12 OFFSET(6) NUMBITS(3) [],
        SMP11 OFFSET(3) NUMBITS(3) [],
        SMP10 OFFSET(0) NUMBITS(3) []
    ],
    /// Sample time register 2
    SMPR2 [
        /// Channel x sampling time selection
        SMP9 OFFSET(27) NUMBITS(3) [],
        SMP8 OFFSET(24) NUMBITS(3) [],
        SMP7 OFFSET(21) NUMBITS(3) [],
        SMP6 OFFSET(18) NUMBITS(3) [],
        SMP5 OFFSET(15) NUMBITS(3) [],
        SMP4 OFFSET(12) NUMBITS(3) [],
        SMP3 OFFSET(9) NUMBITS(3) [],
        SMP2 OFFSET(6) NUMBITS(3) [],
        SMP1 OFFSET(3) NUMBITS(3) [],
        SMP0 OFFSET(0) NUMBITS(3) []
    ],
    /// injected channel data offsetregister x
    JOFR [
        /// Data offsetfor injected channel x
        JOFFSET OFFSET(0) NUMBITS(12) []
    ],
    /// Watchdog higher threshold register
    HTR [
        /// Analog watchdog higher threshold
        HT OFFSET(0) NUMBITS(12) []
    ],
    /// Watchdog lower threshold register
    LTR [
        /// Analog watchdog lower threshold
        LT OFFSET(0) NUMBITS(12) []
    ],
    /// Regular sequence register 1
    SQR1 [
        /// Regular channel sequence length
        L OFFSET(20) NUMBITS(3) [],
        /// 16th conversion in regular sequence
        SQ16 OFFSET(15) NUMBITS(5) [],
        /// 15th conversion in regular sequence
        SQ15 OFFSET(10) NUMBITS(5) [],
        /// 14th conversion in regular sequence
        SQ14 OFFSET(5) NUMBITS(5) [],
        /// 13th conversion in regular sequence
        SQ13 OFFSET(0) NUMBITS(5) []
    ],
    /// Regular sequence register 2
    SQR2 [
        /// 12th conversion in regular sequence
        SQ12 OFFSET(25) NUMBITS(5) [],
        /// 11th conversion in regular sequence
        SQ11 OFFSET(20) NUMBITS(5) [],
        /// 10th conversion in regular sequence
        SQ10 OFFSET(15) NUMBITS(5) [],
        /// 9th conversion in regular sequence
        SQ9 OFFSET(10) NUMBITS(5) [],
        /// 8th conversion in regular sequence
        SQ8 OFFSET(5) NUMBITS(5) [],
        /// 7th conversion in regular sequence
        SQ7 OFFSET(0) NUMBITS(5) []
    ],
    /// Regular sequence register 3
    SQR3 [
        /// 6th conversion in regular sequence
        SQ6 OFFSET(25) NUMBITS(5) [],
        /// 5th conversion in regular sequence
        SQ5 OFFSET(20) NUMBITS(5) [],
        /// 4th conversion in regular sequence
        SQ4 OFFSET(15) NUMBITS(5) [],
        /// 3rd conversion in regular sequence
        SQ3 OFFSET(10) NUMBITS(5) [],
        /// 2nd conversion in regular sequence
        SQ2 OFFSET(5) NUMBITS(5) [],
        /// 1st conversion in regular sequence
        SQ1 OFFSET(0) NUMBITS(5) []
    ],
    /// Injected sequence register
    JSQR [
        /// Note:  When JL[1:0]=3 (4 injected conversions in the sequencer), the ADC converts the channels
        ///      in the following order: JSQ1[4:0], JSQ2[4:0], JSQ3[4:0], and JSQ4[4:0].
        ///      When JL=2 (3 injected conversions in the sequencer), the ADC converts the channels in the
        ///      following order: JSQ2[4:0], JSQ3[4:0], and JSQ4[4:0].
        ///      When JL=1 (2 injected conversions in the sequencer), the ADC converts the channels in
        ///      starting from JSQ3[4:0], and then JSQ4[4:0].
        ///      When JL=0 (1 injected conversion in the sequencer), the ADC converts only JSQ4[4:0]
        ///      channel.
        /// Injected sequence length
        JL OFFSET(20) NUMBITS(2) [],
        /// 4th conversion in injected sequence
        JSQ4 OFFSET(15) NUMBITS(5) [],
        /// 3rd conversion in injected sequence
        JSQ3 OFFSET(15) NUMBITS(5) [],
        /// 2nd conversion in injected sequence
        JSQ2 OFFSET(15) NUMBITS(5) [],
        /// 1st conversion in injected sequence
        JSQ1 OFFSET(15) NUMBITS(5) []
    ],
    /// Injected data register x
    JDR [
        /// Injected data
        JDATA OFFSET(0) NUMBITS(16) []
    ],
    /// Regular data register
    DR [
        /// Regular data
        DATA OFFSET(0) NUMBITS(16) []
    ],
    /// Common status register
    CSR [
        /// Overrun flag of ADC1
        OVR1 OFFSET(5) NUMBITS(1) [],
        /// Regular channel Start flag of ADC1
        STRT1 OFFSET(4) NUMBITS(1) [],
        /// Injected channel Start flag of ADC1
        JSTRT1 OFFSET(3) NUMBITS(1) [],
        /// Injected channel end of conversion of ADC1
        JEOC1 OFFSET(2) NUMBITS(1) [],
        /// End of conversion of ADC1
        EOC1 OFFSET(1) NUMBITS(1) [],
        /// Analog watchdog flag of ADC1
        AWD1 OFFSET(0) NUMBITS(1) []
    ],
    /// Common control register
    CCR [
        /// Temperature sensor and VREFINT enable
        TSVREFE OFFSET(23) NUMBITS(1) [],
        /// VBAT enable
        VBATE OFFSET(22) NUMBITS(1) [],
        /// ADC prescaler
        ADCPRE OFFSET(16) NUMBITS(2) []
    ]
];

const ADC1_BASE: StaticRef<AdcRegisters> =
    unsafe { StaticRef::new(0x4001_2000 as *const AdcRegisters) };

const ADC_COMMON_BASE: StaticRef<AdcCommonRegisters> =
    unsafe { StaticRef::new(0x4001_2300 as *const AdcCommonRegisters) };

#[allow(dead_code)]
#[repr(u32)]
#[derive(Copy, Clone, PartialEq)]
pub enum Channel {
    Channel0 = 0b00000,
    Channel1 = 0b00001,
    Channel2 = 0b00010,
    Channel3 = 0b00011,
    Channel4 = 0b00100,
    Channel5 = 0b00101,
    Channel6 = 0b00110,
    Channel7 = 0b00111,
    Channel8 = 0b01000,
    Channel9 = 0b01001,
    Channel10 = 0b01010,
    Channel11 = 0b01011,
    Channel12 = 0b01100,
    Channel13 = 0b01101,
    Channel14 = 0b01110,
    Channel15 = 0b01111,
    Channel16 = 0b10000,
    Channel17 = 0b10001,
    Channel18 = 0b10010,
}

#[allow(dead_code)]
#[repr(u32)]
enum DataResolution {
    Bit12 = 0b00,
    Bit10 = 0b01,
    Bit8 = 0b10,
    Bit6 = 0b11,
}

#[derive(Copy, Clone, PartialEq)]
enum ADCStatus {
    Idle,
    Off,
    OneSample,
}

pub struct Adc<'a> {
    registers: StaticRef<AdcRegisters>,
    common_registers: StaticRef<AdcCommonRegisters>,
    clock: AdcClock<'a>,
    status: Cell<ADCStatus>,
    client: OptionalCell<&'static dyn hil::adc::Client>,
}

impl<'a> Adc<'a> {
    pub const fn new(rcc: &'a rcc::Rcc) -> Adc {
        Adc {
            registers: ADC1_BASE,
            common_registers: ADC_COMMON_BASE,
            clock: AdcClock(rcc::PeripheralClock::new(
                rcc::PeripheralClockType::APB2(rcc::PCLK2::ADC1),
                rcc,
            )),
            status: Cell::new(ADCStatus::Off),
            client: OptionalCell::empty(),
        }
    }

    pub fn enable(&self) {
        // Enable adc clock
        self.enable_clock();

        // Enable ADC
        self.registers.cr2.modify(CR2::ADON::SET);

        // set idle state
        self.status.set(ADCStatus::Idle);
    }

    pub fn handle_interrupt(&self) {
        // Check if regular group conversion ended
        if self.registers.sr.is_set(SR::EOC) {
            // Clear interrupt
            self.registers.cr1.modify(CR1::EOCIE::CLEAR);
            if self.status.get() == ADCStatus::OneSample {
                // set state
                self.status.set(ADCStatus::Idle);
            }
            self.client
                .map(|client| client.sample_ready(self.registers.dr.read(DR::DATA) as u16));
        }
    }

    pub fn is_enabled_clock(&self) -> bool {
        self.clock.is_enabled()
    }

    pub fn enable_clock(&self) {
        self.clock.enable();
    }

    pub fn disable_clock(&self) {
        self.clock.disable();
    }

    pub fn enable_temperature(&self) {
        self.common_registers.ccr.modify(CCR::TSVREFE::SET);
    }
}

struct AdcClock<'a>(rcc::PeripheralClock<'a>);

impl ClockInterface for AdcClock<'_> {
    fn is_enabled(&self) -> bool {
        self.0.is_enabled()
    }

    fn enable(&self) {
        self.0.enable();
    }

    fn disable(&self) {
        self.0.disable();
    }
}

impl hil::adc::Adc for Adc<'_> {
    type Channel = Channel;

    fn sample(&self, channel: &Self::Channel) -> ReturnCode {
        if self.status.get() == ADCStatus::Off {
            self.enable();
        }
        if *channel as u32 == 18 {
            self.enable_temperature();
        }
        if self.status.get() == ADCStatus::Idle {
            self.status.set(ADCStatus::OneSample);
            self.registers.sqr1.modify(SQR1::L.val(0b0000));
            self.registers.sqr3.modify(SQR3::SQ1.val(*channel as u32));
            self.registers.cr1.modify(CR1::EOCIE::SET);
            self.registers.cr2.modify(CR2::SWSTART::SET);
            ReturnCode::SUCCESS
        } else {
            ReturnCode::EBUSY
        }
    }

    fn sample_continuous(&self, _channel: &Self::Channel, _frequency: u32) -> ReturnCode {
        ReturnCode::ENOSUPPORT
    }

    fn stop_sampling(&self) -> ReturnCode {
        ReturnCode::ENOSUPPORT
    }

    fn get_resolution_bits(&self) -> usize {
        12
    }

    fn get_voltage_reference_mv(&self) -> Option<usize> {
        Some(3300)
    }

    fn set_client(&self, client: &'static dyn hil::adc::Client) {
        self.client.set(client);
    }
}

/// Not yet supported
impl hil::adc::AdcHighSpeed for Adc<'_> {
    /// Capture buffered samples from the ADC continuously at a given
    /// frequency, calling the client whenever a buffer fills up. The client is
    /// then expected to either stop sampling or provide an additional buffer
    /// to sample into. Note that due to hardware constraints the maximum
    /// frequency range of the ADC is from 187 kHz to 23 Hz (although its
    /// precision is limited at higher frequencies due to aliasing).
    ///
    /// - `channel`: the ADC channel to sample
    /// - `frequency`: frequency to sample at
    /// - `buffer1`: first buffer to fill with samples
    /// - `length1`: number of samples to collect (up to buffer length)
    /// - `buffer2`: second buffer to fill once the first is full
    /// - `length2`: number of samples to collect (up to buffer length)
    fn sample_highspeed(
        &self,
        _channel: &Self::Channel,
        _frequency: u32,
        _buffer1: &'static mut [u16],
        _length1: usize,
        _buffer2: &'static mut [u16],
        _length2: usize,
    ) -> (
        ReturnCode,
        Option<&'static mut [u16]>,
        Option<&'static mut [u16]>,
    ) {
        (ReturnCode::ENOSUPPORT, None, None)
    }

    /// Provide a new buffer to send on-going buffered continuous samples to.
    /// This is expected to be called after the `samples_ready` callback.
    ///
    /// - `buf`: buffer to fill with samples
    /// - `length`: number of samples to collect (up to buffer length)
    fn provide_buffer(
        &self,
        _buf: &'static mut [u16],
        _length: usize,
    ) -> (ReturnCode, Option<&'static mut [u16]>) {
        (ReturnCode::ENOSUPPORT, None)
    }

    /// Reclaim buffers after the ADC is stopped.
    /// This is expected to be called after `stop_sampling`.
    fn retrieve_buffers(
        &self,
    ) -> (
        ReturnCode,
        Option<&'static mut [u16]>,
        Option<&'static mut [u16]>,
    ) {
        (ReturnCode::ENOSUPPORT, None, None)
    }
}
