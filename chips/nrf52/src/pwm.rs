//! PWM driver for nRF52.

use kernel::common::cells::VolatileCell;
use kernel::common::registers::{register_bitfields, ReadWrite, WriteOnly};
use kernel::common::StaticRef;
use kernel::hil;
use kernel::ReturnCode;
use nrf5x;

#[repr(C)]
struct PwmRegisters {
    _reserved0: [u8; 4],
    /// Stops PWM pulse generation on all channels at the end of current PWM period
    tasks_stop: WriteOnly<u32, TASK::Register>,
    /// Loads the first PWM value on all enabled channels
    tasks_seqstart: [WriteOnly<u32, TASK::Register>; 2],
    /// Steps by one value in the current sequence on all enabled channels if DECODER.MO
    tasks_nextstep: WriteOnly<u32, TASK::Register>,
    _reserved1: [u8; 240],
    /// Response to STOP task, emitted when PWM pulses are no longer generated
    events_stopped: ReadWrite<u32, EVENT::Register>,
    /// First PWM period started on sequence 0
    events_seqstarted: [ReadWrite<u32, EVENT::Register>; 2],
    /// Emitted at end of every sequence 0, when last value from RAM has been
    /// applied to the wave counter
    events_seqend: [ReadWrite<u32, EVENT::Register>; 2],
    /// Emitted at the end of each PWM period
    events_pwmperiodend: ReadWrite<u32, EVENT::Register>,
    /// Concatenated sequences have been played the amount of times defined in LOOP.CNT
    events_loopsdone: ReadWrite<u32, EVENT::Register>,
    _reserved2: [u8; 224],
    /// Shortcut register
    shorts: ReadWrite<u32, SHORTS::Register>,
    _reserved3: [u8; 252],
    /// Enable or disable interrupt
    inten: ReadWrite<u32, INTEN::Register>,
    /// Enable interrupt
    intenset: ReadWrite<u32, INTEN::Register>,
    /// Disable interrupt
    intenclr: ReadWrite<u32, INTEN::Register>,
    _reserved4: [u8; 500],
    /// PWM module enable register
    enable: ReadWrite<u32, ENABLE::Register>,
    /// Selects operating mode of the wave counter
    mode: ReadWrite<u32, MODE::Register>,
    /// Value up to which the pulse generator counter counts
    countertop: ReadWrite<u32, COUNTERTOP::Register>,
    /// Configuration for PWM_CLK
    prescaler: ReadWrite<u32, PRESCALER::Register>,
    /// Configuration of the decoder
    decoder: ReadWrite<u32, DECODER::Register>,
    /// Amount of playback of a loop
    loopreg: ReadWrite<u32, LOOP::Register>,
    _reserved5: [u8; 8],
    seq0: PwmSeqRegisters,
    _reserved6: [u8; 16],
    seq1: PwmSeqRegisters,
    _reserved7: [u8; 16],
    psel_out: [VolatileCell<nrf5x::pinmux::Pinmux>; 4],
}

#[repr(C)]
struct PwmSeqRegisters {
    seq_ptr: VolatileCell<*const u16>,
    seq_cnt: ReadWrite<u32, SEQ_CNT::Register>,
    seq_refresh: ReadWrite<u32, SEQ_REFRESH::Register>,
    seq_enddelay: ReadWrite<u32, SEQ_ENDDELAY::Register>,
}

register_bitfields![u32,
    SHORTS [
        /// Shortcut between EVENTS_SEQEND[0] event and TASKS_STOP task
        SEQEND0_STOP 0,
        /// Shortcut between EVENTS_SEQEND[1] event and TASKS_STOP task
        SEQEND1_STOP 1,
        /// Shortcut between EVENTS_LOOPSDONE event and TASKS_SEQSTART[0] task
        LOOPSDONE_SEQSTART0 2,
        /// Shortcut between EVENTS_LOOPSDONE event and TASKS_SEQSTART[1] task
        LOOPSDONE_SEQSTART1 3,
        /// Shortcut between EVENTS_LOOPSDONE event and TASKS_STOP task
        LOOPSDONE_STOP 4
    ],
    INTEN [
        /// Enable or disable interrupt on EVENTS_STOPPED event
        STOPPED 1,
        /// Enable or disable interrupt on EVENTS_SEQSTARTED[0] event
        SEQSTARTED0 2,
        /// Enable or disable interrupt on EVENTS_SEQSTARTED[1] event
        SEQSTARTED1 3,
        /// Enable or disable interrupt on EVENTS_SEQEND[0] event
        SEQEND0 4,
        /// Enable or disable interrupt on EVENTS_SEQEND[1] event
        SEQEND1 5,
        /// Enable or disable interrupt on EVENTS_PWMPERIODEND event
        PWMPERIODEND 6,
        /// Enable or disable interrupt on EVENTS_LOOPSDONE event
        LOOPSDONE 7
    ],
    ENABLE [
        ENABLE 0
    ],
    MODE [
        UPDOWN OFFSET(0) NUMBITS(1) [
            Up = 0,
            UpAndDown = 1
        ]
    ],
    COUNTERTOP [
        COUNTERTOP OFFSET(0) NUMBITS(15) []
    ],
    PRESCALER [
        PRESCALER OFFSET(0) NUMBITS(3) [
            DIV_1 = 0,
            DIV_2 = 1,
            DIV_4 = 2,
            DIV_8 = 3,
            DIV_16 = 4,
            DIV_32 = 5,
            DIV_64 = 6,
            DIV_128 = 7
        ]
    ],
    DECODER [
        /// How a sequence is read from RAM and spread to the compare register
        LOAD OFFSET(0) NUMBITS(2) [
            /// 1st half word (16-bit) used in all PWM channels 0..3
            Common = 0,
            /// 1st half word (16-bit) used in channel 0..1; 2nd word in channel 2..3
            Grouped = 1,
            /// 1st half word (16-bit) in ch.0; 2nd in ch.1; ...; 4th in ch.3
            Individual = 2,
            /// 1st half word (16-bit) in ch.0; 2nd in ch.1; ...; 4th in COUNTERTOP
            Waveform = 3
        ],
        /// Selects source for advancing the active sequence
        MODE OFFSET(8) NUMBITS(1) [
            /// SEQ[n].REFRESH is used to determine loading internal compare registers
            RefreshCount = 0,
            /// NEXTSTEP task causes a new value to be loaded to internal compare registers
            NextStep = 1
        ]
    ],
    LOOP [
        /// Number of playbacks of pattern cycles. 0 to disable.
        CNT OFFSET(0) NUMBITS(16) []
    ],
    EVENT [
        EVENT 0
    ],
    TASK [
        TASK 0
    ],
    SEQ_CNT [
        CNT OFFSET(0) NUMBITS(15) []
    ],
    SEQ_REFRESH [
        CNT OFFSET(0) NUMBITS(24) []
    ],
    SEQ_ENDDELAY [
        CNT OFFSET(0) NUMBITS(24) []
    ]
];

