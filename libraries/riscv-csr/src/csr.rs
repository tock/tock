//! `ReadWriteRiscvCsr` type for RISC-V CSRs.

use core::marker::PhantomData;

use tock_registers::registers::{
    Field, FieldValue, IntLike, LocalRegisterCopy, RegisterLongName, TryFromValue,
};

pub const MINSTRETH: usize = 0xB82;
pub const MINSTRET: usize = 0xB02;
pub const MCYCLEH: usize = 0xB80;
pub const MCYCLE: usize = 0xB00;
pub const MIE: usize = 0x304;
pub const MTVEC: usize = 0x305;
pub const MSTATUS: usize = 0x300;
pub const UTVEC: usize = 0x005;
pub const STVEC: usize = 0x105;
pub const MSCRATCH: usize = 0x340;
pub const MEPC: usize = 0x341;
pub const MCAUSE: usize = 0x342;
pub const MTVAL: usize = 0x343;
pub const MIP: usize = 0x344;
pub const PMPCFG0: usize = 0x3A0;
pub const PMPCFG1: usize = 0x3A4;
pub const PMPCFG2: usize = 0x3A8;
pub const PMPCFG3: usize = 0x3AC;
pub const PMPADDR0: usize = 0x3B0;
pub const PMPADDR1: usize = 0x3B4;
pub const PMPADDR2: usize = 0x3B8;
pub const PMPADDR3: usize = 0x3BC;
pub const PMPADDR4: usize = 0x3C0;
pub const PMPADDR5: usize = 0x3C4;
pub const PMPADDR6: usize = 0x3C8;
pub const PMPADDR7: usize = 0x3CC;
pub const PMPADDR8: usize = 0x3D0;
pub const PMPADDR9: usize = 0x3D4;
pub const PMPADDR10: usize = 0x3D8;
pub const PMPADDR11: usize = 0x3DC;
pub const PMPADDR12: usize = 0x3E0;
pub const PMPADDR13: usize = 0x3E4;
pub const PMPADDR14: usize = 0x3E8;
pub const PMPADDR15: usize = 0x3EC;

/// Read/Write registers.
#[derive(Copy, Clone)]
pub struct ReadWriteRiscvCsr<T: IntLike, R: RegisterLongName = ()> {
    value: usize,
    associated_register: PhantomData<R>,
    associated_length: PhantomData<T>,
}

// morally speaking, these should be implemented; however not yet needed
//pub struct WriteOnlyRiscvCsr<T: IntLike, R: RegisterLongName = ()> {
//value: T,
//associated_register: PhantomData<R>}

//pub struct ReadOnlyRiscvCsr<T: IntLike, R: RegisterLongName = ()> {
//value: T,
//associated_register: PhantomData<R>}

impl<R: RegisterLongName> ReadWriteRiscvCsr<u32, R> {
    pub const fn new(value: usize) -> Self {
        ReadWriteRiscvCsr {
            value: value,
            associated_register: PhantomData,
            associated_length: PhantomData,
        }
    }

