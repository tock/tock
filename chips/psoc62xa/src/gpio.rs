// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OxidOS Automotive 2025 SRL.

use kernel::hil::gpio::{Configuration, Configure, Input, Interrupt, Output};
use kernel::utilities::cells::OptionalCell;
use kernel::utilities::registers::{
    interfaces::{ReadWriteable, Readable},
    register_bitfields, register_structs, ReadOnly, ReadWrite,
};
use kernel::utilities::StaticRef;

#[repr(C)]
struct GpioPort {
    prt_out: ReadWrite<u32, PRT_OUT::Register>,
    prt_out_clr: ReadWrite<u32, PRT_OUT::Register>,
    prt_out_set: ReadWrite<u32, PRT_OUT::Register>,
    prt_out_inv: ReadWrite<u32, PRT_OUT::Register>,
    prt_in: ReadOnly<u32, PRT_IN::Register>,
    prt_intr: ReadWrite<u32, PRT_INTR::Register>,
    prt_intr_mask: ReadWrite<u32, PRT_INTR::Register>,
    prt_intr_masked: ReadOnly<u32, PRT_INTR::Register>,
    prt_intr_set: ReadWrite<u32, PRT_INTR::Register>,
    _reserved0: [u32; 7],
    prt_intr_cfg: ReadWrite<u32, PRT_INTR_CFG::Register>,
    prt_cfg: ReadWrite<u32, PRT_CFG::Register>,
    prt_cfg_in: ReadWrite<u32, PRT_CFG_IN::Register>,
    prt_cfg_out: ReadWrite<u32, PRT_CFG_OUT::Register>,
    _reserved1: [u32; 12],
}

