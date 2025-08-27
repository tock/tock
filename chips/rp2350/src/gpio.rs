// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OxidOS Automotive 2025.

use enum_primitive::cast::FromPrimitive;
use enum_primitive::enum_from_primitive;
use kernel::hil;
use kernel::utilities::cells::OptionalCell;
use kernel::utilities::registers::{
    interfaces::{ReadWriteable, Readable, Writeable},
    register_bitfields, register_structs, ReadOnly, ReadWrite,
};
use kernel::utilities::StaticRef;

use crate::chip::Processor;
#[repr(C)]
struct GpioPin {
    status: ReadOnly<u32, GPIOx_STATUS::Register>,
    ctrl: ReadWrite<u32, GPIOx_CTRL::Register>,
}

#[repr(C)]
struct GpioProc {
    enable: [ReadWrite<u32, GPIO_INTRxx::Register>; 6],
    force: [ReadWrite<u32, GPIO_INTRxx::Register>; 6],
    status: [ReadWrite<u32, GPIO_INTRxx::Register>; 6],
}

register_structs! {

    GpioRegisters {
        (0x000 => pin: [GpioPin; 48]),
        (0x180 => _reserved0),

        (0x200 => irqsummary_proc0_secure0: ReadWrite<u32, IRQSUMMARY_PROC0::Register>),

        (0x204 => irqsummary_proc0_secure1: ReadWrite<u32, IRQSUMMARY_PROC0::Register>),

        (0x208 => irqsummary_proc0_nonsecure0: ReadWrite<u32, IRQSUMMARY_PROC0::Register>),

        (0x20C => irqsummary_proc0_nonsecure1: ReadWrite<u32, IRQSUMMARY_PROC0::Register>),

        (0x210 => irqsummary_proc1_secure0: ReadWrite<u32, IRQSUMMARY_PROC1::Register>),

        (0x214 => irqsummary_proc1_secure1: ReadWrite<u32, IRQSUMMARY_PROC1::Register>),

        (0x218 => irqsummary_proc1_nonsecure0: ReadWrite<u32, IRQSUMMARY_PROC1::Register>),

        (0x21C => irqsummary_proc1_nonsecure1: ReadWrite<u32, IRQSUMMARY_PROC1::Register>),

        (0x220 => irqsummary_dormant_wake_secure0: ReadWrite<u32, IRQSUMMARY_PROC0::Register>),

        (0x224 => irqsummary_dormant_wake_secure1: ReadWrite<u32, IRQSUMMARY_PROC1::Register>),

        (0x228 => irqsummary_dormant_wake_nonsecure0: ReadWrite<u32, IRQSUMMARY_PROC0::Register>),

        (0x22C => irqsummary_dormant_wake_nonsecure1: ReadWrite<u32, IRQSUMMARY_PROC1::Register>),
        /// Raw Interrupts
        (0x230 => intr: [ReadWrite<u32, GPIO_INTRxx::Register>; 6]),
        /// Interrupts for procs
        (0x248 => interrupt_proc: [GpioProc; 2]),
        /// Interrupt Enable for dormant_wake
        (0x2D8 => dormant_wake_inte: GpioProc),
        (0x320 => @END),
    },
    GpioPadRegisters {
        /// Voltage select. Per bank control
        (0x000 => voltage_select: ReadWrite<u32, VOLTAGE_SELECT::Register>),

        (0x004 => gpio_pad: [ReadWrite<u32, GPIO_PAD::Register>; 48]),

        (0x0C4 => swclk: ReadWrite<u32, SWCLK::Register>),

        (0x0C8 => swd: ReadWrite<u32, SWD::Register>),
        (0x0CC => @END),
    },
    SIORegisters {
        /// Processor core identifier
        (0x000 => cpuid: ReadWrite<u32>),
        /// Input value for GPIO0...31.
        /// In the Non-secure SIO, Secure-only GPIOs (as per ACCESSCTRL)
        (0x004 => gpio_in: ReadWrite<u32, GPIO_IN::Register>),
        /// Input value on GPIO32...47, QSPI IOs and USB pins
        /// In the Non-secure SIO, Secure-only GPIOs (as per ACCESSCTRL)
        (0x008 => gpio_hi_in: ReadWrite<u32, GPIO_HI_IN::Register>),
        (0x00C => _reserved0),
        /// GPIO0...31 output value
        (0x010 => gpio_out: ReadWrite<u32, GPIO_OUT::Register>),
        /// Output value for GPIO32...47, QSPI IOs and USB pins.
        /// Write to set output level (1/0 -> high/low). Reading back gi
        /// In the Non-secure SIO, Secure-only GPIOs (as per ACCESSCTRL)
        (0x014 => gpio_hi_out: ReadWrite<u32, GPIO_HI_OUT::Register>),
        /// GPIO0...31 output value set
        (0x018 => gpio_out_set: ReadWrite<u32>),
        /// Output value set for GPIO32..47, QSPI IOs and USB pins.
        /// Perform an atomic bit-set on GPIO_HI_OUT, i.e. `GPIO_HI_OUT
        (0x01C => gpio_hi_out_set: ReadWrite<u32, GPIO_HI_OUT_SET::Register>),
        /// GPIO0...31 output value clear
        (0x020 => gpio_out_clr: ReadWrite<u32>),
        /// Output value clear for GPIO32..47, QSPI IOs and USB pins.
        /// Perform an atomic bit-clear on GPIO_HI_OUT, i.e. `GPIO_HI_OU
        (0x024 => gpio_hi_out_clr: ReadWrite<u32, GPIO_HI_OUT_CLR::Register>),
        /// GPIO0...31 output value XOR
        (0x028 => gpio_out_xor: ReadWrite<u32>),
        /// Output value XOR for GPIO32..47, QSPI IOs and USB pins.
        /// Perform an atomic bitwise XOR on GPIO_HI_OUT, i.e. `GPIO_HI_
        (0x02C => gpio_hi_out_xor: ReadWrite<u32, GPIO_HI_OUT_XOR::Register>),
        /// GPIO0...31 output enable
        (0x030 => gpio_oe: ReadWrite<u32, GPIO_OE::Register>),
        /// Output enable value for GPIO32...47, QSPI IOs and USB pins.
        /// Write output enable (1/0 -> output/input). Reading back give
        /// In the Non-secure SIO, Secure-only GPIOs (as per ACCESSCTRL)
        (0x034 => gpio_hi_oe: ReadWrite<u32, GPIO_HI_OE::Register>),
        /// GPIO0...31 output enable set
        (0x038 => gpio_oe_set: ReadWrite<u32>),
        /// Output enable set for GPIO32...47, QSPI IOs and USB pins.
        /// Perform an atomic bit-set on GPIO_HI_OE, i.e. `GPIO_HI_OE |=
        (0x03C => gpio_hi_oe_set: ReadWrite<u32, GPIO_HI_OE_SET::Register>),
        /// GPIO0...31 output enable clear
        (0x040 => gpio_oe_clr: ReadWrite<u32>),
        /// Output enable clear for GPIO32...47, QSPI IOs and USB pins.
        /// Perform an atomic bit-clear on GPIO_HI_OE, i.e. `GPIO_HI_OE
        (0x044 => gpio_hi_oe_clr: ReadWrite<u32, GPIO_HI_OE_CLR::Register>),
        /// GPIO0...31 output enable XOR
        (0x048 => gpio_oe_xor: ReadWrite<u32>),
        /// Output enable XOR for GPIO32...47, QSPI IOs and USB pins.
        /// Perform an atomic bitwise XOR on GPIO_HI_OE, i.e. `GPIO_HI_O
        (0x04C => gpio_hi_oe_xor: ReadWrite<u32, GPIO_HI_OE_XOR::Register>),
        /// Status register for inter-core FIFOs (mailboxes).
        /// There is one FIFO in the core 0 -> core 1 direction, and one
        /// Core 0 can see the read side of the 1->0 FIFO (RX), and the
        /// Core 1 can see the read side of the 0->1 FIFO (RX), and the
        /// The SIO IRQ for each core is the logical OR of the VLD, WOF
        (0x050 => fifo_st: ReadWrite<u32, FIFO_ST::Register>),
        /// Write access to this core's TX FIFO
        (0x054 => fifo_wr: ReadWrite<u32>),
        /// Read access to this core's RX FIFO
        (0x058 => fifo_rd: ReadWrite<u32>),
        /// Spinlock state
        /// A bitmap containing the state of all 32 spinlocks (1=locked)
        /// Mainly intended for debugging.
        (0x05C => spinlock_st: ReadWrite<u32>),
        (0x060 => _reserved1),
        /// Read/write access to accumulator 0
        (0x080 => interp0_accum0: ReadWrite<u32>),
        /// Read/write access to accumulator 1
        (0x084 => interp0_accum1: ReadWrite<u32>),
        /// Read/write access to BASE0 register.
        (0x088 => interp0_base0: ReadWrite<u32>),
        /// Read/write access to BASE1 register.
        (0x08C => interp0_base1: ReadWrite<u32>),
        /// Read/write access to BASE2 register.
        (0x090 => interp0_base2: ReadWrite<u32>),
        /// Read LANE0 result, and simultaneously write lane results to both accumulators (P
        (0x094 => interp0_pop_lane0: ReadWrite<u32>),
        /// Read LANE1 result, and simultaneously write lane results to both accumulators (P
        (0x098 => interp0_pop_lane1: ReadWrite<u32>),
        /// Read FULL result, and simultaneously write lane results to both accumulators (PO
        (0x09C => interp0_pop_full: ReadWrite<u32>),
        /// Read LANE0 result, without altering any internal state (PEEK).
        (0x0A0 => interp0_peek_lane0: ReadWrite<u32>),
        /// Read LANE1 result, without altering any internal state (PEEK).
        (0x0A4 => interp0_peek_lane1: ReadWrite<u32>),
        /// Read FULL result, without altering any internal state (PEEK).
        (0x0A8 => interp0_peek_full: ReadWrite<u32>),
        /// Control register for lane 0
        (0x0AC => interp0_ctrl_lane0: ReadWrite<u32, INTERP0_CTRL_LANE0::Register>),
        /// Control register for lane 1
        (0x0B0 => interp0_ctrl_lane1: ReadWrite<u32, INTERP0_CTRL_LANE1::Register>),
        /// Values written here are atomically added to ACCUM0
        /// Reading yields lane 0's raw shift and mask value (BASE0 not
        (0x0B4 => interp0_accum0_add: ReadWrite<u32>),
        /// Values written here are atomically added to ACCUM1
        /// Reading yields lane 1's raw shift and mask value (BASE1 not
        (0x0B8 => interp0_accum1_add: ReadWrite<u32>),
        /// On write, the lower 16 bits go to BASE0, upper bits to BASE1 simultaneously.
        /// Each half is sign-extended to 32 bits if that lane's SIGNED
        (0x0BC => interp0_base_1and0: ReadWrite<u32>),
        /// Read/write access to accumulator 0
        (0x0C0 => interp1_accum0: ReadWrite<u32>),
        /// Read/write access to accumulator 1
        (0x0C4 => interp1_accum1: ReadWrite<u32>),
        /// Read/write access to BASE0 register.
        (0x0C8 => interp1_base0: ReadWrite<u32>),
        /// Read/write access to BASE1 register.
        (0x0CC => interp1_base1: ReadWrite<u32>),
        /// Read/write access to BASE2 register.
        (0x0D0 => interp1_base2: ReadWrite<u32>),
        /// Read LANE0 result, and simultaneously write lane results to both accumulators (P
        (0x0D4 => interp1_pop_lane0: ReadWrite<u32>),
        /// Read LANE1 result, and simultaneously write lane results to both accumulators (P
        (0x0D8 => interp1_pop_lane1: ReadWrite<u32>),
        /// Read FULL result, and simultaneously write lane results to both accumulators (PO
        (0x0DC => interp1_pop_full: ReadWrite<u32>),
        /// Read LANE0 result, without altering any internal state (PEEK).
        (0x0E0 => interp1_peek_lane0: ReadWrite<u32>),
        /// Read LANE1 result, without altering any internal state (PEEK).
        (0x0E4 => interp1_peek_lane1: ReadWrite<u32>),
        /// Read FULL result, without altering any internal state (PEEK).
        (0x0E8 => interp1_peek_full: ReadWrite<u32>),
        /// Control register for lane 0
        (0x0EC => interp1_ctrl_lane0: ReadWrite<u32, INTERP1_CTRL_LANE0::Register>),
        /// Control register for lane 1
        (0x0F0 => interp1_ctrl_lane1: ReadWrite<u32, INTERP1_CTRL_LANE1::Register>),
        /// Values written here are atomically added to ACCUM0
        /// Reading yields lane 0's raw shift and mask value (BASE0 not
        (0x0F4 => interp1_accum0_add: ReadWrite<u32>),
        /// Values written here are atomically added to ACCUM1
        /// Reading yields lane 1's raw shift and mask value (BASE1 not
        (0x0F8 => interp1_accum1_add: ReadWrite<u32>),
        /// On write, the lower 16 bits go to BASE0, upper bits to BASE1 simultaneously.
        /// Each half is sign-extended to 32 bits if that lane's SIGNED
        (0x0FC => interp1_base_1and0: ReadWrite<u32>),
        /// Reading from a spinlock address will:
        /// - Return 0 if lock is already locked
        /// - Otherwise return nonzero, and simultaneously claim the loc
        /// Writing (any value) releases the lock.
        /// If core 0 and core 1 attempt to claim the same lock simultan
        /// The value returned on success is 0x1 << lock number.
        (0x100 => spinlock: [ReadWrite<u32, SPINLOCK::Register>; 32]),
        /// Trigger a doorbell interrupt on the opposite core.
        /// Write 1 to a bit to set the corresponding bit in DOORBELL_IN
        /// Read to get the status of the doorbells currently asserted o
        (0x180 => doorbell_out_set: ReadWrite<u32>),
        /// Clear doorbells which have been posted to the opposite core. This register is in
        /// Writing 1 to a bit in DOORBELL_OUT_CLR clears the correspond
        /// Reading returns the status of the doorbells currently assert
        (0x184 => doorbell_out_clr: ReadWrite<u32>),
        /// Write 1s to trigger doorbell interrupts on this core. Read to get status of door
        (0x188 => doorbell_in_set: ReadWrite<u32>),
        /// Check and acknowledge doorbells posted to this core. This core's doorbell interr
        /// Write 1 to each bit to clear that bit. The doorbell interrup
        (0x18C => doorbell_in_clr: ReadWrite<u32>),
        /// Detach certain core-local peripherals from Secure SIO, and attach them to Non-se
        /// This register is per-core, and is only present on the Secure
        /// Most SIO hardware is duplicated across the Secure and Non-se
        (0x190 => peri_nonsec: ReadWrite<u32, PERI_NONSEC::Register>),
        (0x194 => _reserved2),
        /// Control the assertion of the standard software interrupt (MIP.MSIP) on the RISC-
        /// Unlike the RISC-V timer, this interrupt is not routed to a n
        /// It is safe for both cores to write to this register on the s
        (0x1A0 => riscv_softirq: ReadWrite<u32, RISCV_SOFTIRQ::Register>),
        /// Control register for the RISC-V 64-bit Machine-mode timer. This timer is only pr
        /// Note whilst this timer follows the RISC-V privileged specifi
        (0x1A4 => mtime_ctrl: ReadWrite<u32, MTIME_CTRL::Register>),
        (0x1A8 => _reserved3),
        /// Read/write access to the high half of RISC-V Machine-mode timer. This register i
        (0x1B0 => mtime: ReadWrite<u32>),
        /// Read/write access to the high half of RISC-V Machine-mode timer. This register i
        (0x1B4 => mtimeh: ReadWrite<u32>),
        /// Low half of RISC-V Machine-mode timer comparator. This register is core-local, i
        /// The timer interrupt is asserted whenever MTIME is greater th
        (0x1B8 => mtimecmp: ReadWrite<u32>),
        /// High half of RISC-V Machine-mode timer comparator. This register is core-local.
        /// The timer interrupt is asserted whenever MTIME is greater th
        (0x1BC => mtimecmph: ReadWrite<u32>),
        /// Control register for TMDS encoder.
        (0x1C0 => tmds_ctrl: ReadWrite<u32, TMDS_CTRL::Register>),
        /// Write-only access to the TMDS colour data register.
        (0x1C4 => tmds_wdata: ReadWrite<u32>),
        /// Get the encoding of one pixel's worth of colour data, packed into a 32-bit value
        /// The PEEK alias does not shift the colour register when read,
        (0x1C8 => tmds_peek_single: ReadWrite<u32>),
        /// Get the encoding of one pixel's worth of colour data, packed into a 32-bit value
        /// The POP alias shifts the colour register when read, as well
        (0x1CC => tmds_pop_single: ReadWrite<u32>),
        /// Get lane 0 of the encoding of two pixels' worth of colour data. Two 10-bit TMDS
        /// The PEEK alias does not shift the colour register when read,
        (0x1D0 => tmds_peek_double_l0: ReadWrite<u32>),
        /// Get lane 0 of the encoding of two pixels' worth of colour data. Two 10-bit TMDS
        /// The POP alias shifts the colour register when read, accordin
        (0x1D4 => tmds_pop_double_l0: ReadWrite<u32>),
        /// Get lane 1 of the encoding of two pixels' worth of colour data. Two 10-bit TMDS
        /// The PEEK alias does not shift the colour register when read,
        (0x1D8 => tmds_peek_double_l1: ReadWrite<u32>),
        /// Get lane 1 of the encoding of two pixels' worth of colour data. Two 10-bit TMDS
        /// The POP alias shifts the colour register when read, accordin
        (0x1DC => tmds_pop_double_l1: ReadWrite<u32>),
        /// Get lane 2 of the encoding of two pixels' worth of colour data. Two 10-bit TMDS
        /// The PEEK alias does not shift the colour register when read,
        (0x1E0 => tmds_peek_double_l2: ReadWrite<u32>),
        /// Get lane 2 of the encoding of two pixels' worth of colour data. Two 10-bit TMDS
        /// The POP alias shifts the colour register when read, accordin
        (0x1E4 => tmds_pop_double_l2: ReadWrite<u32>),
        (0x1E8 => @END),
    }
}
register_bitfields![u32,
GPIOx_STATUS [
    /// interrupt to processors, after override is applied
    IRQTOPROC OFFSET(26) NUMBITS(1) [],
    /// input signal from pad, before filtering and override are applied
    INFROMPAD OFFSET(17) NUMBITS(1) [],
    /// output enable to pad after register override is applied
    OETOPAD OFFSET(13) NUMBITS(1) [],
    /// output signal to pad after register override is applied
    OUTTOPAD OFFSET(9) NUMBITS(1) []
],
GPIOx_CTRL [

    IRQOVER OFFSET(28) NUMBITS(2) [
        /// don't invert the interrupt
        DoNotInvertTheInterrupt = 0,
        /// invert the interrupt
        InvertTheInterrupt = 1,
        /// drive interrupt low
        DriveInterruptLow = 2,
        /// drive interrupt high
        DriveInterruptHigh = 3
    ],

    INOVER OFFSET(16) NUMBITS(2) [
        /// don't invert the peri input
        DoNotInvertThePeriInput = 0,
        /// invert the peri input
        InvertThePeriInput = 1,
        /// drive peri input low
        DrivePeriInputLow = 2,
        /// drive peri input high
        DrivePeriInputHigh = 3
    ],

    OEOVER OFFSET(14) NUMBITS(2) [
        /// drive output enable from peripheral signal selected by funcsel
        DriveOutputEnableFromPeripheralSignalSelectedByFuncsel = 0,
        /// drive output enable from inverse of peripheral signal selected by funcsel
        DriveOutputEnableFromInverseOfPeripheralSignalSelectedByFuncsel = 1,
        /// disable output
        DisableOutput = 2,
        /// enable output
        EnableOutput = 3
    ],

    OUTOVER OFFSET(12) NUMBITS(2) [
        /// drive output from peripheral signal selected by funcsel
        DriveOutputFromPeripheralSignalSelectedByFuncsel = 0,
        /// drive output from inverse of peripheral signal selected by funcsel
        DriveOutputFromInverseOfPeripheralSignalSelectedByFuncsel = 1,
        /// drive output low
        DriveOutputLow = 2,
        /// drive output high
        DriveOutputHigh = 3
    ],
    /// 0-31 -> selects pin function according to the gpio table
/// 31 == NULL
    FUNCSEL OFFSET(0) NUMBITS(5) [

        Jtag_tck = 0
    ]
],
IRQSUMMARY_PROC0 [

    GPIO31 OFFSET(31) NUMBITS(1) [],

    GPIO30 OFFSET(30) NUMBITS(1) [],

    GPIO29 OFFSET(29) NUMBITS(1) [],

    GPIO28 OFFSET(28) NUMBITS(1) [],

    GPIO27 OFFSET(27) NUMBITS(1) [],

    GPIO26 OFFSET(26) NUMBITS(1) [],

    GPIO25 OFFSET(25) NUMBITS(1) [],

    GPIO24 OFFSET(24) NUMBITS(1) [],

    GPIO23 OFFSET(23) NUMBITS(1) [],

    GPIO22 OFFSET(22) NUMBITS(1) [],

    GPIO21 OFFSET(21) NUMBITS(1) [],

    GPIO20 OFFSET(20) NUMBITS(1) [],

    GPIO19 OFFSET(19) NUMBITS(1) [],

    GPIO18 OFFSET(18) NUMBITS(1) [],

    GPIO17 OFFSET(17) NUMBITS(1) [],

    GPIO16 OFFSET(16) NUMBITS(1) [],

    GPIO15 OFFSET(15) NUMBITS(1) [],

    GPIO14 OFFSET(14) NUMBITS(1) [],

    GPIO13 OFFSET(13) NUMBITS(1) [],

    GPIO12 OFFSET(12) NUMBITS(1) [],

    GPIO11 OFFSET(11) NUMBITS(1) [],

    GPIO10 OFFSET(10) NUMBITS(1) [],

    GPIO9 OFFSET(9) NUMBITS(1) [],

    GPIO8 OFFSET(8) NUMBITS(1) [],

    GPIO7 OFFSET(7) NUMBITS(1) [],

    GPIO6 OFFSET(6) NUMBITS(1) [],

    GPIO5 OFFSET(5) NUMBITS(1) [],

    GPIO4 OFFSET(4) NUMBITS(1) [],

    GPIO3 OFFSET(3) NUMBITS(1) [],

    GPIO2 OFFSET(2) NUMBITS(1) [],

    GPIO1 OFFSET(1) NUMBITS(1) [],

    GPIO0 OFFSET(0) NUMBITS(1) []
],
IRQSUMMARY_PROC1 [

    GPIO47 OFFSET(15) NUMBITS(1) [],

    GPIO46 OFFSET(14) NUMBITS(1) [],

    GPIO45 OFFSET(13) NUMBITS(1) [],

    GPIO44 OFFSET(12) NUMBITS(1) [],

    GPIO43 OFFSET(11) NUMBITS(1) [],

    GPIO42 OFFSET(10) NUMBITS(1) [],

    GPIO41 OFFSET(9) NUMBITS(1) [],

    GPIO40 OFFSET(8) NUMBITS(1) [],

    GPIO39 OFFSET(7) NUMBITS(1) [],

    GPIO38 OFFSET(6) NUMBITS(1) [],

    GPIO37 OFFSET(5) NUMBITS(1) [],

    GPIO36 OFFSET(4) NUMBITS(1) [],

    GPIO35 OFFSET(3) NUMBITS(1) [],

    GPIO34 OFFSET(2) NUMBITS(1) [],

    GPIO33 OFFSET(1) NUMBITS(1) [],

    GPIO32 OFFSET(0) NUMBITS(1) []
],
GPIO_INTRxx [

    GPIO7_EDGE_HIGH OFFSET(31) NUMBITS(1) [],

    GPIO7_EDGE_LOW OFFSET(30) NUMBITS(1) [],

    GPIO7_LEVEL_HIGH OFFSET(29) NUMBITS(1) [],

    GPIO7_LEVEL_LOW OFFSET(28) NUMBITS(1) [],

    GPIO6_EDGE_HIGH OFFSET(27) NUMBITS(1) [],

    GPIO6_EDGE_LOW OFFSET(26) NUMBITS(1) [],

    GPIO6_LEVEL_HIGH OFFSET(25) NUMBITS(1) [],

    GPIO6_LEVEL_LOW OFFSET(24) NUMBITS(1) [],

    GPIO5_EDGE_HIGH OFFSET(23) NUMBITS(1) [],

    GPIO5_EDGE_LOW OFFSET(22) NUMBITS(1) [],

    GPIO5_LEVEL_HIGH OFFSET(21) NUMBITS(1) [],

    GPIO5_LEVEL_LOW OFFSET(20) NUMBITS(1) [],

    GPIO4_EDGE_HIGH OFFSET(19) NUMBITS(1) [],

    GPIO4_EDGE_LOW OFFSET(18) NUMBITS(1) [],

    GPIO4_LEVEL_HIGH OFFSET(17) NUMBITS(1) [],

    GPIO4_LEVEL_LOW OFFSET(16) NUMBITS(1) [],

    GPIO3_EDGE_HIGH OFFSET(15) NUMBITS(1) [],

    GPIO3_EDGE_LOW OFFSET(14) NUMBITS(1) [],

    GPIO3_LEVEL_HIGH OFFSET(13) NUMBITS(1) [],

    GPIO3_LEVEL_LOW OFFSET(12) NUMBITS(1) [],

    GPIO2_EDGE_HIGH OFFSET(11) NUMBITS(1) [],

    GPIO2_EDGE_LOW OFFSET(10) NUMBITS(1) [],

    GPIO2_LEVEL_HIGH OFFSET(9) NUMBITS(1) [],

    GPIO2_LEVEL_LOW OFFSET(8) NUMBITS(1) [],

    GPIO1_EDGE_HIGH OFFSET(7) NUMBITS(1) [],

    GPIO1_EDGE_LOW OFFSET(6) NUMBITS(1) [],

    GPIO1_LEVEL_HIGH OFFSET(5) NUMBITS(1) [],

    GPIO1_LEVEL_LOW OFFSET(4) NUMBITS(1) [],

    GPIO0_EDGE_HIGH OFFSET(3) NUMBITS(1) [],

    GPIO0_EDGE_LOW OFFSET(2) NUMBITS(1) [],

    GPIO0_LEVEL_HIGH OFFSET(1) NUMBITS(1) [],

    GPIO0_LEVEL_LOW OFFSET(0) NUMBITS(1) []
],
VOLTAGE_SELECT [

    VOLTAGE_SELECT OFFSET(0) NUMBITS(1) [
        /// Set voltage to 3.3V (DVDD >= 2V5)
        SetVoltageTo33VDVDD2V5 = 0,
        /// Set voltage to 1.8V (DVDD <= 1V8)
        SetVoltageTo18VDVDD1V8 = 1
    ]
],
GPIO_PAD [
    /// Pad isolation control. Remove this once the pad is configured by software.
    ISO OFFSET(8) NUMBITS(1) [],
    /// Output disable. Has priority over output enable from peripherals
    OD OFFSET(7) NUMBITS(1) [],
    /// Input enable
    IE OFFSET(6) NUMBITS(1) [],
    /// Drive strength.
    DRIVE OFFSET(4) NUMBITS(2) [

        _2mA = 0,
        _4mA = 1,
        _8mA = 2,
        _12mA = 3
    ],
    /// Pull up enable
    PUE OFFSET(3) NUMBITS(1) [],
    /// Pull down enable
    PDE OFFSET(2) NUMBITS(1) [],
    /// Enable schmitt trigger
    SCHMITT OFFSET(1) NUMBITS(1) [],
    /// Slew rate control. 1 = Fast, 0 = Slow
    SLEWFAST OFFSET(0) NUMBITS(1) []
],
SWCLK [
    /// Pad isolation control. Remove this once the pad is configured by software.
    ISO OFFSET(8) NUMBITS(1) [],
    /// Output disable. Has priority over output enable from peripherals
    OD OFFSET(7) NUMBITS(1) [],
    /// Input enable
    IE OFFSET(6) NUMBITS(1) [],
    /// Drive strength.
    DRIVE OFFSET(4) NUMBITS(2) [

        _2mA = 0,
        _4mA = 1,
        _8mA = 2,
        _12mA = 3
    ],
    /// Pull up enable
    PUE OFFSET(3) NUMBITS(1) [],
    /// Pull down enable
    PDE OFFSET(2) NUMBITS(1) [],
    /// Enable schmitt trigger
    SCHMITT OFFSET(1) NUMBITS(1) [],
    /// Slew rate control. 1 = Fast, 0 = Slow
    SLEWFAST OFFSET(0) NUMBITS(1) []
],
SWD [
    /// Pad isolation control. Remove this once the pad is configured by software.
    ISO OFFSET(8) NUMBITS(1) [],
    /// Output disable. Has priority over output enable from peripherals
    OD OFFSET(7) NUMBITS(1) [],
    /// Input enable
    IE OFFSET(6) NUMBITS(1) [],
    /// Drive strength.
    DRIVE OFFSET(4) NUMBITS(2) [

        _2mA = 0,
        _4mA = 1,
        _8mA = 2,
        _12mA = 3
    ],
    /// Pull up enable
    PUE OFFSET(3) NUMBITS(1) [],
    /// Pull down enable
    PDE OFFSET(2) NUMBITS(1) [],
    /// Enable schmitt trigger
    SCHMITT OFFSET(1) NUMBITS(1) [],
    /// Slew rate control. 1 = Fast, 0 = Slow
    SLEWFAST OFFSET(0) NUMBITS(1) []
],
CPUID [
    /// Value is 0 when read from processor core 0, and 1 when read from processor core
    CPUID OFFSET(0) NUMBITS(32) []
],
GPIO_IN [

    GPIO_IN OFFSET(0) NUMBITS(32) []
],
GPIO_HI_IN [
    /// Input value on QSPI SD0 (MOSI), SD1 (MISO), SD2 and SD3 pins
    QSPI_SD OFFSET(28) NUMBITS(4) [],
    /// Input value on QSPI CSn pin
    QSPI_CSN OFFSET(27) NUMBITS(1) [],
    /// Input value on QSPI SCK pin
    QSPI_SCK OFFSET(26) NUMBITS(1) [],
    /// Input value on USB D- pin
    USB_DM OFFSET(25) NUMBITS(1) [],
    /// Input value on USB D+ pin
    USB_DP OFFSET(24) NUMBITS(1) [],
    /// Input value on GPIO32...47
    GPIO OFFSET(0) NUMBITS(16) []
],
GPIO_OUT [
    /// Set output level (1/0 -> high/low) for GPIO0...31. Reading back gives the last v
    /// If core 0 and core 1 both write to GPIO_OUT simultan
    /// In the Non-secure SIO, Secure-only GPIOs (as per ACC
    GPIO_OUT OFFSET(0) NUMBITS(32) []
],
GPIO_HI_OUT [
    /// Output value for QSPI SD0 (MOSI), SD1 (MISO), SD2 and SD3 pins
    QSPI_SD OFFSET(28) NUMBITS(4) [],
    /// Output value for QSPI CSn pin
    QSPI_CSN OFFSET(27) NUMBITS(1) [],
    /// Output value for QSPI SCK pin
    QSPI_SCK OFFSET(26) NUMBITS(1) [],
    /// Output value for USB D- pin
    USB_DM OFFSET(25) NUMBITS(1) [],
    /// Output value for USB D+ pin
    USB_DP OFFSET(24) NUMBITS(1) [],
    /// Output value for GPIO32...47
    GPIO OFFSET(0) NUMBITS(16) []
],
GPIO_OUT_SET [
    /// Perform an atomic bit-set on GPIO_OUT, i.e. `GPIO_OUT |= wdata`
    GPIO_OUT_SET OFFSET(0) NUMBITS(32) []
],
GPIO_HI_OUT_SET [

    QSPI_SD OFFSET(28) NUMBITS(4) [],

    QSPI_CSN OFFSET(27) NUMBITS(1) [],

    QSPI_SCK OFFSET(26) NUMBITS(1) [],

    USB_DM OFFSET(25) NUMBITS(1) [],

    USB_DP OFFSET(24) NUMBITS(1) [],

    GPIO OFFSET(0) NUMBITS(16) []
],
GPIO_OUT_CLR [
    /// Perform an atomic bit-clear on GPIO_OUT, i.e. `GPIO_OUT &= ~wdata`
    GPIO_OUT_CLR OFFSET(0) NUMBITS(32) []
],
GPIO_HI_OUT_CLR [

    QSPI_SD OFFSET(28) NUMBITS(4) [],

    QSPI_CSN OFFSET(27) NUMBITS(1) [],

    QSPI_SCK OFFSET(26) NUMBITS(1) [],

    USB_DM OFFSET(25) NUMBITS(1) [],

    USB_DP OFFSET(24) NUMBITS(1) [],

    GPIO OFFSET(0) NUMBITS(16) []
],
GPIO_OUT_XOR [
    /// Perform an atomic bitwise XOR on GPIO_OUT, i.e. `GPIO_OUT ^= wdata`
    GPIO_OUT_XOR OFFSET(0) NUMBITS(32) []
],
GPIO_HI_OUT_XOR [

    QSPI_SD OFFSET(28) NUMBITS(4) [],

    QSPI_CSN OFFSET(27) NUMBITS(1) [],

    QSPI_SCK OFFSET(26) NUMBITS(1) [],

    USB_DM OFFSET(25) NUMBITS(1) [],

    USB_DP OFFSET(24) NUMBITS(1) [],

    GPIO OFFSET(0) NUMBITS(16) []
],
GPIO_OE [
    /// Set output enable (1/0 -> output/input) for GPIO0...31. Reading back gives the l
    /// If core 0 and core 1 both write to GPIO_OE simultane
    /// In the Non-secure SIO, Secure-only GPIOs (as per ACC
    GPIO_OE OFFSET(0) NUMBITS(32) []
],
GPIO_HI_OE [
    /// Output enable value for QSPI SD0 (MOSI), SD1 (MISO), SD2 and SD3 pins
    QSPI_SD OFFSET(28) NUMBITS(4) [],
    /// Output enable value for QSPI CSn pin
    QSPI_CSN OFFSET(27) NUMBITS(1) [],
    /// Output enable value for QSPI SCK pin
    QSPI_SCK OFFSET(26) NUMBITS(1) [],
    /// Output enable value for USB D- pin
    USB_DM OFFSET(25) NUMBITS(1) [],
    /// Output enable value for USB D+ pin
    USB_DP OFFSET(24) NUMBITS(1) [],
    /// Output enable value for GPIO32...47
    GPIO OFFSET(0) NUMBITS(16) []
],
GPIO_OE_SET [
    /// Perform an atomic bit-set on GPIO_OE, i.e. `GPIO_OE |= wdata`
    GPIO_OE_SET OFFSET(0) NUMBITS(32) []
],
GPIO_HI_OE_SET [

    QSPI_SD OFFSET(28) NUMBITS(4) [],

    QSPI_CSN OFFSET(27) NUMBITS(1) [],

    QSPI_SCK OFFSET(26) NUMBITS(1) [],

    USB_DM OFFSET(25) NUMBITS(1) [],

    USB_DP OFFSET(24) NUMBITS(1) [],

    GPIO OFFSET(0) NUMBITS(16) []
],
GPIO_OE_CLR [
    /// Perform an atomic bit-clear on GPIO_OE, i.e. `GPIO_OE &= ~wdata`
    GPIO_OE_CLR OFFSET(0) NUMBITS(32) []
],
GPIO_HI_OE_CLR [

    QSPI_SD OFFSET(28) NUMBITS(4) [],

    QSPI_CSN OFFSET(27) NUMBITS(1) [],

    QSPI_SCK OFFSET(26) NUMBITS(1) [],

    USB_DM OFFSET(25) NUMBITS(1) [],

    USB_DP OFFSET(24) NUMBITS(1) [],

    GPIO OFFSET(0) NUMBITS(16) []
],
GPIO_OE_XOR [
    /// Perform an atomic bitwise XOR on GPIO_OE, i.e. `GPIO_OE ^= wdata`
    GPIO_OE_XOR OFFSET(0) NUMBITS(32) []
],
GPIO_HI_OE_XOR [

    QSPI_SD OFFSET(28) NUMBITS(4) [],

    QSPI_CSN OFFSET(27) NUMBITS(1) [],

    QSPI_SCK OFFSET(26) NUMBITS(1) [],

    USB_DM OFFSET(25) NUMBITS(1) [],

    USB_DP OFFSET(24) NUMBITS(1) [],

    GPIO OFFSET(0) NUMBITS(16) []
],
FIFO_ST [
    /// Sticky flag indicating the RX FIFO was read when empty. This read was ignored by
    ROE OFFSET(3) NUMBITS(1) [],
    /// Sticky flag indicating the TX FIFO was written when full. This write was ignored
    WOF OFFSET(2) NUMBITS(1) [],
    /// Value is 1 if this core's TX FIFO is not full (i.e. if FIFO_WR is ready for more
    RDY OFFSET(1) NUMBITS(1) [],
    /// Value is 1 if this core's RX FIFO is not empty (i.e. if FIFO_RD is valid)
    VLD OFFSET(0) NUMBITS(1) []
],
FIFO_WR [

    FIFO_WR OFFSET(0) NUMBITS(32) []
],
FIFO_RD [

    FIFO_RD OFFSET(0) NUMBITS(32) []
],
SPINLOCK_ST [

    SPINLOCK_ST OFFSET(0) NUMBITS(32) []
],
INTERP0_ACCUM0 [

    INTERP0_ACCUM0 OFFSET(0) NUMBITS(32) []
],
INTERP0_ACCUM1 [

    INTERP0_ACCUM1 OFFSET(0) NUMBITS(32) []
],
INTERP0_BASE0 [

    INTERP0_BASE0 OFFSET(0) NUMBITS(32) []
],
INTERP0_BASE1 [

    INTERP0_BASE1 OFFSET(0) NUMBITS(32) []
],
INTERP0_BASE2 [

    INTERP0_BASE2 OFFSET(0) NUMBITS(32) []
],
INTERP0_POP_LANE0 [

    INTERP0_POP_LANE0 OFFSET(0) NUMBITS(32) []
],
INTERP0_POP_LANE1 [

    INTERP0_POP_LANE1 OFFSET(0) NUMBITS(32) []
],
INTERP0_POP_FULL [

    INTERP0_POP_FULL OFFSET(0) NUMBITS(32) []
],
INTERP0_PEEK_LANE0 [

    INTERP0_PEEK_LANE0 OFFSET(0) NUMBITS(32) []
],
INTERP0_PEEK_LANE1 [

    INTERP0_PEEK_LANE1 OFFSET(0) NUMBITS(32) []
],
INTERP0_PEEK_FULL [

    INTERP0_PEEK_FULL OFFSET(0) NUMBITS(32) []
],
INTERP0_CTRL_LANE0 [
    /// Set if either OVERF0 or OVERF1 is set.
    OVERF OFFSET(25) NUMBITS(1) [],
    /// Indicates if any masked-off MSBs in ACCUM1 are set.
    OVERF1 OFFSET(24) NUMBITS(1) [],
    /// Indicates if any masked-off MSBs in ACCUM0 are set.
    OVERF0 OFFSET(23) NUMBITS(1) [],
    /// Only present on INTERP0 on each core. If BLEND mode is enabled:
    /// - LANE1 result is a linear interpolation between BAS
    /// by the 8 LSBs of lane 1 shift and mask value (a frac
    /// 0 and 255/256ths)
    /// - LANE0 result does not have BASE0 added (yields onl
    /// - FULL result does not have lane 1 shift+mask value
    /// LANE1 SIGNED flag controls whether the interpolation
    BLEND OFFSET(21) NUMBITS(1) [],
    /// ORed into bits 29:28 of the lane result presented to the processor on the bus.
    /// No effect on the internal 32-bit datapath. Handy for
    /// of pointers into flash or SRAM.
    FORCE_MSB OFFSET(19) NUMBITS(2) [],
    /// If 1, mask + shift is bypassed for LANE0 result. This does not affect FULL resul
    ADD_RAW OFFSET(18) NUMBITS(1) [],
    /// If 1, feed the opposite lane's result into this lane's accumulator on POP.
    CROSS_RESULT OFFSET(17) NUMBITS(1) [],
    /// If 1, feed the opposite lane's accumulator into this lane's shift + mask hardwar
    /// Takes effect even if ADD_RAW is set (the CROSS_INPUT
    CROSS_INPUT OFFSET(16) NUMBITS(1) [],
    /// If SIGNED is set, the shifted and masked accumulator value is sign-extended to 3
    /// before adding to BASE0, and LANE0 PEEK/POP appear ex
    SIGNED OFFSET(15) NUMBITS(1) [],
    /// The most-significant bit allowed to pass by the mask (inclusive)
    /// Setting MSB < LSB may cause chip to turn inside-out
    MASK_MSB OFFSET(10) NUMBITS(5) [],
    /// The least-significant bit allowed to pass by the mask (inclusive)
    MASK_LSB OFFSET(5) NUMBITS(5) [],
    /// Right-rotate applied to accumulator before masking. By appropriately configuring
    SHIFT OFFSET(0) NUMBITS(5) []
],
INTERP0_CTRL_LANE1 [
    /// ORed into bits 29:28 of the lane result presented to the processor on the bus.
    /// No effect on the internal 32-bit datapath. Handy for
    /// of pointers into flash or SRAM.
    FORCE_MSB OFFSET(19) NUMBITS(2) [],
    /// If 1, mask + shift is bypassed for LANE1 result. This does not affect FULL resul
    ADD_RAW OFFSET(18) NUMBITS(1) [],
    /// If 1, feed the opposite lane's result into this lane's accumulator on POP.
    CROSS_RESULT OFFSET(17) NUMBITS(1) [],
    /// If 1, feed the opposite lane's accumulator into this lane's shift + mask hardwar
    /// Takes effect even if ADD_RAW is set (the CROSS_INPUT
    CROSS_INPUT OFFSET(16) NUMBITS(1) [],
    /// If SIGNED is set, the shifted and masked accumulator value is sign-extended to 3
    /// before adding to BASE1, and LANE1 PEEK/POP appear ex
    SIGNED OFFSET(15) NUMBITS(1) [],
    /// The most-significant bit allowed to pass by the mask (inclusive)
    /// Setting MSB < LSB may cause chip to turn inside-out
    MASK_MSB OFFSET(10) NUMBITS(5) [],
    /// The least-significant bit allowed to pass by the mask (inclusive)
    MASK_LSB OFFSET(5) NUMBITS(5) [],
    /// Right-rotate applied to accumulator before masking. By appropriately configuring
    SHIFT OFFSET(0) NUMBITS(5) []
],
INTERP0_ACCUM0_ADD [

    INTERP0_ACCUM0_ADD OFFSET(0) NUMBITS(24) []
],
INTERP0_ACCUM1_ADD [

    INTERP0_ACCUM1_ADD OFFSET(0) NUMBITS(24) []
],
INTERP0_BASE_1AND0 [

    INTERP0_BASE_1AND0 OFFSET(0) NUMBITS(32) []
],
INTERP1_ACCUM0 [

    INTERP1_ACCUM0 OFFSET(0) NUMBITS(32) []
],
INTERP1_ACCUM1 [

    INTERP1_ACCUM1 OFFSET(0) NUMBITS(32) []
],
INTERP1_BASE0 [

    INTERP1_BASE0 OFFSET(0) NUMBITS(32) []
],
INTERP1_BASE1 [

    INTERP1_BASE1 OFFSET(0) NUMBITS(32) []
],
INTERP1_BASE2 [

    INTERP1_BASE2 OFFSET(0) NUMBITS(32) []
],
INTERP1_POP_LANE0 [

    INTERP1_POP_LANE0 OFFSET(0) NUMBITS(32) []
],
INTERP1_POP_LANE1 [

    INTERP1_POP_LANE1 OFFSET(0) NUMBITS(32) []
],
INTERP1_POP_FULL [

    INTERP1_POP_FULL OFFSET(0) NUMBITS(32) []
],
INTERP1_PEEK_LANE0 [

    INTERP1_PEEK_LANE0 OFFSET(0) NUMBITS(32) []
],
INTERP1_PEEK_LANE1 [

    INTERP1_PEEK_LANE1 OFFSET(0) NUMBITS(32) []
],
INTERP1_PEEK_FULL [

    INTERP1_PEEK_FULL OFFSET(0) NUMBITS(32) []
],
INTERP1_CTRL_LANE0 [
    /// Set if either OVERF0 or OVERF1 is set.
    OVERF OFFSET(25) NUMBITS(1) [],
    /// Indicates if any masked-off MSBs in ACCUM1 are set.
    OVERF1 OFFSET(24) NUMBITS(1) [],
    /// Indicates if any masked-off MSBs in ACCUM0 are set.
    OVERF0 OFFSET(23) NUMBITS(1) [],
    /// Only present on INTERP1 on each core. If CLAMP mode is enabled:
    /// - LANE0 result is shifted and masked ACCUM0, clamped
    /// BASE0 and an upper bound of BASE1.
    /// - Signedness of these comparisons is determined by L
    CLAMP OFFSET(22) NUMBITS(1) [],
    /// ORed into bits 29:28 of the lane result presented to the processor on the bus.
    /// No effect on the internal 32-bit datapath. Handy for
    /// of pointers into flash or SRAM.
    FORCE_MSB OFFSET(19) NUMBITS(2) [],
    /// If 1, mask + shift is bypassed for LANE0 result. This does not affect FULL resul
    ADD_RAW OFFSET(18) NUMBITS(1) [],
    /// If 1, feed the opposite lane's result into this lane's accumulator on POP.
    CROSS_RESULT OFFSET(17) NUMBITS(1) [],
    /// If 1, feed the opposite lane's accumulator into this lane's shift + mask hardwar
    /// Takes effect even if ADD_RAW is set (the CROSS_INPUT
    CROSS_INPUT OFFSET(16) NUMBITS(1) [],
    /// If SIGNED is set, the shifted and masked accumulator value is sign-extended to 3
    /// before adding to BASE0, and LANE0 PEEK/POP appear ex
    SIGNED OFFSET(15) NUMBITS(1) [],
    /// The most-significant bit allowed to pass by the mask (inclusive)
    /// Setting MSB < LSB may cause chip to turn inside-out
    MASK_MSB OFFSET(10) NUMBITS(5) [],
    /// The least-significant bit allowed to pass by the mask (inclusive)
    MASK_LSB OFFSET(5) NUMBITS(5) [],
    /// Right-rotate applied to accumulator before masking. By appropriately configuring
    SHIFT OFFSET(0) NUMBITS(5) []
],
INTERP1_CTRL_LANE1 [
    /// ORed into bits 29:28 of the lane result presented to the processor on the bus.
    /// No effect on the internal 32-bit datapath. Handy for
    /// of pointers into flash or SRAM.
    FORCE_MSB OFFSET(19) NUMBITS(2) [],
    /// If 1, mask + shift is bypassed for LANE1 result. This does not affect FULL resul
    ADD_RAW OFFSET(18) NUMBITS(1) [],
    /// If 1, feed the opposite lane's result into this lane's accumulator on POP.
    CROSS_RESULT OFFSET(17) NUMBITS(1) [],
    /// If 1, feed the opposite lane's accumulator into this lane's shift + mask hardwar
    /// Takes effect even if ADD_RAW is set (the CROSS_INPUT
    CROSS_INPUT OFFSET(16) NUMBITS(1) [],
    /// If SIGNED is set, the shifted and masked accumulator value is sign-extended to 3
    /// before adding to BASE1, and LANE1 PEEK/POP appear ex
    SIGNED OFFSET(15) NUMBITS(1) [],
    /// The most-significant bit allowed to pass by the mask (inclusive)
    /// Setting MSB < LSB may cause chip to turn inside-out
    MASK_MSB OFFSET(10) NUMBITS(5) [],
    /// The least-significant bit allowed to pass by the mask (inclusive)
    MASK_LSB OFFSET(5) NUMBITS(5) [],
    /// Right-rotate applied to accumulator before masking. By appropriately configuring
    SHIFT OFFSET(0) NUMBITS(5) []
],
INTERP1_ACCUM0_ADD [

    INTERP1_ACCUM0_ADD OFFSET(0) NUMBITS(24) []
],
INTERP1_ACCUM1_ADD [

    INTERP1_ACCUM1_ADD OFFSET(0) NUMBITS(24) []
],
INTERP1_BASE_1AND0 [

    INTERP1_BASE_1AND0 OFFSET(0) NUMBITS(32) []
],
SPINLOCK [

    SPINLOCK OFFSET(0) NUMBITS(32) []
],
DOORBELL_OUT_SET [

    DOORBELL_OUT_SET OFFSET(0) NUMBITS(8) []
],
DOORBELL_OUT_CLR [

    DOORBELL_OUT_CLR OFFSET(0) NUMBITS(8) []
],
DOORBELL_IN_SET [

    DOORBELL_IN_SET OFFSET(0) NUMBITS(8) []
],
DOORBELL_IN_CLR [

    DOORBELL_IN_CLR OFFSET(0) NUMBITS(8) []
],
PERI_NONSEC [
    /// IF 1, detach TMDS encoder (of this core) from the Secure SIO, and attach to the
    TMDS OFFSET(5) NUMBITS(1) [],
    /// If 1, detach interpolator 1 (of this core) from the Secure SIO, and attach to th
    INTERP1 OFFSET(1) NUMBITS(1) [],
    /// If 1, detach interpolator 0 (of this core) from the Secure SIO, and attach to th
    INTERP0 OFFSET(0) NUMBITS(1) []
],
RISCV_SOFTIRQ [
    /// Write 1 to atomically clear the core 1 software interrupt flag. Read to get the
    CORE1_CLR OFFSET(9) NUMBITS(1) [],
    /// Write 1 to atomically clear the core 0 software interrupt flag. Read to get the
    CORE0_CLR OFFSET(8) NUMBITS(1) [],
    /// Write 1 to atomically set the core 1 software interrupt flag. Read to get the st
    CORE1_SET OFFSET(1) NUMBITS(1) [],
    /// Write 1 to atomically set the core 0 software interrupt flag. Read to get the st
    CORE0_SET OFFSET(0) NUMBITS(1) []
],
MTIME_CTRL [
    /// If 1, the timer pauses when core 1 is in the debug halt state.
    DBGPAUSE_CORE1 OFFSET(3) NUMBITS(1) [],
    /// If 1, the timer pauses when core 0 is in the debug halt state.
    DBGPAUSE_CORE0 OFFSET(2) NUMBITS(1) [],
    /// If 1, increment the timer every cycle (i.e. run directly from the system clock),
    FULLSPEED OFFSET(1) NUMBITS(1) [],
    /// Timer enable bit. When 0, the timer will not increment automatically.
    EN OFFSET(0) NUMBITS(1) []
],
MTIME [

    MTIME OFFSET(0) NUMBITS(32) []
],
MTIMEH [

    MTIMEH OFFSET(0) NUMBITS(32) []
],
MTIMECMP [

    MTIMECMP OFFSET(0) NUMBITS(32) []
],
MTIMECMPH [

    MTIMECMPH OFFSET(0) NUMBITS(32) []
],
TMDS_CTRL [
    /// Clear the running DC balance state of the TMDS encoders. This bit should be writ
    CLEAR_BALANCE OFFSET(28) NUMBITS(1) [],
    /// When encoding two pixels's worth of symbols in one cycle (a read of a PEEK/POP_D
    /// This control disables that shift, so that both encod
    PIX2_NOSHIFT OFFSET(27) NUMBITS(1) [],
    /// Shift applied to the colour data register with each read of a POP alias register
    /// Reading from the POP_SINGLE register, or reading fro
    /// Reading from a POP_DOUBLE register when PIX2_NOSHIFT
    PIX_SHIFT OFFSET(24) NUMBITS(3) [
        /// Do not shift the colour data register.
        DoNotShiftTheColourDataRegister = 0,
        /// Shift the colour data register by 1 bit
        ShiftTheColourDataRegisterBy1Bit = 1,
        /// Shift the colour data register by 2 bits
        ShiftTheColourDataRegisterBy2Bits = 2,
        /// Shift the colour data register by 4 bits
        ShiftTheColourDataRegisterBy4Bits = 3,
        /// Shift the colour data register by 8 bits
        ShiftTheColourDataRegisterBy8Bits = 4,
        /// Shift the colour data register by 16 bits
        ShiftTheColourDataRegisterBy16Bits = 5
    ],
    /// Enable lane interleaving for reads of PEEK_SINGLE/POP_SINGLE.
    /// When interleaving is disabled, each of the 3 symbols
    /// When interleaving is enabled, the symbols are packed
    INTERLEAVE OFFSET(23) NUMBITS(1) [],
    /// Number of valid colour MSBs for lane 2 (1-8 bits, encoded as 0 through 7). Remai
    L2_NBITS OFFSET(18) NUMBITS(3) [],
    /// Number of valid colour MSBs for lane 1 (1-8 bits, encoded as 0 through 7). Remai
    L1_NBITS OFFSET(15) NUMBITS(3) [],
    /// Number of valid colour MSBs for lane 0 (1-8 bits, encoded as 0 through 7). Remai
    L0_NBITS OFFSET(12) NUMBITS(3) [],
    /// Right-rotate the 16 LSBs of the colour accumulator by 0-15 bits, in order to get
    /// For example, for RGB565 (red most significant), red
    L2_ROT OFFSET(8) NUMBITS(4) [],
    /// Right-rotate the 16 LSBs of the colour accumulator by 0-15 bits, in order to get
    /// For example, for RGB565, green is bits 10:5, so shou
    L1_ROT OFFSET(4) NUMBITS(4) [],
    /// Right-rotate the 16 LSBs of the colour accumulator by 0-15 bits, in order to get
    /// For example, for RGB565 (red most significant), blue
    L0_ROT OFFSET(0) NUMBITS(4) []
],
TMDS_WDATA [

    TMDS_WDATA OFFSET(0) NUMBITS(32) []
],
TMDS_PEEK_SINGLE [

    TMDS_PEEK_SINGLE OFFSET(0) NUMBITS(32) []
],
TMDS_POP_SINGLE [

    TMDS_POP_SINGLE OFFSET(0) NUMBITS(32) []
],
TMDS_PEEK_DOUBLE_L0 [

    TMDS_PEEK_DOUBLE_L0 OFFSET(0) NUMBITS(32) []
],
TMDS_POP_DOUBLE_L0 [

    TMDS_POP_DOUBLE_L0 OFFSET(0) NUMBITS(32) []
],
TMDS_PEEK_DOUBLE_L1 [

    TMDS_PEEK_DOUBLE_L1 OFFSET(0) NUMBITS(32) []
],
TMDS_POP_DOUBLE_L1 [

    TMDS_POP_DOUBLE_L1 OFFSET(0) NUMBITS(32) []
],
TMDS_PEEK_DOUBLE_L2 [

    TMDS_PEEK_DOUBLE_L2 OFFSET(0) NUMBITS(32) []
],
TMDS_POP_DOUBLE_L2 [

    TMDS_POP_DOUBLE_L2 OFFSET(0) NUMBITS(32) []
]
];
const GPIO_BASE: StaticRef<GpioRegisters> =
    unsafe { StaticRef::new(0x40028000 as *const GpioRegisters) };
