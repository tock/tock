//! General Purpose Input/Output driver.

use core::ops::{Index, IndexMut};
use enum_primitive::cast::FromPrimitive;
use enum_primitive::enum_from_primitive;
use kernel::common::cells::OptionalCell;
use kernel::common::registers::{register_bitfields, register_structs, ReadWrite};
use kernel::common::StaticRef;
use kernel::hil::gpio;

pub const GPIO_BASE_RAW: usize = 0x4001_0000; //safe to export outside crate

const GPIO_BASE: StaticRef<GpioRegisters> =
    unsafe { StaticRef::new(GPIO_BASE_RAW as *const GpioRegisters) };

pub struct Port<'a> {
    pins: [GpioPin<'a>; 50],
}

impl<'a> Port<'a> {
    pub const fn new() -> Self {
        Self {
            pins: [
                GpioPin::new(GPIO_BASE, Pin::Pin00),
                GpioPin::new(GPIO_BASE, Pin::Pin01),
                GpioPin::new(GPIO_BASE, Pin::Pin02),
                GpioPin::new(GPIO_BASE, Pin::Pin03),
                GpioPin::new(GPIO_BASE, Pin::Pin04),
                GpioPin::new(GPIO_BASE, Pin::Pin05),
                GpioPin::new(GPIO_BASE, Pin::Pin06),
                GpioPin::new(GPIO_BASE, Pin::Pin07),
                GpioPin::new(GPIO_BASE, Pin::Pin08),
                GpioPin::new(GPIO_BASE, Pin::Pin09),
                GpioPin::new(GPIO_BASE, Pin::Pin10),
                GpioPin::new(GPIO_BASE, Pin::Pin11),
                GpioPin::new(GPIO_BASE, Pin::Pin12),
                GpioPin::new(GPIO_BASE, Pin::Pin13),
                GpioPin::new(GPIO_BASE, Pin::Pin14),
                GpioPin::new(GPIO_BASE, Pin::Pin15),
                GpioPin::new(GPIO_BASE, Pin::Pin16),
                GpioPin::new(GPIO_BASE, Pin::Pin17),
                GpioPin::new(GPIO_BASE, Pin::Pin18),
                GpioPin::new(GPIO_BASE, Pin::Pin19),
                GpioPin::new(GPIO_BASE, Pin::Pin20),
                GpioPin::new(GPIO_BASE, Pin::Pin21),
                GpioPin::new(GPIO_BASE, Pin::Pin22),
                GpioPin::new(GPIO_BASE, Pin::Pin23),
                GpioPin::new(GPIO_BASE, Pin::Pin24),
                GpioPin::new(GPIO_BASE, Pin::Pin25),
                GpioPin::new(GPIO_BASE, Pin::Pin26),
                GpioPin::new(GPIO_BASE, Pin::Pin27),
                GpioPin::new(GPIO_BASE, Pin::Pin28),
                GpioPin::new(GPIO_BASE, Pin::Pin29),
                GpioPin::new(GPIO_BASE, Pin::Pin30),
                GpioPin::new(GPIO_BASE, Pin::Pin31),
                GpioPin::new(GPIO_BASE, Pin::Pin32),
                GpioPin::new(GPIO_BASE, Pin::Pin33),
                GpioPin::new(GPIO_BASE, Pin::Pin34),
                GpioPin::new(GPIO_BASE, Pin::Pin35),
                GpioPin::new(GPIO_BASE, Pin::Pin36),
                GpioPin::new(GPIO_BASE, Pin::Pin37),
                GpioPin::new(GPIO_BASE, Pin::Pin38),
                GpioPin::new(GPIO_BASE, Pin::Pin39),
                GpioPin::new(GPIO_BASE, Pin::Pin40),
                GpioPin::new(GPIO_BASE, Pin::Pin41),
                GpioPin::new(GPIO_BASE, Pin::Pin42),
                GpioPin::new(GPIO_BASE, Pin::Pin43),
                GpioPin::new(GPIO_BASE, Pin::Pin44),
                GpioPin::new(GPIO_BASE, Pin::Pin45),
                GpioPin::new(GPIO_BASE, Pin::Pin46),
                GpioPin::new(GPIO_BASE, Pin::Pin47),
                GpioPin::new(GPIO_BASE, Pin::Pin48),
                GpioPin::new(GPIO_BASE, Pin::Pin49),
            ],
        }
    }
}

impl<'a> Index<usize> for Port<'a> {
    type Output = GpioPin<'a>;

