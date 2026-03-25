use crate::gpio_registers::*;
use crate::hsiom_registers::*;
use kernel::hil::gpio::{Configuration, Configure, Input, Interrupt, Output};
use kernel::utilities::cells::OptionalCell;
use kernel::utilities::registers::interfaces::ReadWriteable;
use kernel::utilities::registers::interfaces::Readable;
use kernel::utilities::registers::interfaces::Writeable;
use kernel::utilities::StaticRef;

const GPIO_BASE: StaticRef<GpioRegisters> =
    unsafe { StaticRef::new(0x42410000 as *const GpioRegisters) };
const HSIOM_BASE: StaticRef<HsiomRegisters> =
    unsafe { StaticRef::new(0x52400000 as *const HsiomRegisters) };

const HIGHZ: u32 = 0;
const PULL_UP: u32 = 2;
const PULL_DOWN: u32 = 3;
const GPIO_HALF: usize = 4;
const HSIOM_SEC_MASK: u32 = 0x1;

#[derive(Clone, Copy, Debug)]
pub enum PsocPin {
    P0_0 = 0,
    P0_1 = 1,
    P0_2 = 2,
    P0_3 = 3,
    P0_4 = 4,
    P0_5 = 5,
    P1_0 = 8,
    P1_1 = 9,
    P1_2 = 10,
    P1_3 = 11,
    P1_4 = 12,
    P1_5 = 13,
    P2_0 = 16,
    P2_1 = 17,
    P2_2 = 18,
    P2_3 = 19,
    P2_4 = 20,
    P2_5 = 21,
    P2_6 = 22,
    P2_7 = 23,
    P3_0 = 24,
    P3_1 = 25,
    P3_2 = 26,
    P3_3 = 27,
    P3_4 = 28,
    P3_5 = 29,
    P4_0 = 32,
    P4_1 = 33,
    P4_2 = 34,
    P4_3 = 35,
    P5_0 = 40,
    P5_1 = 41,
    P5_2 = 42,
    P5_3 = 43,
    P5_4 = 44,
    P5_5 = 45,
    P5_6 = 46,
    P5_7 = 47,
    P6_0 = 48,
    P6_1 = 49,
    P6_2 = 50,
    P6_3 = 51,
    P6_4 = 52,
    P6_5 = 53,
    P6_6 = 54,
    P6_7 = 55,
    P7_0 = 56,
    P7_1 = 57,
    P7_2 = 58,
    P7_3 = 59,
    P7_4 = 60,
    P7_5 = 61,
    P7_6 = 62,
    P7_7 = 63,
    P8_0 = 64,
    P8_1 = 65,
    P8_2 = 66,
    P8_3 = 67,
    P8_4 = 68,
    P8_5 = 69,
    P8_6 = 70,
    P8_7 = 71,
    P9_0 = 72,
    P9_1 = 73,
    P9_2 = 74,
    P9_3 = 75,
    P9_4 = 76,
    P9_5 = 77,
    P9_6 = 78,
    P9_7 = 79,
    P10_0 = 80,
    P10_1 = 81,
    P10_2 = 82,
    P10_3 = 83,
    P10_4 = 84,
    P10_5 = 85,
    P10_6 = 86,
    P10_7 = 87,
    P11_0 = 88,
    P11_1 = 89,
    P11_2 = 90,
    P11_3 = 91,
    P11_4 = 92,
    P11_5 = 93,
    P11_6 = 94,
    P11_7 = 95,
    P12_0 = 96,
    P12_1 = 97,
    P12_2 = 98,
    P12_3 = 99,
    P12_4 = 100,
    P12_5 = 101,
    P12_6 = 102,
    P12_7 = 103,
    P13_0 = 104,
    P13_1 = 105,
    P13_2 = 106,
    P13_3 = 107,
    P13_4 = 108,
    P13_5 = 109,
    P13_6 = 110,
    P13_7 = 111,
}

pub struct PsocPins<'a> {
    pub pins: [Option<GpioPin<'a>>; 112],
}

