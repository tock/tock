// Copyright OxidOS Automotive 2024.

use super::NoSupport;
use crate::{component, Component};
use std::rc::Rc;

pub trait I2c: crate::Component + std::fmt::Debug + std::fmt::Display {}

// TODO: Doc this.
#[component(curr, ident = "mux_i2c")]
pub(crate) struct MuxI2c<I: I2c> {
    pub(crate) _peripheral: Rc<I>,
}

impl<I: I2c + 'static> MuxI2c<I> {
    #![allow(unused)]
    pub(crate) fn insert_get(_spi: Rc<I>, _visited: &mut [Rc<dyn Component>]) -> Rc<Self> {
        unimplemented!()
    }
}
impl<I: I2c + 'static> Component for MuxI2c<I> {}

impl I2c for NoSupport {}