    fn index(&self, index: usize) -> &GpioPin<'a> {
        &self.pins[index]
    }
}

impl<'a> IndexMut<usize> for Port<'a> {
    fn index_mut(&mut self, index: usize) -> &mut GpioPin<'a> {
        &mut self.pins[index]
    }
}

impl Port<'_> {
    pub fn handle_interrupt(&self) {
        let regs = GPIO_BASE;
        let mut irqs = regs.int0stat.get();
        regs.int0clr.set(irqs);

        let mut count = 0;
        while irqs != 0 && count < self.pins.len() {
            if (irqs & 0b1) != 0 {
                self.pins[count].handle_interrupt();
            }
            count += 1;
            irqs >>= 1;
        }

        let mut irqs = regs.int1stat.get();
        regs.int1clr.set(irqs);

        let mut count = 0;
        while irqs != 0 && count < self.pins.len() {
            if (irqs & 0b1) != 0 {
                self.pins[count].handle_interrupt();
            }
            count += 1;
            irqs >>= 1;
        }
    }

    pub fn enable_uart(&self, tx_pin: &GpioPin, rx_pin: &GpioPin) {
        let regs = GPIO_BASE;

        match tx_pin.pin as usize {
            48 => {
                regs.padkey.set(115);
                regs.padreg[12].set(regs.padreg[12].get() & 0xffffff00);
                regs.cfg[6].modify(CFG::GPIO0INTD.val(0x00) + CFG::GPIO0OUTCFG.val(0x00));
                regs.altpadcfgm
                    .modify(ALTPADCFG::PAD0_DS1::CLEAR + ALTPADCFG::PAD0_SR::CLEAR);
                regs.padkey.set(0x00);
            }
            _ => {
                panic!("tx_pin not supported");
            }
        }

        match rx_pin.pin as usize {
            49 => {
                regs.padkey.set(115);
                regs.padreg[12].modify(PADREG::PAD1INPEN::SET);
                regs.cfg[6].modify(CFG::GPIO1INTD.val(0x00) + CFG::GPIO1OUTCFG.val(0x00));
                regs.altpadcfgm
                    .modify(ALTPADCFG::PAD1_DS1::CLEAR + ALTPADCFG::PAD1_SR::CLEAR);
                regs.padkey.set(0x00);
            }
            _ => {
                panic!("rx_pin not supported");
            }
        }
    }

    pub fn enable_i2c(&self, sda: &GpioPin, scl: &GpioPin) {
        let regs = GPIO_BASE;

        match sda.pin as usize {
            25 => {
                regs.padkey.set(115);
                regs.padreg[6].modify(
                    PADREG::PAD1PULL::SET
                        + PADREG::PAD1INPEN::SET
                        + PADREG::PAD1STRNG::SET
                        + PADREG::PAD1FNCSEL.val(0x4),
                );
                regs.cfg[3].modify(CFG::GPIO1INTD.val(0x00) + CFG::GPIO1OUTCFG.val(0x02));
                regs.altpadcfgg
                    .modify(ALTPADCFG::PAD1_DS1::CLEAR + ALTPADCFG::PAD1_SR::CLEAR);
                regs.padkey.set(0x00);
            }
            _ => {
                panic!("sda not supported");
            }
        }

        match scl.pin as usize {
            27 => {
                regs.padkey.set(115);
                regs.padreg[6].modify(
                    PADREG::PAD3PULL::SET
                        + PADREG::PAD3INPEN::SET
                        + PADREG::PAD3STRNG::SET
                        + PADREG::PAD3FNCSEL.val(0x4),
                );
                regs.cfg[3].modify(CFG::GPIO3INTD.val(0x00) + CFG::GPIO3OUTCFG.val(0x02));
                regs.altpadcfgg
                    .modify(ALTPADCFG::PAD3_DS1::CLEAR + ALTPADCFG::PAD3_SR::CLEAR);
                regs.padkey.set(0x00);
            }
            _ => {
                panic!("scl not supported");
            }
        }
    }
}

enum_from_primitive! {
    #[derive(Copy, Clone, Debug, PartialEq)]
    pub enum Pin {
        Pin00, Pin01, Pin02, Pin03, Pin04, Pin05, Pin06, Pin07,
        Pin08, Pin09, Pin10, Pin11, Pin12, Pin13, Pin14, Pin15,
        Pin16, Pin17, Pin18, Pin19, Pin20, Pin21, Pin22, Pin23,
        Pin24, Pin25, Pin26, Pin27, Pin28, Pin29, Pin30, Pin31,
        Pin32, Pin33, Pin34, Pin35, Pin36, Pin37, Pin38, Pin39,
        Pin40, Pin41, Pin42, Pin43, Pin44, Pin45, Pin46, Pin47,
        Pin48, Pin49,
    }
}

