//! Tock board Platform-struct helper macro
//!
//! The exported [`platform_helper`] macro can be used to generate a
//! struct defintion containing references to the installed drivers on
//! a board, as well as automatically implementing the
//! [`kernel::Platform`] trait.
//!
//! ## Usage example
//!
//! The following example will generate a struct called `MyTockBoard`,
//! with fields and types as specified. It will further implement
//! [`kernel::Platform`] on the struct, mapping system call numbers
//! (prior to `=>`) to the respective field in the struct.
//!
//! ```rust
//! platform_helper!(
//!     MyTockBoard,
//!     drivers: {
//!         console: capsules::console::DRIVER_NUM =>
//!             &'static capsules::console::Console<'static>,
//!         alarm: capsules::alarm::DRIVER_NUM =>
//!             &'static capsules::alarm::AlarmDriver<
//!                 'static,
//!                 VirtualMuxAlarm<'static, sam4l::ast::Ast<'static>>,
//!             >,
//!     },
//!     legacy_drivers: {
//!         gpio: capsules::gpio::DRIVER_NUM =>
//!             &'static capsules::gpio::GPIO<'static, sam4l::gpio::GPIOPin<'static>>,
//!     },
//! );
//! ```

#[macro_export]
macro_rules! platform_helper {
    (
        $struct_name:ident,
        drivers: {
            $($did:ident : $dnum:expr => $dty:ty),* $(,)?
        },
        legacy_drivers: {
            $($ldid:ident : $ldnum:expr => $ldty:ty),* $(,)?
        },
        additional_fields: {
            $($aid:ident : $aty:ty),* $(,)?
        } $(,)?
    ) => {
        /// A structure representing this platform that holds references to all
        /// capsules for this platform.
        struct $struct_name {
            // Tock 2.0 drivers
            $($did : $dty,)*

            // Legacy drivers
            $($ldid : $ldty,)*

            // Additional fields
            $($aid : $aty,)*
        }

        impl kernel::Platform for $struct_name {
            /// Mapping of integer syscalls to objects that implement syscalls.
            #[allow(non_upper_case_globals)]
            fn with_driver<F, R>(&self, driver_num: usize, f: F) -> R
            where
                F: FnOnce(Option<Result<&dyn kernel::Driver, &dyn kernel::LegacyDriver>>) -> R,
            {
                use core::borrow::Borrow;
                $(const $did: usize = $dnum;)*
                $(const $ldid: usize = $ldnum;)*

                match driver_num {
                    $($did => f(Some(Ok((self.$did).borrow()))),)*
                    $($ldid => f(Some(Err((self.$ldid).borrow()))),)*
                    _ => f(None),
                }
            }
        }
    };

    // ----- Alternate parameter orderings -----

    // One parameter: drivers
    (
        $struct_name:ident,
        drivers: $dtt:tt $(,)?
    ) => {
        platform_helper!(
            $struct_name,
            drivers: $dtt,
            legacy_drivers: {},
            additional_fields: {},
        );
    };

    // Two parameters:
    // - drivers
    // - legacy_drivers
    (
        $struct_name:ident,
        drivers: $dtt:tt,
        legacy_drivers: $ldtt:tt $(,)?
    ) => {
        platform_helper!(
            $struct_name,
            drivers: $dtt,
            legacy_drivers: $ldtt,
            additional_fields: {},
        );
    };

    // Two parameters:
    // - legacy_drivers
    // - drivers
    (
        $struct_name:ident,
        legacy_drivers: $ldtt:tt,
        drivers: $dtt:tt $(,)?
    ) => {
        platform_helper!(
            $struct_name,
            drivers: $dtt,
            legacy_drivers: $ldtt,
            additional_fields: {},
        );
    };

    // Two parameters:
    // - drivers
    // - additional_fields
    (
        $struct_name:ident,
        drivers: $dtt:tt,
        additional_fields: $att:tt $(,)?
    ) => {
        platform_helper!(
            $struct_name,
            drivers: $dtt,
            legacy_drivers: {},
            additional_fields: $att,
        );
    };

    // Two parameters:
    // - additional_fields
    // - drivers
    (
        $struct_name:ident,
        additional_fields: $att:tt,
        drivers: $dtt:tt $(,)?
    ) => {
        platform_helper!(
            $struct_name,
            drivers: $dtt,
            legacy_drivers: {},
            additional_fields: $att,
        );
    };

    // Three parameters:
    // - drivers
    // - additional_fields
    // - legacy_drivers
    (
        $struct_name:ident,
        drivers: $dtt:tt,
        additional_fields: $att:tt,
        legacy_drivers: $ldtt:tt $(,)?
    ) => {
        platform_helper!(
            $struct_name,
            drivers: $dtt,
            legacy_drivers: $ldtt,
            additional_fields: $att,
        );
    };

    // Three parameters:
    // - legacy_drivers
    // - drivers
    // - additional_fields
    (
        $struct_name:ident,
        legacy_drivers: $ldtt:tt,
        drivers: $dtt:tt,
        additional_fields: $att:tt $(,)?
    ) => {
        platform_helper!(
            $struct_name,
            drivers: $dtt,
            legacy_drivers: $ldtt,
            additional_fields: $att,
        );
    };

    // Three parameters:
    // - legacy_drivers
    // - additional_fields
    // - drivers
    (
        $struct_name:ident,
        legacy_drivers: $ldtt:tt,
        additional_fields: $att:tt,
        drivers: $dtt:tt $(,)?
    ) => {
        platform_helper!(
            $struct_name,
            drivers: $dtt,
            legacy_drivers: $ldtt,
            additional_fields: $att,
        );
    };

    // Three parameters:
    // - additional_fields
    // - drivers
    // - legacy_drivers
    (
        $struct_name:ident,
        additional_fields: $att:tt,
        drivers: $dtt:tt,
        legacy_drivers: $ldtt:tt $(,)?
    ) => {
        platform_helper!(
            $struct_name,
            drivers: $dtt,
            legacy_drivers: $ldtt,
            additional_fields: $att,
        );
    };

    // Three parameters:
    // - additional_fields
    // - legacy_drivers
    // - drivers
    (
        $struct_name:ident,
        additional_fields: $att:tt,
        legacy_drivers: $ldtt:tt,
        drivers: $dtt:tt $(,)?
    ) => {
        platform_helper!(
            $struct_name,
            drivers: $dtt,
            legacy_drivers: $ldtt,
            additional_fields: $att,
        );
    };

}