impl<'a> PsocPins<'a> {
    pub const fn new() -> Self {
        Self {
            pins: [
                Some(GpioPin::new(PsocPin::P0_0)),
                Some(GpioPin::new(PsocPin::P0_1)),
                Some(GpioPin::new(PsocPin::P0_2)),
                Some(GpioPin::new(PsocPin::P0_3)),
                Some(GpioPin::new(PsocPin::P0_4)),
                Some(GpioPin::new(PsocPin::P0_5)),
                None,
                None,
                Some(GpioPin::new(PsocPin::P1_0)),
                Some(GpioPin::new(PsocPin::P1_1)),
                Some(GpioPin::new(PsocPin::P1_2)),
                Some(GpioPin::new(PsocPin::P1_3)),
                Some(GpioPin::new(PsocPin::P1_4)),
                Some(GpioPin::new(PsocPin::P1_5)),
                None,
                None,
                Some(GpioPin::new(PsocPin::P2_0)),
                Some(GpioPin::new(PsocPin::P2_1)),
                Some(GpioPin::new(PsocPin::P2_2)),
                Some(GpioPin::new(PsocPin::P2_3)),
                Some(GpioPin::new(PsocPin::P2_4)),
                Some(GpioPin::new(PsocPin::P2_5)),
                Some(GpioPin::new(PsocPin::P2_6)),
                Some(GpioPin::new(PsocPin::P2_7)),
                Some(GpioPin::new(PsocPin::P3_0)),
                Some(GpioPin::new(PsocPin::P3_1)),
                Some(GpioPin::new(PsocPin::P3_2)),
                Some(GpioPin::new(PsocPin::P3_3)),
                Some(GpioPin::new(PsocPin::P3_4)),
                Some(GpioPin::new(PsocPin::P3_5)),
                None,
                None,
                Some(GpioPin::new(PsocPin::P4_0)),
                Some(GpioPin::new(PsocPin::P4_1)),
                Some(GpioPin::new(PsocPin::P4_2)),
                Some(GpioPin::new(PsocPin::P4_3)),
                None,
                None,
                None,
                None,
                Some(GpioPin::new(PsocPin::P5_0)),
                Some(GpioPin::new(PsocPin::P5_1)),
                Some(GpioPin::new(PsocPin::P5_2)),
                Some(GpioPin::new(PsocPin::P5_3)),
                Some(GpioPin::new(PsocPin::P5_4)),
                Some(GpioPin::new(PsocPin::P5_5)),
                Some(GpioPin::new(PsocPin::P5_6)),
                Some(GpioPin::new(PsocPin::P5_7)),
                Some(GpioPin::new(PsocPin::P6_0)),
                Some(GpioPin::new(PsocPin::P6_1)),
                Some(GpioPin::new(PsocPin::P6_2)),
                Some(GpioPin::new(PsocPin::P6_3)),
                Some(GpioPin::new(PsocPin::P6_4)),
                Some(GpioPin::new(PsocPin::P6_5)),
                Some(GpioPin::new(PsocPin::P6_6)),
                Some(GpioPin::new(PsocPin::P6_7)),
                Some(GpioPin::new(PsocPin::P7_0)),
                Some(GpioPin::new(PsocPin::P7_1)),
                Some(GpioPin::new(PsocPin::P7_2)),
                Some(GpioPin::new(PsocPin::P7_3)),
                Some(GpioPin::new(PsocPin::P7_4)),
                Some(GpioPin::new(PsocPin::P7_5)),
                Some(GpioPin::new(PsocPin::P7_6)),
                Some(GpioPin::new(PsocPin::P7_7)),
                Some(GpioPin::new(PsocPin::P8_0)),
                Some(GpioPin::new(PsocPin::P8_1)),
                Some(GpioPin::new(PsocPin::P8_2)),
                Some(GpioPin::new(PsocPin::P8_3)),
                Some(GpioPin::new(PsocPin::P8_4)),
                Some(GpioPin::new(PsocPin::P8_5)),
                Some(GpioPin::new(PsocPin::P8_6)),
                Some(GpioPin::new(PsocPin::P8_7)),
                Some(GpioPin::new(PsocPin::P9_0)),
                Some(GpioPin::new(PsocPin::P9_1)),
                Some(GpioPin::new(PsocPin::P9_2)),
                Some(GpioPin::new(PsocPin::P9_3)),
                Some(GpioPin::new(PsocPin::P9_4)),
                Some(GpioPin::new(PsocPin::P9_5)),
                Some(GpioPin::new(PsocPin::P9_6)),
                Some(GpioPin::new(PsocPin::P9_7)),
                Some(GpioPin::new(PsocPin::P10_0)),
                Some(GpioPin::new(PsocPin::P10_1)),
                Some(GpioPin::new(PsocPin::P10_2)),
                Some(GpioPin::new(PsocPin::P10_3)),
                Some(GpioPin::new(PsocPin::P10_4)),
                Some(GpioPin::new(PsocPin::P10_5)),
                Some(GpioPin::new(PsocPin::P10_6)),
                Some(GpioPin::new(PsocPin::P10_7)),
                Some(GpioPin::new(PsocPin::P11_0)),
                Some(GpioPin::new(PsocPin::P11_1)),
                Some(GpioPin::new(PsocPin::P11_2)),
                Some(GpioPin::new(PsocPin::P11_3)),
                Some(GpioPin::new(PsocPin::P11_4)),
                Some(GpioPin::new(PsocPin::P11_5)),
                Some(GpioPin::new(PsocPin::P11_6)),
                Some(GpioPin::new(PsocPin::P11_7)),
                Some(GpioPin::new(PsocPin::P12_0)),
                Some(GpioPin::new(PsocPin::P12_1)),
                Some(GpioPin::new(PsocPin::P12_2)),
                Some(GpioPin::new(PsocPin::P12_3)),
                Some(GpioPin::new(PsocPin::P12_4)),
                Some(GpioPin::new(PsocPin::P12_5)),
                Some(GpioPin::new(PsocPin::P12_6)),
                Some(GpioPin::new(PsocPin::P12_7)),
                Some(GpioPin::new(PsocPin::P13_0)),
                Some(GpioPin::new(PsocPin::P13_1)),
                Some(GpioPin::new(PsocPin::P13_2)),
                Some(GpioPin::new(PsocPin::P13_3)),
                Some(GpioPin::new(PsocPin::P13_4)),
                Some(GpioPin::new(PsocPin::P13_5)),
                Some(GpioPin::new(PsocPin::P13_6)),
                Some(GpioPin::new(PsocPin::P13_7)),
            ],
        }
    }

