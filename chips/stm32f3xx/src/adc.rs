
use kernel::common::registers::{register_bitfields, ReadOnly, ReadWrite};
use kernel::common::StaticRef;
use kernel::hil;
use kernel::ClockInterface;

use crate::returncode::ReturnCode;
use crate::rcc;

#[repr(C)]
struct AdcRegisters {
	adc12: AdcSeparateRegisters,
	_reserved0: [u32; 48],
	
	adc34: AdcSeparateRegisters,
	_reserved3: [u32; 109],

	common: AdcCommonRegisters,
}

#[repr(C)]
struct AdcSeparateRegisters {
	
	isr: ReadWrite<u32, ISR::Register>,
	ier: ReadWrite<u32, IER::Register>,
	cr: ReadWrite<u32, CR::Register>,
	cfgr: ReadWrite<u32, CFGR::Register>,
	
	_reserved0: [u8; 4],
	smpr1: ReadWrite<u32, SMPR1::Register>,
	smpr2: ReadWrite<u32, SMPR2::Register>,
	
	_reserved1: [u8; 4],
	tr1: ReadWrite<u32, TR1::Register>,
	tr2: ReadWrite<u32, TR2::Register>,
	tr3: ReadWrite<u32, TR3::Register>,
	
	_reserved2: [u8; 4],
	sqr1: ReadWrite<u32, SQR1::Register>,
	sqr2: ReadWrite<u32, SQR2::Register>,
	sqr3: ReadWrite<u32, SQR3::Register>,
	sqr4: ReadWrite<u32, SQR4::Register>,
	dr: ReadOnly<u32, DR::Register>,
	_reserved3: [u8; 8],
	
	jsqr: ReadWrite<u32, JSQR::Register>,
	_reserved4: [u8; 16],

	ofr1: ReadWrite<u32, OFR::Register>,
	ofr2: ReadWrite<u32, OFR::Register>,
	ofr3: ReadWrite<u32, OFR::Register>,
	ofr4: ReadWrite<u32, OFR::Register>,
	_reserved5: [u8; 16],

	jdr1: ReadOnly<u32, JDR::Register>,
	jdr2: ReadOnly<u32, JDR::Register>,
	jdr3: ReadOnly<u32, JDR::Register>,
	jdr4: ReadOnly<u32, JDR::Register>,
	_reserved6: [u8; 20],

	awd2cr: ReadWrite<u32, AWD2CR::Register>,
	awd3cr: ReadWrite<u32, AWD3CR::Register>,
	_reserved7: [u8; 8],

	difsel: ReadWrite<u32, DIFSEL::Register>,
	calfact: ReadWrite<u32, CALFACT::Register>,
}

#[repr(C)]
struct AdcCommonRegisters {
	csr: ReadOnly<u32, CSR::Register>,
	_reserved0: [u8; 4],

	ccr: ReadWrite<u32, CCR::Register>,
	cdr: ReadOnly<u32, CDR::Register>,
}