const GPIO_PAD_BASE: StaticRef<GpioPadRegisters> =
    unsafe { StaticRef::new(0x40038000 as *const GpioPadRegisters) };
const SIO_BASE: StaticRef<SIORegisters> =
    unsafe { StaticRef::new(0xD0000000 as *const SIORegisters) };

pub struct RPPins<'a> {
    pub pins: [RPGpioPin<'a>; 30],
    gpio_registers: StaticRef<GpioRegisters>,
}

impl<'a> RPPins<'a> {
    pub const fn new() -> Self {
        Self {
            pins: [
                RPGpioPin::new(RPGpio::GPIO0),
                RPGpioPin::new(RPGpio::GPIO1),
                RPGpioPin::new(RPGpio::GPIO2),
                RPGpioPin::new(RPGpio::GPIO3),
                RPGpioPin::new(RPGpio::GPIO4),
                RPGpioPin::new(RPGpio::GPIO5),
                RPGpioPin::new(RPGpio::GPIO6),
                RPGpioPin::new(RPGpio::GPIO7),
                RPGpioPin::new(RPGpio::GPIO8),
                RPGpioPin::new(RPGpio::GPIO9),
                RPGpioPin::new(RPGpio::GPIO10),
                RPGpioPin::new(RPGpio::GPIO11),
                RPGpioPin::new(RPGpio::GPIO12),
                RPGpioPin::new(RPGpio::GPIO13),
                RPGpioPin::new(RPGpio::GPIO14),
                RPGpioPin::new(RPGpio::GPIO15),
                RPGpioPin::new(RPGpio::GPIO16),
                RPGpioPin::new(RPGpio::GPIO17),
                RPGpioPin::new(RPGpio::GPIO18),
                RPGpioPin::new(RPGpio::GPIO19),
                RPGpioPin::new(RPGpio::GPIO20),
                RPGpioPin::new(RPGpio::GPIO21),
                RPGpioPin::new(RPGpio::GPIO22),
                RPGpioPin::new(RPGpio::GPIO23),
                RPGpioPin::new(RPGpio::GPIO24),
                RPGpioPin::new(RPGpio::GPIO25),
                RPGpioPin::new(RPGpio::GPIO26),
                RPGpioPin::new(RPGpio::GPIO27),
                RPGpioPin::new(RPGpio::GPIO28),
                RPGpioPin::new(RPGpio::GPIO29),
            ],
            gpio_registers: GPIO_BASE,
        }
    }

