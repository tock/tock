use proc_macro::TokenStream as PMTokenStream;
use proc_macro2::TokenStream;
use syn::parse_macro_input;
use syn::spanned::Spanned;

#[derive(Copy, Clone, Debug)]
enum GrantDefaultAttr {
    SubscribeNum(u32),
    AllowNum(u32),
    GrantDefaultPropagate,
}

pub fn derive_grant_default_impl(input: PMTokenStream) -> PMTokenStream {
    let target_struct = parse_macro_input!(input as syn::ItemStruct);

    // Extract the target struct identifier for implementing the
    // GrantDefault trait
    let struct_ident = target_struct.ident.clone();

    // Extract all of the fields, putting them into two buckets:
    // - Fields which can be initialized using the usual Default trait
    // - Fields which require special initialization using the
    //   GrantDefault trait
    let mut default_fields: Vec<(syn::Ident, syn::Type)> = vec![];
    let mut grant_default_fields: Vec<(syn::Ident, syn::Type, GrantDefaultAttr)> = vec![];
    let mut current_allow_num: u32 = 0;
    let mut current_subscribe_num: u32 = 0;

    if let syn::Fields::Named(named_fields) = target_struct.fields {
        for field in named_fields.named.iter() {
            let mut parsed_attr: Option<GrantDefaultAttr> = None;

            for attr in field.attrs.iter() {
                if parsed_attr.is_some() {
                    // We have parsed two known (and thus conflicting
                    // attributes) on the same struct field. Complain.
                    field
                        .span()
                        .unstable()
                        .error("deriving GrantDefault does not work with multiple attributes")
                        .emit();
                    return PMTokenStream::default();
                }

                match attr.parse_meta() {
                    Ok(syn::Meta::NameValue(name_value)) => {
                        if name_value
                            .path
                            .get_ident()
                            .map_or(false, |i| i.to_string() == "subscribe_num")
                        {
                            if let syn::Lit::Int(litint) = name_value.lit {
                                let subscribe_num = litint.base10_parse::<u32>().unwrap();
                                if subscribe_num < current_subscribe_num {
                                    attr.span()
                                        .unstable()
                                        .error("subscribe numbers must be linearly ascending")
                                        .emit();
                                    return PMTokenStream::default();
                                }
                                current_subscribe_num = subscribe_num + 1;

                                parsed_attr = Some(GrantDefaultAttr::SubscribeNum(subscribe_num));
                            } else {
                                name_value
                                    .span()
                                    .unstable()
                                    .error("subscribe_num must be an integer")
                                    .emit();
                                return PMTokenStream::default();
                            }
                        } else if name_value
                            .path
                            .get_ident()
                            .map_or(false, |i| i.to_string() == "allow_num")
                        {
                            if let syn::Lit::Int(litint) = name_value.lit {
                                let allow_num = litint.base10_parse::<u32>().unwrap();
                                if allow_num < current_allow_num {
                                    attr.span()
                                        .unstable()
                                        .error("allow numbers must be linearly ascending")
                                        .emit();
                                    return PMTokenStream::default();
                                }
                                current_allow_num = allow_num + 1;

                                parsed_attr = Some(GrantDefaultAttr::AllowNum(allow_num));
                            } else {
                                name_value
                                    .span()
                                    .unstable()
                                    .error("subscribe_num must be an integer")
                                    .emit();
                                return PMTokenStream::default();
                            }
                        }
                    }
                    Ok(syn::Meta::Path(meta_path)) => {
                        if meta_path
                            .get_ident()
                            .map_or(false, |i| i.to_string() == "grant_default_propagate")
                        {
                            parsed_attr = Some(GrantDefaultAttr::GrantDefaultPropagate);
                        }
                    }
                    Err(_) => {
                        attr.span()
                            .unstable()
                            .error("Error parsing attribute")
                            .emit();
                        return PMTokenStream::default();
                    }
                    Ok(syn::Meta::List(_)) => {
                        // Ignore unknown attributes
                    }
                }
            }

            if let Some(grant_default_attr) = parsed_attr {
                grant_default_fields.push((
                    field.ident.clone().unwrap(),
                    field.ty.clone(),
                    grant_default_attr,
                ));
            } else {
                default_fields.push((field.ident.clone().unwrap(), field.ty.clone()));
            }
        }

        // TODO: Sort the allow & subscribe fields
        //
        // Currently we force the user to have the fields pre-sorted
        // in ascending order of the subscribe & allow numbers
        // respectively
        let default_invocations: Vec<TokenStream> = default_fields
            .iter()
            .map(|(ident, ty)| {
                quote! {
                #[allow(clippy::default_trait_access)]
                        let #ident: #ty = Default::default();
                    }
            })
            .collect();

        let grant_default_initializations: Vec<TokenStream> = grant_default_fields
            .iter()
            .map(|(ident, ty, grant_default_attr)| match grant_default_attr {
                GrantDefaultAttr::SubscribeNum(snum) => quote! {
                    let #ident: #ty = cb_factory.build_callback(#snum as u32).unwrap();
                },
                GrantDefaultAttr::AllowNum(anum) => quote! {
                    let #ident: #ty = appslice_factory.build(#anum);
                },
                GrantDefaultAttr::GrantDefaultPropagate => quote! {
                    let #ident: #ty = GrantDefault::grant_default(
                        callback_factory,
                        appslice_factory,
                    );
                },
            })
            .collect();

        let struct_fields: Vec<TokenStream> = default_fields
            .iter()
            .map(|(ident, _)| ident)
            .chain(grant_default_fields.iter().map(|(ident, _, _)| ident))
            .map(|ident| {
                quote! {
                    #ident
                }
            })
            .collect();

        let generated = quote! {
            impl kernel::GrantDefault for #struct_ident {
                fn grant_default(
            _process_id: AppId,
                    cb_factory: &mut kernel::ProcessCallbackFactory,
                    //appslice_factory: &mut AppSliceFactory,
                ) -> Self {
                    #(#default_invocations)*
                    #(#grant_default_initializations)*

                    #struct_ident {
                        #(#struct_fields),*
                    }
                }
            }
        };

        generated.into()
    } else {
        // TODO: Add support for tuple structs and unit structs
        target_struct
            .span()
            .unstable()
            .error("Tuple structs and unit structs are currently not supported.")
            .emit();
        PMTokenStream::default()
    }
}
