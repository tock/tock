use core::convert::TryFrom;
use proc_macro::TokenStream as PMTokenStream;
use proc_macro2::{Ident, TokenStream};
use syn::{Expr, Member};

use crate::AccessType;
use crate::IntegerWidth;

pub(crate) struct LiteXRegisterAbstractionArguments {
    pub name: Ident,
    pub access_type: AccessType,
    pub value_width: IntegerWidth,
    pub wishbone_data_width: IntegerWidth,
    pub base_width: IntegerWidth,
}

fn parse_usize_expr(expr: &Expr) -> Result<usize, TokenStream> {
    use syn::spanned::Spanned;

    if let syn::Expr::Lit(expr_lit) = expr {
        if let syn::Lit::Int(lit_int) = &expr_lit.lit {
            match lit_int.base10_parse::<usize>() {
                Ok(val) => Ok(val),
                Err(err) => {
                    expr.span()
                        .unstable()
                        .error(format!("Can't parse expression to usize: {}", err))
                        .emit();
                    Err(TokenStream::default())
                }
            }
        } else {
            expr.span()
                .unstable()
                .error("Expression is not a literal usize");
            Err(TokenStream::default())
        }
    } else {
        expr.span()
            .unstable()
            .error("Expression is not a literal usize");
        Err(TokenStream::default())
    }
}

fn parse_intwidth_expr(expr: &Expr) -> Result<IntegerWidth, TokenStream> {
    use syn::spanned::Spanned;

    let parsed_usize = parse_usize_expr(expr)?;

    if let Ok(intwidth) = IntegerWidth::try_from(parsed_usize) {
        Ok(intwidth)
    } else {
        expr.span()
            .unstable()
            .error(format!(
                "{} is not a valid register integer width",
                parsed_usize
            ))
            .emit();
        Err(TokenStream::default())
    }
}

pub(crate) fn litex_register_abstraction_parse_arguments(
    input: PMTokenStream,
) -> Result<LiteXRegisterAbstractionArguments, PMTokenStream> {
    use syn::spanned::Spanned;

    let param_struct: syn::ExprStruct = match syn::parse(input) {
        Ok(params) => params,
        Err(err) => return Err(err.to_compile_error().into()),
    };

    let name: Ident = if let Some(ident) = param_struct.path.get_ident() {
        ident.clone()
    } else {
        param_struct
            .path
            .span()
            .unstable()
            .error("A LiteX register abstraction struct cannot have a path-identifier")
            .emit();
        return Err(PMTokenStream::default());
    };

    let fields: Result<Vec<(Ident, Expr)>, Member> = param_struct
        .fields
        .iter()
        .map(|fieldvalue| {
            (if let Member::Named(ident) = &fieldvalue.member {
                Ok(ident.clone())
            } else {
                Err(fieldvalue.member.clone())
            })
            .map(|ident| (ident, fieldvalue.expr.clone()))
        })
        .collect();
    let fields = match fields {
        Ok(f) => f,
        Err(fv) => {
            fv.span().unstable().error("Member must be named").emit();
            return Err(PMTokenStream::default());
        }
    };

    // Parse the inidividual fields
    let mut access_type = None;
    let mut value_width = None;
    let mut wishbone_data_width = None;
    let mut base_width = None;
    let mut endianess = None;

    for (ident, expr) in fields.iter() {
        match ident.to_string().as_str() {
            "value_width" => {
                value_width = Some(parse_intwidth_expr(expr)?);
            }
            "wishbone_data_width" => {
                wishbone_data_width = Some(parse_intwidth_expr(expr)?);
            }
            "base_width" => {
                base_width = Some(parse_intwidth_expr(expr)?);
            }
            "access_type" => {
                let parsed_at = if let syn::Expr::Lit(expr_lit) = expr {
                    if let syn::Lit::Str(lit_str) = &expr_lit.lit {
                        match lit_str.value().as_str() {
                            "read_only" => Ok(AccessType::ReadOnly),
                            "write_only" => Ok(AccessType::WriteOnly),
                            "read_write" => Ok(AccessType::ReadWrite),
                            _ => {
                                expr.span().unstable().error("access_type not one of \"read_only\", \"write_only\", \"read_write\"").emit();
                                Err(TokenStream::default())
                            }
                        }
                    } else {
                        expr.span()
                            .unstable()
                            .error("Expression is not a literal string");
                        Err(TokenStream::default())
                    }
                } else {
                    expr.span()
                        .unstable()
                        .error("Expression is not a literal string");
                    Err(TokenStream::default())
                };

                access_type = Some(parsed_at?);
            }
            "endianess" => {
                let parsed_end = if let syn::Expr::Lit(expr_lit) = expr {
                    if let syn::Lit::Str(lit_str) = &expr_lit.lit {
                        match lit_str.value().as_str() {
                            "big" => Ok("big"),
                            "little" => Ok("little"),
                            _ => {
                                expr.span()
                                    .unstable()
                                    .error("access_type not one of \"big\", \"little\"")
                                    .emit();
                                Err(TokenStream::default())
                            }
                        }
                    } else {
                        expr.span()
                            .unstable()
                            .error("Expression is not a literal string");
                        Err(TokenStream::default())
                    }
                } else {
                    expr.span()
                        .unstable()
                        .error("Expression is not a literal string");
                    Err(TokenStream::default())
                };

                endianess = Some(parsed_end?);
            }
            _ => {
                ident
                    .span()
                    .unstable()
                    .error(format!("Unknown key \"{}\"", ident.to_string()))
                    .emit();
                return Err(PMTokenStream::default());
            }
        }
    }

    if let Some(e) = endianess {
        if e != "big" {
            param_struct
                .span()
                .unstable()
                .error("Litte-endian registers are currently not supported")
                .emit();
            return Err(PMTokenStream::default());
        }
    } else {
        param_struct
            .span()
            .unstable()
            .error("Missing parameter \"endianess\"")
            .emit();
        return Err(PMTokenStream::default());
    };

    let access_type = if let Some(v) = access_type {
        v
    } else {
        param_struct
            .span()
            .unstable()
            .error("Missing parameter \"access_type\"")
            .emit();
        return Err(PMTokenStream::default());
    };
    let value_width = if let Some(v) = value_width {
        v
    } else {
        param_struct
            .span()
            .unstable()
            .error("Missing parameter \"value_width\"")
            .emit();
        return Err(PMTokenStream::default());
    };
    let wishbone_data_width = if let Some(v) = wishbone_data_width {
        v
    } else {
        param_struct
            .span()
            .unstable()
            .error("Missing parameter \"wishbone_data_width\"")
            .emit();
        return Err(PMTokenStream::default());
    };
    let base_width = if let Some(v) = base_width {
        v
    } else {
        param_struct
            .span()
            .unstable()
            .error("Missing parameter \"base_width\"")
            .emit();
        return Err(PMTokenStream::default());
    };

    Ok(LiteXRegisterAbstractionArguments {
        name,
        access_type,
        value_width,
        wishbone_data_width,
        base_width,
    })
}
