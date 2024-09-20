use kernel::debug;
use kernel::hil::gpio::{Configure, FloatingState, Output};
use kernel::utilities::registers::interfaces::{ReadWriteable, Readable, Writeable};
use kernel::utilities::registers::{register_bitfields, register_structs, ReadOnly, ReadWrite};
use kernel::utilities::StaticRef;

use crate::gpio::{GpioFunction, RPGpio, RPGpioPin};

const NUMBER_STATE_MACHINES: usize = 4;
const NUMBER_INSTR_MEMORY_LOCATIONS: usize = 32;
const NUMBER_IRQS: usize = 2;

#[repr(C)]
struct InstrMem {
    /// Write-only access to instruction memory locations 0-31
    instr_mem: ReadWrite<u32, INSTR_MEMx::Register>,
}

#[repr(C)]
struct StateMachine {
    /// Clock divisor register for state machine x
    /// Frequency = clock freq / (CLKDIV_INT + CLKDIV_FRAC / 256)
    clkdiv: ReadWrite<u32, SMx_CLKDIV::Register>,
    /// Execution/behavioural settings for state machine x
    execctrl: ReadWrite<u32, SMx_EXECCTRL::Register>,
    /// Control behaviour of the input/output shift registers for
    /// state machine x
    shiftctrl: ReadWrite<u32, SMx_SHIFTCTRL::Register>,
    /// Current instruction address of state machine x
    addr: ReadOnly<u32, SMx_ADDR::Register>,
    /// Read to see the instruction currently addressed by state
    /// machine x’s program counter Write to execute an instruction
    /// immediately (including jumps) and then resume execution.
    instr: ReadWrite<u32, SMx_INSTR::Register>,
    /// State machine pin control
    pinctrl: ReadWrite<u32, SMx_PINCTRL::Register>,
}

#[repr(C)]
struct Irq {
    /// Interrupt Enable for irq x
    enable0: ReadWrite<u32, IRQ0_INTS::Register>,
    /// Interrupt Force for irq x
    force0: ReadWrite<u32, IRQ0_INTS::Register>,
    /// Interrupt status after masking & forcing for irq x
    status0: ReadOnly<u32, IRQ0_INTS::Register>,
    /// Interrupt Enable for irq x
    enable1: ReadWrite<u32, IRQ1_INTE::Register>,
    /// Interrupt Force for irq x
    force1: ReadWrite<u32, IRQ1_INTE::Register>,
    /// Interrupt status after masking & forcing for irq x
    status1: ReadOnly<u32, IRQ1_INTE::Register>,
}

register_structs! {
PioRegisters {
        /// PIO control register
        (0x000 => ctrl: ReadWrite<u32, CTRL::Register>),
        /// FIFO status register
        (0x004 => fstat: ReadOnly<u32, FSTAT::Register>),
        /// FIFO debug register
        (0x008 => fdebug: ReadWrite<u32, FDEBUG::Register>),
        /// FIFO levels
        (0x00C => flevel: ReadOnly<u32, FLEVEL::Register>),
        /// Direct write access to the TX FIFO for this state machine. Each
        /// write pushes one word to the FIFO. Attempting to write to a full
        /// FIFO has no effect on the FIFO state or contents, and sets the
        /// sticky FDEBUG_TXOVER error flag for this FIFO.
        (0x010 => txf: [ReadWrite<u32, TXFx::Register>; 4]),
        /// Direct read access to the RX FIFO for this state machine. Each
        /// read pops one word from the FIFO. Attempting to read from an empty
        /// FIFO has no effect on the FIFO state, and sets the sticky
        /// FDEBUG_RXUNDER error flag for this FIFO. The data returned
        /// to the system on a read from an empty FIFO is undefined.
        (0x020 => rxf: [ReadOnly<u32, RXFx::Register>; 4]),
        /// State machine IRQ flags register. Write 1 to clear. There are 8
        /// state machine IRQ flags, which can be set, cleared, and waited on
        /// by the state machines. There’s no fixed association between
        /// flags and state machines — any state machine can use any flag.
        /// Any of the 8 flags can be used for timing synchronisation
        /// between state machines, using IRQ and WAIT instructions. The
        /// lower four of these flags are also routed out to system-level
        /// interrupt requests, alongside FIFO status interrupts —
        /// see e.g. IRQ0_INTE.
        (0x030 => irq: ReadWrite<u32, IRQ::Register>),
        /// Writing a 1 to each of these bits will forcibly assert the
        /// corresponding IRQ. Note this is different to the INTF register:
        /// writing here affects PIO internal state. INTF just asserts the
        /// processor-facing IRQ signal for testing ISRs, and is not visible to
        /// the state machines.
        (0x034 => irq_force: ReadWrite<u32, IRQ_FORCE::Register>),
        /// There is a 2-flipflop synchronizer on each GPIO input, which
        /// protects PIO logic from metastabilities. This increases input
        /// delay, and for fast synchronous IO (e.g. SPI) these synchronizers
        /// may need to be bypassed. Each bit in this register corresponds
        /// to one GPIO.
        /// 0 → input is synchronized (default)
        /// 1 → synchronizer is bypassed
        /// If in doubt, leave this register as all zeroes.
        (0x038 => input_sync_bypass: ReadWrite<u32, INPUT_SYNC_BYPASS::Register>),
        /// Read to sample the pad output values PIO is currently driving
        /// to the GPIOs.
        (0x03C => dbg_padout: ReadOnly<u32, DBG_PADOUT::Register>),
        /// Read to sample the pad output enables (direction) PIO is
        /// currently driving to the GPIOs. On RP2040 there are 30 GPIOs,
        /// so the two most significant bits are hardwired to 0.
        (0x040 => dbg_padoe: ReadOnly<u32, DBG_PADOE::Register>),
        /// The PIO hardware has some free parameters that may vary
        /// between chip products.
        (0x044 => dbg_cfginfo: ReadOnly<u32, DBG_CFGINFO::Register>),
        /// Write-only access to instruction memory locations 0-31
        (0x048 => instr_mem: [InstrMem; NUMBER_INSTR_MEMORY_LOCATIONS]),
        /// State Machines
        (0x0c8 => sm: [StateMachine; NUMBER_STATE_MACHINES]),
        /// Raw Interrupts
        (0x128 => intr: ReadWrite<u32, INTR::Register>),
        /// Interrupt Enable for irq0
        (0x12C => irq0_inte: ReadWrite<u32, IRQ0_INTE::Register>),
        /// Interrupt Force for irq0
        (0x130 => irq0_intf: ReadWrite<u32, IRQ0_INTF::Register>),
        /// Interrupt status after masking & forcing for irq0
        (0x134 => irq0_ints: ReadWrite<u32, IRQ0_INTS::Register>),
        /// Interrupt Enable for irq1
        (0x138 => irq1_inte: ReadWrite<u32, IRQ1_INTE::Register>),
        /// Interrupt Force for irq1
        (0x13C => irq1_intf: ReadWrite<u32, IRQ1_INTF::Register>),
        /// Interrupt status after masking & forcing for irq1
        (0x140 => irq1_ints: ReadWrite<u32, IRQ1_INTS::Register>),
        (0x144 => @END),
    }
}