register_structs! {
    pub GpioRegisters {
        (0x00 => padreg: [ReadWrite<u32, PADREG::Register>; 13]),
        (0x34 => _reserved0),
        (0x40 => cfg: [ReadWrite<u32, CFG::Register>; 7]),
        (0x5C => _reserved1),
        (0x60 => padkey: ReadWrite<u32, PADKEY::Register>),
        (0x64 => _reserved2),
        (0x80 => rda: ReadWrite<u32, RDA::Register>),
        (0x84 => rdb: ReadWrite<u32, RDB::Register>),
        (0x88 => wta: ReadWrite<u32, WTA::Register>),
        (0x8C => wtb: ReadWrite<u32, WTB::Register>),
        (0x90 => wtsa: ReadWrite<u32, WTSA::Register>),
        (0x94 => wtsb: ReadWrite<u32, WTSB::Register>),
        (0x98 => wtca: ReadWrite<u32, WTCA::Register>),
        (0x9c => wtcb: ReadWrite<u32, WTCB::Register>),
        (0xA0 => ena: ReadWrite<u32, ENA::Register>),
        (0xA4 => enb: ReadWrite<u32, ENB::Register>),
        (0xA8 => ensa: ReadWrite<u32, ENSA::Register>),
        (0xAC => ensb: ReadWrite<u32, ENSB::Register>),
        (0xB0 => _reserved3),
        (0xB4 => enca: ReadWrite<u32, ENCA::Register>),
        (0xB8 => encb: ReadWrite<u32, ENCB::Register>),
        (0xBC => stmrcap: ReadWrite<u32, STMRCAP::Register>),
        (0xC0 => iom0irq: ReadWrite<u32, IOMIRQ::Register>),
        (0xC4 => iom1irq: ReadWrite<u32, IOMIRQ::Register>),
        (0xC8 => iom2irq: ReadWrite<u32, IOMIRQ::Register>),
        (0xCC => iom3irq: ReadWrite<u32, IOMIRQ::Register>),
        (0xD0 => iom4irq: ReadWrite<u32, IOMIRQ::Register>),
        (0xD4 => iom5irq: ReadWrite<u32, IOMIRQ::Register>),
        (0xD8 => bleif5irq: ReadWrite<u32, IOMIRQ::Register>),
        (0xDC => gpioobs: ReadWrite<u32, GPIOOBS::Register>),
        (0xE0 => altpadcfga: ReadWrite<u32, ALTPADCFG::Register>),
        (0xE4 => altpadcfgb: ReadWrite<u32, ALTPADCFG::Register>),
        (0xE8 => altpadcfgc: ReadWrite<u32, ALTPADCFG::Register>),
        (0xEC => altpadcfgd: ReadWrite<u32, ALTPADCFG::Register>),
        (0xF0 => altpadcfge: ReadWrite<u32, ALTPADCFG::Register>),
        (0xF4 => altpadcfgf: ReadWrite<u32, ALTPADCFG::Register>),
        (0xF8 => altpadcfgg: ReadWrite<u32, ALTPADCFG::Register>),
        (0xFC => altpadcfgh: ReadWrite<u32, ALTPADCFG::Register>),
        (0x100 => altpadcfgi: ReadWrite<u32, ALTPADCFG::Register>),
        (0x104 => altpadcfgj: ReadWrite<u32, ALTPADCFG::Register>),
        (0x108 => altpadcfgk: ReadWrite<u32, ALTPADCFG::Register>),
        (0x10C => altpadcfgl: ReadWrite<u32, ALTPADCFG::Register>),
        (0x110 => altpadcfgm: ReadWrite<u32, ALTPADCFG::Register>),
        (0x114 => scdet: ReadWrite<u32, SCDET::Register>),
        (0x118 => ctencfg: ReadWrite<u32, CTENCFG::Register>),
        (0x11C => _reserved4),
        (0x200 => int0en: ReadWrite<u32, INT0::Register>),
        (0x204 => int0stat: ReadWrite<u32, INT0::Register>),
        (0x208 => int0clr: ReadWrite<u32, INT0::Register>),
        (0x20C => int0set: ReadWrite<u32, INT0::Register>),
        (0x210 => int1en: ReadWrite<u32, INT1::Register>),
        (0x214 => int1stat: ReadWrite<u32, INT1::Register>),
        (0x218 => int1clr: ReadWrite<u32, INT1::Register>),
        (0x21C => int1set: ReadWrite<u32, INT1::Register>),
        (0x220 => @END),
    }
}