    pub fn get_pin(&self, searched_pin: PsocPin) -> &GpioPin<'a> {
        self.pins[searched_pin as usize].as_ref().unwrap()
    }

    pub fn handle_interrupt(&self) {
        for pin in self.pins.iter() {
            pin.as_ref().inspect(|pin| pin.handle_interrupt());
        }
    }
}

#[derive(Clone, Copy)]
pub enum DriveMode {
    HighZ = 0,
    // Reserved = 1,
    PullUp = 2,
    PullDown = 3,
    OpenDrainLow = 4,
    OpenDrainHigh = 5,
    Strong = 6,
    PullUpDown = 7,
}

pub struct GpioPin<'a> {
    registers: StaticRef<GpioRegisters>,
    hsiom_registers: StaticRef<HsiomRegisters>,
    pin: usize,
    port: usize,

    client: OptionalCell<&'a dyn kernel::hil::gpio::Client>,
}

#[derive(Clone, Copy)]
pub enum DriveSelect {
    /// Full drive strength: Max drive current
    Full = 0,
    /// 1/2 drive strength: 1/2 drive current
    Half = 1,
    /// 1/4 drive strength: 1/4 drive current
    Quarter = 2,
    /// 1/8 drive strength: 1/8 drive current
    Eighth = 3,
}

pub struct PreConfig {
    /// Pin output state
    pub out_val: u32,
    /// Drive mode
    pub drive_mode: DriveMode,
    /// HSIOM selection
    pub hsiom: HsiomFunction,
    /// Interrupt edge type
    pub int_edge: bool,
    /// Interrupt enable mask
    pub int_mask: u32,
    /// Input buffer voltage trip type
    pub vtrip: u32,
    /// Output buffer slew rate
    pub fast_slew_rate: bool,
    /// Drive strength
    pub drive_sel: DriveSelect,
    /// SIO pair output buffer mode
    pub vreg_en: bool,
    /// SIO pair input buffer mode
    pub ibuf_mode: u32,
    /// SIO pair input buffer trip point
    pub vtrip_sel: u32,
    /// SIO pair reference voltage for input buffer trip point
    pub vref_sel: u32,
    /// SIO pair regulated voltage output level
    pub voh_sel: u32,
    /// Secure attribute for each pin of a port
    pub non_sec: bool,
}