    #[cfg(all(target_arch = "riscv32", target_os = "none"))]
    #[inline]
    pub fn get(&self) -> u32 {
        let r: u32;
        if self.value == MINSTRETH {
            unsafe {
                asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const MINSTRETH);
            }
        } else if self.value == MINSTRET {
            unsafe {
                asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const MINSTRET);
            }
        } else if self.value == MCYCLEH {
            unsafe {
                asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const MCYCLEH);
            }
        } else if self.value == MCYCLE {
            unsafe {
                asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const MCYCLE);
            }
        } else if self.value == MIE {
            unsafe {
                asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const MIE);
            }
        } else if self.value == MTVEC {
            unsafe {
                asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const MTVEC);
            }
        } else if self.value == MSTATUS {
            unsafe {
                asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const MSTATUS);
            }
        } else if self.value == UTVEC {
            unsafe {
                asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const UTVEC);
            }
        } else if self.value == STVEC {
            unsafe {
                asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const STVEC);
            }
        } else if self.value == MSCRATCH {
            unsafe {
                asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const MSCRATCH);
            }
        } else if self.value == MEPC {
            unsafe {
                asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const MEPC);
            }
        } else if self.value == MCAUSE {
            unsafe {
                asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const MCAUSE);
            }
        } else if self.value == MTVAL {
            unsafe {
                asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const MTVAL);
            }
        } else if self.value == MIP {
            unsafe {
                asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const MIP);
            }
        } else if self.value == PMPCFG0 {
            unsafe {
                asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const PMPCFG0);
            }
        } else if self.value == PMPCFG1 {
            unsafe {
                asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const PMPCFG1);
            }
        } else if self.value == PMPCFG2 {
            unsafe {
                asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const PMPCFG2);
            }
        } else if self.value == PMPCFG3 {
            unsafe {
                asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const PMPCFG3);
            }
        } else if self.value == PMPADDR0 {
            unsafe {
                asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const PMPADDR0);
            }
        } else if self.value == PMPADDR1 {
            unsafe {
                asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const PMPADDR1);
            }
        } else if self.value == PMPADDR2 {
            unsafe {
                asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const PMPADDR2);
            }
        } else if self.value == PMPADDR3 {
            unsafe {
                asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const PMPADDR3);
            }
        } else if self.value == PMPADDR4 {
            unsafe {
                asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const PMPADDR4);
            }
        } else if self.value == PMPADDR5 {
            unsafe {
                asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const PMPADDR5);
            }
        } else {
            panic!("Unsupported CSR read");
        }
        r
    }

    #[cfg(all(target_arch = "riscv32", target_os = "none"))]
    #[inline]
    pub fn set(&self, val_to_set: u32) {
        if self.value == MINSTRETH {
            unsafe {
                asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const MINSTRETH);
            }
        } else if self.value == MINSTRET {
            unsafe {
                asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const MINSTRET);
            }
        } else if self.value == MCYCLEH {
            unsafe {
                asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const MCYCLEH);
            }
        } else if self.value == MCYCLE {
            unsafe {
                asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const MCYCLE);
            }
        } else if self.value == MIE {
            unsafe {
                asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const MIE);
            }
        } else if self.value == MTVEC {
            unsafe {
                asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const MTVEC);
            }
        } else if self.value == MSTATUS {
            unsafe {
                asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const MSTATUS);
            }
        } else if self.value == UTVEC {
            unsafe {
                asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const UTVEC);
            }
        } else if self.value == STVEC {
            unsafe {
                asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const STVEC);
            }
        } else if self.value == MSCRATCH {
            unsafe {
                asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const MSCRATCH);
            }
        } else if self.value == MEPC {
            unsafe {
                asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const MEPC);
            }
        } else if self.value == MCAUSE {
            unsafe {
                asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const MCAUSE);
            }
        } else if self.value == MTVAL {
            unsafe {
                asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const MTVAL);
            }
        } else if self.value == MIP {
            unsafe {
                asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const MIP);
            }
        } else if self.value == PMPCFG0 {
            unsafe {
                asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const PMPCFG0);
            }
        } else if self.value == PMPCFG1 {
            unsafe {
                asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const PMPCFG1);
            }
        } else if self.value == PMPCFG2 {
            unsafe {
                asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const PMPCFG2);
            }
        } else if self.value == PMPCFG3 {
            unsafe {
                asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const PMPCFG3);
            }
        } else if self.value == PMPADDR0 {
            unsafe {
                asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const PMPADDR0);
            }
        } else if self.value == PMPADDR1 {
            unsafe {
                asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const PMPADDR1);
            }
        } else if self.value == PMPADDR2 {
            unsafe {
                asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const PMPADDR2);
            }
        } else if self.value == PMPADDR3 {
            unsafe {
                asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const PMPADDR3);
            }
        } else if self.value == PMPADDR4 {
            unsafe {
                asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const PMPADDR4);
            }
        } else if self.value == PMPADDR5 {
            unsafe {
                asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const PMPADDR5);
            }
        } else {
            panic!("Unsupported CSR write");
        }
    }

    // Mock implementations for tests on Travis-CI.
    #[cfg(not(any(target_arch = "riscv32", target_os = "none")))]
    pub fn get(&self) -> u32 {
        unimplemented!("reading RISC-V CSR {}", self.value)
    }

    #[cfg(not(any(target_arch = "riscv32", target_os = "none")))]
    pub fn set(&self, _val_to_set: u32) {
        unimplemented!("writing RISC-V CSR {}", self.value)
    }

    #[inline]
    pub fn read(&self, field: Field<u32, R>) -> u32 {
        field.read(self.get())
    }

    #[inline]
    pub fn read_as_enum<E: TryFromValue<u32, EnumType = E>>(
        &self,
        field: Field<u32, R>,
    ) -> Option<E> {
        field.read_as_enum(self.get())
    }

    #[inline]
    pub fn extract(&self) -> LocalRegisterCopy<u32, R> {
        LocalRegisterCopy::new(self.get())
    }

    #[inline]
    pub fn write(&self, field: FieldValue<u32, R>) {
        self.set(field.value);
    }

    #[inline]
    pub fn modify(&self, field: FieldValue<u32, R>) {
        self.set(field.modify(self.get()));
    }

    #[inline]
    pub fn modify_no_read(&self, original: LocalRegisterCopy<u32, R>, field: FieldValue<u32, R>) {
        self.set(field.modify(original.get()));
    }
    #[inline]
    pub fn is_set(&self, field: Field<u32, R>) -> bool {
        field.is_set(self.get())
    }

    #[inline]
    pub fn matches_any(&self, field: FieldValue<u32, R>) -> bool {
        field.matches_any(self.get())
    }

    #[inline]
    pub fn matches_all(&self, field: FieldValue<u32, R>) -> bool {
        field.matches_all(self.get())
    }
}
