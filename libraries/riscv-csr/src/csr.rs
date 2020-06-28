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
pub const PMPCFG1: usize = 0x3A1;
pub const PMPCFG2: usize = 0x3A2;
pub const PMPCFG3: usize = 0x3A3;
pub const PMPCFG4: usize = 0x3A4;
pub const PMPCFG5: usize = 0x3A5;
pub const PMPCFG6: usize = 0x3A6;
pub const PMPCFG7: usize = 0x3A7;
pub const PMPCFG8: usize = 0x3A8;
pub const PMPCFG9: usize = 0x3A9;
pub const PMPCFG10: usize = 0x3AA;
pub const PMPCFG11: usize = 0x3AB;
pub const PMPCFG12: usize = 0x3AC;
pub const PMPCFG13: usize = 0x3AD;
pub const PMPCFG14: usize = 0x3AE;
pub const PMPCFG15: usize = 0x3AF;
pub const PMPADDR0: usize = 0x3B0;
pub const PMPADDR1: usize = 0x3B1;
pub const PMPADDR2: usize = 0x3B2;
pub const PMPADDR3: usize = 0x3B3;
pub const PMPADDR4: usize = 0x3B4;
pub const PMPADDR5: usize = 0x3B5;
pub const PMPADDR6: usize = 0x3B6;
pub const PMPADDR7: usize = 0x3B7;
pub const PMPADDR8: usize = 0x3B8;
pub const PMPADDR9: usize = 0x3B9;
pub const PMPADDR10: usize = 0x3BA;
pub const PMPADDR11: usize = 0x3BB;
pub const PMPADDR12: usize = 0x3BC;
pub const PMPADDR13: usize = 0x3BD;
pub const PMPADDR14: usize = 0x3BE;
pub const PMPADDR15: usize = 0x3BF;
pub const PMPADDR16: usize = 0x3C0;
pub const PMPADDR17: usize = 0x3C1;
pub const PMPADDR18: usize = 0x3C2;
pub const PMPADDR19: usize = 0x3C3;
pub const PMPADDR20: usize = 0x3C4;
pub const PMPADDR21: usize = 0x3C5;
pub const PMPADDR22: usize = 0x3C6;
pub const PMPADDR23: usize = 0x3C7;
pub const PMPADDR24: usize = 0x3C8;
pub const PMPADDR25: usize = 0x3C9;
pub const PMPADDR26: usize = 0x3CA;
pub const PMPADDR27: usize = 0x3CB;
pub const PMPADDR28: usize = 0x3CC;
pub const PMPADDR29: usize = 0x3CD;
pub const PMPADDR30: usize = 0x3CE;
pub const PMPADDR31: usize = 0x3CF;
pub const PMPADDR32: usize = 0x3D0;
pub const PMPADDR33: usize = 0x3D1;
pub const PMPADDR34: usize = 0x3D2;
pub const PMPADDR35: usize = 0x3D3;
pub const PMPADDR36: usize = 0x3D4;
pub const PMPADDR37: usize = 0x3D5;
pub const PMPADDR38: usize = 0x3D6;
pub const PMPADDR39: usize = 0x3D7;
pub const PMPADDR40: usize = 0x3D8;
pub const PMPADDR41: usize = 0x3D9;
pub const PMPADDR42: usize = 0x3DA;
pub const PMPADDR43: usize = 0x3DB;
pub const PMPADDR44: usize = 0x3DC;
pub const PMPADDR45: usize = 0x3DD;
pub const PMPADDR46: usize = 0x3DE;
pub const PMPADDR47: usize = 0x3DF;
pub const PMPADDR48: usize = 0x3E0;
pub const PMPADDR49: usize = 0x3E1;
pub const PMPADDR50: usize = 0x3E2;
pub const PMPADDR51: usize = 0x3E3;
pub const PMPADDR52: usize = 0x3E4;
pub const PMPADDR53: usize = 0x3E5;
pub const PMPADDR54: usize = 0x3E6;
pub const PMPADDR55: usize = 0x3E7;
pub const PMPADDR56: usize = 0x3E8;
pub const PMPADDR57: usize = 0x3E9;
pub const PMPADDR58: usize = 0x3EA;
pub const PMPADDR59: usize = 0x3EB;
pub const PMPADDR60: usize = 0x3EC;
pub const PMPADDR61: usize = 0x3ED;
pub const PMPADDR62: usize = 0x3EE;
pub const PMPADDR63: usize = 0x3EF;

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
                llvm_asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const MINSTRETH);
            }
        } else if self.value == MINSTRET {
            unsafe {
                llvm_asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const MINSTRET);
            }
        } else if self.value == MCYCLEH {
            unsafe {
                llvm_asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const MCYCLEH);
            }
        } else if self.value == MCYCLE {
            unsafe {
                llvm_asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const MCYCLE);
            }
        } else if self.value == MIE {
            unsafe {
                llvm_asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const MIE);
            }
        } else if self.value == MTVEC {
            unsafe {
                llvm_asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const MTVEC);
            }
        } else if self.value == MSTATUS {
            unsafe {
                llvm_asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const MSTATUS);
            }
        } else if self.value == UTVEC {
            unsafe {
                llvm_asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const UTVEC);
            }
        } else if self.value == STVEC {
            unsafe {
                llvm_asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const STVEC);
            }
        } else if self.value == MSCRATCH {
            unsafe {
                llvm_asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const MSCRATCH);
            }
        } else if self.value == MEPC {
            unsafe {
                llvm_asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const MEPC);
            }
        } else if self.value == MCAUSE {
            unsafe {
                llvm_asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const MCAUSE);
            }
        } else if self.value == MTVAL {
            unsafe {
                llvm_asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const MTVAL);
            }
        } else if self.value == MIP {
            unsafe {
                llvm_asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const MIP);
            }
        } else if self.value == PMPCFG0 {
            unsafe {
                llvm_asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const PMPCFG0);
            }
        } else if self.value == PMPCFG1 {
            unsafe {
                llvm_asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const PMPCFG1);
            }
        } else if self.value == PMPCFG2 {
            unsafe {
                llvm_asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const PMPCFG2);
            }
        } else if self.value == PMPCFG3 {
            unsafe {
                llvm_asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const PMPCFG3);
            }
        } else if self.value == PMPCFG4 {
            unsafe {
                llvm_asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const PMPCFG4);
            }
        } else if self.value == PMPCFG5 {
            unsafe {
                llvm_asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const PMPCFG5);
            }
        } else if self.value == PMPCFG6 {
            unsafe {
                llvm_asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const PMPCFG6);
            }
        } else if self.value == PMPCFG7 {
            unsafe {
                llvm_asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const PMPCFG7);
            }
        } else if self.value == PMPCFG8 {
            unsafe {
                llvm_asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const PMPCFG8);
            }
        } else if self.value == PMPCFG9 {
            unsafe {
                llvm_asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const PMPCFG9);
            }
        } else if self.value == PMPCFG10 {
            unsafe {
                llvm_asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const PMPCFG10);
            }
        } else if self.value == PMPCFG11 {
            unsafe {
                llvm_asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const PMPCFG11);
            }
        } else if self.value == PMPCFG12 {
            unsafe {
                llvm_asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const PMPCFG12);
            }
        } else if self.value == PMPCFG13 {
            unsafe {
                llvm_asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const PMPCFG13);
            }
        } else if self.value == PMPCFG14 {
            unsafe {
                llvm_asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const PMPCFG14);
            }
        } else if self.value == PMPCFG15 {
            unsafe {
                llvm_asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const PMPCFG15);
            }
        } else if self.value == PMPADDR0 {
            unsafe {
                llvm_asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const PMPADDR0);
            }
        } else if self.value == PMPADDR1 {
            unsafe {
                llvm_asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const PMPADDR1);
            }
        } else if self.value == PMPADDR2 {
            unsafe {
                llvm_asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const PMPADDR2);
            }
        } else if self.value == PMPADDR3 {
            unsafe {
                llvm_asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const PMPADDR3);
            }
        } else if self.value == PMPADDR4 {
            unsafe {
                llvm_asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const PMPADDR4);
            }
        } else if self.value == PMPADDR5 {
            unsafe {
                llvm_asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const PMPADDR5);
            }
        } else if self.value == PMPADDR6 {
            unsafe {
                llvm_asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const PMPADDR6);
            }
        } else if self.value == PMPADDR7 {
            unsafe {
                llvm_asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const PMPADDR7);
            }
        } else if self.value == PMPADDR8 {
            unsafe {
                llvm_asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const PMPADDR8);
            }
        } else if self.value == PMPADDR9 {
            unsafe {
                llvm_asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const PMPADDR9);
            }
        } else if self.value == PMPADDR10 {
            unsafe {
                llvm_asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const PMPADDR10);
            }
        } else if self.value == PMPADDR11 {
            unsafe {
                llvm_asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const PMPADDR11);
            }
        } else if self.value == PMPADDR12 {
            unsafe {
                llvm_asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const PMPADDR12);
            }
        } else if self.value == PMPADDR13 {
            unsafe {
                llvm_asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const PMPADDR13);
            }
        } else if self.value == PMPADDR14 {
            unsafe {
                llvm_asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const PMPADDR14);
            }
        } else if self.value == PMPADDR15 {
            unsafe {
                llvm_asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const PMPADDR15);
            }
        } else if self.value == PMPADDR16 {
            unsafe {
                llvm_asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const PMPADDR16);
            }
        } else if self.value == PMPADDR17 {
            unsafe {
                llvm_asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const PMPADDR17);
            }
        } else if self.value == PMPADDR18 {
            unsafe {
                llvm_asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const PMPADDR18);
            }
        } else if self.value == PMPADDR19 {
            unsafe {
                llvm_asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const PMPADDR19);
            }
        } else if self.value == PMPADDR20 {
            unsafe {
                llvm_asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const PMPADDR20);
            }
        } else if self.value == PMPADDR21 {
            unsafe {
                llvm_asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const PMPADDR21);
            }
        } else if self.value == PMPADDR22 {
            unsafe {
                llvm_asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const PMPADDR22);
            }
        } else if self.value == PMPADDR23 {
            unsafe {
                llvm_asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const PMPADDR23);
            }
        } else if self.value == PMPADDR24 {
            unsafe {
                llvm_asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const PMPADDR24);
            }
        } else if self.value == PMPADDR25 {
            unsafe {
                llvm_asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const PMPADDR25);
            }
        } else if self.value == PMPADDR26 {
            unsafe {
                llvm_asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const PMPADDR26);
            }
        } else if self.value == PMPADDR27 {
            unsafe {
                llvm_asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const PMPADDR27);
            }
        } else if self.value == PMPADDR28 {
            unsafe {
                llvm_asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const PMPADDR28);
            }
        } else if self.value == PMPADDR29 {
            unsafe {
                llvm_asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const PMPADDR29);
            }
        } else if self.value == PMPADDR30 {
            unsafe {
                llvm_asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const PMPADDR30);
            }
        } else if self.value == PMPADDR31 {
            unsafe {
                llvm_asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const PMPADDR31);
            }
        } else if self.value == PMPADDR32 {
            unsafe {
                llvm_asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const PMPADDR32);
            }
        } else if self.value == PMPADDR33 {
            unsafe {
                llvm_asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const PMPADDR33);
            }
        } else if self.value == PMPADDR34 {
            unsafe {
                llvm_asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const PMPADDR34);
            }
        } else if self.value == PMPADDR35 {
            unsafe {
                llvm_asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const PMPADDR35);
            }
        } else if self.value == PMPADDR36 {
            unsafe {
                llvm_asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const PMPADDR36);
            }
        } else if self.value == PMPADDR37 {
            unsafe {
                llvm_asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const PMPADDR37);
            }
        } else if self.value == PMPADDR38 {
            unsafe {
                llvm_asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const PMPADDR38);
            }
        } else if self.value == PMPADDR39 {
            unsafe {
                llvm_asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const PMPADDR39);
            }
        } else if self.value == PMPADDR40 {
            unsafe {
                llvm_asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const PMPADDR40);
            }
        } else if self.value == PMPADDR41 {
            unsafe {
                llvm_asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const PMPADDR41);
            }
        } else if self.value == PMPADDR42 {
            unsafe {
                llvm_asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const PMPADDR42);
            }
        } else if self.value == PMPADDR43 {
            unsafe {
                llvm_asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const PMPADDR43);
            }
        } else if self.value == PMPADDR44 {
            unsafe {
                llvm_asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const PMPADDR44);
            }
        } else if self.value == PMPADDR45 {
            unsafe {
                llvm_asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const PMPADDR45);
            }
        } else if self.value == PMPADDR46 {
            unsafe {
                llvm_asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const PMPADDR46);
            }
        } else if self.value == PMPADDR47 {
            unsafe {
                llvm_asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const PMPADDR47);
            }
        } else if self.value == PMPADDR48 {
            unsafe {
                llvm_asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const PMPADDR48);
            }
        } else if self.value == PMPADDR49 {
            unsafe {
                llvm_asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const PMPADDR49);
            }
        } else if self.value == PMPADDR50 {
            unsafe {
                llvm_asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const PMPADDR50);
            }
        } else if self.value == PMPADDR51 {
            unsafe {
                llvm_asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const PMPADDR51);
            }
        } else if self.value == PMPADDR52 {
            unsafe {
                llvm_asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const PMPADDR52);
            }
        } else if self.value == PMPADDR53 {
            unsafe {
                llvm_asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const PMPADDR53);
            }
        } else if self.value == PMPADDR54 {
            unsafe {
                llvm_asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const PMPADDR54);
            }
        } else if self.value == PMPADDR55 {
            unsafe {
                llvm_asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const PMPADDR55);
            }
        } else if self.value == PMPADDR56 {
            unsafe {
                llvm_asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const PMPADDR56);
            }
        } else if self.value == PMPADDR57 {
            unsafe {
                llvm_asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const PMPADDR57);
            }
        } else if self.value == PMPADDR58 {
            unsafe {
                llvm_asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const PMPADDR58);
            }
        } else if self.value == PMPADDR59 {
            unsafe {
                llvm_asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const PMPADDR59);
            }
        } else if self.value == PMPADDR60 {
            unsafe {
                llvm_asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const PMPADDR60);
            }
        } else if self.value == PMPADDR61 {
            unsafe {
                llvm_asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const PMPADDR61);
            }
        } else if self.value == PMPADDR62 {
            unsafe {
                llvm_asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const PMPADDR62);
            }
        } else if self.value == PMPADDR63 {
            unsafe {
                llvm_asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const PMPADDR63);
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
                llvm_asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const MINSTRETH);
            }
        } else if self.value == MINSTRET {
            unsafe {
                llvm_asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const MINSTRET);
            }
        } else if self.value == MCYCLEH {
            unsafe {
                llvm_asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const MCYCLEH);
            }
        } else if self.value == MCYCLE {
            unsafe {
                llvm_asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const MCYCLE);
            }
        } else if self.value == MIE {
            unsafe {
                llvm_asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const MIE);
            }
        } else if self.value == MTVEC {
            unsafe {
                llvm_asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const MTVEC);
            }
        } else if self.value == MSTATUS {
            unsafe {
                llvm_asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const MSTATUS);
            }
        } else if self.value == UTVEC {
            unsafe {
                llvm_asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const UTVEC);
            }
        } else if self.value == STVEC {
            unsafe {
                llvm_asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const STVEC);
            }
        } else if self.value == MSCRATCH {
            unsafe {
                llvm_asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const MSCRATCH);
            }
        } else if self.value == MEPC {
            unsafe {
                llvm_asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const MEPC);
            }
        } else if self.value == MCAUSE {
            unsafe {
                llvm_asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const MCAUSE);
            }
        } else if self.value == MTVAL {
            unsafe {
                llvm_asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const MTVAL);
            }
        } else if self.value == MIP {
            unsafe {
                llvm_asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const MIP);
            }
        } else if self.value == PMPCFG0 {
            unsafe {
                llvm_asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const PMPCFG0);
            }
        } else if self.value == PMPCFG1 {
            unsafe {
                llvm_asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const PMPCFG1);
            }
        } else if self.value == PMPCFG2 {
            unsafe {
                llvm_asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const PMPCFG2);
            }
        } else if self.value == PMPCFG3 {
            unsafe {
                llvm_asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const PMPCFG3);
            }
        } else if self.value == PMPCFG4 {
            unsafe {
                llvm_asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const PMPCFG4);
            }
        } else if self.value == PMPCFG5 {
            unsafe {
                llvm_asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const PMPCFG5);
            }
        } else if self.value == PMPCFG6 {
            unsafe {
                llvm_asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const PMPCFG6);
            }
        } else if self.value == PMPCFG7 {
            unsafe {
                llvm_asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const PMPCFG7);
            }
        } else if self.value == PMPCFG8 {
            unsafe {
                llvm_asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const PMPCFG8);
            }
        } else if self.value == PMPCFG9 {
            unsafe {
                llvm_asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const PMPCFG9);
            }
        } else if self.value == PMPCFG10 {
            unsafe {
                llvm_asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const PMPCFG10);
            }
        } else if self.value == PMPCFG11 {
            unsafe {
                llvm_asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const PMPCFG11);
            }
        } else if self.value == PMPCFG12 {
            unsafe {
                llvm_asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const PMPCFG12);
            }
        } else if self.value == PMPCFG13 {
            unsafe {
                llvm_asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const PMPCFG13);
            }
        } else if self.value == PMPCFG14 {
            unsafe {
                llvm_asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const PMPCFG14);
            }
        } else if self.value == PMPCFG15 {
            unsafe {
                llvm_asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const PMPCFG15);
            }
        } else if self.value == PMPADDR0 {
            unsafe {
                llvm_asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const PMPADDR0);
            }
        } else if self.value == PMPADDR1 {
            unsafe {
                llvm_asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const PMPADDR1);
            }
        } else if self.value == PMPADDR2 {
            unsafe {
                llvm_asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const PMPADDR2);
            }
        } else if self.value == PMPADDR3 {
            unsafe {
                llvm_asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const PMPADDR3);
            }
        } else if self.value == PMPADDR4 {
            unsafe {
                llvm_asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const PMPADDR4);
            }
        } else if self.value == PMPADDR5 {
            unsafe {
                llvm_asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const PMPADDR5);
            }
        } else if self.value == PMPADDR6 {
            unsafe {
                llvm_asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const PMPADDR6);
            }
        } else if self.value == PMPADDR7 {
            unsafe {
                llvm_asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const PMPADDR7);
            }
        } else if self.value == PMPADDR8 {
            unsafe {
                llvm_asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const PMPADDR8);
            }
        } else if self.value == PMPADDR9 {
            unsafe {
                llvm_asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const PMPADDR9);
            }
        } else if self.value == PMPADDR10 {
            unsafe {
                llvm_asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const PMPADDR10);
            }
        } else if self.value == PMPADDR11 {
            unsafe {
                llvm_asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const PMPADDR11);
            }
        } else if self.value == PMPADDR12 {
            unsafe {
                llvm_asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const PMPADDR12);
            }
        } else if self.value == PMPADDR13 {
            unsafe {
                llvm_asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const PMPADDR13);
            }
        } else if self.value == PMPADDR14 {
            unsafe {
                llvm_asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const PMPADDR14);
            }
        } else if self.value == PMPADDR15 {
            unsafe {
                llvm_asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const PMPADDR15);
            }
        } else if self.value == PMPADDR16 {
            unsafe {
                llvm_asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const PMPADDR16);
            }
        } else if self.value == PMPADDR17 {
            unsafe {
                llvm_asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const PMPADDR17);
            }
        } else if self.value == PMPADDR18 {
            unsafe {
                llvm_asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const PMPADDR18);
            }
        } else if self.value == PMPADDR19 {
            unsafe {
                llvm_asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const PMPADDR19);
            }
        } else if self.value == PMPADDR20 {
            unsafe {
                llvm_asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const PMPADDR20);
            }
        } else if self.value == PMPADDR21 {
            unsafe {
                llvm_asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const PMPADDR21);
            }
        } else if self.value == PMPADDR22 {
            unsafe {
                llvm_asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const PMPADDR22);
            }
        } else if self.value == PMPADDR23 {
            unsafe {
                llvm_asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const PMPADDR23);
            }
        } else if self.value == PMPADDR24 {
            unsafe {
                llvm_asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const PMPADDR24);
            }
        } else if self.value == PMPADDR25 {
            unsafe {
                llvm_asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const PMPADDR25);
            }
        } else if self.value == PMPADDR26 {
            unsafe {
                llvm_asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const PMPADDR26);
            }
        } else if self.value == PMPADDR27 {
            unsafe {
                llvm_asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const PMPADDR27);
            }
        } else if self.value == PMPADDR28 {
            unsafe {
                llvm_asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const PMPADDR28);
            }
        } else if self.value == PMPADDR29 {
            unsafe {
                llvm_asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const PMPADDR29);
            }
        } else if self.value == PMPADDR30 {
            unsafe {
                llvm_asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const PMPADDR30);
            }
        } else if self.value == PMPADDR31 {
            unsafe {
                llvm_asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const PMPADDR31);
            }
        } else if self.value == PMPADDR32 {
            unsafe {
                llvm_asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const PMPADDR32);
            }
        } else if self.value == PMPADDR33 {
            unsafe {
                llvm_asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const PMPADDR33);
            }
        } else if self.value == PMPADDR34 {
            unsafe {
                llvm_asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const PMPADDR34);
            }
        } else if self.value == PMPADDR35 {
            unsafe {
                llvm_asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const PMPADDR35);
            }
        } else if self.value == PMPADDR36 {
            unsafe {
                llvm_asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const PMPADDR36);
            }
        } else if self.value == PMPADDR37 {
            unsafe {
                llvm_asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const PMPADDR37);
            }
        } else if self.value == PMPADDR38 {
            unsafe {
                llvm_asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const PMPADDR38);
            }
        } else if self.value == PMPADDR39 {
            unsafe {
                llvm_asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const PMPADDR39);
            }
        } else if self.value == PMPADDR40 {
            unsafe {
                llvm_asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const PMPADDR40);
            }
        } else if self.value == PMPADDR41 {
            unsafe {
                llvm_asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const PMPADDR41);
            }
        } else if self.value == PMPADDR42 {
            unsafe {
                llvm_asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const PMPADDR42);
            }
        } else if self.value == PMPADDR43 {
            unsafe {
                llvm_asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const PMPADDR43);
            }
        } else if self.value == PMPADDR44 {
            unsafe {
                llvm_asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const PMPADDR44);
            }
        } else if self.value == PMPADDR45 {
            unsafe {
                llvm_asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const PMPADDR45);
            }
        } else if self.value == PMPADDR46 {
            unsafe {
                llvm_asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const PMPADDR46);
            }
        } else if self.value == PMPADDR47 {
            unsafe {
                llvm_asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const PMPADDR47);
            }
        } else if self.value == PMPADDR48 {
            unsafe {
                llvm_asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const PMPADDR48);
            }
        } else if self.value == PMPADDR49 {
            unsafe {
                llvm_asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const PMPADDR49);
            }
        } else if self.value == PMPADDR50 {
            unsafe {
                llvm_asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const PMPADDR50);
            }
        } else if self.value == PMPADDR51 {
            unsafe {
                llvm_asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const PMPADDR51);
            }
        } else if self.value == PMPADDR52 {
            unsafe {
                llvm_asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const PMPADDR52);
            }
        } else if self.value == PMPADDR53 {
            unsafe {
                llvm_asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const PMPADDR53);
            }
        } else if self.value == PMPADDR54 {
            unsafe {
                llvm_asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const PMPADDR54);
            }
        } else if self.value == PMPADDR55 {
            unsafe {
                llvm_asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const PMPADDR55);
            }
        } else if self.value == PMPADDR56 {
            unsafe {
                llvm_asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const PMPADDR56);
            }
        } else if self.value == PMPADDR57 {
            unsafe {
                llvm_asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const PMPADDR57);
            }
        } else if self.value == PMPADDR58 {
            unsafe {
                llvm_asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const PMPADDR58);
            }
        } else if self.value == PMPADDR59 {
            unsafe {
                llvm_asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const PMPADDR59);
            }
        } else if self.value == PMPADDR60 {
            unsafe {
                llvm_asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const PMPADDR60);
            }
        } else if self.value == PMPADDR61 {
            unsafe {
                llvm_asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const PMPADDR61);
            }
        } else if self.value == PMPADDR62 {
            unsafe {
                llvm_asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const PMPADDR62);
            }
        } else if self.value == PMPADDR63 {
            unsafe {
                llvm_asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const PMPADDR63);
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