impl GpioPin<'_> {
    pub const fn new(id: PsocPin) -> Self {
        Self {
            registers: GPIO_BASE,
            hsiom_registers: HSIOM_BASE,
            pin: (id as usize) % 8,
            port: (id as usize) / 8,
            client: OptionalCell::empty(),
        }
    }

    pub fn preconfigure(&self, preconfig: &PreConfig) {
        self.set_secure_port_nonsecure_pin(false);

        self.set_slew_rate(preconfig.fast_slew_rate);
        self.set_drive_sel(preconfig.drive_sel);
        self.set_hsiom_function(preconfig.hsiom);
        self.configure_drive_mode(preconfig.drive_mode);
        self.set_interrupt_edge(preconfig.int_edge);
        self.set_interrupt_mask(preconfig.int_mask);
        self.set_vtrip(preconfig.vtrip);
        self.set_sio_config(
            preconfig.vreg_en,
            preconfig.ibuf_mode,
            preconfig.vtrip_sel,
            preconfig.vref_sel,
            preconfig.voh_sel,
        );
        self.write_output_raw(preconfig.out_val);

        self.set_secure_port_nonsecure_pin(preconfig.non_sec);
    }

    fn replace_field(value: u32, offset: u32, width: u32, new_field_value: u32) -> u32 {
        let field_mask = ((1u32 << width) - 1) << offset;
        (value & !field_mask) | ((new_field_value << offset) & field_mask)
    }

    fn replace_pin_field(value: u32, pin: usize, width: u32, new_field_value: u32) -> u32 {
        Self::replace_field(value, (pin as u32) * width, width, new_field_value)
    }

    fn set_slew_rate(&self, fast_slew_rate: bool) {
        let register = &self.registers.ports[self.port].prt_slew_ext;
        let old_value = register.get();
        register.set(Self::replace_pin_field(
            old_value,
            self.pin,
            1,
            fast_slew_rate as u32,
        ));
    }

    fn set_drive_sel(&self, drive_sel: DriveSelect) {
        let port_addr = &self.registers.ports[self.port];
        let local_pin = if self.pin < GPIO_HALF {
            self.pin
        } else {
            self.pin - GPIO_HALF
        };

        let bit_offset = (local_pin * 8) as u32;
        let mask = 0x1F_u32 << bit_offset;

        let register = if self.pin < GPIO_HALF {
            &port_addr.prt_drive_ext0
        } else {
            &port_addr.prt_drive_ext1
        };
        let drive_sel_value = (drive_sel as u32) << bit_offset;

        let old_value = register.get();
        register.set((old_value & !mask) | (drive_sel_value & mask));
    }

    fn set_interrupt_edge(&self, edge: bool) {
        let register = &self.registers.ports[self.port].prt_intr_cfg;
        let old_value = register.get();
        register.set(Self::replace_pin_field(old_value, self.pin, 2, edge as u32));
    }

    fn set_interrupt_mask(&self, mask: u32) {
        let register = &self.registers.ports[self.port].prt_intr_mask;
        let old_value = register.get();
        register.set(Self::replace_pin_field(old_value, self.pin, 1, mask));
    }

    fn set_vtrip(&self, vtrip: u32) {
        let register = &self.registers.ports[self.port].prt_cfg_in;
        let old_value = register.get();
        register.set(Self::replace_pin_field(old_value, self.pin, 1, vtrip));
    }

    fn set_sio_config(
        &self,
        vreg_en: bool,
        ibuf_mode: u32,
        vtrip_sel: u32,
        vref_sel: u32,
        voh_sel: u32,
    ) {
        let register = &self.registers.ports[self.port].prt_cfg_sio;

        let pin_shift = ((self.pin & 0x1) as u32) << 3;
        let pin_mask = 0xFF_u32 << pin_shift;

        let sio_cfg = (vreg_en as u32 & 0x1)
            | ((ibuf_mode & 0x1) << 1)
            | ((vtrip_sel & 0x1) << 2)
            | ((vref_sel & 0x3) << 3)
            | ((voh_sel & 0x7) << 5);

        let temp_reg = register.get() & !pin_mask;
        let temp_reg2 = temp_reg | ((sio_cfg << pin_shift) & pin_mask);
        register.set(temp_reg2);
    }

    fn write_output_raw(&self, out_val: u32) {
        let register = &self.registers.ports[self.port].prt_out;
        let old_value = register.get();
        register.set(Self::replace_pin_field(old_value, self.pin, 1, out_val));
    }

    fn set_hsiom_function(&self, function: HsiomFunction) {
        let port_addr = &self.hsiom_registers.ports[self.port];
        let local_pin = if self.pin < GPIO_HALF {
            self.pin
        } else {
            self.pin - GPIO_HALF
        };

        let bit_offset = (local_pin * 8) as u32;
        let mask = 0x1F_u32 << bit_offset;
        let function_value = (function as u32) << bit_offset;

        let register = if self.pin < GPIO_HALF {
            &port_addr.port_sel0
        } else {
            &port_addr.port_sel1
        };

        let old_value = register.get();
        register.set((old_value & !mask) | (function_value & mask));
    }

    fn set_secure_port_nonsecure_pin(&self, nonsecure: bool) {
        let register = &self.hsiom_registers.secure_prts[self.port].secure_prt_nonsecure_mask;
        let pin_shift = self.pin as u32;
        let bit_mask = HSIOM_SEC_MASK << pin_shift;
        let new_bit = (nonsecure as u32) << pin_shift;

        let old_value = register.get();
        register.set((old_value & !bit_mask) | new_bit);
    }

    pub fn get_configuration(&self) -> Configuration {
        let (input_buffer, high_impedance) = if self.pin == 0 {
            (
                self.registers.ports[self.port]
                    .prt_cfg
                    .is_set(PRT_CFG::IN_EN0),
                self.registers.ports[self.port]
                    .prt_cfg
                    .read(PRT_CFG::DRIVE_MODE0)
                    == HIGHZ,
            )
        } else if self.pin == 1 {
            (
                self.registers.ports[self.port]
                    .prt_cfg
                    .is_set(PRT_CFG::IN_EN1),
                self.registers.ports[self.port]
                    .prt_cfg
                    .read(PRT_CFG::DRIVE_MODE1)
                    == HIGHZ,
            )
        } else if self.pin == 2 {
            (
                self.registers.ports[self.port]
                    .prt_cfg
                    .is_set(PRT_CFG::IN_EN2),
                self.registers.ports[self.port]
                    .prt_cfg
                    .read(PRT_CFG::DRIVE_MODE2)
                    == HIGHZ,
            )
        } else if self.pin == 3 {
            (
                self.registers.ports[self.port]
                    .prt_cfg
                    .is_set(PRT_CFG::IN_EN3),
                self.registers.ports[self.port]
                    .prt_cfg
                    .read(PRT_CFG::DRIVE_MODE3)
                    == HIGHZ,
            )
        } else if self.pin == 4 {
            (
                self.registers.ports[self.port]
                    .prt_cfg
                    .is_set(PRT_CFG::IN_EN4),
                self.registers.ports[self.port]
                    .prt_cfg
                    .read(PRT_CFG::DRIVE_MODE4)
                    == HIGHZ,
            )
        } else if self.pin == 5 {
            (
                self.registers.ports[self.port]
                    .prt_cfg
                    .is_set(PRT_CFG::IN_EN5),
                self.registers.ports[self.port]
                    .prt_cfg
                    .read(PRT_CFG::DRIVE_MODE5)
                    == HIGHZ,
            )
        } else if self.pin == 6 {
            (
                self.registers.ports[self.port]
                    .prt_cfg
                    .is_set(PRT_CFG::IN_EN6),
                self.registers.ports[self.port]
                    .prt_cfg
                    .read(PRT_CFG::DRIVE_MODE6)
                    == HIGHZ,
            )
        } else {
            (
                self.registers.ports[self.port]
                    .prt_cfg
                    .is_set(PRT_CFG::IN_EN7),
                self.registers.ports[self.port]
                    .prt_cfg
                    .read(PRT_CFG::DRIVE_MODE7)
                    == HIGHZ,
            )
        };
        match (input_buffer, high_impedance) {
            (false, false) => Configuration::Output,
            (false, true) => Configuration::LowPower,
            (true, true) => Configuration::Input,
            (true, false) => Configuration::InputOutput,
        }
    }

    pub fn configure_drive_mode(&self, drive_mode: DriveMode) {
        if self.pin == 0 {
            self.registers.ports[self.port]
                .prt_cfg
                .modify(PRT_CFG::DRIVE_MODE0.val(drive_mode as u32));
        } else if self.pin == 1 {
            self.registers.ports[self.port]
                .prt_cfg
                .modify(PRT_CFG::DRIVE_MODE1.val(drive_mode as u32));
        } else if self.pin == 2 {
            self.registers.ports[self.port]
                .prt_cfg
                .modify(PRT_CFG::DRIVE_MODE2.val(drive_mode as u32));
        } else if self.pin == 3 {
            self.registers.ports[self.port]
                .prt_cfg
                .modify(PRT_CFG::DRIVE_MODE3.val(drive_mode as u32));
        } else if self.pin == 4 {
            self.registers.ports[self.port]
                .prt_cfg
                .modify(PRT_CFG::DRIVE_MODE4.val(drive_mode as u32));
        } else if self.pin == 5 {
            self.registers.ports[self.port]
                .prt_cfg
                .modify(PRT_CFG::DRIVE_MODE5.val(drive_mode as u32));
        } else if self.pin == 6 {
            self.registers.ports[self.port]
                .prt_cfg
                .modify(PRT_CFG::DRIVE_MODE6.val(drive_mode as u32));
        } else {
            self.registers.ports[self.port]
                .prt_cfg
                .modify(PRT_CFG::DRIVE_MODE7.val(drive_mode as u32));
        }
    }

    pub fn configure_input(&self, input_enable: bool) {
        if self.pin == 0 {
            self.registers.ports[self.port]
                .prt_cfg
                .modify(PRT_CFG::IN_EN0.val(input_enable as u32));
        } else if self.pin == 1 {
            self.registers.ports[self.port]
                .prt_cfg
                .modify(PRT_CFG::IN_EN1.val(input_enable as u32));
        } else if self.pin == 2 {
            self.registers.ports[self.port]
                .prt_cfg
                .modify(PRT_CFG::IN_EN2.val(input_enable as u32));
        } else if self.pin == 3 {
            self.registers.ports[self.port]
                .prt_cfg
                .modify(PRT_CFG::IN_EN3.val(input_enable as u32));
        } else if self.pin == 4 {
            self.registers.ports[self.port]
                .prt_cfg
                .modify(PRT_CFG::IN_EN4.val(input_enable as u32));
        } else if self.pin == 5 {
            self.registers.ports[self.port]
                .prt_cfg
                .modify(PRT_CFG::IN_EN5.val(input_enable as u32));
        } else if self.pin == 6 {
            self.registers.ports[self.port]
                .prt_cfg
                .modify(PRT_CFG::IN_EN6.val(input_enable as u32));
        } else {
            self.registers.ports[self.port]
                .prt_cfg
                .modify(PRT_CFG::IN_EN7.val(input_enable as u32));
        }
    }

    pub fn handle_interrupt(&self) {
        if self.is_pending() {
            let bitfield = match self.pin {
                0 => PRT_INTR::EDGE0,
                1 => PRT_INTR::EDGE1,
                2 => PRT_INTR::EDGE2,
                3 => PRT_INTR::EDGE3,
                4 => PRT_INTR::EDGE4,
                5 => PRT_INTR::EDGE5,
                6 => PRT_INTR::EDGE6,
                _ => PRT_INTR::EDGE7,
            };
            self.registers.ports[self.port]
                .prt_intr
                .modify(bitfield.val(1));
            self.client.map(|client| client.fired());
        }
    }
}

