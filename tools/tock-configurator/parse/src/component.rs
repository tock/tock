// Copyright OxidOS Automotive 2024.

use std::any::{Any, TypeId};
use std::rc::Rc;

/// Constants used in the identifying process for component nodes that
/// are sure to never be instantiated for the same type twice.
pub mod constants {
    use crate::static_ident;
    pub use once_cell::sync::Lazy;

    pub static PERIPHERALS: Lazy<String> = static_ident!("peripherals");
    pub static CHIP: Lazy<String> = static_ident!("chip");
}

/// A trait for objects that define a Tock `capsule`.
///
/// Besides the [`crate::Component`] implementation, these types must provide the code
/// expression that returns the driver number for the capsule.
///
/// # Example
///
/// ```rust,ignore
/// use quote::quote;
///
/// impl Capsule for Console {
///     fn driver_num() -> proc_macro2::TokenStream {
///         quote!(capsules::core::console::DRIVER_NUM);
///     }
/// }
/// ```
pub trait Capsule: crate::Component {
    fn driver_num(&self) -> proc_macro2::TokenStream;
}

/// A trait for objects that define variables represented by an **unique** identifier.
///
/// Additionally, the procedural macro [`parse_macros::component`] provides an implementation
/// of the [`Ident`] trait.
///
/// # Example
///
/// ```rust,ignore
/// // Proc macro that implements the `Ident` trait.
/// #[parse::node(ident = "console")]
/// struct Console {
///     /* ... */
/// }
/// ```
pub trait Ident {
    fn ident(&self) -> Result<String, crate::error::Error>;
}

//  TODO: The ident trait must have a static str? so that this is not to be confused?
// This can't be object safe unless... the ident parameter is an uuid (has default),
// and the base is a static/ implemented in the ident function?

impl Ident for super::NoSupport {
    fn ident(&self) -> Result<String, crate::error::Error> {
        Err(crate::error::Error::CodeNotProvided)
    }
}

impl<P: crate::DefaultPeripherals> Ident for P {
    fn ident(&self) -> Result<String, crate::error::Error> {
        Ok(constants::PERIPHERALS.clone())
    }
}

/// A trait for objects that define variables from a function in the platform's main.
///
/// Besides the [`Ident`] implementation, the *component* can optionally provide the type of the
/// variable, the code expression that returns a new instance, and the dependencies for the
/// initialization.
pub trait Component: Ident + AsComponent {
    /// Return the code for the type of the component if needed, else, return `None`.
    fn ty(&self) -> Result<proc_macro2::TokenStream, crate::Error> {
        Err(crate::Error::CodeNotProvided)
    }

    /// Return the code for the initialization expression of the component if needed, else, return `None`.
    fn init_expr(&self) -> Result<proc_macro2::TokenStream, crate::Error> {
        Err(crate::Error::CodeNotProvided)
    }

    /// Return the list of dependencies for the initialization of the component if it has any,
    /// else, return `None`.
    fn dependencies(&self) -> Option<Vec<Rc<dyn Component>>> {
        None
    }
    /// Check if the current component is a peripheral.
    ///
    /// This is needed to avoid the need for multiple structs with
    /// cyclic dependencies to the `Peripherals` "parent" component.
    fn is_peripheral(&self) -> bool {
        false
    }

    /// Return code expression that must be run after the initialization
    /// of this component if it exists.
    fn after_init(&self) -> Option<proc_macro2::TokenStream> {
        None
    }

    /// Return code expression that must be run before the initialization
    /// of this component if it exists.
    fn before_init(&self) -> Option<proc_macro2::TokenStream> {
        None
    }

    /// Return peripheral prelude code before using it inside a capsule.
    ///  TODO: This should be moved (?) in a different `Peripheral` trait,
    /// similar to `Capsule`.
    fn before_usage(&self) -> Option<proc_macro2::TokenStream> {
        None
    }
}

// Used for finding types in a list of `Component` trait objects.
impl dyn Component {
    pub fn is<T: 'static>(&self) -> bool {
        TypeId::of::<T>() == self.type_id()
    }

    pub fn downcast<T: 'static>(self: Rc<Self>) -> Result<Rc<T>, Rc<Self>> {
        if self.is::<T>() {
            unsafe { Ok(Rc::from_raw(Rc::into_raw(self) as _)) }
        } else {
            Err(self)
        }
    }
}

pub trait AsComponent {
    fn as_component(self: Rc<Self>) -> Rc<dyn Component>;
}

impl<C: Component + 'static> AsComponent for C {
    fn as_component(self: Rc<Self>) -> Rc<dyn Component> {
        self
    }
}

pub trait FormatIdent
where
    String: From<Self>,
    Self: Sized,
{
    fn format_ident(self) -> String {
        String::from(self).to_lowercase().replace('-', "_")
    }
}

impl FormatIdent for uuid::Uuid {}
