#![recursion_limit = "128"]

extern crate proc_macro;
extern crate syn;

#[macro_use]
extern crate quote;

use proc_macro::TokenStream;
use quote::ToTokens;
use syn::DeriveInput;

// https://github.com/dtolnay/syn

#[proc_macro_derive(NoPeripheralManagement, attributes(RegisterType))]
pub fn no_clock_control_mmio_hardware(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let input: DeriveInput = syn::parse(input).unwrap();

    let s = input.attrs.iter().find(|a| {
        a.interpret_meta()
            .map_or(false, |m| m.name() == "RegisterType")
    });
    let ss = s.expect("Missing RegisterType. Add `#[RegisterType(TestRegisters)]` after deriving");
    let meta = ss.interpret_meta()
        .expect("RegisterType requires a value, like `#[RegisterType(TestRegisters)]`");
    let reg_type_str = match meta {
        syn::Meta::Word(_) => panic!("RegisterType requires a value, like `#[RegisterType(TestRegisters)]` (got `Word` type)"),
        syn::Meta::List(meta_list) => meta_list.nested,
        syn::Meta::NameValue(_) => panic!("RegisterType requires a value, like `#[RegisterType(TestRegisters)]` (got `NameValue` type)"),
    };
    let reg_type = reg_type_str.into_tokens();

    // Create the implementation
    let name = &input.ident;
    let expanded = quote! {
        // n.b. Need fully qualified paths o/w callee's need `use` imports
        impl ::kernel::common::peripherals::PeripheralManagement<::kernel::NoClockControl>
            for #name {
            type RegisterType = #reg_type;

            fn get_registers(&self) -> &Self::RegisterType {
                &*self.registers
            }

            fn get_clock(&self) -> &::kernel::NoClockControl {
                unsafe { &::kernel::NO_CLOCK_CONTROL }
            }

            fn before_peripheral_access(&self,
                                  _clock: &::kernel::NoClockControl,
                                  _registers: &Self::RegisterType)
            {}
            fn after_peripheral_access(&self,
                                 _clock: &::kernel::NoClockControl,
                                 _registers: &Self::RegisterType)
            {}
        }
    };

    // And return the generated code
    expanded.into()
}
