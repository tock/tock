use tock_registers::fields::{Field, FieldValue};
use tock_registers::interfaces::{Readable, Writeable};
use tock_registers::registers::ReadWrite;
use tock_registers::RegisterLongName;

#[flux_rs::opaque]
#[flux_rs::refined_by(mask: bitvec<32>, shift: bitvec<32>)]
pub struct FieldU32<R: RegisterLongName> {
    inner: Field<u32, R>,
}

#[allow(dead_code)]
impl<R: RegisterLongName> FieldU32<R> {
    #[flux_rs::trusted]
    #[flux_rs::sig(fn(mask: u32, shift: usize) -> FieldU32<R>[bv_int_to_bv32(mask), bv_int_to_bv32(shift)])]
    pub fn new(mask: u32, shift: usize) -> FieldU32<R> {
        Self {
            inner: Field::new(mask, shift),
        }
    }

    /*
        mask: mask << shift,
        value: (value & mask) << shift,
    */
    #[flux_rs::trusted]
    #[flux_rs::sig(fn(&FieldU32<R>[@mask, @shift], value: u32) -> FieldValueU32<R>[bv_shl(mask, shift), bv_shl(bv_and(bv_int_to_bv32(value), mask), shift)])]
    pub fn val(&self, value: u32) -> FieldValueU32<R> {
        FieldValueU32 {
            inner: FieldValue::<u32, R>::new(self.inner.mask, self.inner.shift, value),
        }
    }
}

use core::ops::Add;

#[derive(Copy, Clone)]
#[flux_rs::opaque]
#[flux_rs::refined_by(mask: bitvec<32>, value: bitvec<32>)]
pub struct FieldValueU32<R: RegisterLongName> {
    inner: FieldValue<u32, R>,
}

#[allow(dead_code)]
impl<R: RegisterLongName> Add for FieldValueU32<R> {
    type Output = Self;

    #[inline]
    #[flux_rs::trusted]
    #[flux_rs::sig(fn(FieldValueU32<R>[@mask0, @value0], FieldValueU32<R>[@mask1, @value1]) -> FieldValueU32<R>[bv_or(mask0, mask1), bv_or(value0, value1)])]
    fn add(self, rhs: Self) -> Self {
        FieldValueU32 {
            inner: FieldValue::<u32, R>::new(
                self.inner.mask | rhs.inner.mask,
                0,
                self.inner.value | rhs.inner.value,
            ),
        }
    }
}

#[flux_rs::opaque]
#[flux_rs::refined_by(value: bitvec<32>)]
pub struct ReadWriteU32<R: RegisterLongName = ()> {
    inner: ReadWrite<u32, R>,
}

#[allow(dead_code)]
impl<R: RegisterLongName> ReadWriteU32<R> {
    fn new(_addr: usize) -> Self {
        unimplemented!()
    }

    #[flux_rs::trusted]
    #[flux_rs::sig(fn(&ReadWriteU32<R>[@n]) -> u32[bv_bv32_to_int(n)])]
    pub fn get(&self) -> u32 {
        self.inner.get()
    }

    #[flux_rs::trusted]
    #[flux_rs::sig(fn(reg: &strg ReadWriteU32<R>, u32[@n]) ensures reg: ReadWriteU32<R>[bv_int_to_bv32(n)])]
    pub fn set(&mut self, value: u32) {
        self.inner.set(value)
    }

    //(val & (self.mask << self.shift)) >> self.shift
    #[flux_rs::trusted]
    #[flux_rs::sig(fn(&ReadWriteU32<R>[@n], FieldU32<R>[@mask, @shift]) -> u32[ bv_bv32_to_int(bv_lshr(bv_and(n, bv_shl(mask, shift)), shift))])]
    pub fn read(&self, field: FieldU32<R>) -> u32 {
        self.inner.read(field.inner)
    }

    #[flux_rs::trusted]
    #[flux_rs::sig(fn(reg: &strg ReadWriteU32<R>, FieldValueU32<R>[@mask, @value]) ensures reg: ReadWriteU32<R>[value])]
    pub fn write(&mut self, fieldval: FieldValueU32<R>) {
        self.inner.write(fieldval.inner);
    }
}