register_bitfields![u32,
    ISR [
        JQOVF OFFSET(10) NUMBITS(1) [],
		AWD3 OFFSET(9) NUMBITS(1) [],
		AWD2 OFFSET(8) NUMBITS(1) [],
		AWD1 OFFSET(7) NUMBITS(1) [],
		JEOS OFFSET(6) NUMBITS(1) [],
		JEOC OFFSET(5) NUMBITS(1) [],
		OVR OFFSET(4) NUMBITS(1) [],
		EOS OFFSET(3) NUMBITS(1) [],
		EOC OFFSET(2) NUMBITS(1) [],
		EOSMP OFFSET(1) NUMBITS(1) [],
		ADRDY OFFSET(0) NUMBITS(1) []
	],
	IER [
		JQOVFIE OFFSET(10) NUMBITS(1) [],
		AWD3IE OFFSET(9) NUMBITS(1) [],
		AWD2IE OFFSET(8) NUMBITS(1) [],
		AWD1IE OFFSET(7) NUMBITS(1) [],
		JEOSIE OFFSET(6) NUMBITS(1) [],
		JEOCIE OFFSET(5) NUMBITS(1) [],
		OVRIE OFFSET(4) NUMBITS(1) [],
		EOSIE OFFSET(3) NUMBITS(1) [],
		EOCIE OFFSET(2) NUMBITS(1) [],
		EOSMPIE OFFSET(1) NUMBITS(1) [],
		ADRDYIE OFFSET(0) NUMBITS(1) []
	],
	CR [
		ADCAL OFFSET(31) NUMBITS(1) [],
		ADCALDIF OFFSET(30) NUMBITS(1) [],
		ADVREGEN OFFSET(29) NUMBITS(2) [],
		JADSTP OFFSET(5) NUMBITS(1) [],
		ADSTP OFFSET(4) NUMBITS(1) [],
		JADSTART OFFSET(3) NUMBITS(1) [],
		ADSTART OFFSET(2) NUMBITS(1) [],
		ADDIS OFFSET(1) NUMBITS(1) [],
		ADEN OFFSET(0) NUMBITS(1) []
	],
	CFGR [
		AWD1CH OFFSET(30) NUMBITS(5) [],
		JAUTO OFFSET(25) NUMBITS(1) [],
		JAWD1EN OFFSET(24) NUMBITS(1) [],
		AWD1EN OFFSET(23) NUMBITS(1) [],
		AWD1SGL OFFSET(22) NUMBITS(1) [],
		JQM OFFSET(21) NUMBITS(1) [],
		JDISCEN OFFSET(20) NUMBITS(1) [],
		DISCNUM OFFSET(19) NUMBITS(3) [],
		DISCEN OFFSET(16) NUMBITS(1) [],
		AUTDLY OFFSET(14) NUMBITS(1) [],
		CONT OFFSET(13) NUMBITS(1) [],
		OVRMOD OFFSET(12) NUMBITS(1) [],
		EXTEN OFFSET(11) NUMBITS(2) [],
		EXTSEL OFFSET(9) NUMBITS(4) [],
		ALIGN OFFSET(5) NUMBITS(1) [],
		RES OFFSET(4) NUMBITS(2) [],
		DMACFG OFFSET(1) NUMBITS(1) [],
		DMAEN OFFSET(0) NUMBITS(1) []
	],
	SMPR1 [
		SMP9 OFFSET(29) NUMBITS(3) [],
		SMP8 OFFSET(26) NUMBITS(3) [],
		SMP7 OFFSET(23) NUMBITS(3) [],
		SMP6 OFFSET(20) NUMBITS(3) [],
		SMP5 OFFSET(17) NUMBITS(3) [],
		SMP4 OFFSET(14) NUMBITS(3) [],
		SMP3 OFFSET(11) NUMBITS(3) [],
		SMP2 OFFSET(8) NUMBITS(3) [],
		SMP1 OFFSET(5) NUMBITS(3) []
	],
	SMPR2 [
		SMP18 OFFSET(26) NUMBITS(3) [],
		SMP17 OFFSET(23) NUMBITS(3) [],
		SMP16 OFFSET(20) NUMBITS(3) [],
		SMP15 OFFSET(17) NUMBITS(3) [],
		SMP14 OFFSET(14) NUMBITS(3) [],
		SMP13 OFFSET(11) NUMBITS(3) [],
		SMP12 OFFSET(8) NUMBITS(3) [],
		SMP11 OFFSET(5) NUMBITS(3) [],
		SMP10 OFFSET(2) NUMBITS(3) []
	],
	TR1 [
		HT1 OFFSET(27) NUMBITS(12) [],
		LT1 OFFSET(11) NUMBITS(12) []
	],
	TR2 [
		HT2 OFFSET(23) NUMBITS(8) [],
		LT2 OFFSET(7) NUMBITS(8) []
	],
	TR3 [
		HT3 OFFSET(23) NUMBITS(8) [],
		LT3 OFFSET(7) NUMBITS(8) []
	],
	SQR1 [
		SQ4 OFFSET(28) NUMBITS(5) [],
		SQ3 OFFSET(22) NUMBITS(5) [],
		SQ2 OFFSET(16) NUMBITS(5) [],
		SQ1 OFFSET(10) NUMBITS(5) [],
		L OFFSET(3) NUMBITS(4) []
	],
	SQR2 [
		SQ9 OFFSET(28) NUMBITS(5) [],
		SQ8 OFFSET(22) NUMBITS(5) [],
		SQ7 OFFSET(16) NUMBITS(5) [],
		SQ6 OFFSET(10) NUMBITS(5) [],
		SQ5 OFFSET(4) NUMBITS(5) []
	],
	SQR3 [
		SQ14 OFFSET(28) NUMBITS(5) [],
		SQ13 OFFSET(22) NUMBITS(5) [],
		SQ12 OFFSET(16) NUMBITS(5) [],
		SQ11 OFFSET(10) NUMBITS(5) [],
		SQ10 OFFSET(4) NUMBITS(5) []
	],
	SQR4 [
		SQ16 OFFSET(10) NUMBITS(5) [],
		SQ15 OFFSET(4) NUMBITS(5) []
	],
	DR [
		RDATA OFFSET(15) NUMBITS(16) []
	],
	JSQR [
		JSQ4 OFFSET(30) NUMBITS(5) [],
		JSQ3 OFFSET(24) NUMBITS(5) [],
		JSQ2 OFFSET(18) NUMBITS(5) [],
		JSQ1 OFFSET(12) NUMBITS(5) [],
		JEXTEN OFFSET(7) NUMBITS(2) [],
		JEXTSEL OFFSET(5) NUMBITS(4) [],
		JL OFFSET(1) NUMBITS(2) []
	],
	OFR [
		OFFSET_EN OFFSET(31) NUMBITS(1) [],
		OFFSET_CH OFFSET(30) NUMBITS(5) [],
		OFFSETy OFFSET(11) NUMBITS(12) []
	],
	JDR [
		JDATA OFFSET(15) NUMBITS(16) []
	],
	AWD2CR [
		AWD2CH OFFSET(18) NUMBITS(18) []
	],
	AWD3CR [
		AWD3CH OFFSET(18) NUMBITS(18) []
	],
	DIFSEL [
		DIFSEL OFFSET(18) NUMBITS(18) []
	],
	CALFACT [
		CALFACT_D OFFSET(22) NUMBITS(7) [],
		CALFACT_s OFFSET(6) NUMBITS(7) []
	],
	CSR [
		JQOVF_SLV OFFSET(26) NUMBITS(1) [],
		AWD3_SLV OFFSET(25) NUMBITS(1) [],
		AWD2_SLV OFFSET(24) NUMBITS(1) [],
		AWD1_SLV OFFSET(23) NUMBITS(1) [],
		JEOS_SLV OFFSET(22) NUMBITS(1) [],
		JEOC_SLV OFFSET(21) NUMBITS(1) [],
		OVR_SLV OFFSET(20) NUMBITS(1) [],
		EOS_SLV OFFSET(19) NUMBITS(1) [],
		EOC_SLV OFFSET(18) NUMBITS(1) [],
		EOSMP_SLV OFFSET(17) NUMBITS(1) [],
		ADRDY_SLV OFFSET(16) NUMBITS(1) [],
		JQOVF_MST OFFSET(10) NUMBITS(1) [],
		AWD3_MST OFFSET(9) NUMBITS(1) [],
		AWD2_MST OFFSET(8) NUMBITS(1) [],
		AWD1_MST OFFSET(7) NUMBITS(1) [],
		JEOS_MST OFFSET(6) NUMBITS(1) [],
		JEOC_MST OFFSET(5) NUMBITS(1) [],
		OVR_MST OFFSET(4) NUMBITS(1) [],
		EOS_MST OFFSET(3) NUMBITS(1) [],
		EOC_MST OFFSET(2) NUMBITS(1) [],
		EOSMP_MST OFFSET(1) NUMBITS(1) [],
		ADRDY_MST OFFSET(0) NUMBITS(1) []
	],
	CCR [
		VBATEN OFFSET(24) NUMBITS(1) [],
		TSEN OFFSET(23) NUMBITS(1) [],
		VREFEN OFFSET(22) NUMBITS(1) [],
		CKMODE OFFSET(17) NUMBITS(2) [],
		MDMA OFFSET(15) NUMBITS(2) [],
		DMACFG OFFSET(13) NUMBITS(1) [],
		DELAY OFFSET(11) NUMBITS(4) [],
		DUAL OFFSET(4) NUMBITS(5) []
	],
	CDR [
		RDATA_SLV OFFSET(31) NUMBITS(16) [],
		RDATA_MST OFFSET(15) NUMBITS(16) []
	]
];

