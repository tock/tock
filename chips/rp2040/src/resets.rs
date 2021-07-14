use kernel::utilities::registers::interfaces::{ReadWriteable, Readable, Writeable};
use kernel::utilities::registers::{register_bitfields, register_structs, FieldValue, ReadWrite};
use kernel::utilities::StaticRef;

register_structs! {
    ResetsRegisters {
        /// Reset control. If a bit is set it means the peripheral is in reset. 0 means the
        (0x000 => reset: ReadWrite<u32, RESET::Register>),
        /// Watchdog select. If a bit is set then the watchdog will reset this peripheral wh
        (0x004 => wdsel: ReadWrite<u32, WDSEL::Register>),
        /// Reset done. If a bit is set then a reset done signal has been returned by the pe
        (0x008 => reset_done: ReadWrite<u32, RESET_DONE::Register>),
        (0x00C => @END),
    }
}
register_bitfields![u32,
    RESET [

        usbctrl OFFSET(24) NUMBITS(1) [],

        uart1 OFFSET(23) NUMBITS(1) [],

        uart0 OFFSET(22) NUMBITS(1) [],

        timer OFFSET(21) NUMBITS(1) [],

        tbman OFFSET(20) NUMBITS(1) [],

        sysinfo OFFSET(19) NUMBITS(1) [],

        syscfg OFFSET(18) NUMBITS(1) [],

        spi1 OFFSET(17) NUMBITS(1) [],

        spi0 OFFSET(16) NUMBITS(1) [],

        rtc OFFSET(15) NUMBITS(1) [],

        pwm OFFSET(14) NUMBITS(1) [],

        pll_usb OFFSET(13) NUMBITS(1) [],

        pll_sys OFFSET(12) NUMBITS(1) [],

        pio1 OFFSET(11) NUMBITS(1) [],

        pio0 OFFSET(10) NUMBITS(1) [],

        pads_qspi OFFSET(9) NUMBITS(1) [],

        pads_bank0 OFFSET(8) NUMBITS(1) [],

        jtag OFFSET(7) NUMBITS(1) [],

        io_qspi OFFSET(6) NUMBITS(1) [],

        io_bank0 OFFSET(5) NUMBITS(1) [],

        i2c1 OFFSET(4) NUMBITS(1) [],

        i2c0 OFFSET(3) NUMBITS(1) [],

        dma OFFSET(2) NUMBITS(1) [],

        busctrl OFFSET(1) NUMBITS(1) [],

        adc OFFSET(0) NUMBITS(1) []
    ],
    WDSEL [

        usbctrl OFFSET(24) NUMBITS(1) [],

        uart1 OFFSET(23) NUMBITS(1) [],

        uart0 OFFSET(22) NUMBITS(1) [],

        timer OFFSET(21) NUMBITS(1) [],

        tbman OFFSET(20) NUMBITS(1) [],

        sysinfo OFFSET(19) NUMBITS(1) [],

        syscfg OFFSET(18) NUMBITS(1) [],

        spi1 OFFSET(17) NUMBITS(1) [],

        spi0 OFFSET(16) NUMBITS(1) [],

        rtc OFFSET(15) NUMBITS(1) [],

        pwm OFFSET(14) NUMBITS(1) [],

        pll_usb OFFSET(13) NUMBITS(1) [],

        pll_sys OFFSET(12) NUMBITS(1) [],

        pio1 OFFSET(11) NUMBITS(1) [],

        pio0 OFFSET(10) NUMBITS(1) [],

        pads_qspi OFFSET(9) NUMBITS(1) [],

        pads_bank0 OFFSET(8) NUMBITS(1) [],

        jtag OFFSET(7) NUMBITS(1) [],

        io_qspi OFFSET(6) NUMBITS(1) [],

        io_bank0 OFFSET(5) NUMBITS(1) [],

        i2c1 OFFSET(4) NUMBITS(1) [],

        i2c0 OFFSET(3) NUMBITS(1) [],

        dma OFFSET(2) NUMBITS(1) [],

        busctrl OFFSET(1) NUMBITS(1) [],

        adc OFFSET(0) NUMBITS(1) []
    ],
    RESET_DONE [

        usbctrl OFFSET(24) NUMBITS(1) [],

        uart1 OFFSET(23) NUMBITS(1) [],

        uart0 OFFSET(22) NUMBITS(1) [],

        timer OFFSET(21) NUMBITS(1) [],

        tbman OFFSET(20) NUMBITS(1) [],

        sysinfo OFFSET(19) NUMBITS(1) [],

        syscfg OFFSET(18) NUMBITS(1) [],

        spi1 OFFSET(17) NUMBITS(1) [],

        spi0 OFFSET(16) NUMBITS(1) [],

        rtc OFFSET(15) NUMBITS(1) [],

        pwm OFFSET(14) NUMBITS(1) [],

        pll_usb OFFSET(13) NUMBITS(1) [],

        pll_sys OFFSET(12) NUMBITS(1) [],

        pio1 OFFSET(11) NUMBITS(1) [],

        pio0 OFFSET(10) NUMBITS(1) [],

        pads_qspi OFFSET(9) NUMBITS(1) [],

        pads_bank0 OFFSET(8) NUMBITS(1) [],

        jtag OFFSET(7) NUMBITS(1) [],

        io_qspi OFFSET(6) NUMBITS(1) [],

        io_bank0 OFFSET(5) NUMBITS(1) [],

        i2c1 OFFSET(4) NUMBITS(1) [],

        i2c0 OFFSET(3) NUMBITS(1) [],

        dma OFFSET(2) NUMBITS(1) [],

        busctrl OFFSET(1) NUMBITS(1) [],

        adc OFFSET(0) NUMBITS(1) []
    ]
];
const RESETS_BASE: StaticRef<ResetsRegisters> =
    unsafe { StaticRef::new(0x4000C000 as *const ResetsRegisters) };