    pub fn get_pin(&self, pin: RPGpio) -> &'a RPGpioPin {
        &self.pins[pin as usize]
    }

    pub fn handle_interrupt(&self) {
        for bank_no in 0..4 {
            let current_val = self.gpio_registers.intr[bank_no].get();
            let enabled_val = self.gpio_registers.interrupt_proc[0].enable[bank_no].get();
            for pin in 0..8 {
                let l_low_reg_no = pin * 4;
                if (current_val & enabled_val & (1 << l_low_reg_no)) != 0 {
                    self.pins[pin + bank_no * 8].handle_interrupt();
                } else if (current_val & enabled_val & (1 << (l_low_reg_no + 1))) != 0 {
                    self.pins[pin + bank_no * 8].handle_interrupt();
                } else if (current_val & enabled_val & (1 << (l_low_reg_no + 2))) != 0 {
                    self.gpio_registers.intr[bank_no].set(current_val & (1 << (l_low_reg_no + 2)));
                    self.pins[pin + bank_no * 8].handle_interrupt();
                } else if (current_val & enabled_val & (1 << (l_low_reg_no + 3))) != 0 {
                    self.gpio_registers.intr[bank_no].set(current_val & (1 << (l_low_reg_no + 3)));
                    self.pins[pin + bank_no * 8].handle_interrupt();
                }
            }
        }
    }
}

