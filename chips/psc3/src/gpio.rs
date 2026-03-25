use kernel::hil::gpio::{Configuration, Configure, Input, Interrupt, Output};
use kernel::utilities::cells::OptionalCell;
use kernel::utilities::registers::{
    interfaces::{ReadWriteable, Readable},
    register_bitfields, register_structs, ReadOnly, ReadWrite,
};
use kernel::utilities::StaticRef;

#[repr(C)]
struct GpioPort {
    /// Port output data register
    prt_out: ReadWrite<u32, PRT_OUT::Register>,

    /// Port output data clear register
    prt_out_clr: ReadWrite<u32, PRT_OUT::Register>,
    /// Port output data set register
    prt_out_set: ReadWrite<u32, PRT_OUT::Register>,
    /// Port output data invert register
    prt_out_inv: ReadWrite<u32, PRT_OUT::Register>,
    /// Port input state register
    prt_in: ReadOnly<u32, PRT_IN::Register>,
    /// Port interrupt status register
    prt_intr: ReadWrite<u32, PRT_INTR::Register>,
    /// Port interrupt mask register
    prt_intr_mask: ReadWrite<u32, PRT_INTR::Register>,
    /// Port interrupt masked status register
    prt_intr_masked: ReadOnly<u32, PRT_INTR::Register>,
    /// Port interrupt set register
    prt_intr_set: ReadWrite<u32, PRT_INTR::Register>,
    _reserved0: [u32; 7], //0x24 - 0x40
    /// Port interrupt configuration register
    prt_intr_cfg: ReadWrite<u32, PRT_INTR_CFG::Register>,
    /// Port configuration register
    prt_cfg: ReadWrite<u32, PRT_CFG::Register>,
    /// Port input buffer configuration register
    prt_cfg_in: ReadWrite<u32, PRT_CFG_IN::Register>,
    /// Port output buffer configuration register
    prt_cfg_out: ReadWrite<u32, PRT_CFG_OUT::Register>,
    /// Port SIO configuration register
    prt_cfg_sio: ReadWrite<u32, PRT_CFG_SIO::Register>,
    _reserved1: [u32; 1], // 0x54-0x58
    /// Port input buffer AUTOLVL configuration register for S40E GPIO
    prt_cfg_in_autolvl: ReadWrite<u32, PRT_CFG_IN_AUTOLVL::Register>,
    _reserved2: [u32; 1], // 0x5C-0x60
    /// Port output buffer configuration register 2
    prt_0_cfg_out2: ReadWrite<u32, PRT_CFG_OUT2::Register>,
    /// Port output buffer drive sel extension configuration register
    prt_0_cfg_slew_ext: ReadWrite<u32, PRT_CFG_SLEW_EXT::Register>,
    /// Port output buffer drive sel extension configuration register
    prt_0_cfg_drive_ext0: ReadWrite<u32, PRT_CFG_DRIVE_EXT0::Register>,
    /// Port output buffer drive sel extension configuration register
    prt_0_cfg_drive_ext1: ReadWrite<u32, PRT_CFG_DRIVE_EXT1::Register>,
    _reserved3: [u32; 4], // 0x70-0x80
}