register_bitfields![u32,
CTRL [
    /// Restart a state machine’s clock divider from an initial
    /// phase of 0. Clock dividers are free-running, so once
    /// started, their output (including fractional jitter) is
    /// completely determined by the integer/fractional divisor
    /// configured in SMx_CLKDIV. This means that, if multiple
    /// clock dividers with the same divisor are restarted
    /// simultaneously, by writing multiple 1 bits to this field, the
    /// execution clocks of those state machines will run in
    /// precise lockstep.
    /// - SM_ENABLE does not stop the clock divider from running
    /// - CLKDIV_RESTART can be written to whilst the state machine is running
    CLKDIV3_RESTART OFFSET(11) NUMBITS(1) [],
    CLKDIV2_RESTART OFFSET(10) NUMBITS(1) [],
    CLKDIV1_RESTART OFFSET(9) NUMBITS(1) [],
    CLKDIV0_RESTART OFFSET(8) NUMBITS(1) [],
    /// Write 1 to instantly clear internal SM state which may be
    /// otherwise difficult to access and will affect future
    /// execution.
    /// Specifically, the following are cleared: input and output
    /// shift counters; the contents of the input shift register; the
    /// delay counter; the waiting-on-IRQ state; any stalled
    /// instruction written to SMx_INSTR or run by OUT/MOV
    /// EXEC; any pin write left asserted due to OUT_STICKY.
    SM3_RESTART OFFSET(7) NUMBITS(1) [],
    SM2_RESTART OFFSET(6) NUMBITS(1) [],
    SM1_RESTART OFFSET(5) NUMBITS(1) [],
    SM0_RESTART OFFSET(4) NUMBITS(1) [],
    /// Enable/disable each of the four state machines by writing
    /// 1/0 to each of these four bits. When disabled, a state
    /// machine will cease executing instructions, except those
    /// written directly to SMx_INSTR by the system. Multiple bits
    /// can be set/cleared at once to run/halt multiple state
    /// machines simultaneously.
    SM3_ENABLE OFFSET(3) NUMBITS(1) [],
    SM2_ENABLE OFFSET(2) NUMBITS(1) [],
    SM1_ENABLE OFFSET(1) NUMBITS(1) [],
    SM0_ENABLE OFFSET(0) NUMBITS(1) [],
],
FSTAT [
    /// State machine TX FIFO is empty
    TXEMPTY3 OFFSET(27) NUMBITS(1) [],
    TXEMPTY2 OFFSET(26) NUMBITS(1) [],
    TXEMPTY1 OFFSET(25) NUMBITS(1) [],
    TXEMPTY0 OFFSET(24) NUMBITS(1) [],
    /// State machine TX FIFO is full
    TXFULL3 OFFSET(19) NUMBITS(1) [],
    TXFULL2 OFFSET(18) NUMBITS(1) [],
    TXFULL1 OFFSET(17) NUMBITS(1) [],
    TXFULL0 OFFSET(16) NUMBITS(1) [],
    /// State machine RX FIFO is empty
    RXEMPTY OFFSET(8) NUMBITS(4) [],
    /// State machine RX FIFO is full
    RXFULL OFFSET(0) NUMBITS(4) []
],
FDEBUG [
    /// State machine has stalled on empty TX FIFO during a
    /// blocking PULL, or an OUT with autopull enabled. Write 1 to
    /// clear.
    TXSTALL OFFSET(24) NUMBITS(4) [],
    /// TX FIFO overflow (i.e. write-on-full by the system) has
    /// occurred. Write 1 to clear. Note that write-on-full does not
    /// alter the state or contents of the FIFO in any way, but the
    /// data that the system attempted to write is dropped, so if
    /// this flag is set, your software has quite likely dropped
    /// some data on the floor.
    TXOVER OFFSET(16) NUMBITS(4) [],
    /// RX FIFO underflow (i.e. read-on-empty by the system) has
    /// occurred. Write 1 to clear. Note that read-on-empty does
    /// not perturb the state of the FIFO in any way, but the data
    /// returned by reading from an empty FIFO is undefined, so
    /// this flag generally only becomes set due to some kind of
    /// software error.
    RXUNDER OFFSET(8) NUMBITS(4) [],
    /// State machine has stalled on full RX FIFO during a
    /// blocking PUSH, or an IN with autopush enabled. This flag
    /// is also set when a nonblocking PUSH to a full FIFO took
    /// place, in which case the state machine has dropped data.
    /// Write 1 to clear.
    RXSTALL OFFSET(0) NUMBITS(4) []
],
FLEVEL [
    RX3 OFFSET(28) NUMBITS(4) [],
    TX3 OFFSET(24) NUMBITS(4) [],
    RX2 OFFSET(20) NUMBITS(4) [],
    TX2 OFFSET(16) NUMBITS(4) [],
    RX1 OFFSET(12) NUMBITS(4) [],
    TX1 OFFSET(8) NUMBITS(4) [],
    RX0 OFFSET(4) NUMBITS(4) [],
    TX0 OFFSET(0) NUMBITS(4) []
],
TXFx [
    TXF OFFSET(0) NUMBITS(32) []
],
RXFx [
    RXF OFFSET(0) NUMBITS(32) []
],
IRQ [
    IRQ OFFSET(0) NUMBITS(8) []
],
IRQ_FORCE [
    IRQ_FORCE OFFSET(0) NUMBITS(8) []
],
INPUT_SYNC_BYPASS [
    INPUT_SYNC_BYPASS OFFSET(0) NUMBITS(32) []
],
DBG_PADOUT [
    DBG_PADOUT OFFSET(0) NUMBITS(32) []
],
DBG_PADOE [
    DBG_PADOE OFFSET(0) NUMBITS(32) []
],
DBG_CFGINFO [
    /// The size of the instruction memory, measured in units of
    /// one instruction
    IMEM_SIZE OFFSET(16) NUMBITS(6) [],
    /// The number of state machines this PIO instance is
    /// equipped with.
    SM_COUNT OFFSET(8) NUMBITS(4) [],
    /// The depth of the state machine TX/RX FIFOs, measured in
    /// words.
    FIFO_DEPTH OFFSET(0) NUMBITS(6) []
],
INSTR_MEMx [
    /// Write-only access to instruction memory location x
    INSTR_MEM OFFSET(0) NUMBITS(16) []
],
SMx_CLKDIV [
    /// Effective frequency is sysclk/(int + frac/256).
    /// Value of 0 is interpreted as 65536. If INT is 0, FRAC must
    /// also be 0.
    INT OFFSET(16) NUMBITS(16) [],
    /// Fractional part of clock divisor
    FRAC OFFSET(8) NUMBITS(8) []
],
SMx_EXECCTRL [
    /// If 1, an instruction written to SMx_INSTR is stalled, and
    /// latched by the state machine. Will clear to 0 once this
    /// instruction completes.
    EXEC_STALLED OFFSET(31) NUMBITS(1) [],
    /// If 1, the MSB of the Delay/Side-set instruction field is used
    /// as side-set enable, rather than a side-set data bit. This
    /// allows instructions to perform side-set optionally, rather
    /// than on every instruction, but the maximum possible side-
    /// set width is reduced from 5 to 4. Note that the value of
    /// PINCTRL_SIDESET_COUNT is inclusive of this enable bit.
    SIDE_EN OFFSET(30) NUMBITS(1) [],
    /// If 1, side-set data is asserted to pin directions, instead of
    /// pin values
    SIDE_PINDIR OFFSET(29) NUMBITS(1) [],
    /// The GPIO number to use as condition for JMP PIN.
    /// Unaffected by input mapping.
    JMP_PIN OFFSET(24) NUMBITS(5) [],
    /// Which data bit to use for inline OUT enable
    OUT_EN_SEL OFFSET(19) NUMBITS(5) [],
    /// If 1, use a bit of OUT data as an auxiliary write enable
    /// When used in conjunction with OUT_STICKY, writes with
    /// an enable of 0 will
    /// deassert the latest pin write. This can create useful
    /// masking/override behaviour
    /// due to the priority ordering of state machine pin writes
    /// (SM0 < SM1 < …)
    INLINE_OUT_EN OFFSET(18) NUMBITS(1) [],
    /// Continuously assert the most recent OUT/SET to the pins
    OUT_STICKY OFFSET(17) NUMBITS(1) [],
    /// After reaching this address, execution is wrapped to
    /// wrap_bottom.
    /// If the instruction is a jump, and the jump condition is true,
    /// the jump takes priority.
    WRAP_TOP OFFSET(12) NUMBITS(5) [],
    /// After reaching wrap_top, execution is wrapped to this
    /// address.
    WRAP_BOTTOM OFFSET(7) NUMBITS(5) [],
    STATUS_SEL OFFSET(4) NUMBITS(1) [],
    /// Comparison level for the MOV x, STATUS instruction
    STATUS_N OFFSET(0) NUMBITS(4) []
],
SMx_SHIFTCTRL [
    /// When 1, RX FIFO steals the TX FIFO’s storage, and
    /// becomes twice as deep.
    /// TX FIFO is disabled as a result (always reads as both full
    /// and empty).
    /// FIFOs are flushed when this bit is changed.
    FJOIN_RX OFFSET(31) NUMBITS(1) [],
    /// When 1, TX FIFO steals the RX FIFO’s storage, and
    /// becomes twice as deep.
    /// RX FIFO is disabled as a result (always reads as both full
    /// and empty).
    /// FIFOs are flushed when this bit is changed.
    FJOIN_TX OFFSET(30) NUMBITS(1) [],
    /// Number of bits shifted out of OSR before autopull, or
    /// conditional pull (PULL IFEMPTY), will take place.
    /// Write 0 for value of 32.
    PULL_THRESH OFFSET(25) NUMBITS(5) [],
    /// Number of bits shifted into ISR before autopush, or
    /// conditional push (PUSH IFFULL), will take place.
    /// Write 0 for value of 32
    PUSH_THRESH OFFSET(20) NUMBITS(5) [],
    OUT_SHIFTDIR OFFSET(19) NUMBITS(1) [
        ShiftRight = 1,
        ShiftLeft = 0
    ],
    IN_SHIFTDIR OFFSET(18) NUMBITS(1) [
        ShiftRight = 1,
        ShiftLeft = 0
    ],
    /// Pull automatically when the output shift register is
    /// emptied, i.e. on or following an OUT instruction which
    /// causes the output shift counter to reach or exceed
    /// PULL_THRESH.
    AUTOPULL OFFSET(17) NUMBITS(1) [],
    /// Push automatically when the input shift register is filled,
    /// i.e. on an IN instruction which causes the input shift
    /// counter to reach or exceed PUSH_THRESH.
    AUTOPUSH OFFSET(16) NUMBITS(1) []
],
SMx_ADDR [
    ADDR OFFSET(0) NUMBITS(5) []
],
SMx_INSTR [
    INSTR OFFSET(0) NUMBITS(16) []
],
SMx_PINCTRL [
    /// The number of MSBs of the Delay/Side-set instruction
    /// field which are used for side-set. Inclusive of the enable
    /// bit, if present. Minimum of 0 (all delay bits, no side-set)
    /// and maximum of 5 (all side-set, no delay).
    SIDESET_COUNT OFFSET(29) NUMBITS(3) [],
    /// The number of pins asserted by a SET. In the range 0 to 5
    // inclusive.
    SET_COUNT OFFSET(26) NUMBITS(3) [],
    /// The number of pins asserted by an OUT PINS, OUT
    /// PINDIRS or MOV PINS instruction. In the range 0 to 32
    /// inclusive.
    OUT_COUNT OFFSET(20) NUMBITS(6) [],
    /// The pin which is mapped to the least-significant bit of a
    /// state machine’s IN data bus. Higher-numbered pins are
    /// mapped to consecutively more-significant data bits, with a
    /// modulo of 32 applied to pin number.
    IN_BASE OFFSET(15) NUMBITS(5) [],
    /// The lowest-numbered pin that will be affected by a side-
    /// set operation. The MSBs of an instruction’s side-set/delay
    /// field (up to 5, determined by SIDESET_COUNT) are used
    /// for side-set data, with the remaining LSBs used for delay.
    /// The least-significant bit of the side-set portion is the bit
    /// written to this pin, with more-significant bits written to
    /// higher-numbered pins.
    SIDESET_BASE OFFSET(10) NUMBITS(5) [],
    /// The lowest-numbered pin that will be affected by a SET
    /// PINS or SET PINDIRS instruction. The data written to this
    /// pin is the least-significant bit of the SET data.
    SET_BASE OFFSET(5) NUMBITS(5) [],
    /// The lowest-numbered pin that will be affected by an OUT
    /// PINS, OUT PINDIRS or MOV PINS instruction. The data
    /// written to this pin will always be the least-significant bit of
    /// the OUT or MOV data.
    OUT_BASE OFFSET(0) NUMBITS(5) []
],
    INTR [
    SM3 OFFSET(11) NUMBITS(1) [],
    SM2 OFFSET(10) NUMBITS(1) [],
    SM1 OFFSET(9) NUMBITS(1) [],
    SM0 OFFSET(8) NUMBITS(1) [],
    SM3_TXNFULL OFFSET(7) NUMBITS(1) [],
    SM2_TXNFULL OFFSET(6) NUMBITS(1) [],
    SM1_TXNFULL OFFSET(5) NUMBITS(1) [],
    SM0_TXNFULL OFFSET(4) NUMBITS(1) [],
    SM3_RXNEMPTY OFFSET(3) NUMBITS(1) [],
    SM2_RXNEMPTY OFFSET(2) NUMBITS(1) [],
    SM1_RXNEMPTY OFFSET(1) NUMBITS(1) [],
    SM0_RXNEMPTY OFFSET(0) NUMBITS(1) []
],

   IRQ0_INTE [
    SM3 OFFSET(11) NUMBITS(1) [],
    SM2 OFFSET(10) NUMBITS(1) [],
    SM1 OFFSET(9) NUMBITS(1) [],
    SM0 OFFSET(8) NUMBITS(1) [],
    SM3_TXNFULL OFFSET(7) NUMBITS(1) [],
    SM2_TXNFULL OFFSET(6) NUMBITS(1) [],
    SM1_TXNFULL OFFSET(5) NUMBITS(1) [],
    SM0_TXNFULL OFFSET(4) NUMBITS(1) [],
    SM3_RXNEMPTY OFFSET(3) NUMBITS(1) [],
    SM2_RXNEMPTY OFFSET(2) NUMBITS(1) [],
    SM1_RXNEMPTY OFFSET(1) NUMBITS(1) [],
    SM0_RXNEMPTY OFFSET(0) NUMBITS(1) []
],

IRQ0_INTF [
    SM3 OFFSET(11) NUMBITS(1) [],
    SM2 OFFSET(10) NUMBITS(1) [],
    SM1 OFFSET(9) NUMBITS(1) [],
    SM0 OFFSET(8) NUMBITS(1) [],
    SM3_TXNFULL OFFSET(7) NUMBITS(1) [],
    SM2_TXNFULL OFFSET(6) NUMBITS(1) [],
    SM1_TXNFULL OFFSET(5) NUMBITS(1) [],
    SM0_TXNFULL OFFSET(4) NUMBITS(1) [],
    SM3_RXNEMPTY OFFSET(3) NUMBITS(1) [],
    SM2_RXNEMPTY OFFSET(2) NUMBITS(1) [],
    SM1_RXNEMPTY OFFSET(1) NUMBITS(1) [],
    SM0_RXNEMPTY OFFSET(0) NUMBITS(1) []
],
IRQ0_INTS [
    SM3 OFFSET(0) NUMBITS(1) [],
    SM2 OFFSET(0) NUMBITS(1) [],
    SM1 OFFSET(0) NUMBITS(1) [],
    SM0 OFFSET(0) NUMBITS(1) [],
    SM3_TXNFULL OFFSET(0) NUMBITS(1) [],
    SM2_TXNFULL OFFSET(0) NUMBITS(1) [],
    SM1_TXNFULL OFFSET(0) NUMBITS(1) [],
    SM0_TXNFULL OFFSET(0) NUMBITS(1) [],
    SM3_RXNEMPTY OFFSET(0) NUMBITS(1) [],
    SM2_RXNEMPTY OFFSET(0) NUMBITS(1) [],
    SM1_RXNEMPTY OFFSET(0) NUMBITS(1) [],
    SM0_RXNEMPTY OFFSET(0) NUMBITS(1) []
],
    IRQ1_INTE [
    SM3 OFFSET(11) NUMBITS(1) [],
    SM2 OFFSET(10) NUMBITS(1) [],
    SM1 OFFSET(9) NUMBITS(1) [],
    SM0 OFFSET(8) NUMBITS(1) [],
    SM3_TXNFULL OFFSET(7) NUMBITS(1) [],
    SM2_TXNFULL OFFSET(6) NUMBITS(1) [],
    SM1_TXNFULL OFFSET(5) NUMBITS(1) [],
    SM0_TXNFULL OFFSET(4) NUMBITS(1) [],
    SM3_RXNEMPTY OFFSET(3) NUMBITS(1) [],
    SM2_RXNEMPTY OFFSET(2) NUMBITS(1) [],
    SM1_RXNEMPTY OFFSET(1) NUMBITS(1) [],
    SM0_RXNEMPTY OFFSET(0) NUMBITS(1) []
],
IRQ1_INTF [
    SM3 OFFSET(11) NUMBITS(1) [],
    SM2 OFFSET(10) NUMBITS(1) [],
    SM1 OFFSET(9) NUMBITS(1) [],
    SM0 OFFSET(8) NUMBITS(1) [],
    SM3_TXNFULL OFFSET(7) NUMBITS(1) [],
    SM2_TXNFULL OFFSET(6) NUMBITS(1) [],
    SM1_TXNFULL OFFSET(5) NUMBITS(1) [],
    SM0_TXNFULL OFFSET(4) NUMBITS(1) [],
    SM3_RXNEMPTY OFFSET(3) NUMBITS(1) [],
    SM2_RXNEMPTY OFFSET(2) NUMBITS(1) [],
    SM1_RXNEMPTY OFFSET(1) NUMBITS(1) [],
    SM0_RXNEMPTY OFFSET(0) NUMBITS(1) []
],
IRQ1_INTS [
    SM3 OFFSET(11) NUMBITS(1) [],
    SM2 OFFSET(10) NUMBITS(1) [],
    SM1 OFFSET(9) NUMBITS(1) [],
    SM0 OFFSET(8) NUMBITS(1) [],
    SM3_TXNFULL OFFSET(7) NUMBITS(1) [],
    SM2_TXNFULL OFFSET(6) NUMBITS(1) [],
    SM1_TXNFULL OFFSET(5) NUMBITS(1) [],
    SM0_TXNFULL OFFSET(4) NUMBITS(1) [],
    SM3_RXNEMPTY OFFSET(3) NUMBITS(1) [],
    SM2_RXNEMPTY OFFSET(2) NUMBITS(1) [],
    SM1_RXNEMPTY OFFSET(1) NUMBITS(1) [],
    SM0_RXNEMPTY OFFSET(0) NUMBITS(1) []
]
];