enum_from_primitive! {
    #[derive(Copy, Clone, PartialEq)]
    #[repr(usize)]
    #[rustfmt::skip]
    pub enum RPGpio {
        GPIO0=0, GPIO1=1, GPIO2=2, GPIO3=3, GPIO4=4, GPIO5=5, GPIO6=6, GPIO7=7,
        GPIO8=8, GPIO9=9, GPIO10=10, GPIO11=11, GPIO12=12, GPIO13=13, GPIO14=14, GPIO15=15,
        GPIO16=16, GPIO17=17, GPIO18=18, GPIO19=19, GPIO20=20, GPIO21=21, GPIO22=22, GPIO23=23,
        GPIO24=24, GPIO25=25, GPIO26=26, GPIO27=27, GPIO28=28, GPIO29=29
    }
}
enum_from_primitive! {
    #[derive(Copy, Clone, PartialEq)]
    #[repr(u32)]
    #[rustfmt::skip]

    pub enum GpioFunction {
       SPI = 1,
       UART = 2,
       I2C = 3,
       PWM = 4,
       SIO = 5,
       PIO0 = 6,
       PIO1 = 7,
       PIO2 = 8,
       XIP = 9,
       USB = 0xa,
       NULL = 0x1f
    }
}

pub struct RPGpioPin<'a> {
    pin: usize,
    client: OptionalCell<&'a dyn hil::gpio::Client>,
    gpio_registers: StaticRef<GpioRegisters>,
    gpio_pad_registers: StaticRef<GpioPadRegisters>,
    sio_registers: StaticRef<SIORegisters>,
}

