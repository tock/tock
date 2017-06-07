//! Macros for defining USBC registers

#[macro_export]
macro_rules! reg {
    [ $offset:expr, $description:expr, $name:ident, "RW" ] => {
        #[allow(dead_code)]
        pub const $name: Reg = unsafe { Reg::new((USBC_BASE + $offset) as *mut u32) };
    };

    [ $offset:expr, $description:expr, $name:ident, "R" ] => {
        #[allow(dead_code)]
        pub const $name: RegR = unsafe { RegR::new((USBC_BASE + $offset) as *mut u32) };
    };

    [ $offset:expr, $description:expr, $name:ident, "W" ] => {
        #[allow(dead_code)]
        pub const $name: RegW = unsafe { RegW::new((USBC_BASE + $offset) as *mut u32) };
    };
}

#[macro_export]
macro_rules! regs {
    [ $offset:expr, $description:expr, $name:ident, "RW", $count:expr ] => {
        #[allow(dead_code)]
        pub const $name: [Reg; $count] = unsafe { [Reg::new((USBC_BASE + $offset) as *mut u32); $count] };
    };

    [ $offset:expr, $description:expr, $name:ident, "R", $count:expr ] => {
        #[allow(dead_code)]
        pub const $name: [RegR; $count] = unsafe { [RegR::new((USBC_BASE + $offset) as *mut u32); $count] };
    };

    [ $offset:expr, $description:expr, $name:ident, "W", $count:expr ] => {
        #[allow(dead_code)]
        pub const $name: [RegW; $count] = unsafe { [RegW::new((USBC_BASE + $offset) as *mut u32); $count] };
    };
}

#[macro_export]
macro_rules! bitfield {
    [ $reg:ident, $field:ident, "RW", $t:ty, $shift:expr, $bits:expr ] => {
        #[allow(dead_code)]
        pub const $field: BitField<$t> = BitField::new($reg, $shift, $bits);
    };

    [ $reg:ident, $field:ident, "W", $t:ty, $shift:expr, $bits:expr ] => {
        #[allow(dead_code)]
        pub const $field: BitFieldW<$t> = BitFieldW::new($reg, $shift, $bits);
    };

    [ $reg:ident, $field:ident, "R", $t:ty, $shift:expr, $bits:expr ] => {
        #[allow(dead_code)]
        pub const $field: BitFieldR<$t> = BitFieldR::new($reg, $shift, $bits);
    };
}
