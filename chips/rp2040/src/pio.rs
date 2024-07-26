use core::ffi::c_uint;

use crate::gpio::{GpioFunction, RPGpioPin};
use kernel::deferred_call::DeferredCallClient;
use kernel::hil::gpio::Output;
use kernel::utilities::registers::interfaces::{ReadWriteable, Readable, Writeable};
use kernel::utilities::registers::{register_bitfields, register_structs, ReadOnly, ReadWrite};
use kernel::utilities::StaticRef;

const NUMBER_STATE_MACHINES: usize = 4;
const NUMBER_INSTR_MEMORY_LOCATIONS: usize = 32;
const NUMBER_IRQS: usize = 2;

#[repr(C)]
struct InstrMem {
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
    enable: ReadWrite<u32, SM_INT::Register>,
    /// Interrupt Force for irq x
    force: ReadWrite<u32, SM_INT::Register>,
    /// Interrupt status after masking & forcing for irq x
    status: ReadOnly<u32, SM_INT::Register>,
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
        (0x128 => intr: ReadOnly<u32, SM_INT::Register>),
        /// Irq 1 and 0 - Interrupt Enable, Force, Status
        (0x12c => irqx: [Irq; NUMBER_IRQS]),
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
    CLKDIV_RESTART OFFSET(8) NUMBITS(4) [],
    /// Write 1 to instantly clear internal SM state which may be
    /// otherwise difficult to access and will affect future
    /// execution.
    /// Specifically, the following are cleared: input and output
    /// shift counters; the contents of the input shift register; the
    /// delay counter; the waiting-on-IRQ state; any stalled
    /// instruction written to SMx_INSTR or run by OUT/MOV
    /// EXEC; any pin write left asserted due to OUT_STICKY.
    SM_RESTART OFFSET(4) NUMBITS(4) [],
    /// Enable/disable each of the four state machines by writing
    /// 1/0 to each of these four bits. When disabled, a state
    /// machine will cease executing instructions, except those
    /// written directly to SMx_INSTR by the system. Multiple bits
    /// can be set/cleared at once to run/halt multiple state
    /// machines simultaneously.
    SM_ENABLE OFFSET(0) NUMBITS(4) []
],
FSTAT [
    /// State machine TX FIFO is empty
    TXEMPTY OFFSET(24) NUMBITS(4) [],
    /// State machine TX FIFO is full
    TXFULL OFFSET(16) NUMBITS(4) [],
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
SM_INT [
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
]
];

const PIO_0_BASE_ADDRESS: usize = 0x50200000;
const PIO_1_BASE_ADDRESS: usize = 0x50300000;
const PIO0_BASE: StaticRef<PioRegisters> =
    unsafe { StaticRef::new(PIO_0_BASE_ADDRESS as *const PioRegisters) };
const PIO1_BASE: StaticRef<PioRegisters> =
    unsafe { StaticRef::new(PIO_1_BASE_ADDRESS as *const PioRegisters) };

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum SMNumber {
    SM0 = 0,
    SM1 = 1,
    SM2 = 2,
    SM3 = 3,
}

#[derive(PartialEq)]
pub enum PIONumber {
    PIO0 = 0,
    PIO1 = 1,
}

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
}

#[derive(Clone, Copy)]
pub enum PioMovStatusType {
    StatusTxLessthan = 0,
    StatusRxLessthan = 1,
}

pub struct StateMachineConfiguration {
    out_pins_count: u32,
    out_pins_base: u32,
    set_pins_count: u32,
    set_pins_base: u32,
    in_pins_base: u32,
    side_set_base: u32,
    side_set_enable: bool,
    side_set_bit_count: u32,
    side_set_pindirs: bool,
    wrap: u32,
    wrap_to: u32,
    in_shift_direction_right: bool,
    in_autopush: bool,
    in_push_threshold: u32,
    out_shift_direction_right: bool,
    out_autopull: bool,
    out_pull_threshold: u32,
    jmp_pin: u32,
    out_special_sticky: bool,
    out_special_has_enable_pin: bool,
    out_special_enable_pin_index: u32,
    mov_status_sel: PioMovStatusType,
    mov_status_n: u32,
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
            side_set_enable: false,
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
        }
    }
}

