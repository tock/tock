// Copyright OxidOS Automotive 2024.

use std::error::Error;
use std::rc::Rc;

use crate::config::{Capsule, Configuration};
use crate::{AlarmDriver, Console, MuxAlarm, MuxUart, RngCapsule, TemperatureCapsule};
use crate::{Chip, Platform, Scheduler};

/// The context provided for Tock's `main` file.
///
/// This should be created from a [`Configuration`], as it's meant to be the glue between
/// the user's agnostic configuration and the Tock's specific internals needed for the code generation
/// process.
pub struct Context<C: Chip> {
    pub platform: Rc<Platform<C>>,
    pub chip: Rc<C>,
    pub process_count: usize,
    pub stack_size: usize,
}

impl<C: Chip> Context<C> {
    pub fn from_config(
        chip: C,
        config: Configuration<C::Peripherals>,
    ) -> Result<Self, Box<dyn Error>> {
        let mut visited = Vec::new();
        let mut capsules = Vec::new();

        // Iterate over the capsules and insert them into the current platform's capsule list.
        for capsule_config in config.capsules() {
            match capsule_config {
                Capsule::Console { uart, baud_rate } => {
                    let mux_uart = MuxUart::insert_get(Rc::clone(uart), *baud_rate, &mut visited);
                    capsules.push(Console::get(mux_uart) as Rc<dyn crate::Capsule>)
                }
                Capsule::Alarm { timer } => {
                    let mux_alarm = MuxAlarm::insert_get(Rc::clone(timer), &mut visited);
                    capsules.push(AlarmDriver::get(mux_alarm) as Rc<dyn crate::Capsule>)
                }
                Capsule::Temperature { temp } => capsules
                    .push(TemperatureCapsule::get(Rc::clone(temp)) as Rc<dyn crate::Capsule>),
                Capsule::Rng { rng } => {
                    capsules.push(RngCapsule::get(Rc::clone(rng)) as Rc<dyn crate::Capsule>)
                }
                _ => {}
            };
        }

        Ok(Self {
            platform: Rc::new(Platform::<C>::new(
                config.r#type,
                capsules,
                Scheduler::insert_get(config.scheduler, &mut visited),
                chip.systick()?,
            )),
            chip: Rc::new(chip),
            process_count: config.process_count,
            stack_size: config.stack_size.into(),
        })
    }
}