const ADC_BASE: StaticRef<AdcRegisters> =
	unsafe {StaticRef::new(0x5000_0000 as *const AdcRegisters)};

#[allow(dead_code)]
#[repr(u32)]
enum ChannelId {
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

pub struct Adc {
	registers: StaticRef<AdcRegisters>,
	clock: AdcClock,
}

pub static mut ADC: Adc = Adc::new();

impl Adc {
	const fn new() -> Adc {
		Adc {
			registers: ADC_BASE,
			clock12: AdcClock(rcc::PeripheralClock::AHB3(rcc::HCLK::ADC12)),
			#[cfg(feature = "stm32f303vct6")]
			clock34: AdcClock(rcc::PeripheralClock::AHB3(rcc::HCLK::ADC34)),
		}
	}

	// 12
	pub fn is_enabled_clock12(&self) -> bool {
		self.clock12.is_enabled()
	}

	pub fn enable_clock12(&self) {
        self.clock12.enable();
    }

    pub fn disable_clock12(&self) {
        self.clock12.disable();
	}
	
	// 34
	pub fn is_enabled_clock34(&self) -> bool {
		self.clock34.is_enabled()
	}

	pub fn enable_clock34(&self) {
        self.clock34.enable();
    }

    pub fn disable_clock34(&self) {
        self.clock34.disable();
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
	fn sample(&self, channel: &Self::Channel) -> ReturnCode {
		
	}
}