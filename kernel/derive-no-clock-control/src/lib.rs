extern crate proc_macro;
#[macro_use]
extern crate quote;
extern crate syn;

use proc_macro::TokenStream;

#[proc_macro_derive(NoClockControlMMIORegisters)]
pub fn no_clock_control_mmio_registers(input: TokenStream) -> TokenStream {
    // Get string of type definition
    let s = input.to_string();

    // Parse the string of Rust code
    let ast = syn::parse_derive_input(&s).unwrap();

    // Create the implementation
    let gen = impl_no_clock_control_mmio_registers(&ast);

    // And return the generated code
    gen.parse().unwrap()
}

fn impl_no_clock_control_mmio_registers(ast: &syn::DeriveInput) -> quote::Tokens {
    let name = &ast.ident;
    quote! {
        // n.b. Need fully qualified paths o/w callee's need `use` imports
        impl ::kernel::MMIOClockGuard<::kernel::NoClockControl> for #name {
            fn before_mmio_access(&self, _clock: &::kernel::NoClockControl) {}
            fn after_mmio_access(&self, _clock: &::kernel::NoClockControl) {}
        }
    }
}

#[proc_macro_derive(NoClockControlMMIOHardware)]
pub fn no_clock_control_mmio_hardware(input: TokenStream) -> TokenStream {
    // Get string of type definition
    let s = input.to_string();

    // Parse the string of Rust code
    let ast = syn::parse_derive_input(&s).unwrap();

    // Create the implementation
    let gen = impl_no_clock_control_mmio_hardware(&ast);

    // And return the generated code
    gen.parse().unwrap()
}

fn impl_no_clock_control_mmio_hardware(ast: &syn::DeriveInput) -> quote::Tokens {
    let name = &ast.ident;
    quote! {
        // n.b. Need fully qualified paths o/w callee's need `use` imports
        impl ::kernel::MMIOClockInterface<::kernel::NoClockControl> for #name {
            fn get_clock(&self) -> &::kernel::NoClockControl {
                unsafe { &::kernel::NO_CLOCK_CONTROL }
            }
        }
    }
}
