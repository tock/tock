//! Analog to Digital Converter Peripheral

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
    isr: ReadWrite<u32, ISR::Register>,
    ier: ReadWrite<u32, IER::Register>,
    cr: ReadWrite<u32, CR::Register>,
    cfgr: ReadWrite<u32, CFGR::Register>,

    _reserved0: [u32; 1],
    smpr1: ReadWrite<u32, SMPR1::Register>,
    smpr2: ReadWrite<u32, SMPR2::Register>,

    _reserved1: [u32; 1],
    tr1: ReadWrite<u32, TR1::Register>,
    tr2: ReadWrite<u32, TR2::Register>,
    tr3: ReadWrite<u32, TR3::Register>,

    _reserved2: [u32; 1],
    sqr1: ReadWrite<u32, SQR1::Register>,
    sqr2: ReadWrite<u32, SQR2::Register>,
    sqr3: ReadWrite<u32, SQR3::Register>,
    sqr4: ReadWrite<u32, SQR4::Register>,
    dr: ReadOnly<u32, DR::Register>,
    _reserved3: [u32; 2],

    jsqr: ReadWrite<u32, JSQR::Register>,
    _reserved4: [u32; 4],

    ofr1: ReadWrite<u32, OFR::Register>,
    ofr2: ReadWrite<u32, OFR::Register>,
    ofr3: ReadWrite<u32, OFR::Register>,
    ofr4: ReadWrite<u32, OFR::Register>,
    _reserved5: [u32; 4],

    jdr1: ReadOnly<u32, JDR::Register>,
    jdr2: ReadOnly<u32, JDR::Register>,
    jdr3: ReadOnly<u32, JDR::Register>,
    jdr4: ReadOnly<u32, JDR::Register>,
    _reserved6: [u32; 4],

    awd2cr: ReadWrite<u32, AWD2CR::Register>,
    awd3cr: ReadWrite<u32, AWD3CR::Register>,
    _reserved7: [u32; 2],

    difsel: ReadWrite<u32, DIFSEL::Register>,
    calfact: ReadWrite<u32, CALFACT::Register>,
}

#[repr(C)]
struct AdcCommonRegisters {
    csr: ReadOnly<u32, CSR::Register>,
    _reserved0: [u32; 1],

    ccr: ReadWrite<u32, CCR::Register>,
    cdr: ReadOnly<u32, CDR::Register>,
}

