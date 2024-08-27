// Copyright OxidOS Automotive 2024.

use quote::format_ident;

use crate::{platform::peripherals::uart, Capsule, Ident};
use std::rc::Rc;

/// The [`Console`] capsule can be configured through the UART device used
/// by the capsule. Could be either the raw UART device or a virtual one that wraps it.
#[parse_macros::component(curr, ident = "console")]
pub struct Console<U: uart::Uart> {
    pub(crate) mux_uart: Rc<uart::MuxUart<U>>,
}

impl<U: uart::Uart + 'static> Console<U> {
    pub fn get(mux_uart: Rc<uart::MuxUart<U>>) -> Rc<Self> {
        Rc::new(Self::new(mux_uart))
    }
}

impl<U: uart::Uart + 'static> crate::Component for Console<U> {
    fn dependencies(&self) -> Option<Vec<Rc<dyn crate::Component>>> {
        Some(vec![self.mux_uart.clone()])
    }

    fn ty(&self) -> Result<proc_macro2::TokenStream, crate::Error> {
        Ok(quote::quote!(capsules_core::console::Console<'static>))
    }

    fn init_expr(&self) -> Result<proc_macro2::TokenStream, crate::Error> {
        let mux_uart = format_ident!("{}", self.mux_uart.ident()?);
        let driver_num = self.driver_num();

        Ok(quote::quote! {
            components::console::ConsoleComponent::new(
                board_kernel,
                #driver_num,
                #mux_uart,
            )
            .finalize(components::console_component_static!());
        })
    }
}

impl<U: uart::Uart + 'static> crate::Capsule for Console<U> {
    fn driver_num(&self) -> proc_macro2::TokenStream {
        quote::quote! {
            capsules_core::console::DRIVER_NUM
        }
    }
}