#[allow(dead_code)]
impl<'a> RPGpioPin<'a> {
    pub const fn new(pin: RPGpio) -> RPGpioPin<'a> {
        RPGpioPin {
            pin: pin as usize,
            client: OptionalCell::empty(),
            gpio_registers: GPIO_BASE,
            gpio_pad_registers: GPIO_PAD_BASE,
            sio_registers: SIO_BASE,
        }
    }

    fn get_mode(&self) -> hil::gpio::Configuration {
        //TODO - read alternate function
        let pad_output_disable = !self.gpio_pad_registers.gpio_pad[self.pin].is_set(GPIO_PAD::OD);
        let pin_mask = 1 << self.pin;
        let sio_output_enable = (self.sio_registers.gpio_oe.read(GPIO_OE::GPIO_OE) & pin_mask) != 0;

        match (pad_output_disable, sio_output_enable) {
            (true, true) => hil::gpio::Configuration::Output,
            (true, false) => hil::gpio::Configuration::Input,
            (false, _) => hil::gpio::Configuration::LowPower,
        }
    }

    fn read_pin(&self) -> bool {
        //TODO - read alternate function
        let value = self.sio_registers.gpio_out.read(GPIO_OUT::GPIO_OUT) & (1 << self.pin);
        value != 0
    }

    pub fn set_function(&self, f: GpioFunction) {
        self.activate_pads();
        self.gpio_registers.pin[self.pin]
            .ctrl
            .write(GPIOx_CTRL::FUNCSEL.val(f as u32));

        // Remove the pad isolation
        self.gpio_pad_registers.gpio_pad[self.pin].modify(GPIO_PAD::ISO::CLEAR);
    }

    fn get_pullup_pulldown(&self) -> hil::gpio::FloatingState {
        //TODO - read alternate function
        let pullup = self.gpio_pad_registers.gpio_pad[self.pin].read(GPIO_PAD::PUE);
        let pulldown = self.gpio_pad_registers.gpio_pad[self.pin].read(GPIO_PAD::PDE);

        match (pullup, pulldown) {
            (0, 0) => hil::gpio::FloatingState::PullNone,
            (0, 1) => hil::gpio::FloatingState::PullDown,
            (1, 0) => hil::gpio::FloatingState::PullUp,
            _ => panic!("Invalid GPIO floating state."),
        }
    }

    pub fn activate_pads(&self) {
        self.gpio_pad_registers.gpio_pad[self.pin].modify(GPIO_PAD::OD::CLEAR + GPIO_PAD::IE::SET);
    }

    pub fn deactivate_pads(&self) {
        self.gpio_pad_registers.gpio_pad[self.pin].modify(GPIO_PAD::OD::SET + GPIO_PAD::IE::CLEAR);
    }

    pub fn handle_interrupt(&self) {
        self.client.map(|client| client.fired());
    }

    pub fn make_output(&self) {
        self.set_function(GpioFunction::SIO);
        self.activate_pads();
        self.sio_registers.gpio_oe_set.set(1 << self.pin);
    }

    pub fn set_pin(&self) {
        self.sio_registers.gpio_out_set.set(1 << self.pin);
    }
}