impl Input for GpioPin<'_> {
    fn read(&self) -> bool {
        match self.get_configuration() {
            Configuration::Input => {
                let bitfield = match self.pin {
                    0 => PRT_IN::IN0,
                    1 => PRT_IN::IN1,
                    2 => PRT_IN::IN2,
                    3 => PRT_IN::IN3,
                    4 => PRT_IN::IN4,
                    5 => PRT_IN::IN5,
                    6 => PRT_IN::IN6,
                    _ => PRT_IN::IN7,
                };
                self.registers.ports[self.port].prt_in.is_set(bitfield)
            }
            Configuration::Output => {
                let bitfield = match self.pin {
                    0 => PRT_OUT::OUT0,
                    1 => PRT_OUT::OUT1,
                    2 => PRT_OUT::OUT2,
                    3 => PRT_OUT::OUT3,
                    4 => PRT_OUT::OUT4,
                    5 => PRT_OUT::OUT5,
                    6 => PRT_OUT::OUT6,
                    _ => PRT_OUT::OUT7,
                };
                self.registers.ports[self.port].prt_out.is_set(bitfield)
            }
            _ => false,
        }
    }
}

impl Output for GpioPin<'_> {
    fn set(&self) {
        match self.get_configuration() {
            Configuration::Output | Configuration::InputOutput => {
                let bitfield = match self.pin {
                    0 => PRT_OUT::OUT0,
                    1 => PRT_OUT::OUT1,
                    2 => PRT_OUT::OUT2,
                    3 => PRT_OUT::OUT3,
                    4 => PRT_OUT::OUT4,
                    5 => PRT_OUT::OUT5,
                    6 => PRT_OUT::OUT6,
                    _ => PRT_OUT::OUT7,
                };
                self.registers.ports[self.port]
                    .prt_out
                    .modify(bitfield.val(1));
            }
            _ => (),
        }
    }

    fn clear(&self) {
        match self.get_configuration() {
            Configuration::Output | Configuration::InputOutput => {
                let bitfield = match self.pin {
                    0 => PRT_OUT::OUT0,
                    1 => PRT_OUT::OUT1,
                    2 => PRT_OUT::OUT2,
                    3 => PRT_OUT::OUT3,
                    4 => PRT_OUT::OUT4,
                    5 => PRT_OUT::OUT5,
                    6 => PRT_OUT::OUT6,
                    _ => PRT_OUT::OUT7,
                };
                self.registers.ports[self.port]
                    .prt_out
                    .modify(bitfield.val(0));
            }
            _ => (),
        }
    }

    fn toggle(&self) -> bool {
        if self.read() {
            self.clear();
            false
        } else {
            self.set();
            true
        }
    }
}

