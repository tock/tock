//!  Qdec driver, nRF5x-family
//!  set_client(), enable, get_ticks,
//!  The nRF5x quadrature decoder
//!
#[allow(unused_imports)]
use core;
use kernel::common::cells::OptionalCell;
use kernel::common::registers::{
    register_bitfields, register_structs, ReadOnly, ReadWrite, WriteOnly,
};
use kernel::common::StaticRef;
use kernel::ReturnCode;
use nrf5x::pinmux;
// In this section I declare a struct called QdecRegisters, which contains all the
// relevant registers as outlined in the Nordic 5x specification of the Qdec.
register_structs! {
    pub QdecRegisters {
        /// Start Qdec sampling
        (0x000 => tasks_start: WriteOnly<u32, Task::Register>),
        /// Stop Qdec sampling
        (0x004 => tasks_stop: WriteOnly<u32, Task::Register>),
        /// Read and clear ACC and ACCDBL
        (0x008 => tasks_readclracc: WriteOnly<u32, Task::Register>),
        /// Read and clear ACC
        (0x00C => tasks_rdclracc: WriteOnly<u32, Task::Register>),
        /// Read nad clear ACCDBL
        (0x010 => tasks_rdclrdbl: WriteOnly<u32, Task::Register>),
        ///Reserve space so tasks_rdclrdbl has access to its entire address space (?)
        (0x0014 => _reserved),
        /// All the events which have interrupts!
        (0x100 => events_arr: [ReadWrite<u32, Event::Register>; 5]),
        (0x0114 => _reserved2),
        /// Shortcut register
        (0x200 => shorts: ReadWrite<u32, Shorts::Register>),
        (0x204 => _reserved3),
        /// Enable interrupt
        (0x304 => intenset: ReadWrite<u32, Inte::Register>),
        /// Disable Interrupt
        (0x308 => intenclr: ReadWrite<u32, Inte::Register>),
        (0x30C => _reserved4),
        /// Enable the quad decoder
        (0x500 => enable: ReadWrite<u32, Task::Register>),
        /// Set the LED output pin polarity
        (0x504 => ledpol: WriteOnly<u32, LedPol::Register>),
        /// Sampling-rate register
        (0x508 => sample_per: WriteOnly<u32, SampPer::Register>),
        /// Sample register (receives all samples)
        (0x50C => sample: WriteOnly<u32, Sample::Register>),
        /// Reportper
        (0x510 => report_per: ReadOnly<u32, ReportPer::Register>),
        /// Accumulating motion-sample values register
        (0x514 => acc: ReadOnly<u32, Acc::Register>),
        (0x518 => acc_read: ReadOnly<u32, Acc::Register>),
        (0x51C => reserved6),
        (0x520 => psel_a: ReadWrite<u32, PinSelect::Register>),
        (0x524 => psel_b: ReadWrite<u32, PinSelect::Register>),
        (0x528 => reserved5),
        (0x544 => accdbl: ReadOnly<u32, AccDbl::Register>),
        (0x548 => accdbl_read: ReadOnly<u32, AccDbl::Register>),
        (0x54C => @END),
    }
}

