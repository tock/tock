// Copyright OxidOS Automotive 2024.

use quote::{format_ident, quote};

use crate::{platform::timer, Capsule, Ident};
use std::rc::Rc;

/// The [`AlarmDriver`] capsule can be configured through the alarm
/// used by the capsule. Could be either a raw timer or a virtual one that wraps it.
#[parse_macros::component(curr, ident = "alarm")]
pub struct AlarmDriver<T: timer::Timer + 'static> {
    pub(crate) mux_alarm: Rc<timer::MuxAlarm<T>>,
}

impl<T: timer::Timer + 'static> AlarmDriver<T> {
    pub fn get(mux_alarm: Rc<timer::MuxAlarm<T>>) -> Rc<Self> {
        Rc::new(Self::new(mux_alarm))
    }
}

impl<T: timer::Timer> crate::Component for AlarmDriver<T> {
    fn ty(&self) -> Result<proc_macro2::TokenStream, crate::Error> {
        Ok(quote! { capsules_core::alarm::AlarmDriver<
              'static,
              capsules_core::virtualizers::virtual_alarm::VirtualMuxAlarm<
                  'static,
                  nrf52::rtc::Rtc<'static>>>
        })
    }

    fn init_expr(&self) -> Result<proc_macro2::TokenStream, crate::Error> {
        let mux_alarm = format_ident!("{}", self.mux_alarm.as_ref().ident()?);
        let driver_num = self.driver_num();
        let timer_ty = self.mux_alarm.peripheral.as_ref().ty()?;

        Ok(quote! {
            components::alarm::AlarmDriverComponent::new(
                board_kernel,
                #driver_num,
                #mux_alarm,
            ).finalize(components::alarm_component_static!(#timer_ty));
        })
    }

    fn dependencies(&self) -> Option<Vec<Rc<dyn crate::Component>>> {
        Some(vec![self.mux_alarm.clone()])
    }
}

impl<T: timer::Timer + 'static> crate::Capsule for AlarmDriver<T> {
    fn driver_num(&self) -> proc_macro2::TokenStream {
        quote!(capsules_core::alarm::DRIVER_NUM)
    }
}