const PWM0_BASE: StaticRef<PwmRegisters> =
    unsafe { StaticRef::new(0x4001C000 as *const PwmRegisters) };

pub static mut PWM0: Pwm = Pwm::new(PWM0_BASE);

/// `DUTY_CYCLES` is a static array that must be passed to the PWM hardware.
/// The nRF52 hardware uses this static array in memory to enable switching
/// between multiple duty cycles automatically while generating the PWM output.
/// This isn't ideal from a Rust perspective, but the peripheral hardware must
/// be passed a pointer.
static mut DUTY_CYCLES: [u16; 4] = [0; 4];

pub struct Pwm {
    registers: StaticRef<PwmRegisters>,
}

impl Pwm {
    const fn new(registers: StaticRef<PwmRegisters>) -> Pwm {
        Pwm {
            registers: registers,
        }
    }

    fn start_pwm(
        &self,
        pin: &nrf5x::pinmux::Pinmux,
        frequency_hz: usize,
        duty_cycle: usize,
    ) -> ReturnCode {
        let prescaler = 0;
        let counter_top = (16000000 / frequency_hz) >> prescaler;

        // Use the passed in duty cycle to calculate the value we pass to the
        // hardware. A 50% duty cycle is half of counter_top, a 10% duty cycle
        // would be 90% of counter_top.
        //
        //                               duty_cycle
        //  dc_out = counter_top * (1 -  ---------- )
        //                                5333333
        let dc_out = counter_top - ((3 * duty_cycle) / frequency_hz);

        // Configure the pin
        self.registers.psel_out[0].set(*pin);

        // Start by enabling the peripheral.
        self.registers.enable.write(ENABLE::ENABLE::SET);
        // Want count up mode.
        self.registers.mode.write(MODE::UPDOWN::Up);
        // Disable loop (repeat) mode.
        self.registers.loopreg.write(LOOP::CNT.val(0));
        // Set the decoder settings.
        self.registers
            .decoder
            .write(DECODER::LOAD::Common + DECODER::MODE::RefreshCount);
        // Set the prescaler.
        self.registers.prescaler.write(PRESCALER::PRESCALER::DIV_1);
        // Set the value to count to.
        self.registers
            .countertop
            .write(COUNTERTOP::COUNTERTOP.val(counter_top as u32));

        // Setup the duty cycles
        unsafe {
            DUTY_CYCLES[0] = dc_out as u16;
            self.registers.seq0.seq_ptr.set(&DUTY_CYCLES as *const u16);
        }
        self.registers.seq0.seq_cnt.write(SEQ_CNT::CNT.val(1));
        self.registers
            .seq0
            .seq_refresh
            .write(SEQ_REFRESH::CNT.val(0));
        self.registers
            .seq0
            .seq_enddelay
            .write(SEQ_ENDDELAY::CNT.val(0));

        // Start
        self.registers.tasks_seqstart[0].write(TASK::TASK::SET);

        ReturnCode::SUCCESS
    }

    fn stop_pwm(&self, _pin: &nrf5x::pinmux::Pinmux) -> ReturnCode {
        self.registers.tasks_stop.write(TASK::TASK::SET);
        self.registers.enable.write(ENABLE::ENABLE::CLEAR);
        ReturnCode::SUCCESS
    }
}

impl hil::pwm::Pwm for Pwm {
    type Pin = nrf5x::pinmux::Pinmux;

    fn start(&self, pin: &Self::Pin, frequency: usize, duty_cycle: usize) -> ReturnCode {
        self.start_pwm(pin, frequency, duty_cycle)
    }

    fn stop(&self, pin: &Self::Pin) -> ReturnCode {
        self.stop_pwm(pin)
    }

    fn get_maximum_frequency_hz(&self) -> usize {
        // Counter runs at 16 MHz, and the minimum value for the COUNTERTOP
        // register is 3. 16000000 / 3 = 5333333
        5333333
    }

    fn get_maximum_duty_cycle(&self) -> usize {
        // We use the max frequency as the max duty cycle as well. This makes
        // calculating `dc_out` straightforward.
        5333333
    }
}
