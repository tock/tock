use parse::{component, Component};
use quote::quote;

#[derive(Debug)]
#[component(serde, ident = "systick")]
pub struct Systick;

impl Component for Systick {
    fn ty(&self) -> Result<parse::proc_macro2::TokenStream, parse::Error> {
        Ok(quote! {
            cortexm4::systick::SysTick
        })
    }

    fn init_expr(&self) -> Result<parse::proc_macro2::TokenStream, parse::Error> {
        Ok(quote! {
            cortexm4::systick::SysTick::new_with_calibration(64000000)
        })
    }
}
