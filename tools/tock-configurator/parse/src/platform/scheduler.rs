// Copyright OxidOS Automotive 2024.

use crate::Component;
use quote::quote;
use std::rc::Rc;

/// The types of schedulers supported by Tock.
#[derive(serde::Serialize, serde::Deserialize, Clone, Copy, Default)]
pub enum SchedulerType {
    #[default]
    Cooperative,
    RoundRobin,
}

impl std::fmt::Display for SchedulerType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SchedulerType::RoundRobin => write!(f, "RoundRobin"),
            SchedulerType::Cooperative => write!(f, "Cooperative"),
        }
    }
}

#[parse_macros::component(serde, curr, ident = "scheduler")]
pub struct Scheduler {
    pub r#type: SchedulerType,
}

impl Scheduler {
    pub fn insert_get(ty: SchedulerType, visited: &mut Vec<Rc<dyn Component>>) -> Rc<Self> {
        let scheduler = Rc::new(Scheduler::new(ty));
        visited.push(scheduler.clone() as Rc<dyn Component>);

        scheduler
    }
}

impl crate::Component for Scheduler {
    fn ty(&self) -> Result<proc_macro2::TokenStream, crate::Error> {
        let ty = match self.r#type {
            SchedulerType::RoundRobin => {
                quote!(kernel::scheduler::round_robin::RoundRobinSched<'static>)
            }
            SchedulerType::Cooperative => {
                quote!(kernel::scheduler::cooperative::CooperativeSched<'static>)
            }
        };

        Ok(ty)
    }

    fn init_expr(&self) -> Result<proc_macro2::TokenStream, crate::Error> {
        let init_expr = match self.r#type {
            SchedulerType::Cooperative => quote! {
            components::sched::cooperative::CooperativeComponent::new(&*core::ptr::addr_of!(PROCESSES))
                .finalize(components::cooperative_component_static!(NUM_PROCS))
            },
            SchedulerType::RoundRobin => quote! {
                            components::sched::round_robin::RoundRobinComponent::new(&*core::ptr::addr_of!(PROCESSES))
            .finalize(components::round_robin_component_static!(NUM_PROCS))
            },
        };

        Ok(init_expr)
    }
}