register_structs! {
    /// GPIO port control/configuration
    GpioRegisters {
        (0x000 => ports: [GpioPort; 10]),
        (0x500 => _reserved0),
        /// Secure Interrupt port cause register 0
        (0x7000 => sec_intr_cause0: ReadWrite<u32, SEC_INTR_CAUSE0::Register>),
        (0x7004 => _reserved1),
        /// Interrupt port cause register 0
        (0x8000 => intr_cause0: ReadWrite<u32, INTR_CAUSE0::Register>),
        (0x8004 => _reserved2),
        /// Extern power supply detection register
        (0x8010 => vdd_active: ReadWrite<u32, VDD_ACTIVE::Register>),
        /// Supply detection interrupt register
        (0x8014 => vdd_intr: ReadWrite<u32, VDD_INTR::Register>),
        /// Supply detection interrupt mask register
        (0x8018 => vdd_intr_mask: ReadWrite<u32, VDD_INTR_MASK::Register>),
        /// Supply detection interrupt masked register
        (0x801C => vdd_intr_masked: ReadWrite<u32, VDD_INTR_MASKED::Register>),
        /// Supply detection interrupt set register
        (0x8020 => vdd_intr_set: ReadWrite<u32, VDD_INTR_SET::Register>),
        (0x8024 => @END),
    }
}
register_bitfields![u32,
SEC_INTR_CAUSE0 [
    /// Each IO port has an associated bit field in this register. The bit field reflects the IO port's interrupt line (bit field i reflects 'gpio_interrupts[i]' for IO port i). The register is used when the system uses a combined interrupt line 'gpio_interrupt'. The software ISR reads the register to determine which IO port(s) is responsible for the combined interrupt line. Once, the IO port(s) is determined, the IO port's GPIO_PRT_INTR register is read to determine the IO pin(s) in the IO port that caused the interrupt.
    /// '0': Port has no pending interrupt
    /// '1': Port has pending interrupt
    PORT_INT OFFSET(0) NUMBITS(32) []
],
INTR_CAUSE0 [
    /// Each IO port has an associated bit field in this register. The bit field reflects the IO port's interrupt line (bit field i reflects 'gpio_interrupts[i]' for IO port i). The register is used when the system uses a combined interrupt line 'gpio_interrupt'. The software ISR reads the register to determine which IO port(s) is responsible for the combined interrupt line. Once, the IO port(s) is determined, the IO port's GPIO_PRT_INTR register is read to determine the IO pin(s) in the IO port that caused the interrupt.
    /// '0': Port has no pending interrupt
    /// '1': Port has pending interrupt
    PORT_INT OFFSET(0) NUMBITS(32) []
],
VDD_ACTIVE [
    /// Indicates presence or absence of VDDIO supplies (i.e. other than VDDD, VDDA) on the device (supplies are numbered 0..n-1).  Note that VDDIO supplies have basic (crude) supply detectors only.  If separate, robust, brown-out detection is desired on IO supplies, on-chip or off-chip analog resources need to provide it.  For these bits to work reliable, the supply must be within valid spec range (per datasheet) or held at ground.  Any in-between voltage has an undefined result.
    /// '0': Supply is not present
    /// '1': Supply is present
    ///
    /// When multiple VDDIO supplies are present, they will be assigned in alphanumeric ascending order to these bits during implementation.
    /// For example 'vddusb, vddio_0, vddio_a, vbackup, vddio_r, vddio_1' are present then they will be assigned to these bits as below:
    /// 0: vbackup,
    /// 1: vddio_0,
    /// 2: vddio_1,
    /// 3: vddio_a,
    /// 4: vddio_r,
    /// 5: vddusb'
    VDDIO_ACTIVE OFFSET(0) NUMBITS(16) [],
    /// Same as VDDIO_ACTIVE for the analog supply VDDA.
    VDDA_ACTIVE OFFSET(30) NUMBITS(1) [],
    /// This bit indicates presence of the VDDD supply.  This bit will always read-back 1.  The VDDD supply has robust brown-out protection monitoring and it is not possible to read back this register without a valid supply. (This bit is used in certain test-modes to observe the brown-out detector status.)
    VDDD_ACTIVE OFFSET(31) NUMBITS(1) []
],
VDD_INTR [
    /// Supply state change detected.
    /// '0': No change to supply detected
    /// '1': Change to supply detected
    VDDIO_ACTIVE OFFSET(0) NUMBITS(16) [],
    /// Same as VDDIO_ACTIVE for the analog supply VDDA.
    VDDA_ACTIVE OFFSET(30) NUMBITS(1) [],
    /// The VDDD supply is always present during operation so a supply transition can not occur. This bit will always read back '1'.
    VDDD_ACTIVE OFFSET(31) NUMBITS(1) []
],
VDD_INTR_MASK [
    /// Masks supply interrupt on VDDIO.
    /// '0': VDDIO interrupt forwarding disabled
    /// '1': VDDIO interrupt forwarding enabled
    VDDIO_ACTIVE OFFSET(0) NUMBITS(16) [],
    /// Same as VDDIO_ACTIVE for the analog supply VDDA.
    VDDA_ACTIVE OFFSET(30) NUMBITS(1) [],
    /// Same as VDDIO_ACTIVE for the digital supply VDDD.
    VDDD_ACTIVE OFFSET(31) NUMBITS(1) []
],
VDD_INTR_MASKED [
    /// Supply transition detected AND masked
    /// '0': Interrupt was not forwarded to CPU
    /// '1': Interrupt occurred and was forwarded to CPU
    VDDIO_ACTIVE OFFSET(0) NUMBITS(16) [],
    /// Same as VDDIO_ACTIVE for the analog supply VDDA.
    VDDA_ACTIVE OFFSET(30) NUMBITS(1) [],
    /// Same as VDDIO_ACTIVE for the digital supply VDDD.
    VDDD_ACTIVE OFFSET(31) NUMBITS(1) []
],
VDD_INTR_SET [
    /// Sets supply interrupt.
    /// '0': Interrupt state not affected
    /// '1': Interrupt set
    VDDIO_ACTIVE OFFSET(0) NUMBITS(16) [],
    /// Same as VDDIO_ACTIVE for the analog supply VDDA.
    VDDA_ACTIVE OFFSET(30) NUMBITS(1) [],
    /// Same as VDDIO_ACTIVE for the digital supply VDDD.
    VDDD_ACTIVE OFFSET(31) NUMBITS(1) []
],
PRT_OUT [
    /// IO output data for pin 0
    /// '0': Output state set to '0'
    /// '1': Output state set to '1'
    OUT0 OFFSET(0) NUMBITS(1) [],
    /// IO output data for pin 1
    OUT1 OFFSET(1) NUMBITS(1) [],
    /// IO output data for pin 2
    OUT2 OFFSET(2) NUMBITS(1) [],
    /// IO output data for pin 3
    OUT3 OFFSET(3) NUMBITS(1) [],
    /// IO output data for pin 4
    OUT4 OFFSET(4) NUMBITS(1) [],
    /// IO output data for pin 5
    OUT5 OFFSET(5) NUMBITS(1) [],
    /// IO output data for pin 6
    OUT6 OFFSET(6) NUMBITS(1) [],
    /// IO output data for pin 7
    OUT7 OFFSET(7) NUMBITS(1) []
],
PRT_OUT_CLR [
    /// IO clear output for pin 0:
    /// '0': Output state not affected.
    /// '1': Output state set to '0'.
    OUT0 OFFSET(0) NUMBITS(1) [],
    /// IO clear output for pin 1
    OUT1 OFFSET(1) NUMBITS(1) [],
    /// IO clear output for pin 2
    OUT2 OFFSET(2) NUMBITS(1) [],
    /// IO clear output for pin 3
    OUT3 OFFSET(3) NUMBITS(1) [],
    /// IO clear output for pin 4
    OUT4 OFFSET(4) NUMBITS(1) [],
    /// IO clear output for pin 5
    OUT5 OFFSET(5) NUMBITS(1) [],
    /// IO clear output for pin 6
    OUT6 OFFSET(6) NUMBITS(1) [],
    /// IO clear output for pin 7
    OUT7 OFFSET(7) NUMBITS(1) []
],
PRT_OUT_SET [
    /// IO set output for pin 0:
    /// '0': Output state not affected.
    /// '1': Output state set to '1'.
    OUT0 OFFSET(0) NUMBITS(1) [],
    /// IO set output for pin 1
    OUT1 OFFSET(1) NUMBITS(1) [],
    /// IO set output for pin 2
    OUT2 OFFSET(2) NUMBITS(1) [],
    /// IO set output for pin 3
    OUT3 OFFSET(3) NUMBITS(1) [],
    /// IO set output for pin 4
    OUT4 OFFSET(4) NUMBITS(1) [],
    /// IO set output for pin 5
    OUT5 OFFSET(5) NUMBITS(1) [],
    /// IO set output for pin 6
    OUT6 OFFSET(6) NUMBITS(1) [],
    /// IO set output for pin 7
    OUT7 OFFSET(7) NUMBITS(1) []
],
PRT_OUT_INV [
    /// IO invert output for pin 0:
    /// '0': Output state not affected.
    /// '1': Output state inverted ('0' => '1', '1' => '0').
    OUT0 OFFSET(0) NUMBITS(1) [],
    /// IO invert output for pin 1
    OUT1 OFFSET(1) NUMBITS(1) [],
    /// IO invert output for pin 2
    OUT2 OFFSET(2) NUMBITS(1) [],
    /// IO invert output for pin 3
    OUT3 OFFSET(3) NUMBITS(1) [],
    /// IO invert output for pin 4
    OUT4 OFFSET(4) NUMBITS(1) [],
    /// IO invert output for pin 5
    OUT5 OFFSET(5) NUMBITS(1) [],
    /// IO invert output for pin 6
    OUT6 OFFSET(6) NUMBITS(1) [],
    /// IO invert output for pin 7
    OUT7 OFFSET(7) NUMBITS(1) []
],
PRT_IN [
    /// IO pin state for pin 0
    /// '0': Low logic level present on pin.
    /// '1': High logic level present on pin.
    /// On reset assertion , IN register will get reset. The Pad value takes 2 clock cycles to be reflected into IN Register.  The default value is transient.
    IN0 OFFSET(0) NUMBITS(1) [],
    /// IO pin state for pin 1
    IN1 OFFSET(1) NUMBITS(1) [],
    /// IO pin state for pin 2
    IN2 OFFSET(2) NUMBITS(1) [],
    /// IO pin state for pin 3
    IN3 OFFSET(3) NUMBITS(1) [],
    /// IO pin state for pin 4
    IN4 OFFSET(4) NUMBITS(1) [],
    /// IO pin state for pin 5
    IN5 OFFSET(5) NUMBITS(1) [],
    /// IO pin state for pin 6
    IN6 OFFSET(6) NUMBITS(1) [],
    /// IO pin state for pin 7
    IN7 OFFSET(7) NUMBITS(1) [],
    /// Reads of this register return the logical state of the filtered pin as selected in the INTR_CFG.FLT_SEL register.
    FLT_IN OFFSET(8) NUMBITS(1) []
],
PRT_INTR [
    /// Edge detect for IO pin 0
    /// '0': No edge was detected on pin.
    /// '1': An edge was detected on pin.
    EDGE0 OFFSET(0) NUMBITS(1) [],
    /// Edge detect for IO pin 1
    EDGE1 OFFSET(1) NUMBITS(1) [],
    /// Edge detect for IO pin 2
    EDGE2 OFFSET(2) NUMBITS(1) [],
    /// Edge detect for IO pin 3
    EDGE3 OFFSET(3) NUMBITS(1) [],
    /// Edge detect for IO pin 4
    EDGE4 OFFSET(4) NUMBITS(1) [],
    /// Edge detect for IO pin 5
    EDGE5 OFFSET(5) NUMBITS(1) [],
    /// Edge detect for IO pin 6
    EDGE6 OFFSET(6) NUMBITS(1) [],
    /// Edge detect for IO pin 7
    EDGE7 OFFSET(7) NUMBITS(1) [],
    /// Edge detected on filtered pin selected by INTR_CFG.FLT_SEL
    FLT_EDGE OFFSET(8) NUMBITS(1) [],
    /// IO pin state for pin 0
    IN_IN0 OFFSET(16) NUMBITS(1) [],
    /// IO pin state for pin 1
    IN_IN1 OFFSET(17) NUMBITS(1) [],
    /// IO pin state for pin 2
    IN_IN2 OFFSET(18) NUMBITS(1) [],
    /// IO pin state for pin 3
    IN_IN3 OFFSET(19) NUMBITS(1) [],
    /// IO pin state for pin 4
    IN_IN4 OFFSET(20) NUMBITS(1) [],
    /// IO pin state for pin 5
    IN_IN5 OFFSET(21) NUMBITS(1) [],
    /// IO pin state for pin 6
    IN_IN6 OFFSET(22) NUMBITS(1) [],
    /// IO pin state for pin 7
    IN_IN7 OFFSET(23) NUMBITS(1) [],
    /// Filtered pin state for pin selected by INTR_CFG.FLT_SEL
    FLT_IN_IN OFFSET(24) NUMBITS(1) []
],
PRT_INTR_MASK [
    /// Masks edge interrupt on IO pin 0
    /// '0': Pin interrupt forwarding disabled
    /// '1': Pin interrupt forwarding enabled
    EDGE0 OFFSET(0) NUMBITS(1) [],
    /// Masks edge interrupt on IO pin 1
    EDGE1 OFFSET(1) NUMBITS(1) [],
    /// Masks edge interrupt on IO pin 2
    EDGE2 OFFSET(2) NUMBITS(1) [],
    /// Masks edge interrupt on IO pin 3
    EDGE3 OFFSET(3) NUMBITS(1) [],
    /// Masks edge interrupt on IO pin 4
    EDGE4 OFFSET(4) NUMBITS(1) [],
    /// Masks edge interrupt on IO pin 5
    EDGE5 OFFSET(5) NUMBITS(1) [],
    /// Masks edge interrupt on IO pin 6
    EDGE6 OFFSET(6) NUMBITS(1) [],
    /// Masks edge interrupt on IO pin 7
    EDGE7 OFFSET(7) NUMBITS(1) [],
    /// Masks edge interrupt on filtered pin selected by INTR_CFG.FLT_SEL
    FLT_EDGE OFFSET(8) NUMBITS(1) []
],
PRT_INTR_MASKED [
    /// Edge detected AND masked on IO pin 0
    /// '0': Interrupt was not forwarded to CPU
    /// '1': Interrupt occurred and was forwarded to CPU
    EDGE0 OFFSET(0) NUMBITS(1) [],
    /// Edge detected and masked on IO pin 1
    EDGE1 OFFSET(1) NUMBITS(1) [],
    /// Edge detected and masked on IO pin 2
    EDGE2 OFFSET(2) NUMBITS(1) [],
    /// Edge detected and masked on IO pin 3
    EDGE3 OFFSET(3) NUMBITS(1) [],
    /// Edge detected and masked on IO pin 4
    EDGE4 OFFSET(4) NUMBITS(1) [],
    /// Edge detected and masked on IO pin 5
    EDGE5 OFFSET(5) NUMBITS(1) [],
    /// Edge detected and masked on IO pin 6
    EDGE6 OFFSET(6) NUMBITS(1) [],
    /// Edge detected and masked on IO pin 7
    EDGE7 OFFSET(7) NUMBITS(1) [],
    /// Edge detected and masked on filtered pin selected by INTR_CFG.FLT_SEL
    FLT_EDGE OFFSET(8) NUMBITS(1) []
],
PRT_INTR_SET [
    /// Sets edge detect interrupt for IO pin 0
    /// '0': Interrupt state not affected
    /// '1': Interrupt set
    EDGE0 OFFSET(0) NUMBITS(1) [],
    /// Sets edge detect interrupt for IO pin 1
    EDGE1 OFFSET(1) NUMBITS(1) [],
    /// Sets edge detect interrupt for IO pin 2
    EDGE2 OFFSET(2) NUMBITS(1) [],
    /// Sets edge detect interrupt for IO pin 3
    EDGE3 OFFSET(3) NUMBITS(1) [],
    /// Sets edge detect interrupt for IO pin 4
    EDGE4 OFFSET(4) NUMBITS(1) [],
    /// Sets edge detect interrupt for IO pin 5
    EDGE5 OFFSET(5) NUMBITS(1) [],
    /// Sets edge detect interrupt for IO pin 6
    EDGE6 OFFSET(6) NUMBITS(1) [],
    /// Sets edge detect interrupt for IO pin 7
    EDGE7 OFFSET(7) NUMBITS(1) [],
    /// Sets edge detect interrupt for filtered pin selected by INTR_CFG.FLT_SEL
    FLT_EDGE OFFSET(8) NUMBITS(1) []
],
PRT_INTR_CFG [
    /// Sets which edge will trigger an IRQ for IO pin 0
    EDGE0_SEL OFFSET(0) NUMBITS(2) [],
    /// Sets which edge will trigger an IRQ for IO pin 1
    EDGE1_SEL OFFSET(2) NUMBITS(2) [],
    /// Sets which edge will trigger an IRQ for IO pin 2
    EDGE2_SEL OFFSET(4) NUMBITS(2) [],
    /// Sets which edge will trigger an IRQ for IO pin 3
    EDGE3_SEL OFFSET(6) NUMBITS(2) [],
    /// Sets which edge will trigger an IRQ for IO pin 4
    EDGE4_SEL OFFSET(8) NUMBITS(2) [],
    /// Sets which edge will trigger an IRQ for IO pin 5
    EDGE5_SEL OFFSET(10) NUMBITS(2) [],
    /// Sets which edge will trigger an IRQ for IO pin 6
    EDGE6_SEL OFFSET(12) NUMBITS(2) [],
    /// Sets which edge will trigger an IRQ for IO pin 7
    EDGE7_SEL OFFSET(14) NUMBITS(2) [],
    /// Sets which edge will trigger an IRQ for the glitch filtered pin (selected by INTR_CFG.FLT_SEL
    FLT_EDGE_SEL OFFSET(16) NUMBITS(2) [
        /// Disabled
        Disabled = 0,
        /// Rising edge
        RisingEdge = 1,
        /// Falling edge
        FallingEdge = 2,
        /// Both rising and falling edges
        BothRisingAndFallingEdges = 3
    ],
    /// Selects which pin is routed through the 50ns glitch filter to provide a glitch-safe interrupt.
    FLT_SEL OFFSET(18) NUMBITS(3) []
],
PRT_CFG [
    /// The GPIO drive mode for IO pin 0. Resistive pull-up and pull-down is selected in the drive mode.
    /// Note: when initializing IO's that are connected to a live bus (such as I2C), make sure the peripheral and HSIOM (HSIOM_PRT_SELx) is properly configured  before turning the IO on here to avoid producing glitches on the bus.
    /// Note: that peripherals other than GPIO & UDB/DSI directly control both the output and output-enable of the output buffer (peripherals can drive strong 0 or strong 1 in any mode except OFF='0').
    /// Note: D_OUT, D_OUT_EN are pins of GPIO cell.
    DRIVE_MODE0 OFFSET(0) NUMBITS(3) [
        /// Output buffer is off creating a high impedance input
        /// D_OUT = '0': High Impedance
        /// D_OUT = '1': High Impedance
        HIGHZ = 0,
        /// N/A
        NA = 1,
        /// Resistive pull up
        ///
        /// For GPIO & UDB/DSI peripherals:
        /// When D_OUT_EN = 1:
        ///    D_OUT = '0': Strong pull down
        ///    D_OUT = '1': Weak/resistive pull up
        /// When D_OUT_EN = 0:
        ///    D_OUT = '0': High impedance
        ///    D_OUT = '1': High impedance
        ///
        /// For peripherals other than GPIO & UDB/DSI:
        /// When D_OUT_EN = 1:
        ///    D_OUT = '0': Strong pull down
        ///    D_OUT = '1': Strong pull up
        /// When D_OUT_EN = 0:
        ///    D_OUT = '0': Weak/resistive pull up
        ///    D_OUT = '1': Weak/resistive pull up
        PULLUP = 2,
        /// Resistive pull down
        ///
        /// For GPIO & UDB/DSI peripherals:
        /// When D_OUT_EN = 1:
        ///    D_OUT = '0': Weak/resistive pull down
        ///    D_OUT = '1': Strong pull up
        /// When D_OUT_EN = 0:
        ///    D_OUT = '0': High impedance
        ///    D_OUT = '1': High impedance
        ///
        /// For peripherals other than GPIO & UDB/DSI:
        /// When D_OUT_EN = 1:
        ///    D_OUT = '0': Strong pull down
        ///    D_OUT = '1': Strong pull up
        /// When D_OUT_EN = 0:
        ///    D_OUT = '0': Weak/resistive pull down
        ///    D_OUT = '1': Weak/resistive pull down
        PULLDOWN = 3,
        /// Open drain, drives low
        ///
        /// For GPIO & UDB/DSI peripherals:
        /// When D_OUT_EN = 1:
        ///    D_OUT = '0': Strong pull down
        ///    D_OUT = '1': High Impedance
        /// When D_OUT_EN = 0:
        ///    D_OUT = '0': High impedance
        ///    D_OUT = '1': High impedance
        ///
        /// For peripherals other than GPIO & UDB/DSI:
        /// When D_OUT_EN = 1:
        ///    D_OUT = '0': Strong pull down
        ///    D_OUT = '1': Strong pull up
        /// When D_OUT_EN = 0:
        ///    D_OUT = '0': High Impedance
        ///    D_OUT = '1': High Impedance
        OD_DRIVESLOW = 4,
        /// Open drain, drives high
        ///
        /// For GPIO & UDB/DSI peripherals:
        /// When D_OUT_EN = 1:
        ///    D_OUT = '0': High Impedance
        ///    D_OUT = '1': Strong pull up
        /// When D_OUT_EN = 0:
        ///    D_OUT = '0': High impedance
        ///    D_OUT = '1': High impedance
        ///
        /// For peripherals other than GPIO & UDB/DSI:
        /// When D_OUT_EN = 1:
        ///    D_OUT = '0': Strong pull down
        ///    D_OUT = '1': Strong pull up
        /// When D_OUT_EN = 0:
        ///    D_OUT = '0': High Impedance
        ///    D_OUT = '1': High Impedance
        OD_DRIVESHIGH = 5,
        /// Strong D_OUTput buffer
        ///
        /// For GPIO & UDB/DSI peripherals:
        /// When D_OUT_EN = 1:
        ///    D_OUT = '0': Strong pull down
        ///    D_OUT = '1': Strong pull up
        /// When D_OUT_EN = 0:
        ///    D_OUT = '0': High impedance
        ///    D_OUT = '1': High impedance
        ///
        /// For peripherals other than GPIO & UDB/DSI:
        /// When D_OUT_EN = 1:
        ///    D_OUT = '0': Strong pull down
        ///    D_OUT = '1': Strong pull up
        /// When D_OUT_EN = 0:
        ///    D_OUT = '0': High Impedance
        ///    D_OUT = '1': High Impedance
        STRONG = 6,
        /// Pull up or pull down
        ///
        /// For GPIO & UDB/DSI peripherals:
        /// When D_OUT_EN = '0':
        ///     GPIO_DSI_OUT = '0': Weak/resistive pull down
        ///     GPIO_DSI_OUT = '1': Weak/resistive pull up
        /// where 'GPIO_DSI_OUT' is a function of PORT_SEL, OUT & DSI_DATA_OUT.
        ///
        /// For peripherals other than GPIO & UDB/DSI:
        /// When D_OUT_EN = 1:
        ///    D_OUT = '0': Strong pull down
        ///    D_OUT = '1': Strong pull up
        /// When D_OUT_EN = 0:
        ///     D_OUT = '0': Weak/resistive pull down
        ///     D_OUT = '1': Weak/resistive pull up
        PULLUP_DOWN = 7
    ],
    /// Enables the input buffer for IO pin 0.  This bit should be cleared when analog signals are present on the pin to avoid crowbar currents.  The output buffer can be used to drive analog signals high or low without issue.
    /// '0': Input buffer disabled
    /// '1': Input buffer enabled
    IN_EN0 OFFSET(3) NUMBITS(1) [],
    /// The GPIO drive mode for IO pin 1
    DRIVE_MODE1 OFFSET(4) NUMBITS(3) [],
    /// Enables the input buffer for IO pin 1
    IN_EN1 OFFSET(7) NUMBITS(1) [],
    /// The GPIO drive mode for IO pin 2
    DRIVE_MODE2 OFFSET(8) NUMBITS(3) [],
    /// Enables the input buffer for IO pin 2
    IN_EN2 OFFSET(11) NUMBITS(1) [],
    /// The GPIO drive mode for IO pin 3
    DRIVE_MODE3 OFFSET(12) NUMBITS(3) [],
    /// Enables the input buffer for IO pin 3
    IN_EN3 OFFSET(15) NUMBITS(1) [],
    /// The GPIO drive mode for IO pin4
    DRIVE_MODE4 OFFSET(16) NUMBITS(3) [],
    /// Enables the input buffer for IO pin 4
    IN_EN4 OFFSET(19) NUMBITS(1) [],
    /// The GPIO drive mode for IO pin 5
    DRIVE_MODE5 OFFSET(20) NUMBITS(3) [],
    /// Enables the input buffer for IO pin 5
    IN_EN5 OFFSET(23) NUMBITS(1) [],
    /// The GPIO drive mode for IO pin 6
    DRIVE_MODE6 OFFSET(24) NUMBITS(3) [],
    /// Enables the input buffer for IO pin 6
    IN_EN6 OFFSET(27) NUMBITS(1) [],
    /// The GPIO drive mode for IO pin 7
    DRIVE_MODE7 OFFSET(28) NUMBITS(3) [],
    /// Enables the input buffer for IO pin 7
    IN_EN7 OFFSET(31) NUMBITS(1) []
],
PRT_CFG_IN [
    /// Configures the pin 0 input buffer mode (trip points and hysteresis)
    VTRIP_SEL0_0 OFFSET(0) NUMBITS(1) [
        /// Input buffer compatible with CMOS and I2C interfaces
        InputBufferCompatibleWithCMOSAndI2CInterfaces = 0,
        /// Input buffer compatible with TTL and MediaLB interfaces
        InputBufferCompatibleWithTTLAndMediaLBInterfaces = 1
    ],
    /// Configures the pin 1 input buffer mode (trip points and hysteresis)
    VTRIP_SEL1_0 OFFSET(1) NUMBITS(1) [],
    /// Configures the pin 2 input buffer mode (trip points and hysteresis)
    VTRIP_SEL2_0 OFFSET(2) NUMBITS(1) [],
    /// Configures the pin 3 input buffer mode (trip points and hysteresis)
    VTRIP_SEL3_0 OFFSET(3) NUMBITS(1) [],
    /// Configures the pin 4 input buffer mode (trip points and hysteresis)
    VTRIP_SEL4_0 OFFSET(4) NUMBITS(1) [],
    /// Configures the pin 5 input buffer mode (trip points and hysteresis)
    VTRIP_SEL5_0 OFFSET(5) NUMBITS(1) [],
    /// Configures the pin 6 input buffer mode (trip points and hysteresis)
    VTRIP_SEL6_0 OFFSET(6) NUMBITS(1) [],
    /// Configures the pin 7 input buffer mode (trip points and hysteresis)
    VTRIP_SEL7_0 OFFSET(7) NUMBITS(1) []
],
PRT_CFG_OUT [
    /// Enables slow slew rate for IO pin 0
    /// '0': Fast slew rate
    /// '1': Slow slew rate
    SLOW0 OFFSET(0) NUMBITS(1) [],
    /// Enables slow slew rate for IO pin 1
    SLOW1 OFFSET(1) NUMBITS(1) [],
    /// Enables slow slew rate for IO pin 2
    SLOW2 OFFSET(2) NUMBITS(1) [],
    /// Enables slow slew rate for IO pin 3
    SLOW3 OFFSET(3) NUMBITS(1) [],
    /// Enables slow slew rate for IO pin 4
    SLOW4 OFFSET(4) NUMBITS(1) [],
    /// Enables slow slew rate for IO pin 5
    SLOW5 OFFSET(5) NUMBITS(1) [],
    /// Enables slow slew rate for IO pin 6
    SLOW6 OFFSET(6) NUMBITS(1) [],
    /// Enables slow slew rate for IO pin 7
    SLOW7 OFFSET(7) NUMBITS(1) [],
    /// Sets the GPIO drive strength for IO pin 0
    DRIVE_SEL0 OFFSET(16) NUMBITS(2) [
        /// N/A
        NA = 0
    ],
    /// Sets the GPIO drive strength for IO pin 1
    DRIVE_SEL1 OFFSET(18) NUMBITS(2) [],
    /// Sets the GPIO drive strength for IO pin 2
    DRIVE_SEL2 OFFSET(20) NUMBITS(2) [],
    /// Sets the GPIO drive strength for IO pin 3
    DRIVE_SEL3 OFFSET(22) NUMBITS(2) [],
    /// Sets the GPIO drive strength for IO pin 4
    DRIVE_SEL4 OFFSET(24) NUMBITS(2) [],
    /// Sets the GPIO drive strength for IO pin 5
    DRIVE_SEL5 OFFSET(26) NUMBITS(2) [],
    /// Sets the GPIO drive strength for IO pin 6
    DRIVE_SEL6 OFFSET(28) NUMBITS(2) [],
    /// Sets the GPIO drive strength for IO pin 7
    DRIVE_SEL7 OFFSET(30) NUMBITS(2) []
],
PRT_CFG_SIO [
    /// The regulated output mode is selected ONLY if the CFG.DRIVE_MODE bits are set to the strong pull up (Z_1 = '5') mode If the CFG.DRIVE_MODE bits are set to any other mode the regulated output buffer will be disabled and the standard CMOS output buffer is used.
    VREG_EN01 OFFSET(0) NUMBITS(1) [],
    /// N/A
    IBUF_SEL01 OFFSET(1) NUMBITS(1) [],
    /// N/A
    VTRIP_SEL01 OFFSET(2) NUMBITS(1) [],
    /// N/A
    VREF_SEL01 OFFSET(3) NUMBITS(2) [],
    /// Selects trip-point of input buffer. In single ended input buffer mode (IBUF01_SEL = '0'):
    /// 0: input buffer functions as a CMOS input buffer.
    /// 1: input buffer functions as a LVTTL input buffer.
    /// In differential input buffer mode (IBUF01_SEL = '1'):                                                                  VTRIP_SEL=0:                                                                                                                                a) VREF_SEL=00, VOH_SEL=X -> Trip point=50 percent of vddio
    /// b) VREF_SEL=01, VOH_SEL=000 -> Trip point=Vohref (buffered)
    /// c) VREF_SEL=01, VOH_SEL=[1-7] -> Input buffer functions as CMOS input buffer.
    /// d) VREF_SEL=10/11, VOH_SEL=000 -> Trip point=Amuxbus_a/b (buffered)
    /// e) VREF_SEL=10/11, VOH_SEL=[1-7]  ->  Input buffer functions as CMOS input buffer.                                                                                                                                             VTRIP_SEL=1:                                                                                                                                a) VREF_SEL=00, VOH_SEL=X -> Trip point=40 percent of vddio
    /// b) VREF_SEL=01, VOH_SEL=000 -> Trip point=0.5*Vohref
    /// c) VREF_SEL=01, VOH_SEL=[1-7] -> Input buffer functions as LVTTL input buffer.                                                                                                                                            d) VREF_SEL=10/11, VOH_SEL=000 -> Trip point=0.5*Amuxbus_a/b (buffered)
    /// e) VREF_SEL=10/11, VOH_SEL=[1-7]  -> Input buffer functions as LVTTL input buffer.
    VOH_SEL01 OFFSET(5) NUMBITS(3) [],
    /// N/A
    VREG_EN23 OFFSET(8) NUMBITS(1) [],
    /// N/A
    IBUF_SEL23 OFFSET(9) NUMBITS(1) [],
    /// N/A
    VTRIP_SEL23 OFFSET(10) NUMBITS(1) [],
    /// N/A
    VREF_SEL23 OFFSET(11) NUMBITS(2) [],
    /// N/A
    VOH_SEL23 OFFSET(13) NUMBITS(3) [],
    /// N/A
    VREG_EN45 OFFSET(16) NUMBITS(1) [],
    /// N/A
    IBUF_SEL45 OFFSET(17) NUMBITS(1) [],
    /// N/A
    VTRIP_SEL45 OFFSET(18) NUMBITS(1) [],
    /// N/A
    VREF_SEL45 OFFSET(19) NUMBITS(2) [],
    /// N/A
    VOH_SEL45 OFFSET(21) NUMBITS(3) [],
    /// N/A
    VREG_EN67 OFFSET(24) NUMBITS(1) [],
    /// N/A
    IBUF_SEL67 OFFSET(25) NUMBITS(1) [],
    /// N/A
    VTRIP_SEL67 OFFSET(26) NUMBITS(1) [],
    /// N/A
    VREF_SEL67 OFFSET(27) NUMBITS(2) [],
    /// N/A
    VOH_SEL67 OFFSET(29) NUMBITS(3) []
],
PRT_CFG_IN_AUTOLVL [
    /// Configures the input buffer mode (trip points and hysteresis) for S40E GPIO upper bit.  Lower bit is still selected by CFG_IN.VTRIP_SEL0_0 field.  This field is used along with CFG_IN.VTRIP_SEL0_0 field as below:
    /// {CFG_IN_AUTOLVL.VTRIP_SEL0_1,CFG_IN.VTRIP_SEL0_0}:
    /// 0,0: CMOS
    /// 0,1: TTL
    /// 1,0: input buffer is compatible with automotive.
    /// 1,1: input buffer is compatible with MediaLB.
    VTRIP_SEL0_1 OFFSET(0) NUMBITS(1) [
        /// Input buffer compatible with CMOS/TTL interfaces as described in CFG_IN.VTRIP_SEL0_0.
        InputBufferCompatibleWithCMOSTTLInterfacesAsDescribedInCFG_INVTRIP_SEL0_0 = 0,
        /// Input buffer compatible with AUTO/MediaLB (elevated Vil) interfaces when used along with CFG_IN.VTRIP_SEL0_0.
        AUTO_OR_MediaLB = 1
    ],
    /// Input buffer compatible with automotive (elevated Vil) interfaces.
    VTRIP_SEL1_1 OFFSET(1) NUMBITS(1) [],
    /// Input buffer compatible with automotive (elevated Vil) interfaces.
    VTRIP_SEL2_1 OFFSET(2) NUMBITS(1) [],
    /// Input buffer compatible with automotive (elevated Vil) interfaces.
    VTRIP_SEL3_1 OFFSET(3) NUMBITS(1) [],
    /// Input buffer compatible with automotive (elevated Vil) interfaces.
    VTRIP_SEL4_1 OFFSET(4) NUMBITS(1) [],
    /// Input buffer compatible with automotive (elevated Vil) interfaces.
    VTRIP_SEL5_1 OFFSET(5) NUMBITS(1) [],
    /// Input buffer compatible with automotive (elevated Vil) interfaces.
    VTRIP_SEL6_1 OFFSET(6) NUMBITS(1) [],
    /// Input buffer compatible with automotive (elevated Vil) interfaces.
    VTRIP_SEL7_1 OFFSET(7) NUMBITS(1) []
],
PRT_CFG_OUT2 [
    /// Sets the Drive Select Trim for  IO pin 0
    /// 0 - Default (50ohms)
    /// 1 - 120ohms
    /// 2 - 90ohms
    /// 3 - 60ohms
    /// 4 - 50ohms
    /// 5 - 30ohms
    /// 6 - 20ohms
    /// 7 - 15ohms
    DS_TRIM0 OFFSET(0) NUMBITS(3) [
        /// N/A
        NA = 0
    ],
    /// Sets the Drive Select Trim for IO pin 1
    DS_TRIM1 OFFSET(3) NUMBITS(3) [],
    /// Sets the Drive Select Trim for IO pin 2
    DS_TRIM2 OFFSET(6) NUMBITS(3) [],
    /// Sets the Drive Select Trim for IO pin 3
    DS_TRIM3 OFFSET(9) NUMBITS(3) [],
    /// Sets the Drive Select Trim for IO pin 4
    DS_TRIM4 OFFSET(12) NUMBITS(3) [],
    /// Sets the Drive Select Trim for IO pin 5
    DS_TRIM5 OFFSET(15) NUMBITS(3) [],
    /// Sets the Drive Select Trim for IO pin 6
    DS_TRIM6 OFFSET(18) NUMBITS(3) [],
    /// Sets the Drive Select Trim for IO pin 7
    DS_TRIM7 OFFSET(21) NUMBITS(3) []
],
PRT_CFG_SLEW_EXT [
    /// Enables slow slew rate for IO pin 0
    /// '0': Fast slew rate
    /// '1': Slow slew rate
    SLEW0 OFFSET(0) NUMBITS(3) [],
    /// Slew rate for IO pin 1
    SLEW1 OFFSET(4) NUMBITS(3) [],
    /// Slew rate for IO pin 2
    SLEW2 OFFSET(8) NUMBITS(3) [],
    /// Slew rate for IO pin 3
    SLEW3 OFFSET(12) NUMBITS(3) [],
    /// Slew rate for IO pin 4
    SLEW4 OFFSET(16) NUMBITS(3) [],
    /// Slew rate for IO pin 5
    SLEW5 OFFSET(20) NUMBITS(3) [],
    /// Slew rate for IO pin 6
    SLEW6 OFFSET(24) NUMBITS(3) [],
    /// Slew rate for IO pin 7
    SLEW7 OFFSET(28) NUMBITS(3) []
],
PRT_CFG_DRIVE_EXT0 [
    /// Sets the GPIO drive strength for IO pin 0
    DRIVE_SEL_EXT0 OFFSET(0) NUMBITS(5) [],
    /// Sets the GPIO drive strength for IO pin 1
    DRIVE_SEL_EXT1 OFFSET(8) NUMBITS(5) [],
    /// Sets the GPIO drive strength for IO pin 2
    DRIVE_SEL_EXT2 OFFSET(16) NUMBITS(5) [],
    /// Sets the GPIO drive strength for IO pin 3
    DRIVE_SEL_EXT3 OFFSET(24) NUMBITS(5) []
],
PRT_CFG_DRIVE_EXT1 [
    /// Sets the GPIO drive strength for IO pin 4
    DRIVE_SEL_EXT4 OFFSET(0) NUMBITS(5) [],
    /// Sets the GPIO drive strength for IO pin 5
    DRIVE_SEL_EXT5 OFFSET(8) NUMBITS(5) [],
    /// Sets the GPIO drive strength for IO pin 6
    DRIVE_SEL_EXT6 OFFSET(16) NUMBITS(5) [],
    /// Sets the GPIO drive strength for IO pin 7
    DRIVE_SEL_EXT7 OFFSET(24) NUMBITS(5) []
],
];
const GPIO_BASE: StaticRef<GpioRegisters> =
    unsafe { StaticRef::new(0x42410000 as *const GpioRegisters) };

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

    pub fn get_pin(&self, searched_pin: PsocPin) -> &'a GpioPin<'_> {
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
