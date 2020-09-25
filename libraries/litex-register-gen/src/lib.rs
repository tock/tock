//! Generate LiteX register abstractions
//!
//! This crate exports a procedural macro to generate LiteX register
//! abstractions over different widths, data types and alignments
//! programmatically.
//!
//! Usage examples can be found in
//! `chips/litex/src/litex_registers.rs`

#![feature(proc_macro_diagnostic)]

extern crate proc_macro;
#[macro_use]
extern crate quote;
extern crate proc_macro2;

use core::convert::TryFrom;
use proc_macro::TokenStream as PMTokenStream;
use proc_macro2::{Ident, TokenStream};

mod arguments;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub(crate) enum AccessType {
    ReadOnly,
    WriteOnly,
    ReadWrite,
}
impl AccessType {
    fn tock_registers_type(&self) -> Ident {
        match self {
            AccessType::ReadOnly => format_ident!("TRReadOnly"),
            AccessType::WriteOnly => format_ident!("TRWriteOnly"),
            AccessType::ReadWrite => format_ident!("TRReadWrite"),
        }
    }
}

#[repr(usize)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub(crate) enum IntegerWidth {
    U8 = 8,
    U16 = 16,
    U32 = 32,
    U64 = 64,
}

impl IntegerWidth {
    fn width(&self) -> usize {
        *self as usize
    }

    fn bytes(&self) -> usize {
        self.width() / 8
    }

    fn to_primitive_type(&self) -> Ident {
        match self {
            IntegerWidth::U8 => format_ident!("u8"),
            IntegerWidth::U16 => format_ident!("u16"),
            IntegerWidth::U32 => format_ident!("u32"),
            IntegerWidth::U64 => format_ident!("u64"),
        }
    }
}

impl TryFrom<&str> for IntegerWidth {
    type Error = ();

    fn try_from(type_name: &str) -> Result<IntegerWidth, Self::Error> {
        match type_name {
            "u8" => Ok(IntegerWidth::U8),
            "u16" => Ok(IntegerWidth::U16),
            "u32" => Ok(IntegerWidth::U32),
            "u64" => Ok(IntegerWidth::U64),
            _ => Err(()),
        }
    }
}

impl TryFrom<usize> for IntegerWidth {
    type Error = ();

    fn try_from(width: usize) -> Result<IntegerWidth, Self::Error> {
        match width {
            8 => Ok(IntegerWidth::U8),
            16 => Ok(IntegerWidth::U16),
            32 => Ok(IntegerWidth::U32),
            64 => Ok(IntegerWidth::U64),
            _ => Err(()),
        }
    }
}

fn padded_partial_register(
    partial_count: usize,
    base_width: IntegerWidth,
    wishbone_data_width: IntegerWidth,
    at: AccessType,
) -> Result<TokenStream, ()> {
    if wishbone_data_width.width() > base_width.width() {
        return Err(());
    }

    let padding = (base_width.width() - wishbone_data_width.width()) / 8;

    let partial_type = {
        let tock_register_base_type = match at {
            AccessType::ReadOnly => format_ident!("TRReadOnly"),
            AccessType::WriteOnly => format_ident!("TRWriteOnly"),
            AccessType::ReadWrite => format_ident!("TRReadWrite"),
        };

        let integer_primitive = wishbone_data_width.to_primitive_type();

        quote! {
            #tock_register_base_type<#integer_primitive>
        }
    };

    let reg_ident = format_ident!("reg_p{}", partial_count);
    let padding_ident = format_ident!("_reserved_{}", partial_count);

    let padded_partial = quote! {
    #reg_ident: #partial_type,
    #padding_ident: [u8; #padding],
    };

    Ok(padded_partial.into())
}

fn partial_register_collection(
    base_width: IntegerWidth,
    wishbone_data_width: IntegerWidth,
    value_width: IntegerWidth,
    at: AccessType,
) -> Result<TokenStream, ()> {
    if wishbone_data_width.width() > value_width.width() {
        let padding = wishbone_data_width.bytes() - value_width.bytes();
        let integer_primitive = value_width.to_primitive_type();
        let tock_registers_type = at.tock_registers_type();

        Ok(quote! {
            reg_p0: #tock_registers_type<#integer_primitive>,
            _reserved_0: [u8; #padding],
        })
    } else {
        let count = value_width.width() / wishbone_data_width.width();
        let partial_registers = (0..count)
            .map(|c| padded_partial_register(c, base_width, wishbone_data_width, at))
            .collect::<Result<Vec<TokenStream>, ()>>()?;

        Ok(quote! {
            #(#partial_registers)*
        })
    }
}

fn litex_register_abstraction_type(
    name: Ident,
    base_width: IntegerWidth,
    wishbone_data_width: IntegerWidth,
    value_width: IntegerWidth,
    at: AccessType,
) -> Result<TokenStream, ()> {
    if wishbone_data_width.width() > base_width.width() {
        return Err(());
    }

    let partial_registers =
        partial_register_collection(base_width, wishbone_data_width, value_width, at)?;

    Ok(quote! {
    #[repr(C)]
    pub struct #name<N: RegisterLongName = ()> {
        #partial_registers
        _regname: PhantomData<N>,
    }
    })
}

