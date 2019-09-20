//! Implementation of the SAM4L TRNG. It provides an implementation of
//! the Entropy32 trait.

use crate::pm;
use kernel::common::cells::OptionalCell;
use kernel::common::registers::{register_bitfields, ReadOnly, WriteOnly};
use kernel::common::StaticRef;
use kernel::hil::entropy::{self, Continue};
use kernel::ReturnCode;

#[repr(C)]
struct TrngRegisters {
    cr: WriteOnly<u32, Control::Register>,
    _reserved0: [u32; 3],
    ier: WriteOnly<u32, Interrupt::Register>,
    idr: WriteOnly<u32, Interrupt::Register>,
    imr: ReadOnly<u32, Interrupt::Register>,
    isr: ReadOnly<u32, Interrupt::Register>,
    _reserved1: [u32; 12],
    odata: ReadOnly<u32, OutputData::Register>,
}

register_bitfields![u32,
    Control [
        /// Security Key
        KEY OFFSET(8) NUMBITS(24) [],
        /// Enables the TRNG to provide random values
        ENABLE OFFSET(0) NUMBITS(1) [
            Disable = 0,
            Enable = 1
        ]
    ],

    Interrupt [
        /// Data Ready
        DATRDY 0
    ],

    OutputData [
        /// Output Data
        ODATA OFFSET(0) NUMBITS(32) []
    ]
];

const BASE_ADDRESS: StaticRef<TrngRegisters> =
    unsafe { StaticRef::new(0x40068000 as *const TrngRegisters) };

pub struct Trng<'a> {
    regs: StaticRef<TrngRegisters>,
    client: OptionalCell<&'a dyn entropy::Client32>,
}

pub static mut TRNG: Trng<'static> = Trng::new();
const KEY: u32 = 0x524e47;

impl Trng<'a> {
    const fn new() -> Trng<'a> {
        Trng {
            regs: BASE_ADDRESS,
            client: OptionalCell::empty(),
        }
    }

    pub fn handle_interrupt(&self) {
        let regs = &*self.regs;

        regs.idr.write(Interrupt::DATRDY::SET);

        self.client.map(|client| {
            let result = client.entropy_available(&mut TrngIter(self), ReturnCode::SUCCESS);
            if let Continue::Done = result {
                // disable controller
                regs.cr
                    .write(Control::KEY.val(KEY) + Control::ENABLE::Disable);
                pm::disable_clock(pm::Clock::PBA(pm::PBAClock::TRNG));
            } else {
                regs.ier.write(Interrupt::DATRDY::SET);
            }
        });
    }
}

struct TrngIter<'a, 'b: 'a>(&'a Trng<'b>);

impl Iterator for TrngIter<'a, 'b> {
    type Item = u32;

    fn next(&mut self) -> Option<u32> {
        let regs = &*self.0.regs;
        if regs.isr.is_set(Interrupt::DATRDY) {
            Some(regs.odata.read(OutputData::ODATA))
        } else {
            None
        }
    }
}

impl entropy::Entropy32<'a> for Trng<'a> {
    fn get(&self) -> ReturnCode {
        let regs = &*self.regs;
        pm::enable_clock(pm::Clock::PBA(pm::PBAClock::TRNG));

        regs.cr
            .write(Control::KEY.val(KEY) + Control::ENABLE::Enable);
        regs.ier.write(Interrupt::DATRDY::SET);
        ReturnCode::SUCCESS
    }

    fn cancel(&self) -> ReturnCode {
        ReturnCode::FAIL
    }

    fn set_client(&'a self, client: &'a dyn entropy::Client32) {
        self.client.set(client);
    }
}