register_bitfields![u32,
    PADREG [
        PAD0PULL OFFSET(0) NUMBITS(1) [],
        PAD0INPEN OFFSET(1) NUMBITS(1) [],
        PAD0STRING OFFSET(2) NUMBITS(1) [],
        PAD0FNCSEL OFFSET(3) NUMBITS(3) [],
        PAD0RSEL OFFSET(6) NUMBITS(2) [],
        PAD1PULL OFFSET(8) NUMBITS(1) [],
        PAD1INPEN OFFSET(9) NUMBITS(1) [],
        PAD1STRNG OFFSET(10) NUMBITS(1) [],
        PAD1FNCSEL OFFSET(11) NUMBITS(3) [],
        PAD1RSEL OFFSET(14) NUMBITS(2) [],
        PAD2PULL OFFSET(16) NUMBITS(1) [],
        PAD2INPEN OFFSET(17) NUMBITS(1) [],
        PAD2STRNG OFFSET(18) NUMBITS(1) [],
        PAD2FNCSEL OFFSET(19) NUMBITS(3) [],
        PAD3PULL OFFSET(24) NUMBITS(1) [],
        PAD3INPEN OFFSET(25) NUMBITS(1) [],
        PAD3STRNG OFFSET(26) NUMBITS(1) [],
        PAD3FNCSEL OFFSET(27) NUMBITS(3) [],
        PAD3RSEL OFFSET(30) NUMBITS(2) []
    ],
    CFG [
        GPIO0INCFG OFFSET(0) NUMBITS(1) [],
        GPIO0OUTCFG OFFSET(1) NUMBITS(2) [],
        GPIO0INTD OFFSET(3) NUMBITS(1) [],
        GPIO1INCFG OFFSET(4) NUMBITS(1) [],
        GPIO1OUTCFG OFFSET(5) NUMBITS(2) [],
        GPIO1INTD OFFSET(7) NUMBITS(1) [],
        GPIO2INCFG OFFSET(8) NUMBITS(1) [],
        GPIO2OUTCFG OFFSET(9) NUMBITS(2) [],
        GPIO2INTD OFFSET(11) NUMBITS(1) [],
        GPIO3INCFG OFFSET(12) NUMBITS(1) [],
        GPIO3OUTCFG OFFSET(13) NUMBITS(2) [],
        GPIO3INTD OFFSET(15) NUMBITS(1) [],
        GPIO4INCFG OFFSET(16) NUMBITS(1) [],
        GPIO4OUTCFG OFFSET(17) NUMBITS(2) [],
        GPIO4INTD OFFSET(19) NUMBITS(1) [],
        GPIO5INCFG OFFSET(20) NUMBITS(1) [],
        GPIO5OUTCFG OFFSET(21) NUMBITS(2) [],
        GPIO5INTD OFFSET(23) NUMBITS(1) [],
        GPIO6INCFG OFFSET(24) NUMBITS(1) [],
        GPIO6OUTCFG OFFSET(25) NUMBITS(2) [],
        GPIO6INTD OFFSET(27) NUMBITS(1) [],
        GPIO7INCFG OFFSET(28) NUMBITS(1) [],
        GPIO7OUTCFG OFFSET(29) NUMBITS(2) [],
        GPIO7INTD OFFSET(31) NUMBITS(1) []
    ],
    PADKEY [
        PADKEY OFFSET(0) NUMBITS(31) []
    ],
    RDA [
        RDA OFFSET(0) NUMBITS(31) []
    ],
    RDB [
        RDB OFFSET(0) NUMBITS(17) []
    ],
    WTA [
        WTA OFFSET(0) NUMBITS(31) []
    ],
    WTB [
        WTB OFFSET(0) NUMBITS(17) []
    ],
    WTSA [
        WTSA OFFSET(0) NUMBITS(31) []
    ],
    WTSB [
        WTSB OFFSET(0) NUMBITS(17) []
    ],
    WTCA [
        WTCA OFFSET(0) NUMBITS(31) []
    ],
    WTCB [
        WTCB OFFSET(0) NUMBITS(17) []
    ],
    ENA [
        ENA OFFSET(0) NUMBITS(31) []
    ],
    ENB [
        ENB OFFSET(0) NUMBITS(17) []
    ],
    ENSA [
        ENSA OFFSET(0) NUMBITS(31) []
    ],
    ENSB [
        ENSB OFFSET(0) NUMBITS(17) []
    ],
    ENCA [
        ENCA OFFSET(0) NUMBITS(31) []
    ],
    ENCB [
        ENCB OFFSET(0) NUMBITS(17) []
    ],
    STMRCAP [
        STSEL0 OFFSET(0) NUMBITS(5) [],
        STPOL0 OFFSET(6) NUMBITS(1) [],
        STSEL1 OFFSET(8) NUMBITS(5) [],
        STPOL1 OFFSET(14) NUMBITS(1) [],
        STSEL2 OFFSET(16) NUMBITS(5) [],
        STPOL2 OFFSET(2) NUMBITS(1) [],
        STSEL3 OFFSET(24) NUMBITS(5) [],
        STPOL3 OFFSET(30) NUMBITS(1) []
    ],
    IOMIRQ [
        IOMIRQ OFFSET(0) NUMBITS(5) []
    ],
    GPIOOBS [
        OBS_DATA OFFSET(0) NUMBITS(15) []
    ],
    ALTPADCFG [
        PAD0_DS1 OFFSET(0) NUMBITS(1) [],
        PAD0_SR OFFSET(4) NUMBITS(1) [],
        PAD1_DS1 OFFSET(8) NUMBITS(1) [],
        PAD1_SR OFFSET(12) NUMBITS(1) [],
        PAD2_DS1 OFFSET(16) NUMBITS(1) [],
        PAD2_SR OFFSET(20) NUMBITS(1) [],
        PAD3_DS1 OFFSET(24) NUMBITS(1) [],
        PAD3_SR OFFSET(28) NUMBITS(1) []
    ],
    SCDET [
        SCDET OFFSET(0) NUMBITS(5) []
    ],
    CTENCFG [
        EN0 OFFSET(0) NUMBITS(1) [],
        EN1 OFFSET(1) NUMBITS(1) [],
        EN2 OFFSET(2) NUMBITS(1) [],
        EN3 OFFSET(3) NUMBITS(1) [],
        EN4 OFFSET(4) NUMBITS(1) [],
        EN5 OFFSET(5) NUMBITS(1) [],
        EN6 OFFSET(6) NUMBITS(1) [],
        EN7 OFFSET(7) NUMBITS(1) [],
        EN8 OFFSET(8) NUMBITS(1) [],
        EN9 OFFSET(9) NUMBITS(1) [],
        EN10 OFFSET(10) NUMBITS(1) [],
        EN11 OFFSET(11) NUMBITS(1) [],
        EN12 OFFSET(12) NUMBITS(1) [],
        EN13 OFFSET(13) NUMBITS(1) [],
        EN14 OFFSET(14) NUMBITS(1) [],
        EN15 OFFSET(15) NUMBITS(1) [],
        EN16 OFFSET(16) NUMBITS(1) [],
        EN17 OFFSET(17) NUMBITS(1) [],
        EN18 OFFSET(18) NUMBITS(1) [],
        EN19 OFFSET(19) NUMBITS(1) [],
        EN20 OFFSET(20) NUMBITS(1) [],
        EN21 OFFSET(21) NUMBITS(1) [],
        EN22 OFFSET(22) NUMBITS(1) [],
        EN23 OFFSET(23) NUMBITS(1) [],
        EN24 OFFSET(24) NUMBITS(1) [],
        EN25 OFFSET(25) NUMBITS(1) [],
        EN26 OFFSET(26) NUMBITS(1) [],
        EN27 OFFSET(27) NUMBITS(1) [],
        EN28 OFFSET(28) NUMBITS(1) [],
        EN29 OFFSET(29) NUMBITS(1) [],
        EN30 OFFSET(30) NUMBITS(1) [],
        EN31 OFFSET(31) NUMBITS(1) []
    ],
    INT0 [
        GPIO0 OFFSET(0) NUMBITS(1) [],
        GPIO1 OFFSET(1) NUMBITS(1) [],
        GPIO2 OFFSET(2) NUMBITS(1) [],
        GPIO3 OFFSET(3) NUMBITS(1) [],
        GPIO4 OFFSET(4) NUMBITS(1) [],
        GPIO5 OFFSET(5) NUMBITS(1) [],
        GPIO6 OFFSET(6) NUMBITS(1) [],
        GPIO7 OFFSET(7) NUMBITS(1) [],
        GPIO8 OFFSET(8) NUMBITS(1) [],
        GPIO9 OFFSET(9) NUMBITS(1) [],
        GPIO10 OFFSET(10) NUMBITS(1) [],
        GPIO11 OFFSET(11) NUMBITS(1) [],
        GPIO12 OFFSET(12) NUMBITS(1) [],
        GPIO13 OFFSET(13) NUMBITS(1) [],
        GPIO14 OFFSET(14) NUMBITS(1) [],
        GPIO15 OFFSET(15) NUMBITS(1) [],
        GPIO16 OFFSET(16) NUMBITS(1) [],
        GPIO17 OFFSET(17) NUMBITS(1) [],
        GPIO18 OFFSET(18) NUMBITS(1) [],
        GPIO19 OFFSET(19) NUMBITS(1) [],
        GPIO20 OFFSET(20) NUMBITS(1) [],
        GPIO21 OFFSET(21) NUMBITS(1) [],
        GPIO22 OFFSET(22) NUMBITS(1) [],
        GPIO23 OFFSET(23) NUMBITS(1) [],
        GPIO24 OFFSET(24) NUMBITS(1) [],
        GPIO25 OFFSET(25) NUMBITS(1) [],
        GPIO26 OFFSET(26) NUMBITS(1) [],
        GPIO27 OFFSET(27) NUMBITS(1) [],
        GPIO28 OFFSET(28) NUMBITS(1) [],
        GPIO29 OFFSET(29) NUMBITS(1) [],
        GPIO30 OFFSET(30) NUMBITS(1) [],
        GPIO31 OFFSET(31) NUMBITS(1) []
    ],
    INT1 [
        GPIO32 OFFSET(0) NUMBITS(1) [],
        GPIO33 OFFSET(1) NUMBITS(1) [],
        GPIO34 OFFSET(2) NUMBITS(1) [],
        GPIO35 OFFSET(3) NUMBITS(1) [],
        GPIO36 OFFSET(4) NUMBITS(1) [],
        GPIO37 OFFSET(5) NUMBITS(1) [],
        GPIO38 OFFSET(6) NUMBITS(1) [],
        GPIO39 OFFSET(7) NUMBITS(1) [],
        GPIO40 OFFSET(8) NUMBITS(1) [],
        GPIO41 OFFSET(9) NUMBITS(1) [],
        GPIO42 OFFSET(10) NUMBITS(1) [],
        GPIO43 OFFSET(11) NUMBITS(1) [],
        GPIO44 OFFSET(12) NUMBITS(1) [],
        GPIO45 OFFSET(13) NUMBITS(1) [],
        GPIO46 OFFSET(14) NUMBITS(1) [],
        GPIO47 OFFSET(15) NUMBITS(1) [],
        GPIO48 OFFSET(16) NUMBITS(1) [],
        GPIO49 OFFSET(17) NUMBITS(1) []
    ]
];