const PIO_0_BASE_ADDRESS: usize = 0x50200000;
const PIO_1_BASE_ADDRESS: usize = 0x50300000;
const PIO0_BASE: StaticRef<PioRegisters> =
    unsafe { StaticRef::new(PIO_0_BASE_ADDRESS as *const PioRegisters) };
const PIO0_XOR_BASE: StaticRef<PioRegisters> =
    unsafe { StaticRef::new((PIO_0_BASE_ADDRESS + 0x1000) as *const PioRegisters) };
const PIO0_SET_BASE: StaticRef<PioRegisters> =
    unsafe { StaticRef::new((PIO_0_BASE_ADDRESS + 0x2000) as *const PioRegisters) };
const PIO0_CLEAR_BASE: StaticRef<PioRegisters> =
    unsafe { StaticRef::new((PIO_0_BASE_ADDRESS + 0x3000) as *const PioRegisters) };
const PIO1_BASE: StaticRef<PioRegisters> =
    unsafe { StaticRef::new(PIO_1_BASE_ADDRESS as *const PioRegisters) };
const PIO1_XOR_BASE: StaticRef<PioRegisters> =
    unsafe { StaticRef::new((PIO_1_BASE_ADDRESS + 0x1000) as *const PioRegisters) };
const PIO1_SET_BASE: StaticRef<PioRegisters> =
    unsafe { StaticRef::new((PIO_1_BASE_ADDRESS + 0x2000) as *const PioRegisters) };