pub enum Peripheral {
    Adc,
    BusController,
    Dma,
    I2c0,
    I2c1,
    IOBank0,
    IOQSpi,
    Jtag,
    PadsBank0,
    PadsQSpi,
    Pio0,
    Pio1,
    PllSys,
    PllUsb,
    Pwm,
    Rtc,
    Spi0,
    Spi1,
    Syscfg,
    SysInfo,
    TBMan,
    Timer,
    Uart0,
    Uart1,
    UsbCtrl,
}

impl Peripheral {
    fn get_reset_field_set(&self) -> FieldValue<u32, RESET::Register> {
        match self {
            Peripheral::Adc => RESET::adc::SET,
            Peripheral::BusController => RESET::busctrl::SET,
            Peripheral::Dma => RESET::dma::SET,
            Peripheral::I2c0 => RESET::i2c0::SET,
            Peripheral::I2c1 => RESET::i2c1::SET,
            Peripheral::IOBank0 => RESET::io_bank0::SET,
            Peripheral::IOQSpi => RESET::io_qspi::SET,
            Peripheral::Jtag => RESET::jtag::SET,
            Peripheral::PadsBank0 => RESET::pads_bank0::SET,
            Peripheral::PadsQSpi => RESET::pads_qspi::SET,
            Peripheral::Pio0 => RESET::pio0::SET,
            Peripheral::Pio1 => RESET::pio1::SET,
            Peripheral::PllSys => RESET::pll_sys::SET,
            Peripheral::PllUsb => RESET::pll_usb::SET,
            Peripheral::Pwm => RESET::pwm::SET,
            Peripheral::Rtc => RESET::rtc::SET,
            Peripheral::Spi0 => RESET::spi0::SET,
            Peripheral::Spi1 => RESET::spi1::SET,
            Peripheral::Syscfg => RESET::syscfg::SET,
            Peripheral::SysInfo => RESET::sysinfo::SET,
            Peripheral::TBMan => RESET::tbman::SET,
            Peripheral::Timer => RESET::timer::SET,
            Peripheral::Uart0 => RESET::uart0::SET,
            Peripheral::Uart1 => RESET::uart1::SET,
            Peripheral::UsbCtrl => RESET::usbctrl::SET,
        }
    }

    fn get_reset_field_clear(&self) -> FieldValue<u32, RESET::Register> {
        match self {
            Peripheral::Adc => RESET::adc::CLEAR,
            Peripheral::BusController => RESET::busctrl::CLEAR,
            Peripheral::Dma => RESET::dma::CLEAR,
            Peripheral::I2c0 => RESET::i2c0::CLEAR,
            Peripheral::I2c1 => RESET::i2c1::CLEAR,
            Peripheral::IOBank0 => RESET::io_bank0::CLEAR,
            Peripheral::IOQSpi => RESET::io_qspi::CLEAR,
            Peripheral::Jtag => RESET::jtag::CLEAR,
            Peripheral::PadsBank0 => RESET::pads_bank0::CLEAR,
            Peripheral::PadsQSpi => RESET::pads_qspi::CLEAR,
            Peripheral::Pio0 => RESET::pio0::CLEAR,
            Peripheral::Pio1 => RESET::pio1::CLEAR,
            Peripheral::PllSys => RESET::pll_sys::CLEAR,
            Peripheral::PllUsb => RESET::pll_usb::CLEAR,
            Peripheral::Pwm => RESET::pwm::CLEAR,
            Peripheral::Rtc => RESET::rtc::CLEAR,
            Peripheral::Spi0 => RESET::spi0::CLEAR,
            Peripheral::Spi1 => RESET::spi1::CLEAR,
            Peripheral::Syscfg => RESET::syscfg::CLEAR,
            Peripheral::SysInfo => RESET::sysinfo::CLEAR,
            Peripheral::TBMan => RESET::tbman::CLEAR,
            Peripheral::Timer => RESET::timer::CLEAR,
            Peripheral::Uart0 => RESET::uart0::CLEAR,
            Peripheral::Uart1 => RESET::uart1::CLEAR,
            Peripheral::UsbCtrl => RESET::usbctrl::CLEAR,
        }
    }