pub struct GpioPin<'a> {
    registers: StaticRef<GpioRegisters>,
    pin: Pin,
    client: OptionalCell<&'a dyn gpio::Client>,
}

impl<'a> GpioPin<'a> {
    pub const fn new(base: StaticRef<GpioRegisters>, pin: Pin) -> GpioPin<'a> {
        GpioPin {
            registers: base,
            pin,
            client: OptionalCell::empty(),
        }
    }

    pub fn handle_interrupt(&self) {
        unimplemented!();
    }
}

impl<'a> gpio::Configure for GpioPin<'a> {
    fn configuration(&self) -> gpio::Configuration {
        unimplemented!();
    }

    fn set_floating_state(&self, _mode: gpio::FloatingState) {
        unimplemented!();
    }

    fn floating_state(&self) -> gpio::FloatingState {
        unimplemented!();
    }

    fn deactivate_to_low_power(&self) {
        self.disable_input();
        self.disable_output();
    }

    fn make_output(&self) -> gpio::Configuration {
        let regs = self.registers;

        // Set the key
        regs.padkey.set(115);

        // Configure the pin as GPIO
        let pagreg_offset = self.pin as usize / 4;
        let pagreg_value = match self.pin as usize % 4 {
            0 => PADREG::PAD0FNCSEL.val(0x3),
            1 => PADREG::PAD1FNCSEL.val(0x3),
            2 => PADREG::PAD2FNCSEL.val(0x3),
            3 => PADREG::PAD3FNCSEL.val(0x3),
            _ => unreachable!(),
        };
        regs.padreg[pagreg_offset].modify(pagreg_value);

        // Set to push/pull
        let cfgreg_offset = self.pin as usize / 8;
        let cfgreg_value = match self.pin as usize % 8 {
            0 => CFG::GPIO0INTD::CLEAR + CFG::GPIO0OUTCFG.val(0x1),
            1 => CFG::GPIO1INTD::CLEAR + CFG::GPIO1OUTCFG.val(0x1),
            2 => CFG::GPIO2INTD::CLEAR + CFG::GPIO2OUTCFG.val(0x1),
            3 => CFG::GPIO3INTD::CLEAR + CFG::GPIO3OUTCFG.val(0x1),
            4 => CFG::GPIO4INTD::CLEAR + CFG::GPIO4OUTCFG.val(0x1),
            5 => CFG::GPIO5INTD::CLEAR + CFG::GPIO5OUTCFG.val(0x1),
            6 => CFG::GPIO6INTD::CLEAR + CFG::GPIO6OUTCFG.val(0x1),
            7 => CFG::GPIO7INTD::CLEAR + CFG::GPIO7OUTCFG.val(0x1),
            _ => unreachable!(),
        };
        regs.cfg[cfgreg_offset].modify(cfgreg_value);

        // Unset key
        regs.padkey.set(0x00);

        gpio::Configuration::Output
    }

