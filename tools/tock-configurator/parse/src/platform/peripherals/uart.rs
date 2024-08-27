// Copyright OxidOS Automotive 2024.

use crate::Component;
use quote::quote;
use std::rc::Rc;

/// The [`Uart`] trait applies to devices that implement the Uart-related traits defined in
/// Tock's HIL.
pub trait Uart: Component + std::fmt::Debug + PartialEq + std::fmt::Display {}

/// Virtual multiplexed UART. Required by the `Console` capsule.
#[parse_macros::component(curr, ident = "mux_uart")]
pub struct MuxUart<U: Uart> {
    peripheral: Rc<U>,
    baud_rate: usize,
}

impl<U: Uart> MuxUart<U> {
    pub fn baud_rate(&self) -> usize {
        self.baud_rate
    }

    pub fn uart(&self) -> Rc<U> {
        self.peripheral.clone()
    }
}

impl<U: Uart + 'static> MuxUart<U> {
    pub fn insert_get(
        peripheral: Rc<U>,
        baud_rate: usize,
        visited: &mut Vec<Rc<dyn Component>>,
    ) -> Rc<Self> {
        // Iterate over the existing nodes.
        for node in visited.iter() {
            // Check if the node is of the `MuxUart` type.
            if let Ok(mux_uart) = node.clone().downcast::<MuxUart<U>>() {
                if mux_uart.uart() == peripheral && baud_rate == mux_uart.baud_rate() {
                    // If a mux uart with the same fields exists, return it.
                    return mux_uart;
                }
            }
        }

        // No mux uart with the same values was found in the node list, so return a new one.
        let mux_uart = Rc::new(MuxUart::new(peripheral, baud_rate));
        visited.push(mux_uart.clone() as Rc<dyn Component>);

        mux_uart
    }
}

impl<U: Uart + 'static> crate::Component for MuxUart<U> {
    fn dependencies(&self) -> Option<Vec<Rc<dyn crate::Component>>> {
        Some(vec![self.peripheral.clone()])
    }

    fn init_expr(&self) -> Result<proc_macro2::TokenStream, crate::Error> {
        let baud_rate = self.baud_rate as u32;
        let uart_ident: proc_macro2::TokenStream =
            self.peripheral.as_ref().ident()?.parse().unwrap();

        let uart_before = self.peripheral.before_usage();
        Ok(quote! {
            {
                #uart_before
                components::console::UartMuxComponent::new(&#uart_ident, #baud_rate)
                .finalize(components::uart_mux_component_static!())
            }
        })
    }
}
