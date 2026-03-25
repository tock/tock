// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OxidOS Automotive 2025 SRL.

use kernel::utilities::registers::{
    interfaces::{ReadWriteable, Readable, Writeable},
    register_bitfields, register_structs, ReadWrite,
};
use kernel::utilities::StaticRef;

register_structs! {
    PeriPClkRegisters {
        // --- Group 0 ---
        (0x000 => gr0_div_cmd: ReadWrite<u32, DIV_CMD::Register>),
        (0x004 => _reserved0),
        (0xC00 => gr0_clock_ctl0: ReadWrite<u32, CLOCK_CTL::Register>),
        (0xC04 => _reserved1),
        (0x1C00 => gr0_div_24_5_ctl0: ReadWrite<u32, DIV_24_5_CTL::Register>),
        (0x1C04 => _reserved2),

        // --- Group 1 ---
        (0x2000 => gr1_div_cmd: ReadWrite<u32, DIV_CMD::Register>),
        (0x2004 => _reserved3),
        (0x2C00 => gr1_clock_ctl0: ReadWrite<u32, CLOCK_CTL::Register>),
        (0x2C04 => gr1_clock_ctl1: ReadWrite<u32, CLOCK_CTL::Register>),
        (0x2C08 => gr1_clock_ctl2: ReadWrite<u32, CLOCK_CTL::Register>),
        (0x2C0C => gr1_clock_ctl3: ReadWrite<u32, CLOCK_CTL::Register>),
        (0x2C10 => gr1_clock_ctl4: ReadWrite<u32, CLOCK_CTL::Register>),
        (0x2C14 => gr1_clock_ctl5: ReadWrite<u32, CLOCK_CTL::Register>),
        (0x2C18 => gr1_clock_ctl6: ReadWrite<u32, CLOCK_CTL::Register>),
        (0x2C1C => _reserved4),
        (0x3000 => gr1_div_8_ctl0: ReadWrite<u32, DIV_8_CTL::Register>),
        (0x3004 => _reserved5),

        // --- Group 4 ---
        (0x8000 => gr4_div_cmd: ReadWrite<u32, DIV_CMD::Register>),
        (0x8004 => _reserved6),
        (0x8C00 => gr4_clock_ctl0: ReadWrite<u32, CLOCK_CTL::Register>),
        (0x8C04 => gr4_clock_ctl1: ReadWrite<u32, CLOCK_CTL::Register>),
        (0x8C08 => gr4_clock_ctl2: ReadWrite<u32, CLOCK_CTL::Register>),
        (0x8C0C => gr4_clock_ctl3: ReadWrite<u32, CLOCK_CTL::Register>),
        (0x8C10 => gr4_clock_ctl4: ReadWrite<u32, CLOCK_CTL::Register>),
        (0x8C14 => gr4_clock_ctl5: ReadWrite<u32, CLOCK_CTL::Register>),
        (0x8C18 => gr4_clock_ctl6: ReadWrite<u32, CLOCK_CTL::Register>),
        (0x8C1C => _reserved7),
        (0x9000 => gr4_div_8_ctl0: ReadWrite<u32, DIV_8_CTL::Register>),
        (0x9004 => gr4_div_8_ctl1: ReadWrite<u32, DIV_8_CTL::Register>),
        (0x9008 => _reserved8),
        (0x9400 => gr4_div_16_ctl0: ReadWrite<u32, DIV_16_CTL::Register>),
        (0x9404 => _reserved9),
        (0x9800 => gr4_div_16_5_ctl0: ReadWrite<u32, DIV_16_5_CTL::Register>),
        (0x9804 => gr4_div_16_5_ctl1: ReadWrite<u32, DIV_16_5_CTL::Register>),
        (0x9808 => gr4_div_16_5_ctl2: ReadWrite<u32, DIV_16_5_CTL::Register>),
        (0x980C => _reserved10),
        (0x9C00 => gr4_div_24_5_ctl0: ReadWrite<u32, DIV_24_5_CTL::Register>),
        (0x9C04 => _reserved11),

        // --- Group 5 ---
        (0xA000 => gr5_div_cmd: ReadWrite<u32, DIV_CMD::Register>),
        (0xA004 => _reserved12),
        (0xAC00 => gr5_clock_ctl0: ReadWrite<u32, CLOCK_CTL::Register>),
        (0xAC04 => gr5_clock_ctl1: ReadWrite<u32, CLOCK_CTL::Register>),
        (0xAC08 => gr5_clock_ctl2: ReadWrite<u32, CLOCK_CTL::Register>),
        (0xAC0C => gr5_clock_ctl3: ReadWrite<u32, CLOCK_CTL::Register>),
        (0xAC10 => gr5_clock_ctl4: ReadWrite<u32, CLOCK_CTL::Register>),
        (0xAC14 => gr5_clock_ctl5: ReadWrite<u32, CLOCK_CTL::Register>),
        (0xAC18 => gr5_clock_ctl6: ReadWrite<u32, CLOCK_CTL::Register>),
        (0xAC1C => gr5_clock_ctl7: ReadWrite<u32, CLOCK_CTL::Register>),
        (0xAC20 => gr5_clock_ctl8: ReadWrite<u32, CLOCK_CTL::Register>),
        (0xAC24 => gr5_clock_ctl9: ReadWrite<u32, CLOCK_CTL::Register>),
        (0xAC28 => gr5_clock_ctl10: ReadWrite<u32, CLOCK_CTL::Register>),
        (0xAC2C => gr5_clock_ctl11: ReadWrite<u32, CLOCK_CTL::Register>),
        (0xAC30 => gr5_clock_ctl12: ReadWrite<u32, CLOCK_CTL::Register>),
        (0xAC34 => gr5_clock_ctl13: ReadWrite<u32, CLOCK_CTL::Register>),
        (0xAC38 => gr5_clock_ctl14: ReadWrite<u32, CLOCK_CTL::Register>),
        (0xAC3C => gr5_clock_ctl15: ReadWrite<u32, CLOCK_CTL::Register>),
        (0xAC40 => gr5_clock_ctl16: ReadWrite<u32, CLOCK_CTL::Register>),
        (0xAC44 => gr5_clock_ctl17: ReadWrite<u32, CLOCK_CTL::Register>),
        (0xAC48 => gr5_clock_ctl18: ReadWrite<u32, CLOCK_CTL::Register>),
        (0xAC4C => gr5_clock_ctl19: ReadWrite<u32, CLOCK_CTL::Register>),
        (0xAC50 => gr5_clock_ctl20: ReadWrite<u32, CLOCK_CTL::Register>),
        (0xAC54 => gr5_clock_ctl21: ReadWrite<u32, CLOCK_CTL::Register>),
        (0xAC58 => _reserved13),
        (0xB000 => gr5_div_8_ctl0: ReadWrite<u32, DIV_8_CTL::Register>),
        (0xB004 => gr5_div_8_ctl1: ReadWrite<u32, DIV_8_CTL::Register>),
        (0xB008 => gr5_div_8_ctl2: ReadWrite<u32, DIV_8_CTL::Register>),
        (0xB00C => gr5_div_8_ctl3: ReadWrite<u32, DIV_8_CTL::Register>),
        (0xB010 => gr5_div_8_ctl4: ReadWrite<u32, DIV_8_CTL::Register>),
        (0xB014 => gr5_div_8_ctl5: ReadWrite<u32, DIV_8_CTL::Register>),
        (0xB018 => gr5_div_8_ctl6: ReadWrite<u32, DIV_8_CTL::Register>),
        (0xB01C => gr5_div_8_ctl7: ReadWrite<u32, DIV_8_CTL::Register>),
        (0xB020 => gr5_div_8_ctl8: ReadWrite<u32, DIV_8_CTL::Register>),
        (0xB024 => gr5_div_8_ctl9: ReadWrite<u32, DIV_8_CTL::Register>),
        (0xB028 => _reserved14),
        (0xB400 => gr5_div_16_ctl0: ReadWrite<u32, DIV_16_CTL::Register>),
        (0xB404 => gr5_div_16_ctl1: ReadWrite<u32, DIV_16_CTL::Register>),
        (0xB408 => gr5_div_16_ctl2: ReadWrite<u32, DIV_16_CTL::Register>),
        (0xB40C => gr5_div_16_ctl3: ReadWrite<u32, DIV_16_CTL::Register>),
        (0xB410 => _reserved15),

        // --- Group 6 ---
        (0xC000 => gr6_div_cmd: ReadWrite<u32, DIV_CMD::Register>),
        (0xC004 => _reserved16),
        (0xCC00 => gr6_clock_ctl0: ReadWrite<u32, CLOCK_CTL::Register>),
        (0xCC04 => _reserved17),
        (0xD800 => gr6_div_16_5_ctl0: ReadWrite<u32, DIV_16_5_CTL::Register>),
        (0xD804 => @END),
    }
}
register_bitfields![u32,
DIV_CMD [
    /// (TYPE_SEL, DIV_SEL) specifies the divider on which the command (DISABLE/ENABLE) is performed.
    ///
    /// If DIV_SEL is '255' and TYPE_SEL is '3' (default/reset value), no divider is specified and no clock signal(s) are generated.
    DIV_SEL OFFSET(0) NUMBITS(8) [],
    /// Specifies the divider type of the divider on which the command is performed
    TYPE_SEL OFFSET(8) NUMBITS(2) [
        DIV8_0 = 0b00,
        DIV16_0 = 0b01,
        DIV16_5 = 0b10,
        DIV24_5 = 0b11,
    ],
    /// (PA_TYPE_SEL, PA_DIV_SEL) specifies the divider to which phase alignment is performed for the clock enable command. Any enabled divider can be used as reference. This allows all dividers to be aligned with each other, even when they are enabled at different times.
    ///
    /// If PA_DIV_SEL is '255' and PA_TYPE_SEL is '3', 'clk_pclk_root[i]' is used as reference.
    PA_DIV_SEL OFFSET(16) NUMBITS(8) [],
    /// Specifies the divider type of the divider to which phase alignment is performed for the clock enable command:
    PA_TYPE_SEL OFFSET(24) NUMBITS(2) [
        DIV8_0 = 0b00,
        DIV16_0 = 0b01,
        DIV16_5 = 0b10,
        DIV24_5 = 0b11,
    ],
    /// Clock divider disable command (mutually exclusive with ENABLE). SW sets this field to '1' and HW sets this field to '0'.
    ///
    /// The DIV_SEL and TYPE_SEL fields specify which divider is to be disabled.
    ///
    /// The HW sets the DISABLE field to '0' immediately and the HW sets the DIV_XXX_CTL.EN field of the divider to '0' immediately.
    DISABLE OFFSET(30) NUMBITS(1) [],
    /// Clock divider enable command (mutually exclusive with DISABLE). Typically, SW sets this field to '1' to enable a divider and HW sets this field to '0' to indicate that divider enabling has completed. When a divider is enabled, its integer and fractional (if present) counters are initialized to '0'. If a divider is to be re-enabled using different integer and fractional divider values, the SW should follow these steps:
    /// 0: Disable the divider using the DIV_CMD.DISABLE field.
    /// 1: Configure the divider's DIV_XXX_CTL register.
    /// 2: Enable the divider using the DIV_CMD_ENABLE field.
    ///
    /// The DIV_SEL and TYPE_SEL fields specify which divider is to be enabled. The enabled divider may be phase aligned to either 'clk_pclk_root[i]' (typical usage) or to ANY enabled divider.
    ///
    /// The PA_DIV_SEL and PA_TYPE_SEL fields specify the reference divider.
    ///
    /// The HW sets the ENABLE field to '0' when the enabling is performed and the HW set the DIV_XXX_CTL.EN field of the divider to '1' when the enabling is performed. Note that enabling with phase alignment to a low frequency divider takes time. E.g. To align to a divider that generates a clock of 'clk_pclk_root[i]'/n (with n being the integer divider value INT_DIV+1), up to n cycles may be required to perform alignment. Phase alignment to 'clk_pclk_root[i]' takes affect immediately. SW can set this field to '0' during phase alignment to abort the enabling process.
    ENABLE OFFSET(31) NUMBITS(1) []
],
CLOCK_CTL [
    /// Specifies one of the dividers of the divider type specified by TYPE_SEL.
    ///
    /// If DIV_SEL is '255' and TYPE_SEL is '3' (default/reset value), no divider is specified and no clock control signal(s) are generated.
    ///
    /// When transitioning a clock between two out-of-phase dividers, spurious clock control signals may be generated for one 'clk_pclk_root[i]' cycle during this transition. These clock control signals may cause a single clock period that is smaller than any of the two divider periods. To prevent these spurious clock signals, the clock multiplexer can be disconnected (DIV_SEL is '255' and TYPE_SEL is '3') for a transition time that is larger than the smaller of the two divider periods.
    DIV_SEL OFFSET(0) NUMBITS(8) [],
    /// Specifies divider type
    TYPE_SEL OFFSET(8) NUMBITS(2) [
        DIV8_0 = 0b00,
        DIV16_0 = 0b01,
        DIV16_5 = 0b10,
        DIV24_5 = 0b11,
    ]
],
DIV_8_CTL [
    /// Divider enabled. HW sets this field to '1' as a result of an ENABLE command. HW sets this field to '0' as a result on a DISABLE command.
    ///
    /// Note that this field is retained. As a result, the divider does NOT have to be re-enabled after transitioning from DeepSleep to Active power mode.
    EN OFFSET(0) NUMBITS(1) [],
    /// Integer division by (1+INT8_DIV). Allows for integer divisions in the range [1, 256]. Note: this type of divider does NOT allow for a fractional division.
    ///
    /// For the generation of a divided clock, the integer division range is restricted to [2, 256].
    ///
    /// For the generation of a 50/50 percent duty cycle digital divided clock, the integer division range is restricted to even numbers in the range [2, 256]. The generation of a 50/50  percent duty cycle analog divided clock has no restrictions.
    ///
    /// Note that this field is retained. However, the counter that is used to implement the division is not and will be initialized by HW to '0' when transitioning from DeepSleep to Active power mode.
    INT8_DIV OFFSET(8) NUMBITS(8) []
],
DIV_16_CTL [
    /// Divider enabled. HW sets this field to '1' as a result of an ENABLE command. HW sets this field to '0' as a result on a DISABLE command.
    ///
    /// Note that this field is retained. As a result, the divider does NOT have to be re-enabled after transitioning from DeepSleep to Active power mode.
    EN OFFSET(0) NUMBITS(1) [],
    /// Integer division by (1+INT16_DIV). Allows for integer divisions in the range [1, 65,536]. Note: this type of divider does NOT allow for a fractional division.
    ///
    /// For the generation of a divided clock, the integer division range is restricted to [2, 65,536].
    ///
    /// For the generation of a 50/50 percent duty cycle digital divided clock, the integer division range is restricted to even numbers in the range [2, 65,536]. The generation of a 50/50  percent duty cycle analog divided clock has no restrictions.
    ///
    /// Note that this field is retained. However, the counter that is used to implement the division is not and will be initialized by HW to '0' when transitioning from DeepSleep to Active power mode.
    INT16_DIV OFFSET(8) NUMBITS(16) []
],
DIV_16_5_CTL [
    /// Divider enabled. HW sets this field to '1' as a result of an ENABLE command. HW sets this field to '0' as a result on a DISABLE command.
    ///
    /// Note that this field is retained. As a result, the divider does NOT have to be re-enabled after transitioning from DeepSleep to Active power mode.
    EN OFFSET(0) NUMBITS(1) [],
    /// Fractional division by (FRAC5_DIV/32). Allows for fractional divisions in the range [0, 31/32]. Note that fractional division results in clock jitter as some clock periods may be 1 'clk_pclk_root[i]' cycle longer than other clock periods.
    ///
    /// Note that this field is retained. However, the counter that is used to implement the division is not and will be initialized by HW to '0' when transitioning from DeepSleep to Active power mode.
    FRAC5_DIV OFFSET(3) NUMBITS(5) [],
    /// Integer division by (1+INT16_DIV). Allows for integer divisions in the range [1, 65,536]. Note: combined with fractional division, this divider type allows for a division in the range [1, 65,536 31/32] in 1/32 increments.
    ///
    /// For the generation of a divided clock, the division range is restricted to [2, 65,536 31/32].
    ///
    /// For the generation of a 50/50 percent duty cycle divided clock, the  division range is restricted to [2, 65,536].
    ///
    /// Note that this field is retained. However, the counter that is used to implement the division is not and will be initialized by HW to '0' when transitioning from DeepSleep to Active power mode.
    INT16_DIV OFFSET(8) NUMBITS(16) []
],
DIV_24_5_CTL [
    /// Divider enabled. HW sets this field to '1' as a result of an ENABLE command. HW sets this field to '0' as a result on a DISABLE command.
    ///
    /// Note that this field is retained. As a result, the divider does NOT have to be re-enabled after transitioning from DeepSleep to Active power mode.
    EN OFFSET(0) NUMBITS(1) [],
    /// Fractional division by (FRAC5_DIV/32). Allows for fractional divisions in the range [0, 31/32]. Note that fractional division results in clock jitter as some clock periods may be 1 'clk_pclk_root[i]' cycle longer than other clock periods.
    ///
    /// Note that this field is retained. However, the counter that is used to implement the division is not and will be initialized by HW to '0' when transitioning from DeepSleep to Active power mode.
    FRAC5_DIV OFFSET(3) NUMBITS(5) [],
    /// Integer division by (1+INT24_DIV). Allows for integer divisions in the range [1, 16,777,216]. Note: combined with fractional division, this divider type allows for a division in the range [1, 16,777,216 31/32] in 1/32 increments.
    ///
    /// For the generation of a divided clock, the integer division range is restricted to [2, 16,777,216 31/32].
    ///
    /// For the generation of a 50/50 percent duty cycle divided clock, the  division range is restricted to [2, 16,777,216].
    ///
    /// Note that this field is retained. However, the counter that is used to implement the division is not and will be initialized by HW to '0' when transitioning from DeepSleep to Active power mode.
    INT24_DIV OFFSET(8) NUMBITS(24) []
],
];

