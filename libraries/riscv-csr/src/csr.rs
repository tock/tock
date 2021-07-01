//! `ReadWriteRiscvCsr` type for RISC-V CSRs.

use core::marker::PhantomData;

use tock_registers::fields::Field;
use tock_registers::interfaces::{Readable, Writeable};
use tock_registers::{RegisterLongName, UIntLike};

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
pub const MSECCFG: usize = 0x390;
pub const MSECCFGH: usize = 0x391;
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
pub struct ReadWriteRiscvCsr<T: UIntLike, R: RegisterLongName, const V: usize> {
    associated_register: PhantomData<R>,
    associated_length: PhantomData<T>,
}

// morally speaking, these should be implemented; however not yet needed
//pub struct WriteOnlyRiscvCsr<T: UIntLike, R: RegisterLongName = ()> {
//value: T,
//associated_register: PhantomData<R>}

//pub struct ReadOnlyRiscvCsr<T: UIntLike, R: RegisterLongName = ()> {
//value: T,
//associated_register: PhantomData<R>}

impl<R: RegisterLongName, const V: usize> ReadWriteRiscvCsr<usize, R, V> {
    pub const fn new() -> Self {
        ReadWriteRiscvCsr {
            associated_register: PhantomData,
            associated_length: PhantomData,
        }
    }

    // Special methods only available on RISC-V CSRs, not found in the
    // usual tock-registers interface, others implemented through the
    // respective [`Readable`] and [`Writeable`] trait implementations

