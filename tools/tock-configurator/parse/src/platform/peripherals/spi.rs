// Copyright OxidOS Automotive 2024.

use std::rc::Rc;

use super::NoSupport;
use crate::{component, Component};

pub trait Spi: crate::Component + std::fmt::Debug + std::fmt::Display {}

///  TODO: Doc this also.
#[component(curr, ident = "mux_spi")]
pub struct MuxSpi<S: Spi> {
    pub(crate) _peripheral: Rc<S>,
}

impl<S: Spi + 'static> Component for MuxSpi<S> {}

impl Spi for NoSupport {}