const PIO1_CLEAR_BASE: StaticRef<PioRegisters> =
    unsafe { StaticRef::new((PIO_1_BASE_ADDRESS + 0x3000) as *const PioRegisters) };

/// There are a total of 4 State Machines per PIO.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum SMNumber {
    SM0 = 0,
    SM1 = 1,
    SM2 = 2,
    SM3 = 3,
}

/// There can be 2 PIOs per RP2040.
#[derive(PartialEq)]
pub enum PIONumber {
    PIO0 = 0,
    PIO1 = 1,
}

impl RPGpio {
    fn from_u32(value: u32) -> RPGpio {
        match value {
            0 => RPGpio::GPIO0,
            1 => RPGpio::GPIO1,
            2 => RPGpio::GPIO2,
            3 => RPGpio::GPIO3,
            4 => RPGpio::GPIO4,
            5 => RPGpio::GPIO5,
            6 => RPGpio::GPIO6,
            7 => RPGpio::GPIO7,
            8 => RPGpio::GPIO8,
            9 => RPGpio::GPIO9,
            10 => RPGpio::GPIO10,
            11 => RPGpio::GPIO11,
            12 => RPGpio::GPIO12,
            13 => RPGpio::GPIO13,
            14 => RPGpio::GPIO14,
            15 => RPGpio::GPIO15,
            16 => RPGpio::GPIO16,
            17 => RPGpio::GPIO17,
            18 => RPGpio::GPIO18,
            19 => RPGpio::GPIO19,
            20 => RPGpio::GPIO20,
            21 => RPGpio::GPIO21,
            22 => RPGpio::GPIO22,
            23 => RPGpio::GPIO23,
            24 => RPGpio::GPIO24,
            25 => RPGpio::GPIO25,
            26 => RPGpio::GPIO26,
            27 => RPGpio::GPIO27,
            28 => RPGpio::GPIO28,
            29 => RPGpio::GPIO29,
            _ => panic!(
                "Unknown value for GPIO pin: {} (should be from 0 to 29)",
                value
            ),
        }
    }
}