    /// Atomically swap the contents of a CSR
    ///
    /// Reads the current value of a CSR and replaces it with the
    /// specified value in a single instruction, returning the
    /// previous value.
    ///
    /// This method corresponds to the RISC-V `CSRRW rd, csr, rs1`
    /// instruction where `rs1 = in(reg) value_to_set` and `rd =
    /// out(reg) <return value>`.
    #[cfg(all(
        any(target_arch = "riscv32", target_arch = "riscv64"),
        target_os = "none"
    ))]
    #[inline]
    pub fn atomic_replace(&self, val_to_set: usize) -> usize {
        let r: usize;
        unsafe {
            asm!("csrrw {rd}, {csr}, {rs1}",
                 rd = out(reg) r,
                 csr = const V,
                 rs1 = in(reg) val_to_set);
        }
        r
    }

    /// Atomically swap the contents of a CSR
    ///
    /// Reads the current value of a CSR and replaces it with the
    /// specified value in a single instruction, returning the
    /// previous value.
    ///
    /// This method corresponds to the RISC-V `CSRRW rd, csr, rs1`
    /// instruction where `rs1 = in(reg) value_to_set` and `rd =
    /// out(reg) <return value>`.
    // Mock implementations for tests on Travis-CI.
    #[cfg(not(any(target_arch = "riscv32", target_arch = "riscv64", target_os = "none")))]
    pub fn atomic_replace(&self, _value_to_set: usize) -> usize {
        unimplemented!("RISC-V CSR {} Atomic Read/Write", V)
    }

    /// Atomically read a CSR and set bits specified in a bitmask
    ///
    /// This method corresponds to the RISC-V `CSRRS rd, csr, rs1`
    /// instruction where `rs1 = in(reg) bitmask` and `rd = out(reg)
    /// <return value>`.
    #[cfg(all(
        any(target_arch = "riscv32", target_arch = "riscv64"),
        target_os = "none"
    ))]
    #[inline]
    pub fn read_and_set_bits(&self, bitmask: usize) -> usize {
        let r: usize;
        unsafe {
            asm!("csrrs {rd}, {csr}, {rs1}",
                 rd = out(reg) r,
                 csr = const V,
                 rs1 = in(reg) bitmask);
        }
        r
    }

    /// Atomically read a CSR and set bits specified in a bitmask
    ///
    /// This method corresponds to the RISC-V `CSRRS rd, csr, rs1`
    /// instruction where `rs1 = in(reg) bitmask` and `rd = out(reg)
    /// <return value>`.
    // Mock implementations for tests on Travis-CI.
    #[cfg(not(any(target_arch = "riscv32", target_arch = "riscv64", target_os = "none")))]
    pub fn read_and_set_bits(&self, bitmask: usize) -> usize {
        unimplemented!(
            "RISC-V CSR {} Atomic Read and Set Bits, bitmask {:04x}",
            V,
            bitmask
        )
    }

    /// Atomically read a CSR and clear bits specified in a bitmask
    ///
    /// This method corresponds to the RISC-V `CSRRC rd, csr, rs1`
    /// instruction where `rs1 = in(reg) bitmask` and `rd = out(reg)
    /// <return value>`.
    #[cfg(all(
        any(target_arch = "riscv32", target_arch = "riscv64"),
        target_os = "none"
    ))]
    #[inline]
    pub fn read_and_clear_bits(&self, bitmask: usize) -> usize {
        let r: usize;
        unsafe {
            asm!("csrrc {rd}, {csr}, {rs1}",
                 rd = out(reg) r,
                 csr = const V,
                 rs1 = in(reg) bitmask);
        }
        r
    }

    /// Atomically read a CSR and clear bits specified in a bitmask
    ///
    /// This method corresponds to the RISC-V `CSRRC rd, csr, rs1`
    /// instruction where `rs1 = in(reg) bitmask` and `rd = out(reg)
    /// <return value>`.
    // Mock implementations for tests on Travis-CI.
    #[cfg(not(any(target_arch = "riscv32", target_arch = "riscv64", target_os = "none")))]
    pub fn read_and_clear_bits(&self, bitmask: usize) -> usize {
        unimplemented!(
            "RISC-V CSR {} Atomic Read and Clear Bits, bitmask {:04x}",
            V,
            bitmask
        )
    }

    /// Atomically read field and set all bits to 1
    ///
    /// This method corresponds to the RISC-V `CSRRS rd, csr, rs1`
    /// instruction, where `rs1` is the bitmask described by the
    /// [`Field`].
    ///
    /// The previous value of the field is returned.
    #[inline]
    pub fn read_and_set_field(&self, field: Field<usize, R>) -> usize {
        field.read(self.read_and_set_bits(field.mask << field.shift))
    }

    /// Atomically read field and set all bits to 0
    ///
    /// This method corresponds to the RISC-V `CSRRC rd, csr, rs1`
    /// instruction, where `rs1` is the bitmask described by the
    /// [`Field`].
    ///
    /// The previous value of the field is returned.
    #[inline]
    pub fn read_and_clear_field(&self, field: Field<usize, R>) -> usize {
        field.read(self.read_and_clear_bits(field.mask << field.shift))
    }
}

impl<R: RegisterLongName, const V: usize> Readable for ReadWriteRiscvCsr<usize, R, V> {
    type T = usize;
    type R = R;

    #[cfg(all(
        any(target_arch = "riscv32", target_arch = "riscv64"),
        target_os = "none"
    ))]
    #[inline]
    fn get(&self) -> usize {
        let r: usize;
        unsafe {
            asm!("csrr {rd}, {csr}", rd = out(reg) r, csr = const V);
        }
        r
    }

    // Mock implementations for tests on Travis-CI.
    #[cfg(not(any(target_arch = "riscv32", target_arch = "riscv64", target_os = "none")))]
    fn get(&self) -> usize {
        unimplemented!("reading RISC-V CSR {}", V)
    }
}

impl<R: RegisterLongName, const V: usize> Writeable for ReadWriteRiscvCsr<usize, R, V> {
    type T = usize;
    type R = R;

    #[cfg(all(
        any(target_arch = "riscv32", target_arch = "riscv64"),
        target_os = "none"
    ))]
    #[inline]
    fn set(&self, val_to_set: usize) {
        unsafe {
            asm!("csrw {csr}, {rs}", rs = in(reg) val_to_set, csr = const V);
        }
    }

    #[cfg(not(any(target_arch = "riscv32", target_arch = "riscv64", target_os = "none")))]
    fn set(&self, _val_to_set: usize) {
        unimplemented!("writing RISC-V CSR {}", V)
    }
}