    fn disable_output(&self) -> gpio::Configuration {
        unimplemented!();
    }

    fn make_input(&self) -> gpio::Configuration {
        unimplemented!();
    }

    fn disable_input(&self) -> gpio::Configuration {
        unimplemented!();
    }
}

impl<'a> gpio::Input for GpioPin<'a> {
    fn read(&self) -> bool {
        let regs = self.registers;

        if (self.pin as usize) < 32 {
            regs.rda.get() & (1 << self.pin as usize) != 0
        } else {
            regs.rdb.get() & (1 << (self.pin as usize - 32)) != 0
        }
    }
}

impl<'a> gpio::Output for GpioPin<'a> {
    fn toggle(&self) -> bool {
        let regs = self.registers;
        let cur_value;

        if (self.pin as usize) < 32 {
            cur_value = (regs.wtsa.get() & 1 << self.pin as usize) != 0;
            if cur_value {
                regs.wta.set(1 << self.pin as usize | regs.wtsa.get());
            } else {
                regs.wta.set(0 << self.pin as usize | regs.wtsa.get());
            }
        } else {
            cur_value = (regs.wtsb.get() & 1 << self.pin as usize) != 0;
            if cur_value {
                regs.wtb.set(1 << self.pin as usize - 32 | regs.wtsb.get());
            } else {
                regs.wtb.set(0 << self.pin as usize - 32 | regs.wtsb.get());
            }
        }

        cur_value
    }