impl Pio {
    pub fn sm_config(&self, sm_number: SMNumber, config: &StateMachineConfiguration) {
        self.set_in_pins(sm_number, config.in_pins_base);
        self.set_out_pins(sm_number, config.out_pins_base, config.out_pins_count);
        self.set_set_pins(sm_number, config.set_pins_base, config.set_pins_count);
        self.set_side_set_pins(sm_number, config.side_set_base);
        self.set_side_set(
            sm_number,
            config.side_set_bit_count,
            config.side_set_enable,
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
        self.set_mov_status(sm_number, config.mov_status_sel, config.mov_status_n)
    }

    pub fn new_pio0() -> Self {
        Self {
            registers: PIO0_BASE,
            pio_number: PIONumber::PIO0,
        }
    }

    pub fn new_pio1() -> Self {
        Self {
            registers: PIO1_BASE,
            pio_number: PIONumber::PIO1,
        }
    }

    pub fn init(&self) {
        // self.new_pio0();
        // self.new_pio1();
        let default_config: StateMachineConfiguration = StateMachineConfiguration::default();
        for state_machine in STATE_MACHINE_NUMBERS {
            self.sm_config(state_machine, &default_config);
            // self.set_counter(channel_number, 0);
            // self.disable_interrupt(channel_number);
        }
        // self.registers.intr.write(CH::CH.val(0));
    }

    // pub fn check_pio_param() -> bool {
    //     if (pio.pio_number != 1 && pio.pio_number != 0) {
    //         return false;
    //     }
    //     return true;
    // }

    // pub fn check_sm_param(sm_number: SMNumber) -> bool {
    //     if (sm_number as u32 == 0) || (sm_number as u32 == (NUMBER_STATE_MACHINES - 1) as u32) {
    //         return true;
    //     }
    //     return false;
    // }

    fn set_in_pins(&self, sm_number: SMNumber, in_base: u32) {
        self.registers.sm[sm_number as usize]
            .pinctrl
            .modify(SMx_PINCTRL::IN_BASE.val(in_base));
    }

    fn set_set_pins(&self, sm_number: SMNumber, set_base: u32, set_count: u32) {
        self.registers.sm[sm_number as usize]
            .pinctrl
            .modify(SMx_PINCTRL::SET_BASE.val(set_base));
        self.registers.sm[sm_number as usize]
            .pinctrl
            .modify(SMx_PINCTRL::SET_COUNT.val(set_count));
    }

    fn set_out_pins(&self, sm_number: SMNumber, out_base: u32, out_count: u32) {
        self.registers.sm[sm_number as usize]
            .pinctrl
            .modify(SMx_PINCTRL::OUT_BASE.val(out_base));
        self.registers.sm[sm_number as usize]
            .pinctrl
            .modify(SMx_PINCTRL::OUT_COUNT.val(out_count));
    }

    pub fn set_enabled(&self, enabled: bool) {
        // if Pio::check_pio_param() && Pio::check_sm_param(sm_number) {
        self.registers.ctrl.modify(match enabled {
            true => CTRL::SM_ENABLE::SET,
            false => CTRL::SM_ENABLE::CLEAR,
        });
        // }
    }

    fn set_in_shift(
        &self,
        sm_number: SMNumber,
        shift_right: bool,
        autopush: bool,
        push_threshold: u32,
    ) {
        self.registers.sm[sm_number as usize]
            .shiftctrl
            .modify(SMx_SHIFTCTRL::IN_SHIFTDIR.val(u32::from(shift_right)));
        self.registers.sm[sm_number as usize]
            .shiftctrl
            .modify(SMx_SHIFTCTRL::AUTOPUSH.val(u32::from(autopush)));
        self.registers.sm[sm_number as usize]
            .shiftctrl
            .modify(SMx_SHIFTCTRL::PUSH_THRESH.val(u32::from(push_threshold)));
    }

    fn set_out_shift(
        &self,
        sm_number: SMNumber,
        shift_right: bool,
        autopull: bool,
        pull_threshold: u32,
    ) {
        self.registers.sm[sm_number as usize]
            .shiftctrl
            .modify(SMx_SHIFTCTRL::OUT_SHIFTDIR.val(u32::from(shift_right)));
        self.registers.sm[sm_number as usize]
            .shiftctrl
            .modify(SMx_SHIFTCTRL::AUTOPULL.val(u32::from(autopull)));
        self.registers.sm[sm_number as usize]
            .shiftctrl
            .modify(SMx_SHIFTCTRL::PULL_THRESH.val(u32::from(pull_threshold)));
    }

    fn set_jmp_pin(&self, sm_number: SMNumber, pin: u32) {
        self.registers.sm[sm_number as usize]
            .execctrl
            .modify(SMx_EXECCTRL::JMP_PIN.val(pin));
    }

    // pub fn set_clkdiv(&self, sm_number: SMNumber, div: c_float){
    //     self.registers.sm[sm_number as usize]
    //         .clkdiv
    //         .modify(SMx_CLKDIV::INT.val(div));
    // }

    fn set_clkdiv_int_frac(&self, sm_number: SMNumber, div_int: c_uint, div_frac: c_uint) {
        //c_uint is u32, shall we use signed u8 or u16 instead?
        self.registers.sm[sm_number as usize]
            .clkdiv
            .modify(SMx_CLKDIV::INT.val(div_int));
        self.registers.sm[sm_number as usize]
            .clkdiv
            .modify(SMx_CLKDIV::FRAC.val(div_frac));
    }

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

    fn set_side_set_pins(&self, sm_number: SMNumber, sideset_base: c_uint) {
        self.registers.sm[sm_number as usize]
            .pinctrl
            .modify(SMx_PINCTRL::SIDESET_BASE.val(sideset_base));
    }

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

    fn gpio_init(&self, pin: RPGpioPin) {
        if self.pio_number == PIONumber::PIO0 {
            pin.set_function(GpioFunction::PIO0)
        } else {
            pin.set_function(GpioFunction::PIO1)
        }
    }

    fn set_wrap(&self, sm_number: SMNumber, wrap_target: u32, wrap: u32) {
        self.registers.sm[sm_number as usize]
            .execctrl
            .modify(SMx_EXECCTRL::WRAP_BOTTOM.val(wrap_target));
        self.registers.sm[sm_number as usize]
            .execctrl
            .modify(SMx_EXECCTRL::WRAP_TOP.val(wrap));
    }

    fn set_mov_status(&self, sm_number: SMNumber, status_sel: PioMovStatusType, status_n: u32) {
        self.registers.sm[sm_number as usize]
            .execctrl
            .modify(SMx_EXECCTRL::STATUS_SEL.val(status_sel as u32));
        self.registers.sm[sm_number as usize]
            .execctrl
            .modify(SMx_EXECCTRL::STATUS_N.val(status_n));
    }
}
