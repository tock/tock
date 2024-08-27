// Copyright OxidOS Automotive 2024.

pub mod capsules;
pub use capsules::*;

pub mod peripherals;
pub use peripherals::*;

pub mod scheduler;
use quote::{format_ident, quote};
pub use scheduler::*;

pub mod scheduler_timer;
pub use scheduler_timer::*;

pub mod syscall_filter;
pub use syscall_filter::*;

use crate::Ident;

/// The platform *(board)* that contains the needed fields for the defined struct
/// to implement the `...` trait.
#[parse_macros::component(curr, ident = "platform")]
pub struct Platform<C: Chip> {
    pub ty: String,
    pub capsules: Vec<std::rc::Rc<dyn crate::Capsule>>,
    pub scheduler: std::rc::Rc<Scheduler>,
    pub systick: std::rc::Rc<C::Systick>,
}

impl<C: Chip + 'static> crate::Component for Platform<C> {
    fn dependencies(&self) -> Option<Vec<std::rc::Rc<dyn crate::Component>>> {
        let mut dependencies = self
            .capsules
            .iter()
            .map(|c| c.clone().as_component() as std::rc::Rc<dyn crate::Component>)
            .collect::<Vec<_>>();

        dependencies.push(self.scheduler.clone());

        Some(dependencies)
    }

    fn ty(&self) -> Result<proc_macro2::TokenStream, crate::Error> {
        let ty = format_ident!("{}", self.ty);
        Ok(quote!(#ty))
    }

    fn init_expr(&self) -> Result<proc_macro2::TokenStream, crate::Error> {
        let ty = self.ty()?;
        let scheduler = format_ident!("{}", self.scheduler.as_ref().ident()?);
        let (systick_id, systick_init) = (
            format_ident!("{}", self.systick.as_ref().ident()?),
            self.systick.init_expr()?,
        );
        let capsules = self
            .capsules
            .iter()
            .map(|c| format_ident!("{}", c.as_ref().ident().unwrap()))
            .collect::<Vec<_>>();

        Ok(quote! {
            #ty {
                #(#capsules,)*
                #scheduler,
                #systick_id: #systick_init
            }
        })
    }
}