impl<'a> hil::gpio::Interrupt<'a> for RPGpioPin<'a> {
    fn set_client(&self, client: &'a dyn hil::gpio::Client) {
        self.client.set(client);
    }

    fn is_pending(&self) -> bool {
        let interrupt_bank_no = self.pin / 8;
        let l_low_reg_no = (self.pin * 4) % 32;
        let current_val = self.gpio_registers.interrupt_proc[0].status[interrupt_bank_no].get();
        (current_val
            & (1 << l_low_reg_no)
            & (1 << (l_low_reg_no + 1))
            & (1 << (l_low_reg_no + 2))
            & (1 << (l_low_reg_no + 3)))
            != 0
    }

    fn enable_interrupts(&self, mode: hil::gpio::InterruptEdge) {
        let interrupt_bank_no = self.pin / 8;
        match mode {
            hil::gpio::InterruptEdge::RisingEdge => {
                let high_reg_no = (self.pin * 4 + 3) % 32;
                let current_val =
                    self.gpio_registers.interrupt_proc[0].enable[interrupt_bank_no].get();
                self.gpio_registers.interrupt_proc[0].enable[interrupt_bank_no]
                    .set((1 << high_reg_no) | current_val);
            }
            hil::gpio::InterruptEdge::FallingEdge => {
                let low_reg_no = (self.pin * 4 + 2) % 32;
                let current_val =
                    self.gpio_registers.interrupt_proc[0].enable[interrupt_bank_no].get();
                self.gpio_registers.interrupt_proc[0].enable[interrupt_bank_no]
                    .set((1 << low_reg_no) | current_val);
            }
            hil::gpio::InterruptEdge::EitherEdge => {
                let low_reg_no = (self.pin * 4 + 2) % 32;
                let high_reg_no = low_reg_no + 1;
                let current_val =
                    self.gpio_registers.interrupt_proc[0].enable[interrupt_bank_no].get();
                self.gpio_registers.interrupt_proc[0].enable[interrupt_bank_no]
                    .set((1 << high_reg_no) | (1 << low_reg_no) | current_val);
            }
        }
    }

    fn disable_interrupts(&self) {
        let interrupt_bank_no = self.pin / 8;
        let low_reg_no = (self.pin * 4 + 2) % 32;
        let high_reg_no = low_reg_no + 1;
        let current_val = self.gpio_registers.interrupt_proc[0].enable[interrupt_bank_no].get();
        self.gpio_registers.interrupt_proc[0].enable[interrupt_bank_no]
            .set(current_val & !(1 << high_reg_no) & !(1 << low_reg_no));
    }
}