register_structs! {
    /// GPIO port control/configuration
    GpioRegisters {
        (0x000 => ports: [GpioPort; 15]),
        (0x780 => _reserved0),
        /// Interrupt port cause register 0
        (0x4000 => intr_cause0: ReadOnly<u32>),
        /// Interrupt port cause register 1
        (0x4004 => intr_cause1: ReadOnly<u32>),
        /// Interrupt port cause register 2
        (0x4008 => intr_cause2: ReadOnly<u32>),
        /// Interrupt port cause register 3
        (0x400C => intr_cause3: ReadOnly<u32>),
        /// Extern power supply detection register
        (0x4010 => vdd_active: ReadOnly<u32, VDD_ACTIVE::Register>),
        /// Supply detection interrupt register
        (0x4014 => vdd_intr: ReadWrite<u32, VDD_INTR::Register>),
        /// Supply detection interrupt mask register
        (0x4018 => vdd_intr_mask: ReadWrite<u32, VDD_INTR_MASK::Register>),
        /// Supply detection interrupt masked register
        (0x401C => vdd_intr_masked: ReadOnly<u32, VDD_INTR_MASKED::Register>),
        /// Supply detection interrupt set register
        (0x4020 => vdd_intr_set: ReadWrite<u32, VDD_INTR_SET::Register>),
        (0x4024 => @END),
    }
}
register_bitfields![u32,
PRT_OUT [
    OUT0 OFFSET(0) NUMBITS(1) [],
    OUT1 OFFSET(1) NUMBITS(1) [],
    OUT2 OFFSET(2) NUMBITS(1) [],
    OUT3 OFFSET(3) NUMBITS(1) [],
    OUT4 OFFSET(4) NUMBITS(1) [],
    OUT5 OFFSET(5) NUMBITS(1) [],
    OUT6 OFFSET(6) NUMBITS(1) [],
    OUT7 OFFSET(7) NUMBITS(1) [],
],
PRT_IN [
    IN0 OFFSET(0) NUMBITS(1) [],
    IN1 OFFSET(1) NUMBITS(1) [],
    IN2 OFFSET(2) NUMBITS(1) [],
    IN3 OFFSET(3) NUMBITS(1) [],
    IN4 OFFSET(4) NUMBITS(1) [],
    IN5 OFFSET(5) NUMBITS(1) [],
    IN6 OFFSET(6) NUMBITS(1) [],
    IN7 OFFSET(7) NUMBITS(1) [],
    FLT_IN OFFSET(8) NUMBITS(1) [],
],
PRT_INTR [
    EDGE0 OFFSET(0) NUMBITS(1) [],
    EDGE1 OFFSET(1) NUMBITS(1) [],
    EDGE2 OFFSET(2) NUMBITS(1) [],
    EDGE3 OFFSET(3) NUMBITS(1) [],
    EDGE4 OFFSET(4) NUMBITS(1) [],
    EDGE5 OFFSET(5) NUMBITS(1) [],
    EDGE6 OFFSET(6) NUMBITS(1) [],
    EDGE7 OFFSET(7) NUMBITS(1) [],
    FLT_EDGE OFFSET(8) NUMBITS(1) [],
    IN_IN OFFSET(16) NUMBITS(8) [],
    FLT_IN_IN OFFSET(24) NUMBITS(1) [],
],
PRT_INTR_CFG [
    EDGE_SEL0 OFFSET(0) NUMBITS(2) [],
    EDGE_SEL1 OFFSET(2) NUMBITS(2) [],
    EDGE_SEL2 OFFSET(4) NUMBITS(2) [],
    EDGE_SEL3 OFFSET(6) NUMBITS(2) [],
    EDGE_SEL4 OFFSET(8) NUMBITS(2) [],
    EDGE_SEL5 OFFSET(10) NUMBITS(2) [],
    EDGE_SEL6 OFFSET(12) NUMBITS(2) [],
    EDGE_SEL7 OFFSET(14) NUMBITS(2) [],
    FLT_EDGE_SEL OFFSET(16) NUMBITS(2) [],
    FLT_SEL OFFSET(18) NUMBITS(3) [],
],
PRT_CFG [
    DRIVE_MODE0 OFFSET(0) NUMBITS(3) [
        HIGHZ = 0,
        PULL_UP = 2,
        PULL_DOWN = 3,
        OD_DRIVESLOW = 4,
        OD_DRIVESHIGH = 5,
        STRONG = 6,
        PULLUP_DOWN = 7,
    ],
    IN_EN0 OFFSET(3) NUMBITS(1) [],
    DRIVE_MODE1 OFFSET(4) NUMBITS(3) [
        HIGHZ = 0,
        PULL_UP = 2,
        PULL_DOWN = 3,
        OD_DRIVESLOW = 4,
        OD_DRIVESHIGH = 5,
        STRONG = 6,
        PULLUP_DOWN = 7,
    ],
    IN_EN1 OFFSET(7) NUMBITS(1) [],
    DRIVE_MODE2 OFFSET(8) NUMBITS(3) [
        HIGHZ = 0,
        PULL_UP = 2,
        PULL_DOWN = 3,
        OD_DRIVESLOW = 4,
        OD_DRIVESHIGH = 5,
        STRONG = 6,
        PULLUP_DOWN = 7,
    ],
    IN_EN2 OFFSET(11) NUMBITS(1) [],
    DRIVE_MODE3 OFFSET(12) NUMBITS(3) [
        HIGHZ = 0,
        PULL_UP = 2,
        PULL_DOWN = 3,
        OD_DRIVESLOW = 4,
        OD_DRIVESHIGH = 5,
        STRONG = 6,
        PULLUP_DOWN = 7,
    ],
    IN_EN3 OFFSET(15) NUMBITS(1) [],
    DRIVE_MODE4 OFFSET(16) NUMBITS(3) [
        HIGHZ = 0,
        PULL_UP = 2,
        PULL_DOWN = 3,
        OD_DRIVESLOW = 4,
        OD_DRIVESHIGH = 5,
        STRONG = 6,
        PULLUP_DOWN = 7,
    ],
    IN_EN4 OFFSET(19) NUMBITS(1) [],
    DRIVE_MODE5 OFFSET(20) NUMBITS(3) [
        HIGHZ = 0,
        PULL_UP = 2,
        PULL_DOWN = 3,
        OD_DRIVESLOW = 4,
        OD_DRIVESHIGH = 5,
        STRONG = 6,
        PULLUP_DOWN = 7,
    ],
    IN_EN5 OFFSET(23) NUMBITS(1) [],
    DRIVE_MODE6 OFFSET(24) NUMBITS(3) [
        HIGHZ = 0,
        PULL_UP = 2,
        PULL_DOWN = 3,
        OD_DRIVESLOW = 4,
        OD_DRIVESHIGH = 5,
        STRONG = 6,
        PULLUP_DOWN = 7,
    ],
    IN_EN6 OFFSET(27) NUMBITS(1) [],
    DRIVE_MODE7 OFFSET(28) NUMBITS(3) [
        HIGHZ = 0,
        PULL_UP = 2,
        PULL_DOWN = 3,
        OD_DRIVESLOW = 4,
        OD_DRIVESHIGH = 5,
        STRONG = 6,
        PULLUP_DOWN = 7,
    ],
    IN_EN7 OFFSET(31) NUMBITS(1) [],
],
PRT_CFG_IN [
    VTRIP_VTRIP_SEL_0 OFFSET(0) NUMBITS(1) [],
    VTRIP_VTRIP_SEL_1 OFFSET(1) NUMBITS(1) [],
    VTRIP_VTRIP_SEL_2 OFFSET(2) NUMBITS(1) [],
    VTRIP_VTRIP_SEL_3 OFFSET(3) NUMBITS(1) [],
    VTRIP_VTRIP_SEL_4 OFFSET(4) NUMBITS(1) [],
    VTRIP_VTRIP_SEL_5 OFFSET(5) NUMBITS(1) [],
    VTRIP_VTRIP_SEL_6 OFFSET(6) NUMBITS(1) [],
    VTRIP_VTRIP_SEL_7 OFFSET(7) NUMBITS(1) [],
],
PRT_CFG_OUT [
    SLOW0 OFFSET(0) NUMBITS(1) [],
    SLOW1 OFFSET(0) NUMBITS(1) [],
    SLOW2 OFFSET(0) NUMBITS(1) [],
    SLOW3 OFFSET(0) NUMBITS(1) [],
    SLOW4 OFFSET(0) NUMBITS(1) [],
    SLOW5 OFFSET(0) NUMBITS(1) [],
    SLOW6 OFFSET(0) NUMBITS(1) [],
    SLOW7 OFFSET(0) NUMBITS(1) [],
    DRIVE_SEL0 OFFSET(16) NUMBITS(2) [],
    DRIVE_SEL1 OFFSET(18) NUMBITS(2) [],
    DRIVE_SEL2 OFFSET(20) NUMBITS(2) [],
    DRIVE_SEL3 OFFSET(22) NUMBITS(2) [],
    DRIVE_SEL4 OFFSET(24) NUMBITS(2) [],
    DRIVE_SEL5 OFFSET(26) NUMBITS(2) [],
    DRIVE_SEL6 OFFSET(28) NUMBITS(2) [],
    DRIVE_SEL7 OFFSET(30) NUMBITS(2) [],
],
INTR_CAUSE0 [
    PORT_INT OFFSET(0) NUMBITS(32) []
],
INTR_CAUSE1 [
    PORT_INT OFFSET(0) NUMBITS(32) []
],
INTR_CAUSE2 [
    PORT_INT OFFSET(0) NUMBITS(32) []
],
INTR_CAUSE3 [
    PORT_INT OFFSET(0) NUMBITS(32) []
],
VDD_ACTIVE [
    VDDIO_ACTIVE OFFSET(0) NUMBITS(16) [],
    VDDA_ACTIVE OFFSET(30) NUMBITS(1) [],
    VDDD_ACTIVE OFFSET(31) NUMBITS(1) []
],
VDD_INTR [
    VDDIO_ACTIVE OFFSET(0) NUMBITS(16) [],
    VDDA_ACTIVE OFFSET(30) NUMBITS(1) [],
    VDDD_ACTIVE OFFSET(31) NUMBITS(1) []
],
VDD_INTR_MASK [
    VDDIO_ACTIVE OFFSET(0) NUMBITS(16) [],
    VDDA_ACTIVE OFFSET(30) NUMBITS(1) [],
    VDDD_ACTIVE OFFSET(31) NUMBITS(1) []
],
VDD_INTR_MASKED [
    VDDIO_ACTIVE OFFSET(0) NUMBITS(16) [],
    VDDA_ACTIVE OFFSET(30) NUMBITS(1) [],
    VDDD_ACTIVE OFFSET(31) NUMBITS(1) []
],
VDD_INTR_SET [
    VDDIO_ACTIVE OFFSET(0) NUMBITS(16) [],
    VDDA_ACTIVE OFFSET(30) NUMBITS(1) [],
    VDDD_ACTIVE OFFSET(31) NUMBITS(1) []
]
];
const GPIO_BASE: StaticRef<GpioRegisters> =
    unsafe { StaticRef::new(0x40310000 as *const GpioRegisters) };

