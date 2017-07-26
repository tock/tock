//! Macros for defining registers

macro_rules! define_register {
    [ $name:ident ] => {
        #[allow(non_snake_case)]
        mod $name {
            use kernel::common::volatile_cell::VolatileCell;

            pub struct $name(pub VolatileCell<u32>);
        }
    };
}

macro_rules! impl_register {
    [ $name:ident, "RW" ] => {
        impl ::usbc::common::register::RegisterRW for $name::$name {
            #[inline]
            fn read(&self) -> u32 {
                self.0.get()
            }

            #[inline]
            fn write(&self, val: u32) {
                self.0.set(val);
            }
        }
    };
    [ $name:ident, "R" ] => {
        impl ::usbc::common::register::RegisterR for $name::$name {
            #[inline]
            fn read(&self) -> u32 {
                self.0.get()
            }
        }
    };
    [ $name:ident, "W" ] => {
        impl ::usbc::common::register::RegisterW for $name::$name {
            #[inline]
            fn write(&self, val: u32) {
                self.0.set(val);
            }
        }
    };
}

#[macro_export]
macro_rules! register {
    [ $base:expr, $offset:expr, $description:expr, $name:ident, $access:tt ] => {

        define_register!($name);
        impl_register!($name, $access);

        pub const $name: ::kernel::common::static_ref::StaticRef<$name::$name> =
            unsafe {
                ::kernel::common::static_ref::StaticRef::new(
                    ($base + $offset) as *const $name::$name)
            };
    };
    [ $base:expr, $offset:expr, $description:expr, $name:ident, $access:tt, $count:expr ] => {

        define_register!($name);
        impl_register!($name, $access);

        pub const $name: ::kernel::common::static_ref::StaticRef<[$name::$name; $count]> =
            unsafe {
                ::kernel::common::static_ref::StaticRef::new(
                    ($base + $offset) as *const [$name::$name; $count])
            };
    };
}

#[macro_export]
macro_rules! registers {
    [ $base:expr, {
        $( $offset:expr => { $( $arg:tt ),* } ),*
    } ] => {
        $( register![ $base, $offset, $( $arg ),* ]; )*
    };
}

#[macro_export]
macro_rules! bitfield {
    [ $reg:ident, $field:ident, "RW", $valty:ty, $shift:expr, $mask:expr ] => {

        #[allow(non_snake_case)]
        mod $field {
            pub struct $field;
        }

        impl $field::$field {
            pub fn write(self, val: $valty) {
                use usbc::common::register::*;

                let w = $reg.read();
                let val_bits = (val.to_word() & $mask) << $shift;
                $reg.write((w & !($mask << $shift)) | val_bits);
            }
        }

        pub const $field: $field::$field = $field::$field;
    };

    [ $reg:ident, $field:ident, "R", $valty:ty, $shift:expr, $mask:expr ] => {

        #[allow(non_snake_case)]
        mod $field {
            pub struct $field;
        }

        impl $field::$field {
            pub fn read(self) -> $valty {
                use usbc::common::register::*;

                FromWord::from_word(($reg.read() >> $shift) & $mask)
            }
        }

        pub const $field: $field::$field = $field::$field;
    };
}
