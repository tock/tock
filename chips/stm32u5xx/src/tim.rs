use kernel::hil::time::{self, Alarm, Ticks, Ticks32};
use kernel::utilities::cells::OptionalCell;
use kernel::utilities::registers::interfaces::{ReadWriteable, Readable, Writeable};
use kernel::utilities::registers::{register_bitfields, register_structs, ReadWrite};
use kernel::utilities::StaticRef;

register_structs! {
    pub TimRegisters {
        /// Control register 1
        (0x000 => cr1: ReadWrite<u32, CR1::Register>),
        (0x004 => _reserved0),
        /// DMA/Interrupt enable register
        (0x00C => dier: ReadWrite<u32, DIER::Register>),
        /// Status register
        (0x010 => sr: ReadWrite<u32, SR::Register>),
        /// Event generation register
        (0x014 => egr: ReadWrite<u32>),
        (0x018 => _reserved1),
        /// Counter
        (0x024 => cnt: ReadWrite<u32>),
        /// Prescaler
        (0x028 => psc: ReadWrite<u32>),
        /// Auto-reload register
        (0x02C => arr: ReadWrite<u32>),
        (0x030 => _reserved2),
        /// Capture/compare register 1
        (0x034 => ccr1: ReadWrite<u32>),
        (0x038 => @END),
    }
}

register_bitfields![u32,
    pub CR1 [
        /// Counter enable
        CEN OFFSET(0) NUMBITS(1) []
    ],
    pub DIER [
        /// Update interrupt enable
        UIE  OFFSET(0) NUMBITS(1) [],
        /// CC1 interrupt enable
        CC1IE OFFSET(1) NUMBITS(1) []
    ],
    pub SR [
        /// Update interrupt flag
        UIF  OFFSET(0) NUMBITS(1) [],
        /// CC1 interrupt flag
        CC1IF OFFSET(1) NUMBITS(1) []
    ]
];

pub struct Tim2<'a> {
    registers: StaticRef<TimRegisters>,
    client: OptionalCell<&'a dyn time::AlarmClient>,
}

impl<'a> Tim2<'a> {
    pub const fn new(base: StaticRef<TimRegisters>) -> Tim2<'a> {
        Tim2 {
            registers: base,
            client: OptionalCell::empty(),
        }
    }

    pub fn enable_clock(&self) {
        // Secure Alias for RCC_APB1ENR1 (from working C code)
        let rcc_apb1enr1 = 0x46020C9C as *mut u32;
        unsafe {
            core::ptr::write_volatile(rcc_apb1enr1, core::ptr::read_volatile(rcc_apb1enr1) | 1);
        }
    }

    pub fn handle_interrupt(&self) {
        // Clear interrupt flag
        self.registers.sr.modify(SR::CC1IF::CLEAR);

        self.client.map(|client| {
            client.alarm();
        });
    }
}

impl time::Time for Tim2<'_> {
    type Frequency = time::Freq32KHz; // Adjust based on your clock
    type Ticks = Ticks32;

    fn now(&self) -> Ticks32 {
        Ticks32::from(self.registers.cnt.get())
    }
}

impl<'a> time::Alarm<'a> for Tim2<'a> {
    fn set_alarm_client(&self, client: &'a dyn time::AlarmClient) {
        self.client.set(client);
    }

    fn set_alarm(&self, reference: Ticks32, dt: Ticks32) {
        let target = reference.wrapping_add(dt);
        self.registers.ccr1.set(target.into_u32());
        self.registers.dier.modify(DIER::CC1IE::SET);
        self.registers.cr1.modify(CR1::CEN::SET);
    }

    fn get_alarm(&self) -> Ticks32 {
        Ticks32::from(self.registers.ccr1.get())
    }

    fn is_armed(&self) -> bool {
        self.registers.dier.is_set(DIER::CC1IE)
    }

    fn disarm(&self) -> Result<(), kernel::ErrorCode> {
        self.registers.dier.modify(DIER::CC1IE::CLEAR);
        Ok(())
    }

    fn minimum_dt(&self) -> Ticks32 {
        Ticks32::from(1)
    }
}
