// Copyright OxidOS Automotive 2024.

#[macro_export]
macro_rules! static_ident {
    ($ident: tt) => {{
        use $crate::FormatIdent;
        ::once_cell::sync::Lazy::new(|| $ident.to_string() + &::uuid::Uuid::new_v4().format_ident())
    }};
}
