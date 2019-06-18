//DISTRIBUTION STATEMENT A. Approved for public release. Distribution is unlimited.
//
//This material is based upon work supported by the Under Secretary of Defense for Research and Engineering under Air Force Contract No. FA8702-15-D-0001. Any opinions, findings, conclusions or recommendations expressed in this material are those of the author(s) and do not necessarily reflect the views of the Under Secretary of Defense for Research and Engineering.
//
//© 2019 Massachusetts Institute of Technology.
//
//The software/firmware is provided to you on an As-Is basis
//
//Delivered to the U.S. Government with Unlimited Rights, as defined in DFARS Part 252.227-7013 or 7014 (Feb 2014). Notwithstanding any copyright notice, U.S. Government rights in this work are defined by DFARS 252.227-7013 or DFARS 252.227-7014 as detailed above. Use of this work other than as specifically authorized by the U.S. Government may violate any copyrights that exist in this work.//


// Note that it looks like this has been replaced MMIO-wise by machine_timer. Still useful for
// hifive1 board. Perhaps this, machine_timer, and the clic.rs belong under boards?
use kernel::common::registers::{ReadOnly, ReadWrite};
use kernel::common::StaticRef;

// memory map as described here: https://sifive.cdn.prismic.io/sifive%2F898b5153-4c06-4085-8d95-2d5fd67e74c6_u54_core_complex_manual_v19_02.pdf
#[repr(C)]
struct ClintRegisters {
    // Interrupt Pending Register
    misp: ReadWrite<u32, MISP::Register>,
    _reserved0: [u32; (0x200_4000 - 0x200_0004) / 0x4],
    mtimecmp1: ReadWrite<u32, MTIME::Register>,
    mtimecmp2: ReadWrite<u32, MTIME::Register>,
    _reserved1: [u32; (0x200_bff8 - 0x200_4008) / 0x4],
    mtime1: ReadWrite<u32, MTIME::Register>,
    mtime2: ReadWrite<u32, MTIME::Register>,
}

register_bitfields![u32,
    MISP [
        MISPBIT OFFSET(0) NUMBITS(1) []
    ],
    MTIME [
        MTIMEBITS OFFSET(0) NUMBITS(32) []
    ]
];

const CLINT_BASE: StaticRef<ClintRegisters> =
unsafe { StaticRef::new(0x200_0000 as *const ClintRegisters) };

pub unsafe fn read_mtimecmp() -> u64 {
    let clint: &ClintRegisters = &*CLINT_BASE;
    clint.mtimecmp1.get() | clint.mtimecmp2.get() << 32
}

pub unsafe fn read_mtime() -> u64 {
    let clint: &ClintRegisters = &*CLINT_BASE;
    let a = (clint.mtime2.get() as u64);
    let b = (clint.mtime1.get() as u64);
    a + (b << 32)
}

pub unsafe fn write_mtime(arg: u32) {
    let clint: &ClintRegisters = &*CLINT_BASE;
    clint.mtime1.set(arg);
}

pub unsafe fn write_mtimecmp1(new_bound: u32) {
    let clint: &ClintRegisters = &*CLINT_BASE;
    clint.mtimecmp1.set(new_bound);
}

pub unsafe fn write_mtimecmp2(new_bound: u32) {
    let clint: &ClintRegisters = &*CLINT_BASE;
    clint.mtimecmp2.set(new_bound);
}

pub unsafe fn write_mtime(new_bound: u64){
    write_mtimecmp1(new_bound & 0xFFFF_FFFF);
    write_mtimecmp2(new_bound >> 32);
}

pub unsafe fn trigger_software_interrupt() {
    let clint: &ClintRegisters = &*CLINT_BASE;
    clint.misp.write(MISP::MISPBIT::SET);
}