const PERI_PCLK_BASE: StaticRef<PeriPClkRegisters> =
    unsafe { StaticRef::new(0x42040000 as *const PeriPClkRegisters) };

pub struct PeriPClk {
    registers: StaticRef<PeriPClkRegisters>,
}

impl PeriPClk {
    pub const fn new() -> PeriPClk {
        PeriPClk {
            registers: PERI_PCLK_BASE,
        }
    }

    // TODO: mtb-pdl-cat1/release-v3.20.0/drivers/source/cy_sysclk_v2.c
    // #ifdef CY_CFG_SYSCLK_CLKPATH1_ENABLED
    //   Cy_SysClk_ClkPath1Init();
    // #endif

    #[no_mangle]
    pub fn init_clocks(&self) {
        self.registers
            .gr4_div_cmd
            .write(DIV_CMD::DISABLE::SET + DIV_CMD::DIV_SEL.val(0) + DIV_CMD::TYPE_SEL::DIV8_0);
        self.registers
            .gr4_div_8_ctl0
            .modify(DIV_8_CTL::INT8_DIV.val(108));
        self.registers.gr4_div_cmd.write(
            DIV_CMD::ENABLE::SET
                + DIV_CMD::DIV_SEL.val(0)
                + DIV_CMD::TYPE_SEL::DIV8_0
                + DIV_CMD::PA_TYPE_SEL.val(3) // set PA masks
                + DIV_CMD::PA_DIV_SEL.val(255),
        );
        while self.registers.gr4_div_cmd.read(DIV_CMD::ENABLE) == 1 {}

        // Group 5 divider 8, index 0
        self.registers
            .gr5_div_cmd
            .write(DIV_CMD::DISABLE::SET + DIV_CMD::DIV_SEL.val(0) + DIV_CMD::TYPE_SEL::DIV8_0);
        self.registers
            .gr5_div_8_ctl0
            .modify(DIV_8_CTL::INT8_DIV.val(239));
        self.registers.gr5_div_cmd.write(
            DIV_CMD::ENABLE::SET
                + DIV_CMD::DIV_SEL.val(0)
                + DIV_CMD::TYPE_SEL::DIV8_0
                + DIV_CMD::PA_TYPE_SEL.val(3) // set PA masks
                + DIV_CMD::PA_DIV_SEL.val(255),
        );
        while self.registers.gr5_div_cmd.read(DIV_CMD::ENABLE) == 1 {}
    }

    pub fn init_peripherals(&self) {
        self.registers
            .gr4_clock_ctl3 // ctl3 = SCB
            .write(CLOCK_CTL::DIV_SEL.val(0) + CLOCK_CTL::TYPE_SEL::DIV8_0);

        self.registers
            .gr5_clock_ctl0 // ctl0 = PCLK_TCPWM0_CLOCK_COUNTER_EN0
            .write(CLOCK_CTL::DIV_SEL.val(0) + CLOCK_CTL::TYPE_SEL::DIV8_0);
    }
}