fn litex_register_abstraction_write(
    name: Ident,
    wishbone_data_width: IntegerWidth,
    value_width: IntegerWidth,
) -> TokenStream {
    let value_type = value_width.to_primitive_type();
    let value_width_bits = value_width.width();
    if wishbone_data_width.width() > value_width.width() {
        quote! {
                impl<N: RegisterLongName> BaseWriteableRegister<#value_type> for #name<N> {
            type Reg = N;
            const REG_WIDTH: usize = #value_width_bits;
                    #[inline]
            fn base_set(&self, value: #value_type) {
            self.reg_p0.set(value)
        }
            }
        }
    } else {
        let value_bytes = value_width.bytes();
        let wishbone_data_type = wishbone_data_width.to_primitive_type();

        let reg_setter = |c, bo, bc| {
            let reg_ident = format_ident!("reg_p{}", c);
            let indicies: Vec<usize> = (bo..(bo + bc)).collect();

            quote! {
                self.#reg_ident.set(#wishbone_data_type::from_be_bytes([#(bytes[#indicies]),*]));
            }
        };

        let setters: Vec<TokenStream> = (0..value_width.bytes())
            .step_by(wishbone_data_width.bytes())
            .enumerate()
            .map(|(c, byte_offset)| reg_setter(c, byte_offset, wishbone_data_width.bytes()))
            .collect();

        quote! {
        impl<N: RegisterLongName> BaseWriteableRegister<#value_type> for #name<N> {
            type Reg = N;
            const REG_WIDTH: usize = #value_width_bits;

            #[inline]
            fn base_set(&self, value: #value_type) {
            let bytes: [u8; #value_bytes]  = #value_type::to_be_bytes(value);

            #(#setters)*
            }
        }
        }
    }
}

fn litex_register_abstraction_read(
    name: Ident,
    wishbone_data_width: IntegerWidth,
    value_width: IntegerWidth,
) -> TokenStream {
    let value_width_bits = value_width.width();
    let value_type = value_width.to_primitive_type();
    if wishbone_data_width.width() > value_width.width() {
        quote! {
                impl<N: RegisterLongName> BaseReadableRegister<#value_type> for #name<N> {
            type Reg = N;
            const REG_WIDTH: usize = #value_width_bits;
                    fn base_get(&self) -> #value_type {
                self.reg_p0.get()
                }
            }
        }
    } else {
        let reg_bytes = wishbone_data_width.bytes();
        let wishbone_type = wishbone_data_width.to_primitive_type();

        let reg_reader = |c| {
            let reg_ident = format_ident!("reg_p{}", c);
            let var_ident = format_ident!("reg_p{}_val", c);

            (
                var_ident.clone(),
                quote! {
                let #var_ident: [u8; #reg_bytes] =
                    #wishbone_type::to_be_bytes(self.#reg_ident.get());
                },
            )
        };
        let readers: Vec<(Ident, TokenStream)> = (0..value_width.bytes())
            .step_by(wishbone_data_width.bytes())
            .enumerate()
            .map(|(c, _)| reg_reader(c))
            .collect();

        let reg_byte_addresser = |var_ident: Ident, bc| {
            let bytes: Vec<usize> = (0..bc).collect();
            quote! {
                #(#var_ident[#bytes]),*
            }
        };
        let addressers: Vec<TokenStream> = readers
            .iter()
            .map(|(var_ident, _)| reg_byte_addresser(var_ident.clone(), reg_bytes))
            .collect();

        let readers_tokens: Vec<TokenStream> = readers.into_iter().map(|(_ident, ts)| ts).collect();

        quote! {
        impl<N: RegisterLongName> BaseReadableRegister<#value_type> for #name<N> {
            type Reg = N;
            const REG_WIDTH: usize = #value_width_bits;

            #[inline]
            fn base_get(&self) -> #value_type {
            #(#readers_tokens)*

            #value_type::from_be_bytes([#(#addressers),*])
            }
        }
        }
    }
}

#[proc_macro]
pub fn litex_register_abstraction(input: PMTokenStream) -> PMTokenStream {
    let params = match arguments::litex_register_abstraction_parse_arguments(input) {
        Ok(params) => params,
        Err(ts) => {
            return ts;
        }
    };

    let ratype = litex_register_abstraction_type(
        params.name.clone(),
        params.base_width,
        params.wishbone_data_width,
        params.value_width,
        params.access_type,
    )
    .unwrap();
    let raread = litex_register_abstraction_read(
        params.name.clone(),
        params.wishbone_data_width,
        params.value_width,
    );
    let rawrite = litex_register_abstraction_write(
        params.name,
        params.wishbone_data_width,
        params.value_width,
    );

    let ts = match params.access_type {
        AccessType::ReadOnly => vec![ratype, raread],
        AccessType::WriteOnly => vec![ratype, rawrite],
        AccessType::ReadWrite => vec![ratype, raread, rawrite],
    };

    let generated = quote! {
    #(#ts)*
    };

    generated.into()
}