    fn set(&self) {
        let regs = self.registers;

        if (self.pin as usize) < 32 {
            regs.wtsa.set(1 << self.pin as usize);
        } else {
            regs.wtsb.set(1 << (self.pin as usize - 32));
        }
    }

    fn clear(&self) {
        let regs = self.registers;

        if (self.pin as usize) < 32 {
            regs.wtca.set(1 << self.pin as usize);
        } else {
            regs.wtcb.set(1 << (self.pin as usize - 32));
        }
    }
}

impl<'a> gpio::Interrupt<'a> for GpioPin<'a> {
    fn set_client(&self, client: &'a dyn gpio::Client) {
        self.client.set(client);
    }

    fn enable_interrupts(&self, mode: gpio::InterruptEdge) {
        let regs = self.registers;

        // Set the key
        regs.padkey.set(115);

        // Configure the pin as GPIO
        let pagreg_offset = self.pin as usize / 4;
        let pagreg_value = match self.pin as usize % 4 {
            0 => PADREG::PAD0FNCSEL.val(0x3),
            1 => PADREG::PAD1FNCSEL.val(0x3),
            2 => PADREG::PAD2FNCSEL.val(0x3),
            3 => PADREG::PAD2FNCSEL.val(0x3),
            _ => unreachable!(),
        };
        regs.padreg[pagreg_offset].modify(pagreg_value);

        // Set the edge mode
        let cfgreg_offset = self.pin as usize / 8;
        match mode {
            gpio::InterruptEdge::RisingEdge => {
                let cfgreg_value = match self.pin as usize % 8 {
                    0 => CFG::GPIO0INTD::CLEAR + CFG::GPIO0INCFG::CLEAR,
                    1 => CFG::GPIO1INTD::CLEAR + CFG::GPIO1INCFG::CLEAR,
                    2 => CFG::GPIO2INTD::CLEAR + CFG::GPIO2INCFG::CLEAR,
                    3 => CFG::GPIO3INTD::CLEAR + CFG::GPIO3INCFG::CLEAR,
                    4 => CFG::GPIO4INTD::CLEAR + CFG::GPIO4INCFG::CLEAR,
                    5 => CFG::GPIO5INTD::CLEAR + CFG::GPIO5INCFG::CLEAR,
                    6 => CFG::GPIO6INTD::CLEAR + CFG::GPIO6INCFG::CLEAR,
                    7 => CFG::GPIO7INTD::CLEAR + CFG::GPIO7INCFG::CLEAR,
                    _ => unreachable!(),
                };
                regs.cfg[cfgreg_offset].modify(cfgreg_value);
            }
            gpio::InterruptEdge::FallingEdge => {
                let cfgreg_value = match self.pin as usize % 8 {
                    0 => CFG::GPIO0INTD::SET + CFG::GPIO0INCFG::CLEAR,
                    1 => CFG::GPIO1INTD::SET + CFG::GPIO1INCFG::CLEAR,
                    2 => CFG::GPIO2INTD::SET + CFG::GPIO2INCFG::CLEAR,
                    3 => CFG::GPIO3INTD::SET + CFG::GPIO3INCFG::CLEAR,
                    4 => CFG::GPIO4INTD::SET + CFG::GPIO4INCFG::CLEAR,
                    5 => CFG::GPIO5INTD::SET + CFG::GPIO5INCFG::CLEAR,
                    6 => CFG::GPIO6INTD::SET + CFG::GPIO6INCFG::CLEAR,
                    7 => CFG::GPIO7INTD::SET + CFG::GPIO7INCFG::CLEAR,
                    _ => unreachable!(),
                };
                regs.cfg[cfgreg_offset].modify(cfgreg_value);
            }
            gpio::InterruptEdge::EitherEdge => {
                let cfgreg_value = match self.pin as usize % 8 {
                    0 => CFG::GPIO0INTD::SET + CFG::GPIO0INCFG::SET,
                    1 => CFG::GPIO1INTD::SET + CFG::GPIO1INCFG::SET,
                    2 => CFG::GPIO2INTD::SET + CFG::GPIO2INCFG::SET,
                    3 => CFG::GPIO3INTD::SET + CFG::GPIO3INCFG::SET,
                    4 => CFG::GPIO4INTD::SET + CFG::GPIO4INCFG::SET,
                    5 => CFG::GPIO5INTD::SET + CFG::GPIO5INCFG::SET,
                    6 => CFG::GPIO6INTD::SET + CFG::GPIO6INCFG::SET,
                    7 => CFG::GPIO7INTD::SET + CFG::GPIO7INCFG::SET,
                    _ => unreachable!(),
                };
                regs.cfg[cfgreg_offset].modify(cfgreg_value);
            }
        }

        // Enable interrupts
        if (self.pin as usize) < 32 {
            regs.int0en.set(1 << self.pin as usize | regs.int0en.get());
        } else {
            regs.int1en
                .set(1 << (self.pin as usize - 32) | regs.int1en.get());
        }

        // Unset key
        regs.padkey.set(0x00);
    }

    fn disable_interrupts(&self) {
        let regs = self.registers;

        // Disable interrupt
        if (self.pin as usize) < 32 {
            regs.int0en
                .set(!(1 << self.pin as usize) & regs.int0en.get());
        } else {
            regs.int1en
                .set(!(1 << (self.pin as usize - 32)) & regs.int0en.get());
        }

        // Clear interrupt
        if (self.pin as usize) < 32 {
            regs.int0clr.set(1 << self.pin as usize);
        } else {
            regs.int1clr.set(1 << (self.pin as usize - 32));
        }
    }

    fn is_pending(&self) -> bool {
        let regs = self.registers;

        regs.int0stat.get() | regs.int1stat.get() != 0
    }
}

impl<'a> gpio::Pin for GpioPin<'a> {}
impl<'a> gpio::InterruptPin<'a> for GpioPin<'a> {}
