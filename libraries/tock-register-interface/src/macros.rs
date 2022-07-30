//! Macros for cleanly defining peripheral registers.

#[macro_export]
macro_rules! registers {
    (
        $(
            $(#[$mod_attr:meta])*
            $vis:vis $name:ident {
                $(
                    $(#[$field_attr:meta])*
                    ($offset:literal => $reg_name:ident: $($tokens:tt)*)
                ),*
                $(,)?
            }
        )*
    ) => {
        $(
            $(#[$mod_attr])*
            $vis mod $name {
                pub trait ReadRegisters {
                    $($crate::read_registers!{$reg_name: $($tokens)*})*
                }

                pub trait WriteRegisters {
                    $($crate::write_registers!{$reg_name: $($tokens)*})*
                }

                pub trait AccessRegisters: ReadRegisters + WriteRegisters {}
                impl<T: ReadRegisters + WriteRegisters> AccessRegisters for T {}

                pub struct Mmio;

                impl ReadRegisters for Mmio {
                    $($crate::mmio_read!{$offset => $reg_name: $($tokens)*})*
                }

                impl WriteRegisters for Mmio {
                    $($crate::mmio_write!{$offset => $reg_name: $($tokens)*})*
                }

                #[repr(C)]
                pub struct Registers<A: AccessRegisters> {
                    $(pub $reg_name: $reg_name<A>,)*

                    access: A,
                }

                $($crate::register_type!{$reg_name})*
                $($crate::field_impls!{$reg_name: $($tokens)*})*
            }
        )*
    };
}

#[macro_export]
macro_rules! register_structs {
    // Pattern that matches a single register struct with an attached module
    // name.
    {
        $(#[$attr:meta])*
        $vis:vis $struct_name:ident, mod $mod_name:ident {
            $($fields:tt)*
        }
    } => {
        $crate::registers!{
            $(#[$attr])*
            $vis $mod_name {
                $($fields)*
            }
        }

        type $struct_name = $mod_name::Registers<$mod_name::Mmio>;
    };
    // Pattern that matches a single register struct with no attached module
    // name. This recursively calls itself with a module name specified, which
    // should be matched by the first pattern.
    {
        $(#[$attr:meta])*
        $vis:vis $struct_name:ident {
            $($fields:tt)*
        }
    } => {
        $crate::register_structs! {
            $(#[$attr])*
            $vis $struct_name, mod register_structs {
                $($fields)*
            }
        }
    };
    // Pattern that matches multiple register structs. At most one of the
    // structs must specify a module name alongside the register name.
    {
        $(
            $(#[$attr:meta])*
            $vis:vis $struct_name:ident$(, mod $mod_name:ident)? {
                $($fields:tt)*
            }
        ),*
    } => {
        $(
            $crate::register_structs! {
                $crate::register_structs! {
                    $(#[$attr])*
                    $vis $struct_name$(, mod $mod_name)? {
                        $($fields)*
                    }
                }
            }
        )*
    };
}

// -----------------------------------------------------------------------------
// Macros below this line are for internal use only (they are used registers!).
// -----------------------------------------------------------------------------

/// For internal use by `tock-registers`.
#[macro_export]
macro_rules! read_registers {
    ($name:ident: [ReadOnly<$reg_ty:ty, $long_ty:ty>; $len:literal]) => {
        fn $name(&self, idx: usize) -> core::result::Result<$reg_ty, $crate::TooLargeIndex>;
    };
    ($name:ident: [ReadWrite<$reg_ty:ty, $long_ty:ty>; $len:literal]) => {
        fn $name(&self, idx: usize) -> core::result::Result<$reg_ty, $crate::TooLargeIndex>;
    };
    ($name:ident: ReadOnly<$reg_ty:ty, $long_ty:ty>) => {
        fn $name(&self) -> $reg_ty;
    };
    ($name:ident: ReadWrite<$reg_ty:ty, $long_ty:ty>) => {
        fn $name(&self) -> $reg_ty;
    };
    ($($tokens:tt)*) => {};
}

/// For internal use by `tock-registers`.
#[macro_export]
macro_rules! write_registers {
    ($name:ident: [ReadWrite<$reg_ty:ty, $long_ty:ty>; $len:literal]) => {
        fn $name(&self, value: $reg_ty, idx: usize) -> core::result::Result<$crate::WriteSuccess, $crate::TooLargeIndex>;
    };
    ($name:ident: [WriteOnly<$reg_ty:ty, $long_ty:ty>; $len:literal]) => {
        fn $name(&self, value: $reg_ty, idx: usize) -> core::result::Result<$crate::WriteSuccess, $crate::TooLargeIndex>;
    };
    ($name:ident: ReadWrite<$reg_ty:ty, $long_ty:ty>) => {
        fn $name(&self, value: $reg_ty);
    };
    ($name:ident: WriteOnly<$reg_ty:ty, $long_ty:ty>) => {
        fn $name(&self, value: $reg_ty);
    };
    ($($tokens:tt)*) => {};
}

/// For internal use by `tock-registers`.
#[macro_export]
macro_rules! mmio_read {
    ($rel_addr:literal => $name:ident: [ReadOnly<$reg_ty:ty, $long_ty:ty>; $len:literal]) => {
        fn $name(&self, idx: usize) -> core::result::Result<$reg_ty, $crate::TooLargeIndex> {
            unsafe { $crate::internal::mmio_read_array::<$rel_addr, $len, _, _>(self, idx) }
        }
    };
    ($rel_addr:literal => $name:ident: [ReadWrite<$reg_ty:ty, $long_ty:ty>; $len:literal]) => {
        fn $name(&self, idx: usize) -> core::result::Result<$reg_ty, $crate::TooLargeIndex> {
            unsafe { $crate::internal::mmio_read_array::<$rel_addr, $len, _, _>(self, idx) }
        }
    };
    ($rel_addr:literal => $name:ident: ReadOnly<$reg_ty:ty, $long_ty:ty>) => {
        fn $name(&self) -> $reg_ty {
            unsafe { $crate::internal::mmio_read::<$rel_addr, _, _>(self) }
        }
    };
    ($rel_addr:literal => $name:ident: ReadWrite<$reg_ty:ty, $long_ty:ty>) => {
        fn $name(&self) -> $reg_ty {
            unsafe { $crate::internal::mmio_read::<$rel_addr, _, _>(self) }
        }
    };
    ($($tokens:tt)*) => {};
}

/// For internal use by `tock-registers`.
#[macro_export]
macro_rules! mmio_write {
    ($rel_addr:literal => $name:ident: [WriteOnly<$reg_ty:ty, $long_ty:ty>; $len:literal]) => {
        fn $name(&self, value: $reg_ty, idx: usize) -> core::result::Result<$crate::WriteSuccess, $crate::TooLargeIndex> {
            unsafe { $crate::internal::mmio_write_array::<$rel_addr, $len, _, _>(self, value, idx) }
        }
    };
    ($rel_addr:literal => $name:ident: [ReadWrite<$reg_ty:ty, $long_ty:ty>; $len:literal]) => {
        fn $name(&self, value: $reg_ty, idx: usize) -> core::result::Result<$crate::WriteSuccess, $crate::TooLargeIndex> {
            unsafe { $crate::internal::mmio_write_array::<$rel_addr, $len, _, _>(self, value, idx) }
        }
    };
    ($rel_addr:literal => $name:ident: WriteOnly<$reg_ty:ty, $long_ty:ty>) => {
        fn $name(&self, value: $reg_ty) {
            unsafe { $crate::internal::mmio_write::<$rel_addr, _, _>(self, value) }
        }
    };
    ($rel_addr:literal => $name:ident: ReadWrite<$reg_ty:ty, $long_ty:ty>) => {
        fn $name(&self, value: $reg_ty) {
            unsafe { $crate::internal::mmio_write::<$rel_addr, _, _>(self, value) }
        }
    };
    ($($tokens:tt)*) => {};
}

/// For internal use by `tock-registers`.
#[macro_export]
macro_rules! register_type {
    ($name:ident) => {
        #[allow(non_camel_case_types)]
        pub struct $name<A: AccessRegisters> {
            _phantom: core::marker::PhantomData<*const A>,
        }

        impl<A: AccessRegisters> $name<A> {
            fn access(&self) -> &A {
                unsafe { &*(self as *const _ as *const A) }
            }
        }
    };
}

/// For internal use by `tock-registers`.
#[macro_export]
macro_rules! field_impls {
    ($name:ident: [ReadOnly<$reg_ty:ty, $long_ty:ty>; $len:literal]) => {
        impl<A: AccessRegisters> $name<A> {
            pub fn read(&self, idx: usize) -> core::result::Result<$reg_ty, $crate::TooLargeIndex> {
                ReadRegisters::$name(self.access(), idx)
            }
        }
    };
    ($name:ident: [ReadWrite<$reg_ty:ty, $long_ty:ty>; $len:literal]) => {
        impl<A: AccessRegisters> $name<A> {
            pub fn read(&self, idx: usize) -> core::result::Result<$reg_ty, $crate::TooLargeIndex> {
                ReadRegisters::$name(self.access(), idx)
            }
            pub fn write(&self, value: $reg_ty, idx: usize) -> core::result::Result<$crate::WriteSuccess, $crate::TooLargeIndex> {
                WriteRegisters::$name(self.access(), value, idx)
            }
        }
    };
    ($name:ident: [WriteOnly<$reg_ty:ty, $long_ty:ty>; $len:literal]) => {
        impl<A: AccessRegisters> $name<A> {
            pub fn write(&self, value: $reg_ty, idx: usize) -> core::result::Result<$crate::WriteSuccess, $crate::TooLargeIndex> {
                WriteRegisters::$name(self.access(), value, idx)
            }
        }
    };
    ($name:ident: ReadOnly<$reg_ty:ty, $long_ty:ty>) => {
        impl<A: AccessRegisters> $name<A> {
            pub fn read(&self) -> $reg_ty {
                ReadRegisters::$name(self.access())
            }
        }
    };
    ($name:ident: ReadWrite<$reg_ty:ty, $long_ty:ty>) => {
        impl<A: AccessRegisters> $name<A> {
            pub fn read(&self) -> $reg_ty {
                ReadRegisters::$name(self.access())
            }
            pub fn write(&self, value: $reg_ty) {
                WriteRegisters::$name(self.access(), value);
            }
        }
    };
    ($name:ident: WriteOnly<$reg_ty:ty, $long_ty:ty>) => {
        impl<A: AccessRegisters> $name<A> {
            pub fn write(&self, value: $reg_ty) {
                WriteRegisters::$name(self.access(), value);
            }
        }
    };
}
