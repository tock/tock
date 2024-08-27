// Copyright OxidOS Automotive 2024.

use crate::{peripherals::rng, Capsule, Component};
use parse_macros::component;
use quote::quote;
use std::rc::Rc;

#[component(curr, ident = "rng")]
pub struct RngCapsule<R: rng::Rng + 'static> {
    peripheral: Rc<R>,
}

impl<R: rng::Rng + 'static> RngCapsule<R> {
    #[inline]
    pub fn get(peripheral: Rc<R>) -> Rc<Self> {
        Rc::new(Self::new(peripheral))
    }
}

impl<R: rng::Rng> Component for RngCapsule<R> {
    fn ty(&self) -> Result<proc_macro2::TokenStream, crate::Error> {
        let inner_ty = self.peripheral.ty()?;
        Ok(quote!(components::rng::RngComponentType<#inner_ty>))
    }

    fn init_expr(&self) -> Result<proc_macro2::TokenStream, crate::Error> {
        let inner_ty = self.peripheral.ty()?;
        let inner_ident: proc_macro2::TokenStream = self.peripheral.ident()?.parse().unwrap();
        Ok(quote! {
             components::rng::RngComponent::new(
                 board_kernel,
                 capsules_core::rng::DRIVER_NUM,
                 &#inner_ident,
             )
             .finalize(components::rng_component_static!(#inner_ty));
        })
    }
}

impl<R: rng::Rng> Capsule for RngCapsule<R> {
    fn driver_num(&self) -> proc_macro2::TokenStream {
        quote!(capsules_core::rng::DRIVER_NUM)
    }
}
