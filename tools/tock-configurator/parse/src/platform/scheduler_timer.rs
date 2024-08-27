// Copyright OxidOS Automotive 2024.

use super::peripherals::timer;
use crate::Component;
use std::rc::Rc;

#[parse_macros::component(curr, ident = "scheduler_timer")]
pub struct SchedulerTimer<T: timer::Timer + 'static> {
    virtual_mux_alarm: Rc<timer::VirtualMuxAlarm<T>>,
}

impl<T: timer::Timer + 'static> SchedulerTimer<T> {
    pub fn get(virtual_mux_alarm: Rc<timer::VirtualMuxAlarm<T>>) -> Rc<Self> {
        Rc::new(Self::new(virtual_mux_alarm))
    }
}

impl<T: timer::Timer + 'static> Component for SchedulerTimer<T> {
    fn dependencies(&self) -> Option<Vec<Rc<dyn Component>>> {
        Some(vec![self.virtual_mux_alarm.clone()])
    }
}

impl<T: timer::Timer + 'static> SchedulerTimer<T> {
    pub fn virtual_mux_alarm(&self) -> Rc<timer::VirtualMuxAlarm<T>> {
        self.virtual_mux_alarm.clone()
    }
}