    fn get_reset_done_field_set(&self) -> FieldValue<u32, RESET_DONE::Register> {
        match self {
            Peripheral::Adc => RESET_DONE::adc::SET,
            Peripheral::BusController => RESET_DONE::busctrl::SET,
            Peripheral::Dma => RESET_DONE::dma::SET,
            Peripheral::I2c0 => RESET_DONE::i2c0::SET,
            Peripheral::I2c1 => RESET_DONE::i2c1::SET,
            Peripheral::IOBank0 => RESET_DONE::io_bank0::SET,
            Peripheral::IOQSpi => RESET_DONE::io_qspi::SET,
            Peripheral::Jtag => RESET_DONE::jtag::SET,
            Peripheral::PadsBank0 => RESET_DONE::pads_bank0::SET,
            Peripheral::PadsQSpi => RESET_DONE::pads_qspi::SET,
            Peripheral::Pio0 => RESET_DONE::pio0::SET,
            Peripheral::Pio1 => RESET_DONE::pio1::SET,
            Peripheral::PllSys => RESET_DONE::pll_sys::SET,
            Peripheral::PllUsb => RESET_DONE::pll_usb::SET,
            Peripheral::Pwm => RESET_DONE::pwm::SET,
            Peripheral::Rtc => RESET_DONE::rtc::SET,
            Peripheral::Spi0 => RESET_DONE::spi0::SET,
            Peripheral::Spi1 => RESET_DONE::spi1::SET,
            Peripheral::Syscfg => RESET_DONE::syscfg::SET,
            Peripheral::SysInfo => RESET_DONE::sysinfo::SET,
            Peripheral::TBMan => RESET_DONE::tbman::SET,
            Peripheral::Timer => RESET_DONE::timer::SET,
            Peripheral::Uart0 => RESET_DONE::uart0::SET,
            Peripheral::Uart1 => RESET_DONE::uart1::SET,
            Peripheral::UsbCtrl => RESET_DONE::usbctrl::SET,
        }
    }
}

pub struct Resets {
    registers: StaticRef<ResetsRegisters>,
}

impl Resets {
    pub const fn new() -> Resets {
        Resets {
            registers: RESETS_BASE,
        }
    }

    pub fn reset(&self, peripherals: &'static [Peripheral]) {
        if peripherals.len() > 0 {
            let mut value: FieldValue<u32, RESET::Register> = peripherals[0].get_reset_field_set();
            for peripheral in peripherals {
                value = value + peripheral.get_reset_field_set();
            }
            self.registers.reset.modify(value);
        }
    }

    pub fn unreset(&self, peripherals: &'static [Peripheral], wait_for: bool) {
        if peripherals.len() > 0 {
            let mut value: FieldValue<u32, RESET::Register> =
                peripherals[0].get_reset_field_clear();
            for peripheral in peripherals {
                value = value + peripheral.get_reset_field_clear();
            }
            self.registers.reset.modify(value);

            if wait_for {
                let mut value_done: FieldValue<u32, RESET_DONE::Register> =
                    peripherals[0].get_reset_done_field_set();
                for peripheral in peripherals {
                    value_done = value_done + peripheral.get_reset_done_field_set();
                }
                while !self.registers.reset_done.matches_all(value_done) {}
            }
        }
    }

    pub fn reset_all_except(&self, peripherals: &'static [Peripheral]) {
        let mut value = 0xFFFFFF;
        for peripheral in peripherals {
            value ^= peripheral.get_reset_field_set().value;
        }
        self.registers.reset.set(value);
    }

    pub fn unreset_all_except(&self, peripherals: &'static [Peripheral], wait_for: bool) {
        let mut value = 0;
        for peripheral in peripherals {
            value |= peripheral.get_reset_field_set().value;
        }

        self.registers.reset.set(value);

        if wait_for {
            value = !value & 0xFFFFF;
            while (self.registers.reset_done.get() & value) != value {}
        }
    }
}