impl Configure for GpioPin<'_> {
    fn configuration(&self) -> Configuration {
        self.get_configuration()
    }

    fn make_input(&self) -> Configuration {
        self.configure_input(true);
        self.get_configuration()
    }

    fn disable_input(&self) -> Configuration {
        self.configure_input(false);
        self.get_configuration()
    }

    fn make_output(&self) -> Configuration {
        self.configure_drive_mode(DriveMode::Strong);
        self.get_configuration()
    }

    fn disable_output(&self) -> Configuration {
        self.configure_drive_mode(DriveMode::HighZ);
        self.get_configuration()
    }

    fn set_floating_state(&self, state: kernel::hil::gpio::FloatingState) {
        match state {
            kernel::hil::gpio::FloatingState::PullUp => {
                self.configure_drive_mode(DriveMode::PullUp);
                self.set();
            }
            kernel::hil::gpio::FloatingState::PullDown => {
                self.configure_drive_mode(DriveMode::PullDown);
                self.clear();
            }
            kernel::hil::gpio::FloatingState::PullNone => {
                self.configure_drive_mode(DriveMode::HighZ)
            }
        }
    }

    fn deactivate_to_low_power(&self) {
        self.configure_drive_mode(DriveMode::HighZ);
        self.configure_input(false);
    }

    fn floating_state(&self) -> kernel::hil::gpio::FloatingState {
        let drive_mode = if self.pin == 0 {
            self.registers.ports[self.port]
                .prt_cfg
                .read(PRT_CFG::DRIVE_MODE0)
        } else if self.pin == 1 {
            self.registers.ports[self.port]
                .prt_cfg
                .read(PRT_CFG::DRIVE_MODE1)
        } else if self.pin == 2 {
            self.registers.ports[self.port]
                .prt_cfg
                .read(PRT_CFG::DRIVE_MODE2)
        } else if self.pin == 3 {
            self.registers.ports[self.port]
                .prt_cfg
                .read(PRT_CFG::DRIVE_MODE3)
        } else if self.pin == 4 {
            self.registers.ports[self.port]
                .prt_cfg
                .read(PRT_CFG::DRIVE_MODE4)
        } else if self.pin == 5 {
            self.registers.ports[self.port]
                .prt_cfg
                .read(PRT_CFG::DRIVE_MODE5)
        } else if self.pin == 6 {
            self.registers.ports[self.port]
                .prt_cfg
                .read(PRT_CFG::DRIVE_MODE6)
        } else {
            self.registers.ports[self.port]
                .prt_cfg
                .read(PRT_CFG::DRIVE_MODE7)
        };
        if drive_mode == PULL_UP {
            kernel::hil::gpio::FloatingState::PullUp
        } else if drive_mode == PULL_DOWN {
            kernel::hil::gpio::FloatingState::PullDown
        } else {
            kernel::hil::gpio::FloatingState::PullNone
        }
    }
}

