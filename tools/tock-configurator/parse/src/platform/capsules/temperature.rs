// Copyright OxidOS Automotive 2024.

use crate::{temp, Capsule, Component};
use parse_macros::component;
use quote::quote;
use std::rc::Rc;

#[component(curr, ident = "temperature")]
pub struct Temperature<T: temp::Temperature + 'static> {
    /// Temperature driver used by the capsule.
    peripheral: Rc<T>,
}

impl<T: temp::Temperature + 'static> Temperature<T> {
    pub fn get(inner: Rc<T>) -> Rc<Self> {
        Rc::new(Self::new(inner))
    }
}

impl<T: temp::Temperature> Component for Temperature<T> {
    fn ty(&self) -> Result<proc_macro2::TokenStream, crate::Error> {
        let temp_ty = self.peripheral.ty()?;
        Ok(quote! {
            components::temperature::TemperatureComponentType<#temp_ty>
        })
    }

    fn init_expr(&self) -> Result<proc_macro2::TokenStream, crate::Error> {
        let temp_ident: proc_macro2::TokenStream = self.peripheral.ident()?.parse().unwrap();
        let temp_ty = self.peripheral.ty()?;
        Ok(quote! {
                components::temperature::TemperatureComponent::new(
                    board_kernel,
                    capsules_extra::temperature::DRIVER_NUM,
                    &#temp_ident,
                )
                .finalize(components::temperature_component_static!(
                        #temp_ty
                ))
        })
    }
}

impl<T: temp::Temperature> Capsule for Temperature<T> {
    fn driver_num(&self) -> proc_macro2::TokenStream {
        quote!(capsules_extra::temperature::DRIVER_NUM)
    }
}
