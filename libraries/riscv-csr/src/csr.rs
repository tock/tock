//! `ReadWriteRiscvCsr` type for RISC-V CSRs.

use core::marker::PhantomData;

use tock_registers::registers::{
    Field, FieldValue, IntLike, LocalRegisterCopy, RegisterLongName, TryFromValue,
};

/// Read/Write registers.
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

impl<T: IntLike, R: RegisterLongName> ReadWriteRiscvCsr<T, R> {
    pub const fn new(value: usize) -> Self {
        ReadWriteRiscvCsr {
            value: value,
            associated_register: PhantomData,
            associated_length: PhantomData,
        }
    }

    #[cfg(all(target_arch = "riscv32", target_os = "none"))]
    #[inline]
    pub fn get(&self) -> T {
        let r: T;
        unsafe { llvm_asm!("csrr $0, $1" : "=r"(r) : "i"(self.value) :: "volatile") }
        r
    }

    #[cfg(all(target_arch = "riscv32", target_os = "none"))]
    #[inline]
    pub fn set(&self, val_to_set: T) {
        unsafe { llvm_asm!("csrw $0, $1" :: "i"(self.value), "r"(val_to_set) :: "volatile") }
    }

    // Mock implementations for tests on Travis-CI.
    #[cfg(not(any(target_arch = "riscv32", target_os = "none")))]
    pub fn get(&self) -> T {
        unimplemented!("reading RISC-V CSR {}", self.value)
    }

    #[cfg(not(any(target_arch = "riscv32", target_os = "none")))]
    pub fn set(&self, _val_to_set: T) {
        unimplemented!("writing RISC-V CSR {}", self.value)
    }

    #[inline]
    pub fn read(&self, field: Field<T, R>) -> T {
        field.read(self.get())
    }

    #[inline]
    pub fn read_as_enum<E: TryFromValue<T, EnumType = E>>(&self, field: Field<T, R>) -> Option<E> {
        field.read_as_enum(self.get())
    }

    #[inline]
    pub fn extract(&self) -> LocalRegisterCopy<T, R> {
        LocalRegisterCopy::new(self.get())
    }

    #[inline]
    pub fn write(&self, field: FieldValue<T, R>) {
        self.set(field.value);
    }

    #[inline]
    pub fn modify(&self, field: FieldValue<T, R>) {
        self.set(field.modify(self.get()));
    }

    #[inline]
    pub fn modify_no_read(&self, original: LocalRegisterCopy<T, R>, field: FieldValue<T, R>) {
        self.set(field.modify(original.get()));
    }
    #[inline]
    pub fn is_set(&self, field: Field<T, R>) -> bool {
        field.is_set(self.get())
    }

    #[inline]
    pub fn matches_any(&self, field: FieldValue<T, R>) -> bool {
        field.matches_any(self.get())
    }

    #[inline]
    pub fn matches_all(&self, field: FieldValue<T, R>) -> bool {
        field.matches_all(self.get())
    }
}