register_bitfields![u32,
    Task [
        /// Enable
        ENABLE 0
    ],
    Shorts [
        /// Write '1' to Enable shortcut on EVENTS_COMPARE\[0\] event
        REPORTRDY_READCLRACC 0,
        /// Write '1' to Enable shortcut on EVENTS_COMPARE\[1\] event
        SAMPLERDY_STOP 1,
        /// Write '1' to Enable shortcut on EVENTS_COMPARE\[2\] event
        REPORTRDY_RDCLRACC 2,
        /// Write '1' to Enable shortcut on EVENTS_COMPARE\[3\] event
        REPORTRDY_STOP 3,
        /// Write '1' to Enable shortcut on EVENTS_COMPARE\[4\] event
        DBLRDY_RDCLRDBL 4,
        /// Write '1' to Enable shortcut on EVENTS_COMPARE\[5\] event
        DBLRDY_STOP 5,
        /// Write '1' to Enable shortcut on EVENTS_COMPARE\[6\] event
        SAMPLERDY_READCLRACC 6
    ],
    Event [
        /// Ready to receive interrupts
        READY 0
    ],
    PinSelect [
        /// Pins the QDEC is attached to
        Pin OFFSET(0) NUMBITS(5),
        /// Port the QDEC is using
        Port OFFSET(5) NUMBITS(1),
        /// Connects pins
        Connect OFFSET(31) NUMBITS(1)
    ],
    Inte [
        /// Write '1' to Enable interrupt on EVENTS_COMPARE\[0\] event
        SAMPLERDY 0,
        /// Write '1' to Enable interrupt on EVENTS_COMPARE\[1\] event
        REPORTRDY 1,
        /// Write '1' to Enable interrupt on EVENTS_COMPARE\[2\] event
        ACCOF 2,
        /// Write '1' to Enable interrupt on EVENTS_COMPARE\[3\] event
        DBLRDY 3,
        /// Write '1' to Enable interrupt on EVENTS_COMPARE\[4\] event
        STOPPED 4
    ],
    LedPol [
        /// LED pin polarity
        LedPol OFFSET(0) NUMBITS(1) [
            /// Active: Low
            ActiveLow = 0,
            /// Active: High
            ActiveHigh = 1
        ]
    ],
    Sample [
        /// Last motion sample
        SAMPLE 1
    ],
    SampPer [
        /// Sample period
        SAMPLEPER OFFSET(0) NUMBITS(4) [
            /// Sample every 128 microseconds
            us128 = 0,
            /// Sample every 256 microseconds
            us256 = 1,
            /// Sample every 512 microseconds
            us512 = 2,
            /// Sample every 1024 microseconds
            us1024 = 3,
            /// Sample every 2048 microseconds
            us2048 = 4,
            /// Sample every 4096 microseconds
            us4096 = 5,
            /// Sample every 8192 microseconds
            us8192 = 6,
            /// Sample every 16384 microseconds
            us16384 = 7,
            /// Sample every 32 milliseconds
            ms32 = 8,
            /// Sample every 65 milliseconds
            ms65 = 9,
            /// Sample every 131 milliseconds
            ms131 = 10
        ]
    ],
    ReportPer [
        /// Captures several samples before being sent out by REPORTRDY event
        REPORTPER OFFSET(0) NUMBITS(4) [
            /// 10 samples/report
            hz10 = 0,
            /// 40 samples/report
            hz40 = 1,
            /// 80 samples/report
            hz80 = 2,
            /// 120 samples/report
            hz120 = 3,
            /// 160 samples/report
            hz160 = 4,
            /// 200 samples/report
            hz200 = 5,
            /// 240 samples/report
            hz240 = 6,
            /// 280 samples/report
            hz280 = 7,
            /// 1 sample/report
            hz1 = 8
        ]
    ],
    Acc [
        /// Valid motion value samples 
        ACC OFFSET(0) NUMBITS(32)
    ],
    AccDbl [
        /// Invalid motion value samples
        ACCDBL OFFSET(0) NUMBITS(4)
    ]
];

/// This defines the beginning of memory which is memory-mapped to the Qdec
/// This base is declared under the Registers Table 3
const QDEC_BASE: StaticRef<QdecRegisters> =
    unsafe { StaticRef::new(0x40012000 as *const QdecRegisters) };

pub static mut QDEC: Qdec = Qdec::new();

/// Enum defining the current QDEC state of started or stopped
#[derive(PartialEq, Eq)]
enum QdecState {
    Start,
    Stop,
}
/// Qdec type declaration: gives the Qdec instance registers and a client
pub struct Qdec {
    registers: StaticRef<QdecRegisters>,
    client: OptionalCell<&'static dyn kernel::hil::qdec::QdecClient>,
    state: QdecState,
}

/// Qdec impl: provides the Qdec type with vital functionality including:
impl Qdec {
    const fn new() -> Qdec {
        let qdec = Qdec {
            registers: QDEC_BASE,
            client: OptionalCell::empty(),
            state: QdecState::Start,
        };
        qdec
    }