/// The FIFO queues can be joined together for twice the length in one direction.
#[derive(PartialEq)]
pub enum PioFifoJoin {
    PioFifoJoinNone = 0,
    PioFifoJoinTx = 1,
    PioFifoJoinRx = 2,
}

const STATE_MACHINE_NUMBERS: [SMNumber; NUMBER_STATE_MACHINES] =
    [SMNumber::SM0, SMNumber::SM1, SMNumber::SM2, SMNumber::SM3];

pub struct Pio {
    registers: StaticRef<PioRegisters>,
    pio_number: PIONumber,
    xor_registers: StaticRef<PioRegisters>,
    set_registers: StaticRef<PioRegisters>,
    clear_registers: StaticRef<PioRegisters>,
}

/// 'MOV STATUS' types.
#[derive(Clone, Copy)]
pub enum PioMovStatusType {
    StatusTxLessthan = 0,
    StatusRxLessthan = 1,
}

/// PIO State Machine configuration structure
///
/// Used to initialize a PIO with all of its state machines.
pub struct StateMachineConfiguration {
    pub out_pins_count: u32,
    pub out_pins_base: u32,
    pub set_pins_count: u32,
    pub set_pins_base: u32,
    pub in_pins_base: u32,
    pub side_set_base: u32,
    pub side_set_opt_enable: bool,
    pub side_set_bit_count: u32,
    pub side_set_pindirs: bool,
    pub wrap: u32,
    pub wrap_to: u32,
    pub in_shift_direction_right: bool,
    pub in_autopush: bool,
    pub in_push_threshold: u32,
    pub out_shift_direction_right: bool,
    pub out_autopull: bool,
    pub out_pull_threshold: u32,
    pub jmp_pin: u32,
    pub out_special_sticky: bool,
    pub out_special_has_enable_pin: bool,
    pub out_special_enable_pin_index: u32,
    pub mov_status_sel: PioMovStatusType,
    pub mov_status_n: u32,
    pub div_int: u32,
    pub div_frac: u32,
}

impl Default for StateMachineConfiguration {
    fn default() -> Self {
        StateMachineConfiguration {
            out_pins_count: 32,
            out_pins_base: 0,
            set_pins_count: 0,
            set_pins_base: 0,
            in_pins_base: 0,
            side_set_base: 0,
            side_set_opt_enable: false,
            side_set_bit_count: 0,
            side_set_pindirs: false,
            wrap: 31,
            wrap_to: 0,
            in_shift_direction_right: true,
            in_autopush: false,
            in_push_threshold: 32,
            out_shift_direction_right: true,
            out_autopull: false,
            out_pull_threshold: 32,
            jmp_pin: 0,
            out_special_sticky: false,
            out_special_has_enable_pin: false,
            out_special_enable_pin_index: 0,
            mov_status_sel: PioMovStatusType::StatusTxLessthan,
            mov_status_n: 0,
            div_int: 0,
            div_frac: 0,
        }
    }
}

impl Pio {
    /// State machine configuration with any config structure.
    pub fn sm_config(&self, sm_number: SMNumber, config: &StateMachineConfiguration) {
        self.set_in_pins(sm_number, config.in_pins_base);
        self.set_out_pins(sm_number, config.out_pins_base, config.out_pins_count);
        self.set_set_pins(sm_number, config.set_pins_base, config.set_pins_count);
        self.set_side_set_pins(sm_number, config.side_set_base);
        self.set_side_set(
            sm_number,
            config.side_set_bit_count,
            config.side_set_opt_enable,
            config.side_set_pindirs,
        );
        self.set_in_shift(
            sm_number,
            config.in_shift_direction_right,
            config.in_autopush,
            config.in_push_threshold,
        );
        self.set_out_shift(
            sm_number,
            config.out_shift_direction_right,
            config.out_autopull,
            config.out_pull_threshold,
        );
        self.set_jmp_pin(sm_number, config.jmp_pin);
        self.set_wrap(sm_number, config.wrap_to, config.wrap);
        self.set_mov_status(sm_number, config.mov_status_sel, config.mov_status_n);
        self.set_out_special(
            sm_number,
            config.out_special_sticky,
            config.out_special_has_enable_pin,
            config.out_special_enable_pin_index,
        );
        self.set_clkdiv_int_frac(sm_number, config.div_int, config.div_frac);
    }

    /// Resets the state machine to a consistent state, and configures it.
    pub fn sm_init(&self, sm_number: SMNumber) {
        self.restart_sm(sm_number);
        self.sm_clkdiv_restart(sm_number);
        self.sm_clear_fifos(sm_number);
        self.registers.sm[sm_number as usize]
            .instr
            .modify(SMx_INSTR::INSTR.val(0));
        self.sm_set_enabled(sm_number, true);
    }

    /// Set a state machine's state to enabled or to disabled.
    pub fn sm_set_enabled(&self, sm_number: SMNumber, enabled: bool) {
        match sm_number {
            SMNumber::SM0 => self.registers.ctrl.modify(match enabled {
                true => CTRL::SM0_ENABLE::SET,
                false => CTRL::SM0_ENABLE::CLEAR,
            }),
            SMNumber::SM1 => self.registers.ctrl.modify(match enabled {
                true => CTRL::SM1_ENABLE::SET,
                false => CTRL::SM1_ENABLE::CLEAR,
            }),
            SMNumber::SM2 => self.registers.ctrl.modify(match enabled {
                true => CTRL::SM2_ENABLE::SET,
                false => CTRL::SM2_ENABLE::CLEAR,
            }),
            SMNumber::SM3 => self.registers.ctrl.modify(match enabled {
                true => CTRL::SM3_ENABLE::SET,
                false => CTRL::SM3_ENABLE::CLEAR,
            }),
        }
    }