impl hil::gpio::Configure for RPGpioPin<'_> {
    fn configuration(&self) -> hil::gpio::Configuration {
        self.get_mode()
    }
    /// Set output mode
    fn make_output(&self) -> hil::gpio::Configuration {
        self.set_function(GpioFunction::SIO);
        self.activate_pads();
        self.sio_registers.gpio_oe_set.set(1 << self.pin);
        self.get_mode()
    }
    /// Disable pad output
    fn disable_output(&self) -> hil::gpio::Configuration {
        self.set_function(GpioFunction::SIO);
        self.gpio_pad_registers.gpio_pad[self.pin].modify(GPIO_PAD::OD::SET);
        self.get_mode()
    }
    /// Set input mode
    fn make_input(&self) -> hil::gpio::Configuration {
        self.set_function(GpioFunction::SIO);
        self.activate_pads();
        self.sio_registers.gpio_oe_clr.set(1 << self.pin);
        self.get_mode()
    }
    /// Disable input mode, will set pin to output mode
    fn disable_input(&self) -> hil::gpio::Configuration {
        self.make_output();
        self.get_mode()
    }
    fn deactivate_to_low_power(&self) {
        self.set_function(GpioFunction::SIO);
        self.gpio_pad_registers.gpio_pad[self.pin].modify(GPIO_PAD::OD::SET);
    }

    fn set_floating_state(&self, mode: hil::gpio::FloatingState) {
        match mode {
            hil::gpio::FloatingState::PullUp => self.gpio_pad_registers.gpio_pad[self.pin]
                .modify(GPIO_PAD::PUE::SET + GPIO_PAD::PDE::CLEAR),
            hil::gpio::FloatingState::PullDown => self.gpio_pad_registers.gpio_pad[self.pin]
                .modify(GPIO_PAD::PUE::CLEAR + GPIO_PAD::PDE::SET),
            hil::gpio::FloatingState::PullNone => self.gpio_pad_registers.gpio_pad[self.pin]
                .modify(GPIO_PAD::PUE::CLEAR + GPIO_PAD::PDE::CLEAR),
        }
    }

    fn floating_state(&self) -> hil::gpio::FloatingState {
        self.get_pullup_pulldown()
    }

    fn is_input(&self) -> bool {
        let mode = self.get_mode();
        match mode {
            hil::gpio::Configuration::Input => true,
            hil::gpio::Configuration::InputOutput => true,
            _ => false,
        }
    }

    fn is_output(&self) -> bool {
        let mode = self.get_mode();
        match mode {
            hil::gpio::Configuration::Output => true,
            hil::gpio::Configuration::InputOutput => true,
            _ => false,
        }
    }
}

