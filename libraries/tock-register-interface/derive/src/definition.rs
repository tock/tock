// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.
// Copyright Google LLC 2024.

use crate::{Input, Safety};
use proc_macro2::TokenStream;
use quote::quote;

/// Generates the peripheral trait definition.
pub fn definition(input: &Input) -> TokenStream {
    let comments = &input.comments;
    let cfgs = &input.cfgs;
    let visibility = &input.visibility;
    let name = &input.name;
    let fields = input.fields.iter().filter_map(|field| {
        let register = field.contents.register()?;
        let cfgs = &field.cfgs;
        let name = &register.name;
        let tock_registers = &input.tock_registers;
        let data_type = &register.data_type;
        let read_bound = match &register.read {
            None => quote![],
            Some(Safety::Safe(op) | Safety::Unsafe(op)) => quote![+ #tock_registers::#op],
        };
        let write_bound = match &register.write {
            None => quote![],
            Some(Safety::Safe(op) | Safety::Unsafe(op)) => quote![+ #tock_registers::#op],
        };
        let comments = &field.comments;
        Some(quote! {
            #(#cfgs)*
            type #name<'s>: #tock_registers::Register<DataType = #data_type>
                #read_bound #write_bound where Self: 's;
            #(#comments)* #(#cfgs)* fn #name(&self) -> Self::#name<'_>;
        })
    });
    quote! {
        #(#comments)*
        #(#cfgs)*
        #[allow(non_camel_case_types)]
        #visibility trait #name {
            #(#fields)*
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::assert_tokens_eq;
    use syn::parse_quote;

    #[test]
    fn attributes() {
        assert_tokens_eq(
            definition(&parse_quote! {tock_registers;
                #[allow_bus_adapter]
                #[cfg(feature = "A")]
                /// Doc comment 1
                #[cfg(feature = "B")]
                /// Doc comment 2
                Foo {
                    #[cfg(feature = "C")]
                    /// Doc comment 3
                    #[cfg(feature = "D")]
                    /// Doc comment 4
                    0x0 => a: u32 { Read },

                    #[cfg(feature = "E")]
                    /// Doc comment 5
                    #[cfg(feature = "F")]
                    /// Doc comment 6
                    0x4 => _,

                    #[cfg(feature = "G")]
                    /// Doc comment 7
                    #[cfg(feature = "H")]
                    /// Doc comment 8
                    0x5 => b: [u8; 2] { Write },
                }
            }),
            quote! {
                /// Doc comment 1
                /// Doc comment 2
                #[cfg(feature = "A")]
                #[cfg(feature = "B")]
                #[allow(non_camel_case_types)]
                trait Foo {
                    #[cfg(feature = "C")]
                    #[cfg(feature = "D")]
                    type a<'s>: tock_registers::Register<DataType = u32>
                        + tock_registers::Read
                    where
                        Self: 's;
                    /// Doc comment 3
                    /// Doc comment 4
                    #[cfg(feature = "C")]
                    #[cfg(feature = "D")]
                    fn a(&self) -> Self::a<'_>;

                    #[cfg(feature = "G")]
                    #[cfg(feature = "H")]
                    type b<'s>: tock_registers::Register<DataType = [u8; 2]>
                        + tock_registers::Write
                    where
                        Self: 's;
                    /// Doc comment 7
                    /// Doc comment 8
                    #[cfg(feature = "G")]
                    #[cfg(feature = "H")]
                    fn b(&self) -> Self::b<'_>;
                }
            },
        );
    }

    #[test]
    fn trait_bounds() {
        assert_tokens_eq(
            definition(&parse_quote! {tock_registers;
                Foo {
                    0x0 => no_ops: u8 {},
                    0x1 => _,
                    0x2 => safe_write: u16 { Write },
                    _ => unsafe_write: u32 { UnsafeWrite },
                    _ => safe_read: A { Read },
                    _ => safe_rw: Aliased<B, C> { Read, Write },
                    _ => read_unsafe_write: u16 { Read, UnsafeWrite },
                    _ => unsafe_read: u32 { UnsafeRead },
                    _ => write_unsafe_read: D { UnsafeRead, Write },
                    _ => unsafe_read_write: u8 { UnsafeRead, UnsafeWrite },
                }
            }),
            quote! {
                #[allow(non_camel_case_types)]
                trait Foo {
                    type no_ops<'s>: tock_registers::Register<DataType = u8> where Self: 's;
                    fn no_ops(&self) -> Self::no_ops<'_>;

                    type safe_write<'s>: tock_registers::Register<DataType = u16>
                        + tock_registers::Write
                    where
                        Self: 's;
                    fn safe_write(&self) -> Self::safe_write<'_>;

                    type unsafe_write<'s>: tock_registers::Register<DataType = u32>
                        + tock_registers::UnsafeWrite
                    where
                        Self: 's;
                    fn unsafe_write(&self) -> Self::unsafe_write<'_>;

                    type safe_read<'s>: tock_registers::Register<DataType = A>
                        + tock_registers::Read
                    where
                        Self: 's;
                    fn safe_read(&self) -> Self::safe_read<'_>;

                    type safe_rw<'s>: tock_registers::Register<DataType = Aliased<B, C> >
                        + tock_registers::Read
                        + tock_registers::Write
                    where
                        Self: 's;
                    fn safe_rw(&self) -> Self::safe_rw<'_>;

                    type read_unsafe_write<'s>: tock_registers::Register<DataType = u16>
                        + tock_registers::Read
                        + tock_registers::UnsafeWrite
                    where
                        Self: 's;
                    fn read_unsafe_write(&self) -> Self::read_unsafe_write<'_>;

                    type unsafe_read<'s>: tock_registers::Register<DataType = u32>
                        + tock_registers::UnsafeRead
                    where
                        Self: 's;
                    fn unsafe_read(&self) -> Self::unsafe_read<'_>;

                    type write_unsafe_read<'s>: tock_registers::Register<DataType = D>
                        + tock_registers::UnsafeRead
                        + tock_registers::Write
                    where
                        Self: 's;
                    fn write_unsafe_read(&self) -> Self::write_unsafe_read<'_>;

                    type unsafe_read_write<'s>: tock_registers::Register<DataType = u8>
                        + tock_registers::UnsafeRead
                        + tock_registers::UnsafeWrite
                    where
                        Self: 's;
                    fn unsafe_read_write(&self) -> Self::unsafe_read_write<'_>;
                }
            },
        );
    }
}