    /// Setup the function select for a GPIO to use output from the given PIO instance.
    pub fn gpio_init(&self, pin: &RPGpioPin) {
        if self.pio_number == PIONumber::PIO1 {
            pin.set_function(GpioFunction::PIO1)
        } else {
            pin.set_function(GpioFunction::PIO0)
        }
    }

    /// Create a new PIO0 struct.
    pub fn new_pio0() -> Self {
        Self {
            registers: PIO0_BASE,
            xor_registers: PIO0_XOR_BASE,
            set_registers: PIO0_SET_BASE,
            clear_registers: PIO0_CLEAR_BASE,
            pio_number: PIONumber::PIO0,
        }
    }

    /// Create a new PIO1 struct.
    pub fn new_pio1() -> Self {
        Self {
            registers: PIO1_BASE,
            xor_registers: PIO1_XOR_BASE,
            set_registers: PIO1_SET_BASE,
            clear_registers: PIO1_CLEAR_BASE,
            pio_number: PIONumber::PIO1,
        }
    }

    /// Set every config for the IN pins.
    ///
    /// in_base => the starting location for the input pins
    fn set_in_pins(&self, sm_number: SMNumber, in_base: u32) {
        self.registers.sm[sm_number as usize]
            .pinctrl
            .modify(SMx_PINCTRL::IN_BASE.val(in_base));
    }

    /// Set every config for the SET pins.
    ///
    /// set_base => the starting location for the SET pins
    ///
    /// set_count => the number of SET pins
    fn set_set_pins(&self, sm_number: SMNumber, set_base: u32, set_count: u32) {
        self.registers.sm[sm_number as usize]
            .pinctrl
            .modify(SMx_PINCTRL::SET_BASE.val(set_base));
        self.registers.sm[sm_number as usize]
            .pinctrl
            .modify(SMx_PINCTRL::SET_COUNT.val(set_count));
    }

    /// Set every config for the OUT pins.
    ///
    /// out_base => the starting location for the OUT pins
    ///
    /// out_count => the number of OUT pins
    fn set_out_pins(&self, sm_number: SMNumber, out_base: u32, out_count: u32) {
        self.registers.sm[sm_number as usize]
            .pinctrl
            .modify(SMx_PINCTRL::OUT_BASE.val(out_base));
        self.registers.sm[sm_number as usize]
            .pinctrl
            .modify(SMx_PINCTRL::OUT_COUNT.val(out_count));
    }

    /// Setup 'in' shifting parameters.
    /// ```
    ///  shift_right => true to shift ISR to right
    ///              => false to shift ISR to left
    ///  autopush => true to enable, false to disable
    ///  push_threshold => threshold in bits to shift in before auto/conditional re-pushing of the ISR
    /// ```
    fn set_in_shift(
        &self,
        sm_number: SMNumber,
        shift_right: bool,
        autopush: bool,
        push_threshold: u32,
    ) {
        self.registers.sm[sm_number as usize]
            .shiftctrl
            .modify(SMx_SHIFTCTRL::IN_SHIFTDIR.val(shift_right.into()));
        self.registers.sm[sm_number as usize]
            .shiftctrl
            .modify(SMx_SHIFTCTRL::AUTOPUSH.val(autopush.into()));
        self.registers.sm[sm_number as usize]
            .shiftctrl
            .modify(SMx_SHIFTCTRL::PUSH_THRESH.val(push_threshold));
    }

    /// Setup 'out' shifting parameters.
    /// ```
    /// shift_right => `true` to shift OSR to right
    ///             => `false` to shift OSR to left
    /// autopull => true to enable, false to disable
    /// pull_threshold => threshold in bits to shift out before auto/conditional re-pulling of the OSR
    /// ```
    fn set_out_shift(
        &self,
        sm_number: SMNumber,
        shift_right: bool,
        autopull: bool,
        pull_threshold: u32,
    ) {
        self.registers.sm[sm_number as usize]
            .shiftctrl
            .modify(SMx_SHIFTCTRL::OUT_SHIFTDIR.val(shift_right.into()));
        self.registers.sm[sm_number as usize]
            .shiftctrl
            .modify(SMx_SHIFTCTRL::AUTOPULL.val(autopull.into()));
        self.registers.sm[sm_number as usize]
            .shiftctrl
            .modify(SMx_SHIFTCTRL::PULL_THRESH.val(pull_threshold));
    }

    /// Set the 'jmp' pin.
    ///
    /// pin => the raw GPIO pin number to use as the source for a jmp pin instruction
    fn set_jmp_pin(&self, sm_number: SMNumber, pin: u32) {
        self.registers.sm[sm_number as usize]
            .execctrl
            .modify(SMx_EXECCTRL::JMP_PIN.val(pin));
    }

    /// Set the clock divider for a state machine.
    ///
    /// div_int => Integer part of the divisor
    ///
    /// div_frac => Fractional part in 1/256ths
    fn set_clkdiv_int_frac(&self, sm_number: SMNumber, div_int: u32, div_frac: u32) {
        self.registers.sm[sm_number as usize]
            .clkdiv
            .modify(SMx_CLKDIV::INT.val(div_int));
        self.registers.sm[sm_number as usize]
            .clkdiv
            .modify(SMx_CLKDIV::FRAC.val(div_frac));
    }

    /// Setup the FIFO joining in a state machine.
    ///
    /// fifo_join => specifies the join type - see the `PioFifoJoin` type
    fn set_fifo_join(&self, sm_number: SMNumber, fifo_join: PioFifoJoin) {
        if fifo_join == PioFifoJoin::PioFifoJoinRx {
            self.registers.sm[sm_number as usize]
                .shiftctrl
                .modify(SMx_SHIFTCTRL::FJOIN_RX.val(fifo_join as u32));
        } else if fifo_join == PioFifoJoin::PioFifoJoinTx {
            self.registers.sm[sm_number as usize]
                .shiftctrl
                .modify(SMx_SHIFTCTRL::FJOIN_TX.val(fifo_join as u32));
        }
    }

    /// Set the starting location for the sideset pins.
    fn set_side_set_pins(&self, sm_number: SMNumber, sideset_base: u32) {
        self.registers.sm[sm_number as usize]
            .pinctrl
            .modify(SMx_PINCTRL::SIDESET_BASE.val(sideset_base));
    }

    /// Set every config for the SIDESET pins.
    ///```
    /// bit_count => number of SIDESET bits per instruction - max 5
    /// optional => true to use the topmost sideset bit as a flag for whether to apply side set on that instruction
    ///          => false to use sideset with every instruction
    /// pindirs => true to affect pin direction
    ///         => false to affect value of a pin
    /// ```
    fn set_side_set(&self, sm_number: SMNumber, bit_count: u32, optional: bool, pindirs: bool) {
        self.registers.sm[sm_number as usize]
            .pinctrl
            .modify(SMx_PINCTRL::SIDESET_COUNT.val(bit_count));
        self.registers.sm[sm_number as usize]
            .execctrl
            .modify(SMx_EXECCTRL::SIDE_EN.val(optional as u32));
        self.registers.sm[sm_number as usize]
            .execctrl
            .modify(SMx_EXECCTRL::SIDE_PINDIR.val(pindirs as u32));
    }