impl<'a> Interrupt<'a> for GpioPin<'a> {
    fn set_client(&self, client: &'a dyn kernel::hil::gpio::Client) {
        self.client.set(client);
    }

    fn enable_interrupts(&self, mode: kernel::hil::gpio::InterruptEdge) {
        let edge_value = match mode {
            kernel::hil::gpio::InterruptEdge::RisingEdge => 1,
            kernel::hil::gpio::InterruptEdge::FallingEdge => 2,
            kernel::hil::gpio::InterruptEdge::EitherEdge => 3,
        };
        if self.pin == 0 {
            self.registers.ports[self.port]
                .prt_intr_cfg
                .modify(PRT_INTR_CFG::EDGE0_SEL.val(edge_value));
        } else if self.pin == 1 {
            self.registers.ports[self.port]
                .prt_intr_cfg
                .modify(PRT_INTR_CFG::EDGE1_SEL.val(edge_value));
        } else if self.pin == 2 {
            self.registers.ports[self.port]
                .prt_intr_cfg
                .modify(PRT_INTR_CFG::EDGE2_SEL.val(edge_value));
        } else if self.pin == 3 {
            self.registers.ports[self.port]
                .prt_intr_cfg
                .modify(PRT_INTR_CFG::EDGE3_SEL.val(edge_value));
        } else if self.pin == 4 {
            self.registers.ports[self.port]
                .prt_intr_cfg
                .modify(PRT_INTR_CFG::EDGE4_SEL.val(edge_value));
        } else if self.pin == 5 {
            self.registers.ports[self.port]
                .prt_intr_cfg
                .modify(PRT_INTR_CFG::EDGE5_SEL.val(edge_value));
        } else if self.pin == 6 {
            self.registers.ports[self.port]
                .prt_intr_cfg
                .modify(PRT_INTR_CFG::EDGE6_SEL.val(edge_value));
        } else {
            self.registers.ports[self.port]
                .prt_intr_cfg
                .modify(PRT_INTR_CFG::EDGE7_SEL.val(edge_value));
        }
        let bitfield = match self.pin {
            0 => PRT_INTR::EDGE0,
            1 => PRT_INTR::EDGE1,
            2 => PRT_INTR::EDGE2,
            3 => PRT_INTR::EDGE3,
            4 => PRT_INTR::EDGE4,
            5 => PRT_INTR::EDGE5,
            6 => PRT_INTR::EDGE6,
            _ => PRT_INTR::EDGE7,
        };
        self.registers.ports[self.port]
            .prt_intr_mask
            .modify(bitfield.val(1));
    }

    fn disable_interrupts(&self) {
        let bitfield = match self.pin {
            0 => PRT_INTR::EDGE0,
            1 => PRT_INTR::EDGE1,
            2 => PRT_INTR::EDGE2,
            3 => PRT_INTR::EDGE3,
            4 => PRT_INTR::EDGE4,
            5 => PRT_INTR::EDGE5,
            6 => PRT_INTR::EDGE6,
            _ => PRT_INTR::EDGE7,
        };
        self.registers.ports[self.port]
            .prt_intr_mask
            .modify(bitfield.val(0));
    }

    fn is_pending(&self) -> bool {
        let bitfield = match self.pin {
            0 => PRT_INTR::EDGE0,
            1 => PRT_INTR::EDGE1,
            2 => PRT_INTR::EDGE2,
            3 => PRT_INTR::EDGE3,
            4 => PRT_INTR::EDGE4,
            5 => PRT_INTR::EDGE5,
            6 => PRT_INTR::EDGE6,
            _ => PRT_INTR::EDGE7,
        };
        self.registers.ports[self.port].prt_intr.is_set(bitfield)
    }
}
