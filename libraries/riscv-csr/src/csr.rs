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
        match self.value {
            MINSTRETH => unsafe {
                asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const MINSTRETH);
            },
            MINSTRET => unsafe {
                asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const MINSTRET);
            },
            MCYCLEH => unsafe {
                asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const MCYCLEH);
            },
            MCYCLE => unsafe {
                asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const MCYCLE);
            },
            MIE => unsafe {
                asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const MIE);
            },
            MTVEC => unsafe {
                asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const MTVEC);
            },
            MSTATUS => unsafe {
                asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const MSTATUS);
            },
            UTVEC => unsafe {
                asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const UTVEC);
            },
            STVEC => unsafe {
                asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const STVEC);
            },
            MSCRATCH => unsafe {
                asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const MSCRATCH);
            },
            MEPC => unsafe {
                asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const MEPC);
            },
            MCAUSE => unsafe {
                asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const MCAUSE);
            },
            MTVAL => unsafe {
                asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const MTVAL);
            },
            MIP => unsafe {
                asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const MIP);
            },
            PMPCFG0 => unsafe {
                asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const PMPCFG0);
            },
            PMPCFG1 => unsafe {
                asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const PMPCFG1);
            },
            PMPCFG2 => unsafe {
                asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const PMPCFG2);
            },
            PMPCFG3 => unsafe {
                asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const PMPCFG3);
            },
            PMPCFG4 => unsafe {
                asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const PMPCFG4);
            },
            PMPCFG5 => unsafe {
                asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const PMPCFG5);
            },
            PMPCFG6 => unsafe {
                asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const PMPCFG6);
            },
            PMPCFG7 => unsafe {
                asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const PMPCFG7);
            },
            PMPCFG8 => unsafe {
                asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const PMPCFG8);
            },
            PMPCFG9 => unsafe {
                asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const PMPCFG9);
            },
            PMPCFG10 => unsafe {
                asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const PMPCFG10);
            },
            PMPCFG11 => unsafe {
                asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const PMPCFG11);
            },
            PMPCFG12 => unsafe {
                asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const PMPCFG12);
            },
            PMPCFG13 => unsafe {
                asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const PMPCFG13);
            },
            PMPCFG14 => unsafe {
                asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const PMPCFG14);
            },
            PMPCFG15 => unsafe {
                asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const PMPCFG15);
            },
            PMPADDR0 => unsafe {
                asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const PMPADDR0);
            },
            PMPADDR1 => unsafe {
                asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const PMPADDR1);
            },
            PMPADDR2 => unsafe {
                asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const PMPADDR2);
            },
            PMPADDR3 => unsafe {
                asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const PMPADDR3);
            },
            PMPADDR4 => unsafe {
                asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const PMPADDR4);
            },
            PMPADDR5 => unsafe {
                asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const PMPADDR5);
            },
            PMPADDR6 => unsafe {
                asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const PMPADDR6);
            },
            PMPADDR7 => unsafe {
                asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const PMPADDR7);
            },
            PMPADDR8 => unsafe {
                asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const PMPADDR8);
            },
            PMPADDR9 => unsafe {
                asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const PMPADDR9);
            },
            PMPADDR10 => unsafe {
                asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const PMPADDR10);
            },
            PMPADDR11 => unsafe {
                asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const PMPADDR11);
            },
            PMPADDR12 => unsafe {
                asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const PMPADDR12);
            },
            PMPADDR13 => unsafe {
                asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const PMPADDR13);
            },
            PMPADDR14 => unsafe {
                asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const PMPADDR14);
            },
            PMPADDR15 => unsafe {
                asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const PMPADDR15);
            },
            PMPADDR16 => unsafe {
                asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const PMPADDR16);
            },
            PMPADDR17 => unsafe {
                asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const PMPADDR17);
            },
            PMPADDR18 => unsafe {
                asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const PMPADDR18);
            },
            PMPADDR19 => unsafe {
                asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const PMPADDR19);
            },
            PMPADDR20 => unsafe {
                asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const PMPADDR20);
            },
            PMPADDR21 => unsafe {
                asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const PMPADDR21);
            },
            PMPADDR22 => unsafe {
                asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const PMPADDR22);
            },
            PMPADDR23 => unsafe {
                asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const PMPADDR23);
            },
            PMPADDR24 => unsafe {
                asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const PMPADDR24);
            },
            PMPADDR25 => unsafe {
                asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const PMPADDR25);
            },
            PMPADDR26 => unsafe {
                asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const PMPADDR26);
            },
            PMPADDR27 => unsafe {
                asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const PMPADDR27);
            },
            PMPADDR28 => unsafe {
                asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const PMPADDR28);
            },
            PMPADDR29 => unsafe {
                asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const PMPADDR29);
            },
            PMPADDR30 => unsafe {
                asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const PMPADDR30);
            },
            PMPADDR31 => unsafe {
                asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const PMPADDR31);
            },
            PMPADDR32 => unsafe {
                asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const PMPADDR32);
            },
            PMPADDR33 => unsafe {
                asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const PMPADDR33);
            },
            PMPADDR34 => unsafe {
                asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const PMPADDR34);
            },
            PMPADDR35 => unsafe {
                asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const PMPADDR35);
            },
            PMPADDR36 => unsafe {
                asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const PMPADDR36);
            },
            PMPADDR37 => unsafe {
                asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const PMPADDR37);
            },
            PMPADDR38 => unsafe {
                asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const PMPADDR38);
            },
            PMPADDR39 => unsafe {
                asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const PMPADDR39);
            },
            PMPADDR40 => unsafe {
                asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const PMPADDR40);
            },
            PMPADDR41 => unsafe {
                asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const PMPADDR41);
            },
            PMPADDR42 => unsafe {
                asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const PMPADDR42);
            },
            PMPADDR43 => unsafe {
                asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const PMPADDR43);
            },
            PMPADDR44 => unsafe {
                asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const PMPADDR44);
            },
            PMPADDR45 => unsafe {
                asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const PMPADDR45);
            },
            PMPADDR46 => unsafe {
                asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const PMPADDR46);
            },
            PMPADDR47 => unsafe {
                asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const PMPADDR47);
            },
            PMPADDR48 => unsafe {
                asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const PMPADDR48);
            },
            PMPADDR49 => unsafe {
                asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const PMPADDR49);
            },
            PMPADDR50 => unsafe {
                asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const PMPADDR50);
            },
            PMPADDR51 => unsafe {
                asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const PMPADDR51);
            },
            PMPADDR52 => unsafe {
                asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const PMPADDR52);
            },
            PMPADDR53 => unsafe {
                asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const PMPADDR53);
            },
            PMPADDR54 => unsafe {
                asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const PMPADDR54);
            },
            PMPADDR55 => unsafe {
                asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const PMPADDR55);
            },
            PMPADDR56 => unsafe {
                asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const PMPADDR56);
            },
            PMPADDR57 => unsafe {
                asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const PMPADDR57);
            },
            PMPADDR58 => unsafe {
                asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const PMPADDR58);
            },
            PMPADDR59 => unsafe {
                asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const PMPADDR59);
            },
            PMPADDR60 => unsafe {
                asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const PMPADDR60);
            },
            PMPADDR61 => unsafe {
                asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const PMPADDR61);
            },
            PMPADDR62 => unsafe {
                asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const PMPADDR62);
            },
            PMPADDR63 => unsafe {
                asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const PMPADDR63);
            },
            _ => panic!("Unsupported CSR read"),
        }
        r
    }

    #[cfg(all(target_arch = "riscv32", target_os = "none"))]
    #[inline]
    pub fn set(&self, val_to_set: u32) {
        match self.value {
            MINSTRETH => unsafe {
                asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const MINSTRETH);
            },
            MINSTRET => unsafe {
                asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const MINSTRET);
            },
            MCYCLEH => unsafe {
                asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const MCYCLEH);
            },
            MCYCLE => unsafe {
                asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const MCYCLE);
            },
            MIE => unsafe {
                asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const MIE);
            },
            MTVEC => unsafe {
                asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const MTVEC);
            },
            MSTATUS => unsafe {
                asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const MSTATUS);
            },
            UTVEC => unsafe {
                asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const UTVEC);
            },
            STVEC => unsafe {
                asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const STVEC);
            },
            MSCRATCH => unsafe {
                asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const MSCRATCH);
            },
            MEPC => unsafe {
                asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const MEPC);
            },
            MCAUSE => unsafe {
                asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const MCAUSE);
            },
            MTVAL => unsafe {
                asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const MTVAL);
            },
            MIP => unsafe {
                asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const MIP);
            },
            PMPCFG0 => unsafe {
                asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const PMPCFG0);
            },
            PMPCFG1 => unsafe {
                asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const PMPCFG1);
            },
            PMPCFG2 => unsafe {
                asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const PMPCFG2);
            },
            PMPCFG3 => unsafe {
                asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const PMPCFG3);
            },
            PMPCFG4 => unsafe {
                asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const PMPCFG4);
            },
            PMPCFG5 => unsafe {
                asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const PMPCFG5);
            },
            PMPCFG6 => unsafe {
                asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const PMPCFG6);
            },
            PMPCFG7 => unsafe {
                asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const PMPCFG7);
            },
            PMPCFG8 => unsafe {
                asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const PMPCFG8);
            },
            PMPCFG9 => unsafe {
                asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const PMPCFG9);
            },
            PMPCFG10 => unsafe {
                asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const PMPCFG10);
            },
            PMPCFG11 => unsafe {
                asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const PMPCFG11);
            },
            PMPCFG12 => unsafe {
                asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const PMPCFG12);
            },
            PMPCFG13 => unsafe {
                asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const PMPCFG13);
            },
            PMPCFG14 => unsafe {
                asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const PMPCFG14);
            },
            PMPCFG15 => unsafe {
                asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const PMPCFG15);
            },
            PMPADDR0 => unsafe {
                asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const PMPADDR0);
            },
            PMPADDR1 => unsafe {
                asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const PMPADDR1);
            },
            PMPADDR2 => unsafe {
                asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const PMPADDR2);
            },
            PMPADDR3 => unsafe {
                asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const PMPADDR3);
            },
            PMPADDR4 => unsafe {
                asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const PMPADDR4);
            },
            PMPADDR5 => unsafe {
                asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const PMPADDR5);
            },
            PMPADDR6 => unsafe {
                asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const PMPADDR6);
            },
            PMPADDR7 => unsafe {
                asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const PMPADDR7);
            },
            PMPADDR8 => unsafe {
                asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const PMPADDR8);
            },
            PMPADDR9 => unsafe {
                asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const PMPADDR9);
            },
            PMPADDR10 => unsafe {
                asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const PMPADDR10);
            },
            PMPADDR11 => unsafe {
                asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const PMPADDR11);
            },
            PMPADDR12 => unsafe {
                asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const PMPADDR12);
            },
            PMPADDR13 => unsafe {
                asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const PMPADDR13);
            },
            PMPADDR14 => unsafe {
                asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const PMPADDR14);
            },
            PMPADDR15 => unsafe {
                asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const PMPADDR15);
            },
            PMPADDR16 => unsafe {
                asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const PMPADDR16);
            },
            PMPADDR17 => unsafe {
                asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const PMPADDR17);
            },
            PMPADDR18 => unsafe {
                asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const PMPADDR18);
            },
            PMPADDR19 => unsafe {
                asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const PMPADDR19);
            },
            PMPADDR20 => unsafe {
                asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const PMPADDR20);
            },
            PMPADDR21 => unsafe {
                asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const PMPADDR21);
            },
            PMPADDR22 => unsafe {
                asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const PMPADDR22);
            },
            PMPADDR23 => unsafe {
                asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const PMPADDR23);
            },
            PMPADDR24 => unsafe {
                asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const PMPADDR24);
            },
            PMPADDR25 => unsafe {
                asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const PMPADDR25);
            },
            PMPADDR26 => unsafe {
                asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const PMPADDR26);
            },
            PMPADDR27 => unsafe {
                asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const PMPADDR27);
            },
            PMPADDR28 => unsafe {
                asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const PMPADDR28);
            },
            PMPADDR29 => unsafe {
                asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const PMPADDR29);
            },
            PMPADDR30 => unsafe {
                asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const PMPADDR30);
            },
            PMPADDR31 => unsafe {
                asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const PMPADDR31);
            },
            PMPADDR32 => unsafe {
                asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const PMPADDR32);
            },
            PMPADDR33 => unsafe {
                asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const PMPADDR33);
            },
            PMPADDR34 => unsafe {
                asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const PMPADDR34);
            },
            PMPADDR35 => unsafe {
                asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const PMPADDR35);
            },
            PMPADDR36 => unsafe {
                asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const PMPADDR36);
            },
            PMPADDR37 => unsafe {
                asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const PMPADDR37);
            },
            PMPADDR38 => unsafe {
                asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const PMPADDR38);
            },
            PMPADDR39 => unsafe {
                asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const PMPADDR39);
            },
            PMPADDR40 => unsafe {
                asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const PMPADDR40);
            },
            PMPADDR41 => unsafe {
                asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const PMPADDR41);
            },
            PMPADDR42 => unsafe {
                asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const PMPADDR42);
            },
            PMPADDR43 => unsafe {
                asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const PMPADDR43);
            },
            PMPADDR44 => unsafe {
                asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const PMPADDR44);
            },
            PMPADDR45 => unsafe {
                asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const PMPADDR45);
            },
            PMPADDR46 => unsafe {
                asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const PMPADDR46);
            },
            PMPADDR47 => unsafe {
                asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const PMPADDR47);
            },
            PMPADDR48 => unsafe {
                asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const PMPADDR48);
            },
            PMPADDR49 => unsafe {
                asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const PMPADDR49);
            },
            PMPADDR50 => unsafe {
                asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const PMPADDR50);
            },
            PMPADDR51 => unsafe {
                asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const PMPADDR51);
            },
            PMPADDR52 => unsafe {
                asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const PMPADDR52);
            },
            PMPADDR53 => unsafe {
                asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const PMPADDR53);
            },
            PMPADDR54 => unsafe {
                asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const PMPADDR54);
            },
            PMPADDR55 => unsafe {
                asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const PMPADDR55);
            },
            PMPADDR56 => unsafe {
                asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const PMPADDR56);
            },
            PMPADDR57 => unsafe {
                asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const PMPADDR57);
            },
            PMPADDR58 => unsafe {
                asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const PMPADDR58);
            },
            PMPADDR59 => unsafe {
                asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const PMPADDR59);
            },
            PMPADDR60 => unsafe {
                asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const PMPADDR60);
            },
            PMPADDR61 => unsafe {
                asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const PMPADDR61);
            },
            PMPADDR62 => unsafe {
                asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const PMPADDR62);
            },
            PMPADDR63 => unsafe {
                asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const PMPADDR63);
            },
            _ => panic!("Unsupported CSR write"),
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
