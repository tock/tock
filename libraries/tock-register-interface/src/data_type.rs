// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.
// Copyright Google LLC 2024.

use crate::UIntLike;

pub trait ArrayDataType {
    type Element: UIntLike;
    const LEN: usize;
}
impl<U: UIntLike, const LEN: usize> ArrayDataType for [U; LEN] {
    type Element = U;
    const LEN: usize = LEN;
}

//pub trait DataType: private::Sealed {}

//impl<U: UIntLike> private::Sealed for U {}
//impl<U: UIntLike> DataType for U {}

//impl<U: UIntLike, const LEN: usize> private::Sealed for [U; LEN] {}
//impl<U: UIntLike, const LEN: usize> DataType for [U; LEN] {}

//mod private {
//    pub trait Sealed {}
//}