const HIGHZ: u32 = 0;
const PULL_UP: u32 = 2;
const PULL_DOWN: u32 = 3;

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

    pub fn get_pin(&self, searched_pin: PsocPin) -> &'a GpioPin {
        self.pins[searched_pin as usize].as_ref().unwrap()
    }

    pub fn handle_interrupt(&self) {
        for pin in self.pins.iter() {
            pin.as_ref().inspect(|pin| pin.handle_interrupt());
        }
    }
}

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
    pin: usize,
    port: usize,

    client: OptionalCell<&'a dyn kernel::hil::gpio::Client>,
}

impl GpioPin<'_> {
    pub const fn new(id: PsocPin) -> Self {
        Self {
            registers: GPIO_BASE,
            pin: (id as usize) % 8,
            port: (id as usize) / 8,
            client: OptionalCell::empty(),
        }
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
                .modify(PRT_INTR_CFG::EDGE_SEL0.val(edge_value));
        } else if self.pin == 1 {
            self.registers.ports[self.port]
                .prt_intr_cfg
                .modify(PRT_INTR_CFG::EDGE_SEL1.val(edge_value));
        } else if self.pin == 2 {
            self.registers.ports[self.port]
                .prt_intr_cfg
                .modify(PRT_INTR_CFG::EDGE_SEL2.val(edge_value));
        } else if self.pin == 3 {
            self.registers.ports[self.port]
                .prt_intr_cfg
                .modify(PRT_INTR_CFG::EDGE_SEL3.val(edge_value));
        } else if self.pin == 4 {
            self.registers.ports[self.port]
                .prt_intr_cfg
                .modify(PRT_INTR_CFG::EDGE_SEL4.val(edge_value));
        } else if self.pin == 5 {
            self.registers.ports[self.port]
                .prt_intr_cfg
                .modify(PRT_INTR_CFG::EDGE_SEL5.val(edge_value));
        } else if self.pin == 6 {
            self.registers.ports[self.port]
                .prt_intr_cfg
                .modify(PRT_INTR_CFG::EDGE_SEL6.val(edge_value));
        } else {
            self.registers.ports[self.port]
                .prt_intr_cfg
                .modify(PRT_INTR_CFG::EDGE_SEL7.val(edge_value));
        };
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