impl hil::gpio::Output for RPGpioPin<'_> {
    fn set(&self) {
        // For performance this match might be skipped
        match self.get_mode() {
            hil::gpio::Configuration::Output | hil::gpio::Configuration::InputOutput => {
                self.sio_registers.gpio_out_set.set(1 << self.pin);
            }
            _ => {}
        }
    }

    fn clear(&self) {
        // For performance this match might be skipped
        match self.get_mode() {
            hil::gpio::Configuration::Output | hil::gpio::Configuration::InputOutput => {
                self.sio_registers.gpio_out_clr.set(1 << self.pin);
            }
            _ => {}
        }
    }

    fn toggle(&self) -> bool {
        match self.get_mode() {
            hil::gpio::Configuration::Output | hil::gpio::Configuration::InputOutput => {
                self.sio_registers.gpio_out_xor.set(1 << self.pin);
            }
            _ => {}
        }
        self.read_pin()
    }
}

impl hil::gpio::Input for RPGpioPin<'_> {
    fn read(&self) -> bool {
        let value = self.sio_registers.gpio_in.read(GPIO_IN::GPIO_IN) & (1 << self.pin);
        value != 0
    }
}

pub struct SIO {
    registers: StaticRef<SIORegisters>,
}

impl SIO {
    pub const fn new() -> Self {
        Self {
            registers: SIO_BASE,
        }
    }

    pub fn handle_proc_interrupt(&self, for_processor: Processor) {
        match for_processor {
            Processor::Processor0 => {
                // read data from the fifo
                self.registers.fifo_rd.get();
                self.registers.fifo_st.set(0xff);
            }
            Processor::Processor1 => {
                if self.registers.cpuid.get() == 1 {
                    panic!("Kernel should not run on processor 1");
                } else {
                    panic!("SIO_PROC1_IRQ should be ignored for processor 1");
                }
            }
        }
    }

    pub fn get_processor(&self) -> Processor {
        let proc_id = self.registers.cpuid.get();
        match proc_id {
            0 => Processor::Processor0,
            1 => Processor::Processor1,
            _ => panic!("SIO CPUID cannot be {}", proc_id),
        }
    }
}
