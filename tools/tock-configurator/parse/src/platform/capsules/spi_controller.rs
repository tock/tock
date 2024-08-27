// Copyright OxidOS Automotive 2024.

use crate::{component, spi, Capsule, Component};
use std::rc::Rc;

///  TODO: Doc this also.
#[component(curr, ident = "spi_controller")]
pub struct SpiController<S: spi::Spi> {
    pub inner: Rc<spi::MuxSpi<S>>,
}

impl<S: spi::Spi + 'static> Component for SpiController<S> {
    fn ty(&self) -> Result<proc_macro2::TokenStream, crate::Error> {
        Err(crate::Error::CodeNotProvided)
    }

    fn init_expr(&self) -> Result<proc_macro2::TokenStream, crate::Error> {
        Err(crate::Error::CodeNotProvided)
    }

    fn dependencies(&self) -> Option<Vec<Rc<dyn Component>>> {
        Some(vec![self.inner.clone()])
    }
}

impl<S: spi::Spi + 'static> Capsule for SpiController<S> {
    fn driver_num(&self) -> proc_macro2::TokenStream {
        todo!()
    }
}