    /// Set the wrap addresses for a state machine.
    ///
    /// wrap_target => the instruction memory address to wrap to
    ///
    /// wrap => the instruction memory address after which the program counters wraps to the target
    fn set_wrap(&self, sm_number: SMNumber, wrap_target: u32, wrap: u32) {
        self.registers.sm[sm_number as usize]
            .execctrl
            .modify(SMx_EXECCTRL::WRAP_BOTTOM.val(wrap_target));
        self.registers.sm[sm_number as usize]
            .execctrl
            .modify(SMx_EXECCTRL::WRAP_TOP.val(wrap));
    }

    /// Set source for 'mov status' in a state machine.
    ///
    /// status_sel => comparison used for the `MOV x, STATUS` instruction
    ///
    /// status_n => comparison level for the `MOV x, STATUS` instruction
    fn set_mov_status(&self, sm_number: SMNumber, status_sel: PioMovStatusType, status_n: u32) {
        self.registers.sm[sm_number as usize]
            .execctrl
            .modify(SMx_EXECCTRL::STATUS_SEL.val(status_sel as u32));
        self.registers.sm[sm_number as usize]
            .execctrl
            .modify(SMx_EXECCTRL::STATUS_N.val(status_n));
    }

    /// Set special OUT operations in a state machine.
    /// ```
    /// sticky => true to enable sticky output (rere-asserting most recent OUT/SET pin values on subsequent cycles)
    ///        => false to disable sticky output
    /// has_enable_pin => true to enable auxiliary OUT enable pin
    ///                => false to disable auxiliary OUT enable pin
    /// enable_pin_index => pin index for auxiliary OUT enable
    /// ```
    fn set_out_special(
        &self,
        sm_number: SMNumber,
        sticky: bool,
        has_enable_pin: bool,
        enable_pin_index: u32,
    ) {
        self.registers.sm[sm_number as usize]
            .execctrl
            .modify(SMx_EXECCTRL::OUT_STICKY.val(sticky as u32));
        self.registers.sm[sm_number as usize]
            .execctrl
            .modify(SMx_EXECCTRL::INLINE_OUT_EN.val(has_enable_pin as u32));
        self.registers.sm[sm_number as usize]
            .execctrl
            .modify(SMx_EXECCTRL::OUT_EN_SEL.val(enable_pin_index));
    }

    /// Use a state machine to set the same pin direction for multiple consecutive pins for the PIO instance.
    /// This is the pio_sm_set_consecutive_pindirs function from the pico sdk, renamed to be more clear.
    /// ```
    /// pin => starting pin
    /// count => how many pins (including the base) should be changed
    /// is_out => true to set the pin as OUT
    ///        => false to set the pin as IN
    /// ```
    fn set_pins_out(&self, sm_number: SMNumber, mut pin: u32, mut count: u32, is_out: bool) {
        let pinctrl = self.registers.sm[sm_number as usize].pinctrl.get();
        let execctrl = self.registers.sm[sm_number as usize].execctrl.get();
        self.registers.sm[sm_number as usize]
            .execctrl
            .modify(SMx_EXECCTRL::OUT_STICKY.val(0));
        let mut pindir_val: u8 = 0x00;
        if is_out {
            pindir_val = 0x1f;
        }
        while count > 5 {
            self.registers.sm[sm_number as usize]
                .pinctrl
                .modify(SMx_PINCTRL::SET_COUNT.val(5));
            self.registers.sm[sm_number as usize]
                .pinctrl
                .modify(SMx_PINCTRL::SET_BASE.val(pin));
            self.sm_exec(
                sm_number,
                ((0b11100000100 as u32) << 5) | (pindir_val as u32),
            );
            count -= 5;
            pin = (pin + 5) & 0x1f;
        }
        self.registers.sm[sm_number as usize]
            .pinctrl
            .modify(SMx_PINCTRL::SET_COUNT.val(count));
        self.registers.sm[sm_number as usize]
            .pinctrl
            .modify(SMx_PINCTRL::SET_BASE.val(pin));
        self.sm_exec(
            sm_number,
            ((0b11100000100 as u32) << 5) | (pindir_val as u32),
        );
        self.registers.sm[sm_number as usize].execctrl.set(execctrl);
        self.registers.sm[sm_number as usize].pinctrl.set(pinctrl);
    }

    /// Immediately execute an instruction on a state machine.
    fn sm_exec(&self, sm_number: SMNumber, instr: u32) {
        self.registers.sm[sm_number as usize]
            .instr
            .modify(SMx_INSTR::INSTR.val(instr));
    }

    /// Write a word of data to a state machine’s TX FIFO.
    pub fn sm_put(&self, sm_number: SMNumber, data: u32) {
        self.registers.txf[sm_number as usize].set(data);
        // self.registers.txf[sm_number as usize].modify(TXFx::TXF.val(data));
    }

    pub fn wait(&self) {}

    /// Wait until a state machine's TX FIFO is empty, then write a word of data to it.
    pub fn sm_put_blocking(&self, sm_number: SMNumber, data: u32) {
        while self.registers.fstat.read(FSTAT::TXFULL0) != 0 {
            self.wait();
        }
        self.registers.txf[sm_number as usize].set(data);
    }

    /// Restart a state machine.
    pub fn restart_sm(&self, sm_number: SMNumber) {
        // SET Reg
        match sm_number {
            SMNumber::SM0 => self.registers.ctrl.modify(CTRL::SM0_RESTART::SET),
            SMNumber::SM1 => self.registers.ctrl.modify(CTRL::SM1_RESTART::SET),
            SMNumber::SM2 => self.registers.ctrl.modify(CTRL::SM2_RESTART::SET),
            SMNumber::SM3 => self.registers.ctrl.modify(CTRL::SM3_RESTART::SET),
        }
    }

    /// Clear a state machine’s TX and RX FIFOs.
    fn sm_clear_fifos(&self, sm_number: SMNumber) {
        // XOR Reg
        self.xor_registers.sm[sm_number as usize]
            .shiftctrl
            .modify(SMx_SHIFTCTRL::FJOIN_RX::SET);
        // XOR Reg
        self.xor_registers.sm[sm_number as usize]
            .shiftctrl
            .modify(SMx_SHIFTCTRL::FJOIN_RX::SET);
    }

    /// Restart a state machine's clock divider.
    pub fn sm_clkdiv_restart(&self, sm_number: SMNumber) {
        match sm_number {
            // SET Reg
            SMNumber::SM0 => self.registers.ctrl.modify(CTRL::CLKDIV0_RESTART::SET),
            SMNumber::SM1 => self.registers.ctrl.modify(CTRL::CLKDIV1_RESTART::SET),
            SMNumber::SM2 => self.registers.ctrl.modify(CTRL::CLKDIV2_RESTART::SET),
            SMNumber::SM3 => self.registers.ctrl.modify(CTRL::CLKDIV3_RESTART::SET),
        }
    }

