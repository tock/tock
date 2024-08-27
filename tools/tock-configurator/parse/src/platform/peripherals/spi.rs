// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024
// Copyright OxidOS Automotive SRL 2024
//
// Author: Irina Nita <irina.nita@oxidos.io>
// Author: Darius Jipa <darius.jipa@oxidos.io>

use std::rc::Rc;

use super::NoSupport;
use crate::{component, Component};

pub trait Spi: crate::Component + std::fmt::Debug + std::fmt::Display {}

#[component(curr, ident = "mux_spi")]
pub struct MuxSpi<S: Spi> {
    pub(crate) _peripheral: Rc<S>,
}

impl<S: Spi + 'static> Component for MuxSpi<S> {}

impl Spi for NoSupport {}
