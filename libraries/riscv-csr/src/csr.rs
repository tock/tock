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

    #[inline]
    pub fn get(&self) -> T {
        let r: T;
        unsafe { asm!("csrr $0, $1" : "=r"(r) : "i"(self.value) :: "volatile") }
        r
    }

    #[inline]
    pub fn set(&self, val_to_set: T) {
        unsafe { asm!("csrw $0, $1" :: "i"(self.value), "r"(val_to_set) :: "volatile") }
    }

    #[inline]
    pub fn read(&self, field: Field<T, R>) -> T {
        (self.get() & (field.mask << field.shift)) >> field.shift
    }

    #[inline]
    pub fn read_as_enum<E: TryFromValue<T, EnumType = E>>(&self, field: Field<T, R>) -> Option<E> {
        let val: T = self.read(field);

        E::try_from(val)
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
        let reg: T = self.get();
        self.set((reg & !field.mask) | field.value);
    }

    #[inline]
    pub fn modify_no_read(&self, original: LocalRegisterCopy<T, R>, field: FieldValue<T, R>) {
        self.set((original.get() & !field.mask) | field.value);
    }
    #[inline]
    pub fn is_set(&self, field: Field<T, R>) -> bool {
        self.read(field) != T::zero()
    }

    #[inline]
    pub fn matches_any(&self, field: FieldValue<T, R>) -> bool {
        self.get() & field.mask != T::zero()
    }

    #[inline]
    pub fn matches_all(&self, field: FieldValue<T, R>) -> bool {
        self.get() & field.mask == field.value
    }
}