    // Call this with add_program(include_bytes!("path_to_file")).
    pub fn add_program(&self, program: &[u8]) {
        self.clear_instr_registers();
        let iter = program.chunks(2);
        let mut x = 0;
        for i in iter {
            self.registers.instr_mem[x]
                .instr_mem
                .modify(INSTR_MEMx::INSTR_MEM.val((i[0] as u32) << 8 | (i[1] as u32)));
            x += 1;
            if x == 32 {
                break;
            }
        }
        // debug!("Program added")
    }

    /// Clears all of a PIO instance's instruction memory.
    fn clear_instr_registers(&self) {
        for i in 0..31 {
            self.registers.instr_mem[i]
                .instr_mem
                .modify(INSTR_MEMx::INSTR_MEM::CLEAR);
        }
    }

    /// Initialize a new PIO with the same default configuration for all four state machines.
    pub fn init(&self) {
        let default_config: StateMachineConfiguration = StateMachineConfiguration::default();
        for state_machine in STATE_MACHINE_NUMBERS {
            self.sm_config(state_machine, &default_config);
        }
    }

    pub fn blinking_hello_program_init(
        &mut self,
        pio_number: PIONumber,
        sm_number: SMNumber,
        pin: u32,
        config: &StateMachineConfiguration,
    ) {
        self.sm_config(sm_number, config);
        self.pio_number = pio_number;
        self.gpio_init(&RPGpioPin::new(RPGpio::from_u32(
            pin, /*RPGpio::GPIO7*/
        )));
        self.sm_set_enabled(sm_number, false);
        self.set_pins_out(sm_number, pin, 1, true);
        self.set_set_pins(sm_number, pin, 1);
        self.sm_init(sm_number);
    }

    pub fn blink_program_init(
        &mut self,
        pio_number: PIONumber,
        sm_number: SMNumber,
        pin: u32,
        config: &StateMachineConfiguration,
    ) {
        self.sm_config(sm_number, config);
        self.pio_number = pio_number;
        self.gpio_init(&RPGpioPin::new(RPGpio::from_u32(pin)));
        self.sm_set_enabled(sm_number, false);
        self.set_pins_out(sm_number, pin, 1, true);
        self.set_set_pins(sm_number, pin, 1);
        self.sm_init(sm_number);
    }

    pub fn sideset_program_init(
        &mut self,
        pio_number: PIONumber,
        sm_number: SMNumber,
        pin: u32,
        config: &StateMachineConfiguration,
    ) {
        self.sm_config(sm_number, config);
        self.pio_number = pio_number;
        self.gpio_init(&RPGpioPin::new(RPGpio::from_u32(pin)));
        self.gpio_init(&RPGpioPin::new(RPGpio::GPIO7));
        self.sm_set_enabled(sm_number, false);
        self.set_pins_out(sm_number, pin, 1, true);
        self.set_pins_out(sm_number, 7, 1, true);
        self.set_set_pins(sm_number, pin, 1);
        self.set_side_set_pins(sm_number, 7);
        self.sm_init(sm_number);
    }

    pub fn hello_program_init(
        &mut self,
        pio_number: PIONumber,
        sm_number: SMNumber,
        pin: u32,
        config: &StateMachineConfiguration,
    ) {
        self.sm_config(sm_number, config);
        self.pio_number = pio_number;
        self.gpio_init(&RPGpioPin::new(RPGpio::from_u32(pin)));
        self.sm_set_enabled(sm_number, false);
        self.sm_put(sm_number, 0b11001100110011001100110011001100);
        self.set_pins_out(sm_number, pin, 1, true);
        self.sm_init(sm_number);
        // self.sm_put_blocking(sm_number, 1);
        // for _ in 1..100 {
        //     self.wait();
        // }
        // self.sm_put_blocking(sm_number, 0);
        // for _ in 1..100 {
        //     self.wait();
        // }
        // self.sm_put_blocking(sm_number, 1);
    }

    pub fn pwm_program_init(
        &mut self,
        pio_number: PIONumber,
        sm_number: SMNumber,
        pin: u32,
        pwm_period: u32,
        config: &StateMachineConfiguration,
    ) {
        self.sm_config(sm_number, config);
        self.pio_number = pio_number;
        self.gpio_init(&RPGpioPin::new(RPGpio::from_u32(pin)));
        self.sm_set_enabled(sm_number, false);
        self.set_pins_out(sm_number, pin, 1, true);
        self.set_side_set_pins(sm_number, pin);
        self.sm_init(sm_number);
        self.sm_put_blocking(sm_number, pwm_period);
        self.sm_exec(sm_number, 0x8080 as u32); // pull
        self.sm_exec(sm_number, 0x60c0 as u32); // out isr, 1
    }

    /// Returns current instruction running on the state machine.
    pub fn debugger(&self, sm_number: SMNumber) {
        debug!(
            "SM0:{}",
            self.registers.sm[sm_number as usize]
                .instr
                .read(SMx_INSTR::INSTR)
        );
    }

    pub fn read_sideset_reg(&self, sm_number: SMNumber) {
        debug!(
            "{}",
            self.registers.sm[sm_number as usize]
                .pinctrl
                .read(SMx_PINCTRL::SIDESET_COUNT)
        );
        debug!(
            "{}",
            self.registers.sm[sm_number as usize]
                .execctrl
                .read(SMx_EXECCTRL::SIDE_EN)
        );
        debug!(
            "{}",
            self.registers.sm[sm_number as usize]
                .execctrl
                .read(SMx_EXECCTRL::SIDE_PINDIR)
        );
        debug!(
            "{}",
            self.registers.sm[sm_number as usize]
                .pinctrl
                .read(SMx_PINCTRL::SIDESET_BASE)
        );
    }

    pub fn read_txf(&self, sm_number: SMNumber) -> u32 {
        self.registers.txf[sm_number as usize].read(TXFx::TXF)
    }

    pub fn txf_full_0(&self) -> u32 {
        self.registers.fstat.read(FSTAT::TXFULL0)
    }

    pub fn read_dbg_padout(&self) {
        debug!("{}", self.registers.dbg_padout.read(DBG_PADOUT::DBG_PADOUT));
    }

    pub fn read_fdebug(&self, tx: bool, stall: bool) {
        if tx {
            if stall {
                debug!("{}", self.registers.fdebug.read(FDEBUG::TXSTALL))
            } else {
                debug!("{}", self.registers.fdebug.read(FDEBUG::TXOVER))
            }
        } else if stall {
            debug!("{}", self.registers.fdebug.read(FDEBUG::RXSTALL))
        } else {
            debug!("{}", self.registers.fdebug.read(FDEBUG::RXUNDER))
        }
    }

    // pub fn read_set_base(&self, sm_number: SMNumber) -> u32 {
    //     self.registers.sm[sm_number as usize]
    //         .pinctrl
    //         .read(SMx_PINCTRL::SET_BASE)
    // }

    // pub fn read_set_count(&self, sm_number: SMNumber) -> u32 {
    //     self.registers.sm[sm_number as usize]
    //         .pinctrl
    //         .read(SMx_PINCTRL::SET_COUNT)
    // }
}
