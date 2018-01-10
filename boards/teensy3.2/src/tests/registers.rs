#[allow(unused)]

use common::regs::ReadWrite;

struct Registers {
    c1: ReadWrite<u8, Control>,
}

// Some made up register fields.
bitfields! { u8,
    C1 Control [
        CLKS  (6, Mask(0b11)) [],
        PRDIV (4, Mask(0b11)) [
            Div32 = 2
        ]
    ]
}

const BASE: *mut Registers = 0x2000_0000 as *mut Registers; 

#[inline(never)]
pub fn register_test() {
    unsafe {
        let regs: &mut Registers = ::core::mem::transmute(BASE);

        regs.c1.set(1 << 5);

        regs.c1.modify(C1::CLKS.val(3) +
                       C1::PRDIV.val(1));

        regs.c1.write(C1::CLKS.val(1) +
                      C1::PRDIV::Div32);

        while regs.c1.read(C1::CLKS) != 1 {}
    }
}
