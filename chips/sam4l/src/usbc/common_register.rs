#![allow(dead_code)]

use core::marker::PhantomData;

/// A memory-mapped register
#[repr(C)]
#[derive(Copy, Clone)]
pub struct Reg(*mut u32);

impl Reg {
    pub const unsafe fn new(addr: *mut u32) -> Self {
        Reg(addr)
    }

    #[inline]
    pub fn read(self) -> u32 {
        unsafe { ::core::ptr::read_volatile(self.0) }
    }

    #[inline]
    pub fn write(self, val: u32) {
        unsafe { ::core::ptr::write_volatile(self.0, val); }
    }

    #[inline]
    pub fn set_bit(self, bit_index: u32) {
        let w = self.read();
        let bit = 1 << bit_index;
        self.write(w | bit);
    }

    #[inline]
    pub fn clear_bit(self, bit_index: u32) {
        let w = self.read();
        let bit = 1 << bit_index;
        self.write(w & (!bit));
    }
}

/// A write-only memory-mapped register, where any zero bits written have no effect
#[repr(C)]
#[derive(Copy, Clone)]
pub struct RegW(*mut u32);

impl RegW {
    pub const unsafe fn new(addr: *mut u32) -> Self {
        RegW(addr)
    }

    #[inline]
    pub fn write(self, val: u32) {
        unsafe { ::core::ptr::write_volatile(self.0, val) }
    }

    #[inline]
    pub fn set_bit(self, bit_index: u32) {
        // For this kind of register, reads always return zero
        // and zero bits have no effect, so we simply write the
        // single bit requested.
        let bit = 1 << bit_index;
        self.write(bit);
    }
}

/// A read-only memory-mapped register
#[repr(C)]
#[derive(Copy, Clone)]
pub struct RegR(*const u32);

impl RegR {
    pub const unsafe fn new(addr: *const u32) -> Self {
        RegR(addr)
    }

    #[inline]
    pub fn read(self) -> u32 {
        unsafe { ::core::ptr::read_volatile(self.0) }
    }
}

/// A bitfield of a memory-mapped register
pub struct BitField<T> {
    /// The register that hosts this bitfield
    reg: Reg,
    /// Bit offset of the value within a word
    shift: u32,
    /// Bit pattern of the value (e.g. 0b111 for a three-bit field)
    bits: u32,
    phantom: PhantomData<*mut T>,
}

impl<T: ToWord> BitField<T> {
    pub const fn new(reg: Reg, shift: u32, bits: u32) -> Self {
        BitField { reg: reg, shift: shift, bits: bits, phantom: PhantomData }
    }

    #[inline]
    pub fn write(self, val: T) {
        let w = self.reg.read();
        let mask = self.bits << self.shift;
        let val_bits = (val.to_word() & self.bits) << self.shift;
        self.reg.write(w & !mask | val_bits);
    }
}

/// A bitfield of a write-only memory-mapped register,
/// where any zeros written have no effect
pub struct BitFieldW<T> {
    reg: RegW,
    shift: u32,
    bits: u32,
    phantom: PhantomData<*mut T>,
}

impl<T: ToWord> BitFieldW<T> {
    pub const fn new(reg: RegW, shift: u32, bits: u32) -> Self {
        BitFieldW { reg: reg, shift: shift, bits: bits, phantom: PhantomData }
    }

    #[inline]
    pub fn write(self, val: T) {
        let val_bits = (val.to_word() & self.bits) << self.shift;
        self.reg.write(val_bits);
    }
}

pub struct BitFieldR<T> {
    reg: RegR,
    shift: u32,
    bits: u32,
    phantom: PhantomData<*mut T>,
}

impl<T: FromWord> BitFieldR<T> {
    pub const fn new(reg: RegR, shift: u32, bits: u32) -> Self {
        BitFieldR { reg: reg, shift: shift, bits: bits, phantom: PhantomData }
    }

    #[inline]
    pub fn read(self) -> T {
        FromWord::from_word((self.reg.read() >> self.shift) & self.bits)
    }
}

pub trait ToWord {
    fn to_word(self) -> u32;
}

impl ToWord for u32 {
    #[inline]
    fn to_word(self) -> u32 { self }
}

impl ToWord for u8 {
    #[inline]
    fn to_word(self) -> u32 { self as u32 }
}

impl ToWord for bool {
    #[inline]
    fn to_word(self) -> u32 { if self { 1 } else { 0 } }
}

pub trait FromWord {
    fn from_word(u32) -> Self;
}

impl FromWord for u32 {
    #[inline]
    fn from_word(w: u32) -> Self { w }
}

impl FromWord for bool {
    #[inline]
    fn from_word(w: u32) -> bool {
        if w & 1 == 1 { true } else { false }
    }
}
