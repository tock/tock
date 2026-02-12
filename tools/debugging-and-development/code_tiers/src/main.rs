use std::env;
use std::fs;
use syn;

const CODE: &'static str = r#"
//! This module contains various silly code that was written by a 4 your old
//! Code Tier: Untrusted

/// Code Tier: Some-middle-tier
struct Bar {

}

/// Foo is a very important method
/// Code Tier: TCB
fn foo() {
}
"#;

fn get_tier(attrs: &Vec<syn::Attribute>) -> Option<String> {
    let mut result = None;
    for att in attrs {
        if let syn::Meta::NameValue(syn::MetaNameValue {
            ref path,
            ref value,
            ..
        }) = att.meta
        {
            if path.segments == syn::parse_quote!(doc) {
                if let syn::Expr::Lit(syn::ExprLit {
                    lit: syn::Lit::Str(v),
                    ..
                }) = value
                {
                    if let Some(("Code Tier", tier)) = v.value().trim().split_once(':') {
                        if result.is_some() {
                            panic!("Only one code tier allowed per item");
                        }
                        result = Some(String::from(tier.trim()));
                    }
                }
            }
        }
    }
    result
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        panic!("must specify paths to check");
    }

    let content = fs::read_to_string(&args[1]).unwrap();
    let ts = syn::parse_file(&content).unwrap();

    if let Some(tier) = get_tier(&ts.attrs) {
        println!("File tier: {tier}");
    }

    for item in ts.items {
        if let Some((ident, tier)) = match item {
            syn::Item::Fn(ref i) => get_tier(&i.attrs).map(|t| (i.sig.ident.clone(), t)),
            syn::Item::Struct(ref i) => get_tier(&i.attrs).map(|t| (i.ident.clone(), t)),
            _ => None,
        } {
            println!("{ident}: {tier}");
        }
    }
}
