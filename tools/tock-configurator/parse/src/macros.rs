// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024
// Copyright OxidOS Automotive SRL 2024
//
// Author: Irina Nita <irina.nita@oxidos.io>
// Author: Darius Jipa <darius.jipa@oxidos.io>

#[macro_export]
macro_rules! static_ident {
    ($ident: tt) => {{
        use $crate::FormatIdent;
        ::once_cell::sync::Lazy::new(|| $ident.to_string() + &::uuid::Uuid::new_v4().format_ident())
    }};
}