register_bitfields![u32,
    ///interrupt and status register
    ISR [
        /// Injected context queue overflow
        JQOVF OFFSET(10) NUMBITS(1) [],
        /// Analog watchdog 3 flag
        AWD3 OFFSET(9) NUMBITS(1) [],
        /// Analog watchdog 2 flag
        AWD2 OFFSET(8) NUMBITS(1) [],
        /// Analog watchdog 1 flag
        AWD1 OFFSET(7) NUMBITS(1) [],
        /// Injected channel end of sequence flag
        JEOS OFFSET(6) NUMBITS(1) [],
        /// Injected channel end of conversion flag
        JEOC OFFSET(5) NUMBITS(1) [],
        /// ADC overrun
        OVR OFFSET(4) NUMBITS(1) [],
        /// End of regular sequence flag
        EOS OFFSET(3) NUMBITS(1) [],
        /// End of conversion flag
        EOC OFFSET(2) NUMBITS(1) [],
        /// End of sampling flag
        EOSMP OFFSET(1) NUMBITS(1) [],
        /// ADC ready
        ADRDY OFFSET(0) NUMBITS(1) []
    ],
    /// Interrupt enable register
    IER [
        /// Injected context queue overflow interrupt enable
        JQOVFIE OFFSET(10) NUMBITS(1) [],
        /// Analog watchdog 3 interrupt enable
        AWD3IE OFFSET(9) NUMBITS(1) [],
        /// Analog watchdog 2 interrupt enable
        AWD2IE OFFSET(8) NUMBITS(1) [],
        /// Analog watchdog 1 interrupt enable
        AWD1IE OFFSET(7) NUMBITS(1) [],
        /// End of injected sequence of conversions interrupt enable
        JEOSIE OFFSET(6) NUMBITS(1) [],
        /// End of injected conversion interrupt enable
        JEOCIE OFFSET(5) NUMBITS(1) [],
        /// Overrun interrupt enable
        OVRIE OFFSET(4) NUMBITS(1) [],
        /// End of regular sequence of conversions interrupt enable
        EOSIE OFFSET(3) NUMBITS(1) [],
        /// End of regular conversion interrupt enable
        EOCIE OFFSET(2) NUMBITS(1) [],
        /// End of sampling flag interrupt enable for regular conversions
        EOSMPIE OFFSET(1) NUMBITS(1) [],
        /// ADC ready interrupt enable
        ADRDYIE OFFSET(0) NUMBITS(1) []
    ],
    /// Control register
    CR [
        /// ADC calibration
        ADCAL OFFSET(31) NUMBITS(1) [],
        /// Differential mode for calibration
        ADCALDIF OFFSET(30) NUMBITS(1) [],
        /// ADC voltage regulator enable
        ADVREGEN OFFSET(28) NUMBITS(2) [],
        /// ADC stop of injected conversion command
        JADSTP OFFSET(5) NUMBITS(1) [],
        /// ADC stop of regular conversion command
        ADSTP OFFSET(4) NUMBITS(1) [],
        /// ADC start of injected conversion
        JADSTART OFFSET(3) NUMBITS(1) [],
        /// ADC start of regular conversion
        ADSTART OFFSET(2) NUMBITS(1) [],
        /// ADC disable command
        ADDIS OFFSET(1) NUMBITS(1) [],
        /// ADC enable control
        ADEN OFFSET(0) NUMBITS(1) []
    ],
    /// Configuration register
    CFGR [
        /// Analog watchdog 1 channel selection
        AWD1CH OFFSET(26) NUMBITS(5) [],
        /// Automatic injected group conversion
        JAUTO OFFSET(25) NUMBITS(1) [],
        /// Analog watchdog 1 enable on injected channels
        JAWD1EN OFFSET(24) NUMBITS(1) [],
        /// Analog watchdog 1 enable on regular channels
        AWD1EN OFFSET(23) NUMBITS(1) [],
        /// Enable the watchdog 1 on a single channel or on all channels
        AWD1SGL OFFSET(22) NUMBITS(1) [],
        /// JSQR queue mode
        JQM OFFSET(21) NUMBITS(1) [],
        /// Discontinuous mode on injected channels
        JDISCEN OFFSET(20) NUMBITS(1) [],
        /// Discontinuous mode channel count
        DISCNUM OFFSET(17) NUMBITS(3) [],
        /// Discontinuous mode for regular channels
        DISCEN OFFSET(16) NUMBITS(1) [],
        /// Delayed conversion mode
        AUTDLY OFFSET(14) NUMBITS(1) [],
        /// Single / continuous conversion mode for regular conversions
        CONT OFFSET(13) NUMBITS(1) [],
        /// Overrun Mode
        OVRMOD OFFSET(12) NUMBITS(1) [],
        /// External trigger enable and polarity selection for regular channels
        EXTEN OFFSET(10) NUMBITS(2) [],
        /// External trigger selection for regular group
        EXTSEL OFFSET(6) NUMBITS(4) [],
        /// Data alignment
        ALIGN OFFSET(5) NUMBITS(1) [],
        /// Data resolution
        RES OFFSET(3) NUMBITS(2) [],
        /// Direct memory access configuration
        DMACFG OFFSET(1) NUMBITS(1) [],
        /// Direct memory access enable
        DMAEN OFFSET(0) NUMBITS(1) []
    ],
    /// Sample time register 1
    SMPR1 [
        /// Channel x sampling time selection
        SMP9 OFFSET(27) NUMBITS(3) [],
        SMP8 OFFSET(24) NUMBITS(3) [],
        SMP7 OFFSET(21) NUMBITS(3) [],
        SMP6 OFFSET(18) NUMBITS(3) [],
        SMP5 OFFSET(15) NUMBITS(3) [],
        SMP4 OFFSET(12) NUMBITS(3) [],
        SMP3 OFFSET(9) NUMBITS(3) [],
        SMP2 OFFSET(6) NUMBITS(3) [],
        SMP1 OFFSET(3) NUMBITS(3) []
    ],
    /// Sample time register 2
    SMPR2 [
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
    /// Watchdog threshold register 1
    TR1 [
        /// Analog watchdog 1 higher threshold
        HT1 OFFSET(16) NUMBITS(12) [],
        /// Analog watchdog 1 lower threshold
        LT1 OFFSET(0) NUMBITS(12) []
    ],
    /// Watchdog threshold register 2
    TR2 [
        /// Analog watchdog 2 higher threshold
        HT2 OFFSET(16) NUMBITS(8) [],
        /// Analog watchdog 2 lower threshold
        LT2 OFFSET(0) NUMBITS(8) []
    ],
    /// Watchdog threshold register 3
    TR3 [
        /// Analog watchdog 3 higher threshold
        HT3 OFFSET(16) NUMBITS(8) [],
        /// Analog watchdog 3 lower threshold
        LT3 OFFSET(0) NUMBITS(8) []
    ],
    /// Regular sequence register 1
    SQR1 [
        /// 4th conversion in regular sequence
        SQ4 OFFSET(24) NUMBITS(5) [],
        /// 3rd conversion in regular sequence
        SQ3 OFFSET(18) NUMBITS(5) [],
        /// 2nd conversion in regular sequence
        SQ2 OFFSET(12) NUMBITS(5) [],
        /// 1st conversion in regular sequence
        SQ1 OFFSET(6) NUMBITS(5) [],
        /// Regular channel sequence length
        L OFFSET(0) NUMBITS(4) []
    ],
    /// Regular sequence register 2
    SQR2 [
        SQ9 OFFSET(24) NUMBITS(5) [],
        /// 9th conversion in regular sequence
        SQ8 OFFSET(18) NUMBITS(5) [],
        /// 8th conversion in regular sequence
        SQ7 OFFSET(12) NUMBITS(5) [],
        /// 7th conversion in regular sequence
        SQ6 OFFSET(6) NUMBITS(5) [],
        /// 6th conversion in regular sequence
        SQ5 OFFSET(0) NUMBITS(5) []
    ],
    /// Regular sequence register 3
    SQR3 [
        /// 14th conversion in regular sequence
        SQ14 OFFSET(24) NUMBITS(5) [],
        /// 13th conversion in regular sequence
        SQ13 OFFSET(18) NUMBITS(5) [],
        /// 12th conversion in regular sequence
        SQ12 OFFSET(12) NUMBITS(5) [],
        /// 11th conversion in regular sequence
        SQ11 OFFSET(6) NUMBITS(5) [],
        /// 10th conversion in regular sequence
        SQ10 OFFSET(0) NUMBITS(5) []
    ],
    /// Regular sequence register 4
    SQR4 [
        /// 16th conversion in regular sequence
        SQ16 OFFSET(6) NUMBITS(5) [],
        /// 15th conversion in regular sequence
        SQ15 OFFSET(0) NUMBITS(5) []
    ],
    /// Regular Data Register
    DR [
        /// Regular Data converted
        RDATA OFFSET(0) NUMBITS(16) []
    ],
    /// Injected sequence register
    JSQR [
        /// 4th conversion in the injected sequence
        JSQ4 OFFSET(26) NUMBITS(5) [],
        /// 3rd conversion in the injected sequence
        JSQ3 OFFSET(20) NUMBITS(5) [],
        /// 2nd conversion in the injected sequence
        JSQ2 OFFSET(14) NUMBITS(5) [],
        /// 1st conversion in the injected sequence
        JSQ1 OFFSET(8) NUMBITS(5) [],
        /// External Trigger Enable and Polarity Selection for injected channels
        JEXTEN OFFSET(6) NUMBITS(2) [],
        /// External Trigger Selection for injected group
        JEXTSEL OFFSET(2) NUMBITS(4) [],
        /// Injected channel sequence length
        JL OFFSET(0) NUMBITS(2) []
    ],
    /// Offset register
    OFR [
        /// Offset y Enable
        OFFSET_EN OFFSET(31) NUMBITS(1) [],
        /// Channel selection for the Data offset y
        OFFSET_CH OFFSET(26) NUMBITS(5) [],
        /// Data offset y for the channel programmed into bits OFFSET_CH[4:0]
        OFFSETy OFFSET(0) NUMBITS(12) []
    ],
    /// Injected data register
    JDR [
        /// Injected data
        JDATA OFFSET(0) NUMBITS(16) []
    ],
    /// Analog Watchdog 2 Configuration Register
    AWD2CR [
        /// Analog watchdog 2 channel selection
        AWD2CH OFFSET(1) NUMBITS(18) []
    ],
    /// Analog Watchdog 3 Configuration Register
    AWD3CR [
        /// Analog watchdog 3 channel selection
        AWD3CH OFFSET(1) NUMBITS(18) []
    ],
    /// Differential Mode Selection Register
    DIFSEL [
        /// Differential mode for channels 18 to 16 r
        /// Differential mode for channels 15 to 1 r/w
        DIFSEL OFFSET(1) NUMBITS(18) []
    ],
    /// Calibration Factors
    CALFACT [
        /// Calibration Factors in differential mode
        CALFACT_D OFFSET(16) NUMBITS(7) [],
        /// Calibration Factors In Single-Ended mode
        CALFACT_S OFFSET(0) NUMBITS(7) []
    ],
    /// Common status register
    CSR [
        /// Injected Context Queue Overflow flag of the slave ADC
        JQOVF_SLV OFFSET(26) NUMBITS(1) [],
        /// Analog watchdog 3 flag of the slave ADC
        AWD3_SLV OFFSET(25) NUMBITS(1) [],
        /// Analog watchdog 2 flag of the slave ADC
        AWD2_SLV OFFSET(24) NUMBITS(1) [],
        /// Analog watchdog 1 flag of the slave ADC
        AWD1_SLV OFFSET(23) NUMBITS(1) [],
        /// End of injected sequence flag of the slave ADC
        JEOS_SLV OFFSET(22) NUMBITS(1) [],
        /// End of injected conversion flag of the slave ADC
        JEOC_SLV OFFSET(21) NUMBITS(1) [],
        /// Overrun flag of the slave ADC
        OVR_SLV OFFSET(20) NUMBITS(1) [],
        /// End of regular sequence flag of the slave ADC
        EOS_SLV OFFSET(19) NUMBITS(1) [],
        /// End of regular conversion of the slave ADC
        EOC_SLV OFFSET(18) NUMBITS(1) [],
        /// End of Sampling phase flag of the slave ADC
        EOSMP_SLV OFFSET(17) NUMBITS(1) [],
        /// Slave ADC ready
        ADRDY_SLV OFFSET(16) NUMBITS(1) [],
        /// Injected Context Queue Overflow flag of the master ADC
        JQOVF_MST OFFSET(10) NUMBITS(1) [],
        /// Analog watchdog 3 flag of the master ADC
        AWD3_MST OFFSET(9) NUMBITS(1) [],
        /// Analog watchdog 2 flag of the master ADC
        AWD2_MST OFFSET(8) NUMBITS(1) [],
        /// Analog watchdog 1 flag of the master ADC
        AWD1_MST OFFSET(7) NUMBITS(1) [],
        /// End of injected sequence flag of the master ADC
        JEOS_MST OFFSET(6) NUMBITS(1) [],
        /// End of injected conversion flag of the master ADC
        JEOC_MST OFFSET(5) NUMBITS(1) [],
        /// Overrun flag of the master ADC
        OVR_MST OFFSET(4) NUMBITS(1) [],
        /// End of regular sequence flag of the master ADC
        EOS_MST OFFSET(3) NUMBITS(1) [],
        /// End of regular conversion of the master ADC
        EOC_MST OFFSET(2) NUMBITS(1) [],
        /// End of Sampling phase flag of the master ADC
        EOSMP_MST OFFSET(1) NUMBITS(1) [],
        /// Master ADC ready
        ADRDY_MST OFFSET(0) NUMBITS(1) []
    ],
    /// Common control register
    CCR [
        /// VBAT enable
        VBATEN OFFSET(24) NUMBITS(1) [],
        /// Temperature sensor enable
        TSEN OFFSET(23) NUMBITS(1) [],
        /// VREFINT enable
        VREFEN OFFSET(22) NUMBITS(1) [],
        /// ADC clock mode
        CKMODE OFFSET(16) NUMBITS(2) [],
        /// Direct memory access mode for dual ADC mode
        MDMA OFFSET(14) NUMBITS(2) [],
        /// DMA configuration (for dual ADC mode)
        DMACFG OFFSET(13) NUMBITS(1) [],
        /// Delay between 2 sampling phases
        DELAY OFFSET(8) NUMBITS(4) [],
        /// Dual ADC mode selection
        DUAL OFFSET(0) NUMBITS(5) []
    ],
    /// Common regular data register for dual mode
    CDR [
        /// Regular data of the slave ADC
        RDATA_SLV OFFSET(16) NUMBITS(16) [],
        /// Regular data of the master ADC
        RDATA_MST OFFSET(0) NUMBITS(16) []
    ]
];

const ADC1_BASE: StaticRef<AdcRegisters> =
    unsafe { StaticRef::new(0x5000_0000 as *const AdcRegisters) };

const ADC12_COMMON_BASE: StaticRef<AdcCommonRegisters> =
    unsafe { StaticRef::new(0x5000_0300 as *const AdcCommonRegisters) };

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
enum DiscontinuousMode {
    OneChannels = 0b000,
    TwoChannels = 0b001,
    ThreeChannels = 0b010,
    FourChannels = 0b011,
    FiveChannels = 0b100,
    SixChannels = 0b101,
    SevenChannels = 0b110,
    EightChannels = 0b111,
}

#[allow(dead_code)]
#[repr(u32)]
enum ExternalTriggerDetection {
    Disabled = 0b00,
    RisingEdge = 0b01,
    FallingEdge = 0b10,
    RisingAndFalling = 0b11,
}

#[allow(dead_code)]
#[repr(u32)]
enum ExternalTriggerSelection {
    Event0 = 0b0000,
    Event1 = 0b0001,
    Event2 = 0b0010,
    Event3 = 0b0011,
    Event4 = 0b0100,
    Event5 = 0b0101,
    Event6 = 0b0110,
    Event7 = 0b0111,
    Event8 = 0b1000,
    Event9 = 0b1001,
    Event10 = 0b1010,
    Event11 = 0b1011,
    Event12 = 0b1100,
    Event13 = 0b1101,
    Event14 = 0b1110,
    Event15 = 0b1111,
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
    PoweringOn,
    OneSample,
    Continuous,
}

pub struct Adc {
    registers: StaticRef<AdcRegisters>,
    common_registers: StaticRef<AdcCommonRegisters>,
    clock: AdcClock,
    status: Cell<ADCStatus>,
    client: OptionalCell<&'static dyn hil::adc::Client>,
    requested: Cell<ADCStatus>,
    requested_channel: Cell<u32>,
    sc_enabled: Cell<bool>,
}

pub static mut ADC1: Adc = Adc::new();

impl Adc {
    const fn new() -> Adc {
        Adc {
            registers: ADC1_BASE,
            common_registers: ADC12_COMMON_BASE,
            clock: AdcClock(rcc::PeripheralClock::AHB(rcc::HCLK::ADC1)),
            status: Cell::new(ADCStatus::Off),
            client: OptionalCell::empty(),
            requested: Cell::new(ADCStatus::Idle),
            requested_channel: Cell::new(0),
            sc_enabled: Cell::new(false),
        }
    }

    pub fn enable_temperature(&self) {
        self.common_registers.ccr.modify(CCR::TSEN::SET);
    }

    pub fn enable(&self) {
        self.status.set(ADCStatus::PoweringOn);

        // Enable adc clock
        self.enable_clock();

        //Set Synchronous clock mode
        self.common_registers.ccr.modify(CCR::CKMODE.val(0b01));

        self.registers.cr.modify(CR::ADVREGEN.val(0b00));
        self.registers.cr.modify(CR::ADVREGEN.val(0b01));

        // Wait for ADVRGEN to enable
        // This needs to be synchronous because there is no interrupt signaling
        // when ADVRGEN becomes enabled
        // we chose 720 because the frequency is 72MHz and it needs 10 us to become enabled
        for _i in 0..720 {
            unsafe {
                llvm_asm!(
                "nop"
            : : : : "volatile" );
            }
        }

        // Enable ADC Ready interrupt
        self.registers.ier.modify(IER::ADRDYIE::SET);

        // Clear registers
        self.registers.isr.modify(ISR::ADRDY::CLEAR);
        self.registers.cr.modify(CR::ADEN::CLEAR);
        self.registers.cr.modify(CR::ADCALDIF::CLEAR);
        self.registers.cr.modify(CR::ADCAL::SET);

        // Wait for calibration
        while self.registers.cr.is_set(CR::ADCAL) {}

        // Enable ADC
        self.registers.cr.modify(CR::ADEN::SET);
        // Enable overrun to overwrite old datas
        self.registers.cfgr.modify(CFGR::OVRMOD::SET);
    }

    pub fn handle_interrupt(&self) {
        // Check if ADC is ready
        if self.registers.isr.is_set(ISR::ADRDY) {
            // Clear interrupt
            self.registers.ier.modify(IER::ADRDYIE::CLEAR);
            // Set Status
            if self.status.get() == ADCStatus::PoweringOn {
                self.status.set(ADCStatus::Idle);
                match self.requested.get() {
                    ADCStatus::OneSample => {
                        self.sample_u32(self.requested_channel.get());
                        return;
                    }
                    _ => {}
                }
            }
        }
        // Check if regular group conversion ended
        if self.registers.isr.is_set(ISR::EOC) {
            // Clear interrupt
            self.registers.ier.modify(IER::EOCIE::CLEAR);
            let data = self.registers.dr.read(DR::RDATA);
            self.client.map(|client| client.sample_ready(data as u16));
            if self.status.get() == ADCStatus::Continuous {
                self.registers.ier.modify(IER::EOCIE::SET);
            }
        }
        // Check if sequence of regular group conversion ended
        if self.registers.isr.is_set(ISR::EOS) {
            // Clear interrupt
            self.registers.ier.modify(IER::EOSIE::CLEAR);
            self.registers.isr.modify(ISR::EOS::SET);
            if self.status.get() == ADCStatus::OneSample {
                // stop adc
                self.registers.cr.modify(CR::ADSTP::SET);
                // set state
                self.status.set(ADCStatus::Idle);
            }
        }
        // Check if sampling ended
        if self.registers.isr.is_set(ISR::EOSMP) {
            // Clear interrupt
            self.registers.ier.modify(IER::EOSMPIE::CLEAR);
            self.registers.isr.modify(ISR::EOSMP::SET);
        }
        // Check if overrun occured
        if self.registers.isr.is_set(ISR::OVR) {
            // Clear interrupt
            self.registers.ier.modify(IER::OVRIE::CLEAR);
            self.registers.isr.modify(ISR::OVR::SET);
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

    fn enable_special_channels(&self) {
        // enabling temperature channel
        if self.requested_channel.get() == 16 {
            self.sc_enabled.set(true);
            self.enable_temperature();
        }
    }

    fn sample_u32(&self, channel: u32) -> ReturnCode {
        if self.sc_enabled.get() == false {
            self.enable_special_channels();
        }
        if self.status.get() == ADCStatus::Idle {
            self.requested.set(ADCStatus::Idle);
            self.status.set(ADCStatus::OneSample);
            self.registers.smpr2.modify(SMPR2::SMP16.val(0b100));
            self.registers.sqr1.modify(SQR1::L.val(0b0000));
            self.registers.sqr1.modify(SQR1::SQ1.val(channel));
            self.registers.ier.modify(IER::EOSIE::SET);
            self.registers.ier.modify(IER::EOCIE::SET);
            self.registers.ier.modify(IER::EOSMPIE::SET);
            self.registers.cr.modify(CR::ADSTART::SET);
            ReturnCode::SUCCESS
        } else {
            ReturnCode::EBUSY
        }
    }
}

struct AdcClock(rcc::PeripheralClock);

impl ClockInterface for AdcClock {
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

impl hil::adc::Adc for Adc {
    type Channel = Channel;

    fn sample(&self, channel: &Self::Channel) -> ReturnCode {
        if self.status.get() == ADCStatus::Off {
            self.requested.set(ADCStatus::OneSample);
            self.requested_channel.set(*channel as u32);
            self.enable();
            ReturnCode::SUCCESS
        } else {
            self.sample_u32(*channel as u32)
        }
    }

    fn sample_continuous(&self, _channel: &Self::Channel, _frequency: u32) -> ReturnCode {
        // Has to be implementer with timers because the frequency is too high
        ReturnCode::ENOSUPPORT
    }

    fn stop_sampling(&self) -> ReturnCode {
        if self.status.get() != ADCStatus::Idle && self.status.get() != ADCStatus::Off {
            self.registers.cr.modify(CR::ADSTP::SET);
            if self.registers.cfgr.is_set(CFGR::CONT) {
                self.registers.cfgr.modify(CFGR::CONT::CLEAR);
            }
            ReturnCode::SUCCESS
        } else {
            ReturnCode::EBUSY
        }
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
impl hil::adc::AdcHighSpeed for Adc {
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