    /// sets pins_a and pins_b to be the output pins for whatever the encoding device is  
    pub fn set_pins(&self, pin_a: pinmux::Pinmux, pin_b: pinmux::Pinmux) {
        let regs = self.registers;
        regs.psel_a.write(
            PinSelect::Pin.val(pin_a.into()) + PinSelect::Port.val(0) + PinSelect::Connect.val(0),
        );
        regs.psel_b.write(
            PinSelect::Pin.val(pin_b.into()) + PinSelect::Port.val(0) + PinSelect::Connect.val(0),
        );
    }

    pub fn set_client(&self, client: &'static dyn kernel::hil::qdec::QdecClient) {
        self.client.set(client);
    }

    /// When an interrupt occurs, check to see if any
    /// of the interrupt register bits are set. If it
    /// is, then put it in the client's bitmask
    pub(crate) fn handle_interrupt(&self) {
        let regs = &*self.registers;
        self.client.map(|client| {
            // For each of 4 possible compare events, if it's happened,
            // clear it and sort its bit in val to pass in callback
            for i in 0..regs.events_arr.len() {
                if regs.events_arr[i].is_set(Event::READY) {
                    regs.events_arr[i].set(0);
                    match i {
                        0 => {
                            client.sample_ready();
                        }
                        1 => { /*For the time being, there will be no implementation in respone
                                 to the firing of the REPORTRDY interrupt. It fires too frequently
                                 in order to utilize it for position measuring. However, by that same 
                                 token, it cannot merely be marked as unimplemented since it would
                                 crash the program.*/ }
                        2 => {
                            client.overflow();
                        }
                        3 => { /*No handling for DBLRDY*/ }
                        4 => {
                            if self.state == QdecState::Stop {
                                self.registers.sample_per.write(SampPer::SAMPLEPER.val(5));
                                self.registers.tasks_start.write(Task::ENABLE::SET);
                            }
                        }
                        _ => panic!("Unsupported interrupt value {}!", i),
                    }
                }
            }
        });
    }

    fn enable_samplerdy_interrupts(&self) {
        let regs = &*self.registers;

        regs.intenset.write(Inte::REPORTRDY::SET); /*SET REPORT READY*/
        regs.intenset.write(Inte::ACCOF::SET); /*SET ACCOF READY*/
    }

    fn enable(&self) {
        let regs = &*self.registers;
        regs.enable.write(Task::ENABLE::SET);
        regs.tasks_start.write(Task::ENABLE::SET);
    }

    fn disable(&self) {
        let regs = &*self.registers;
        regs.enable.write(Task::ENABLE::CLEAR);
        regs.tasks_start.write(Task::ENABLE::CLEAR);
    }

    fn is_enabled(&self) -> ReturnCode {
        let regs = &*self.registers;
        let result = if regs.enable.is_set(Task::ENABLE) {
            ReturnCode::SUCCESS
        } else {
            ReturnCode::FAIL
        };
        result
    }

    fn is_disabled(&self) -> ReturnCode {
        let regs = &*self.registers;
        let result = if regs.enable.is_set(Task::ENABLE) {
            ReturnCode::FAIL
        } else {
            ReturnCode::SUCCESS
        };
        result
    }
}

impl kernel::hil::qdec::QdecDriver for Qdec {
    fn enable_interrupts(&self) -> ReturnCode {
        self.enable_samplerdy_interrupts();
        ReturnCode::SUCCESS
    }

    fn enable_qdec(&self) -> ReturnCode {
        if self.is_enabled() != ReturnCode::SUCCESS {
            self.enable();
        }
        self.is_enabled()
    }

    fn disable_qdec(&self) -> ReturnCode {
        if self.is_enabled() == ReturnCode::SUCCESS {
            self.disable();
        }
        self.is_disabled()
    }

    fn enabled(&self) -> ReturnCode {
        self.is_enabled()
    }

    fn disabled(&self) -> ReturnCode {
        self.is_disabled()
    }

    fn get_acc(&self) -> i32 {
        let regs = &*self.registers;
        regs.tasks_readclracc.write(Task::ENABLE::SET);
        let val = regs.acc_read.read(Acc::ACC);
        val as i32
    }

    fn set_client(&self, client: &'static dyn kernel::hil::qdec::QdecClient) {
        self.client.set(client);
    }
}
