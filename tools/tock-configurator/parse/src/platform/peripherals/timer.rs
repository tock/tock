// Copyright OxidOS Automotive 2024.

use super::NoSupport;
use crate::Component;
use quote::quote;
use std::rc::Rc;

/// A trait that applies to clocks that implement the `Timer`-related traits defined in
/// Tock's HIL.
///
///  TODO: Maybe move to a `Peripheral` trait that implements Component?
pub trait Timer: std::fmt::Debug + PartialEq + std::fmt::Display + Component {
    /// Timer's frequency. Used for providing information in the configuration process.
    fn frequency(&self) -> usize;
}

/// Implementation for the unit type.
impl Timer for NoSupport {
    fn frequency(&self) -> usize {
        0
    }
}

/// Virtual multiplexed alarm. The configurator must resort to this type in case
/// the same alarm may need to be used for multiple capsules/kernel resources.
#[parse_macros::component(curr, ident = "virtual_mux_alarm")]
pub struct VirtualMuxAlarm<T: Timer + 'static> {
    mux_alarm: Rc<MuxAlarm<T>>,
}

impl<T: Timer> VirtualMuxAlarm<T> {
    pub fn mux_alarm(&self) -> Rc<MuxAlarm<T>> {
        self.mux_alarm.clone()
    }
}

impl<T: Timer + 'static> Component for VirtualMuxAlarm<T> {
    fn dependencies(&self) -> Option<Vec<Rc<dyn Component>>> {
        Some(vec![self.mux_alarm.clone()])
    }

    fn ty(&self) -> Result<proc_macro2::TokenStream, crate::Error> {
        let timer_ty = self.mux_alarm.peripheral.ty()?;

        Ok(
            quote!(capsules_core::virtualizers::virtual_alarm::VirtualMuxAlarm<
            'static,
            #timer_ty,
        >),
        )
    }
}

/// Multiplexed alarm. The configurator must resort to this type in case
/// the same alarm may need to be used for multiple capsules/kernel resources.
#[parse_macros::component(curr, ident = "mux_alarm")]
pub struct MuxAlarm<T: Timer + 'static> {
    pub(crate) peripheral: Rc<T>,
}

//  TODO: Remove these clones...
impl<T: Timer> MuxAlarm<T> {
    pub fn timer(&self) -> Rc<T> {
        self.peripheral.clone()
    }
}

impl<T: Timer + 'static> MuxAlarm<T> {
    pub fn insert_get(peripheral: Rc<T>, visited: &mut Vec<Rc<dyn Component>>) -> Rc<Self> {
        for node in visited.iter() {
            if let Ok(mux_alarm) = node.clone().downcast::<MuxAlarm<T>>() {
                if mux_alarm.timer() == peripheral {
                    return mux_alarm;
                }
            }
        }

        let mux_alarm = Rc::new(MuxAlarm::new(peripheral));
        visited.push(mux_alarm.clone() as Rc<dyn Component>);

        mux_alarm
    }
}

impl<T: Timer> crate::Component for MuxAlarm<T> {
    fn ty(&self) -> Result<proc_macro2::TokenStream, crate::Error> {
        let (timer_ident, timer_type): (proc_macro2::TokenStream, _) = (
            self.peripheral.ident()?.parse().unwrap(),
            self.peripheral.ty()?,
        );
        Ok(quote! {
        components::alarm::AlarmMuxComponent::new(#timer_ident)
        .finalize(components::alarm_mux_component_static!(#timer_type))})
    }

    fn init_expr(&self) -> Result<proc_macro2::TokenStream, crate::Error> {
        let timer_type = self.peripheral.ty()?;

        Ok(quote!(components::alarm::AlarmMuxComponent::new(__timer)
            .finalize(components::alarm_mux_component_static!(
                    #timer_type
            ))))
    }

    fn before_init(&self) -> Option<proc_macro2::TokenStream> {
        let timer_ident: proc_macro2::TokenStream =
            self.peripheral.ident().unwrap().parse().unwrap();
        Some(quote! {
            let __timer = &#timer_ident;
        })
    }
}
