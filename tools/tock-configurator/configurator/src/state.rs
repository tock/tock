// Copyright OxidOS Automotive 2024.

use parse::config;
use parse::peripherals::chip::{Chip, DefaultPeripherals};
use parse::peripherals::Gpio;
use parse::scheduler::SchedulerType;
use parse::syscall_filter::SyscallFilterType;

use std::fs::File;
use std::io::Write;
use std::num::NonZeroUsize;
use std::rc::Rc;

use cursive::views::EditView;

use crate::capsule::ConfigMenu;
use crate::items;
use crate::menu::{self, board_config_menu, capsules_menu, kernel_resources_menu};
use crate::menu::{processes_menu, scheduler_menu, stack_menu, syscall_filter_menu};

pub(crate) type ViewStack = Vec<Box<dyn cursive::View>>;
pub(crate) type GpioMap<C> = Vec<(
    <<<C as Chip>::Peripherals as DefaultPeripherals>::Gpio as Gpio>::PinId,
    PinFunction,
)>;

#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(unused)]
pub enum PinFunction {
    None,
    Led,
    Button,
    Gpio,
}

#[derive(Debug)]
pub struct GpioHelper<C: Chip> {
    pub gpio: Rc<<C::Peripherals as DefaultPeripherals>::Gpio>,
    pub pins: GpioMap<C>,
}

impl<C: Chip> GpioHelper<C> {
    pub(crate) fn new(gpio: Rc<<C::Peripherals as DefaultPeripherals>::Gpio>) -> Self {
        let pins = gpio.pins().unwrap();
        Self {
            gpio,
            pins: pins.iter().map(|pin| (*pin, PinFunction::None)).collect(),
        }
    }

    pub fn pins(&self) -> &GpioMap<C> {
        &self.pins
    }
}

/// Inner data to be kept by Cursive.
pub(crate) struct Data<C: Chip> {
    /// The platform configuration.
    pub(crate) platform: parse::Configuration<C::Peripherals>,

    /// The chip that the platform configuration is based on.
    pub(crate) chip: Rc<C>,

    /// The view stack.
    views: ViewStack,

    /// List of pins with their usage.
    pub gpio_list: Option<Vec<GpioHelper<C>>>,
}

impl<C: Chip> Data<C> {
    pub(crate) fn new(chip: C) -> Data<C> {
        let peripherals = Rc::clone(&chip.peripherals());
        Self {
            platform: parse::Configuration::default(),
            chip: Rc::new(chip),
            views: ViewStack::new(),
            gpio_list: peripherals.gpio().ok().map(|list| {
                list.iter()
                    .map(|gpio| GpioHelper::new(Rc::clone(gpio)))
                    .collect()
            }),
        }
    }

    /// Add a view to the view stack.
    pub(crate) fn push_view(&mut self, view: Box<dyn cursive::View>) {
        self.views.push(view)
    }

    /// Pop view from the view stack.
    pub(crate) fn pop_view(&mut self) -> Option<Box<dyn cursive::View>> {
        self.views.pop()
    }

    /// Take the port and returns the helper struct for it.
    pub fn gpio(
        &self,
        gpio: &<<C as Chip>::Peripherals as DefaultPeripherals>::Gpio,
    ) -> Option<&GpioHelper<C>> {
        //  FIXME: The match is unnecessary.
        match self.gpio_list.as_ref() {
            Some(gpio_list) => {
                for helper in gpio_list.iter() {
                    if helper.gpio.as_ref() == gpio {
                        return Some(helper);
                    }
                }
                None
            }
            None => None,
        }
    }

    /// Change the pin status that is stored inside the configurator
    /// inner state.
    pub fn change_pin_status(
        &mut self,
        gpio: Rc<<<C as Chip>::Peripherals as DefaultPeripherals>::Gpio>,
        searched_pin: <<<C as Chip>::Peripherals as DefaultPeripherals>::Gpio as Gpio>::PinId,
        status: PinFunction,
    ) {
        let _ = self.gpio_list.as_mut().map(|gpio_list| {
            let gpio_list = gpio_list.iter_mut().filter(|helper| helper.gpio == gpio);
            gpio_list.for_each(|helper| {
                for pin in helper.pins.iter_mut() {
                    // If the searched pin was found, change its status and exit
                    // the loop.
                    if pin.0 == searched_pin {
                        pin.1 = status;
                        break;
                    }
                }
            });
        });
    }
}

/// Push a layer to the view stack.
pub(crate) fn push_layer<
    V: cursive::view::IntoBoxedView + 'static,
    C: Chip + 'static + serde::ser::Serialize,
>(
    siv: &mut cursive::Cursive,
    layer: V,
) {
    if let Some(old_layer) = siv.pop_layer() {
        // Update user data.
        if let Some(data) = siv.user_data::<Data<C>>() {
            data.push_view(old_layer);
        }
    }

    siv.add_layer(layer);
}

/// Initialize a board configuration session based on the submitted chip.
pub(crate) fn on_chip_submit(siv: &mut cursive::Cursive, submit: &items::SupportedChip) {
    match submit {
        items::SupportedChip::MicroBit => {
            // Initial user data.
            siv.set_user_data::<Data<nrf52833::Chip>>(Data::new(nrf52833::Chip::new()));

            push_layer::<_, nrf52833::Chip>(siv, board_config_menu::<nrf52833::Chip>());
        }
    };
}

/// Initialize a board configuration session based on the submitted chip.
pub(crate) fn on_scheduler_submit<C: Chip + 'static + serde::ser::Serialize>(
    siv: &mut cursive::Cursive,
    submit: &SchedulerType,
) {
    if let Some(data) = siv.user_data::<Data<C>>() {
        data.platform.update_scheduler(*submit);
    }
}

pub(crate) fn on_syscall_filter_submit<C: Chip + 'static + serde::ser::Serialize>(
    siv: &mut cursive::Cursive,
    submit: &SyscallFilterType,
) {
    if let Some(data) = siv.user_data::<Data<C>>() {
        data.platform.update_syscall_filter(*submit);
    }
}

/// Open a new configuration window based on the submitted config field.
pub(crate) fn on_config_submit<C: Chip + 'static + serde::ser::Serialize>(
    siv: &mut cursive::Cursive,
    submit: &items::ConfigurationField,
) {
    // For each one, we need to add a layer.
    if let Some(data) = siv.user_data::<Data<C>>() {
        match submit {
            items::ConfigurationField::Capsules => push_layer::<_, C>(siv, capsules_menu::<C>()),
            items::ConfigurationField::KernelResources => {
                push_layer::<_, C>(siv, kernel_resources_menu::<C>())
            }
            items::ConfigurationField::Processes => {
                let process_count = data.platform.process_count;
                push_layer::<_, C>(siv, processes_menu::<C>(process_count))
            }
            items::ConfigurationField::StackMem => {
                let stack_size: usize = data.platform.stack_size.into();
                push_layer::<_, C>(siv, stack_menu::<C>(stack_size))
            }
            items::ConfigurationField::SysCallFilter => {
                let syscall_filter = data.platform.syscall_filter;
                push_layer::<_, C>(siv, syscall_filter_menu::<C>(syscall_filter))
            }
        }
    };
}

/// Open the corresponding config window based on the submitted kernel resource.
pub(crate) fn on_kernel_resource_submit<C: Chip + 'static + serde::ser::Serialize>(
    siv: &mut cursive::Cursive,
    submit: &items::KernelResources,
) {
    // For each one, we need to add a layer.
    if let Some(data) = siv.user_data::<Data<C>>() {
        match submit {
            items::KernelResources::Scheduler => {
                let scheduler_type = data.platform.scheduler;
                push_layer::<_, C>(siv, scheduler_menu::<C>(scheduler_type));
            } // This will have multiple variants as the support grows.
        }
    }
}

/// Open the corresponding config window based on the submitted capsule.
pub(crate) fn on_capsule_submit<C: Chip + 'static + serde::ser::Serialize>(
    siv: &mut cursive::Cursive,
    submit: &items::SupportedCapsule,
) {
    let data = siv.user_data::<Data<C>>().unwrap();
    let chip = Rc::clone(&data.chip);

    match submit {
        config::Index::CONSOLE => {
            let previous_state = match data.platform.capsule(submit) {
                Some(config::Capsule::Console { uart, baud_rate }) => {
                    Some((Rc::clone(uart), *baud_rate))
                }
                _ => None,
            };

            push_layer::<_, C>(
                siv,
                crate::capsule::console::config::<C>(chip, previous_state),
            )
        }
        config::Index::ALARM => {
            let choice = match data.platform.capsule(&config::Index::ALARM) {
                Some(config::Capsule::Alarm { timer }) => Some(Rc::clone(timer)),
                _ => None,
            };

            push_layer::<_, C>(siv, crate::capsule::alarm::config::<C>(chip, choice))
        }
        config::Index::SPI => {
            let choice = match data.platform.capsule(&config::Index::SPI) {
                Some(config::Capsule::Spi { spi }) => Some(Rc::clone(spi)),
                _ => None,
            };

            push_layer::<_, C>(siv, crate::capsule::spi::config::<C>(chip, choice))
        }
        config::Index::I2C => {
            let choice = match data.platform.capsule(&config::Index::I2C) {
                Some(config::Capsule::I2c { i2c }) => Some(Rc::clone(i2c)),
                _ => None,
            };

            push_layer::<_, C>(siv, crate::capsule::i2c::config::<C>(chip, choice))
        }
        config::Index::BLE => {
            let choice = match data.platform.capsule(&config::Index::BLE) {
                Some(config::Capsule::BleRadio { ble, timer }) => {
                    Some((Rc::clone(timer), Rc::clone(ble)))
                }
                _ => None,
            };

            push_layer::<_, C>(siv, crate::capsule::ble::config::<C>(chip, choice))
        }
        config::Index::FLASH => {
            let choice = match data.platform.capsule(submit) {
                Some(config::Capsule::Flash { flash, buffer_size }) => {
                    Some((Rc::clone(flash), *buffer_size))
                }
                _ => None,
            };

            push_layer::<_, C>(siv, crate::capsule::flash::config::<C>(choice, chip))
        }
        config::Index::LSM303AGR => {
            let previous_state = match data.platform.capsule(submit) {
                Some(config::Capsule::Lsm303agr { i2c, .. }) => Some(Rc::clone(i2c)),
                _ => None,
            };

            siv.pop_layer();
            siv.add_layer(crate::capsule::lsm303agr::config::<C>(chip, previous_state));
        }
        config::Index::TEMPERATURE => {
            let choice = match data.platform.capsule(&config::Index::TEMPERATURE) {
                Some(config::Capsule::Temperature { temp }) => Some(Rc::clone(temp)),
                _ => None,
            };

            push_layer::<_, C>(siv, crate::capsule::temperature::config::<C>(chip, choice))
        }
        config::Index::RNG => {
            let choice = match data.platform.capsule(&config::Index::RNG) {
                Some(config::Capsule::Rng { rng }) => Some(Rc::clone(rng)),
                _ => None,
            };

            push_layer::<_, C>(siv, crate::capsule::rng::config::<C>(chip, choice))
        }
        config::Index::GPIO => {
            push_layer::<_, C>(siv, crate::capsule::gpio::GpioConfig::config(chip))
        }
        _ => unreachable!(),
    }
}

/// Give the next prompt from the GPIO capsule.
#[allow(unused)]
pub(crate) fn on_gpio_submit<
    C: Chip + 'static + serde::Serialize,
    F: 'static
        + Fn(
            Rc<<<C as Chip>::Peripherals as DefaultPeripherals>::Gpio>,
        ) -> cursive::views::LinearLayout,
>(
    siv: &mut cursive::Cursive,
    submit: Rc<<<C as Chip>::Peripherals as DefaultPeripherals>::Gpio>,
    popup: F,
) {
    siv.pop_layer();
    siv.add_layer(popup(submit));
}

/// Exit the current window and go back to the previous one.
pub(crate) fn on_exit_submit<C: Chip + 'static + serde::ser::Serialize>(
    siv: &mut cursive::Cursive,
) {
    // Pop the current layer.
    siv.pop_layer();
    // Go to the back layer.
    if let Some(data) = siv.user_data::<Data<C>>() {
        if let Some(old_layer) = data.pop_view() {
            siv.add_layer(old_layer);
        }
    }
}

/// Exit the current window and go to the "save to JSON" menu.
pub(crate) fn on_quit_submit<C: Chip + 'static + serde::ser::Serialize>(
    siv: &mut cursive::Cursive,
) {
    siv.pop_layer();
    siv.add_layer(menu::save_dialog::<C>())
}

/// Write to the JSON file and quit the configurator.
pub(crate) fn on_name_submit<C: Chip + 'static + serde::Serialize>(
    siv: &mut cursive::Cursive,
    name: &str,
) {
    if let Some(data) = siv.user_data::<Data<C>>() {
        if !name.is_empty() {
            data.platform.update_type(name)
        }
        write_json(data);
    }
    siv.quit();
}

/// Save the process count to use in the JSON.
pub(crate) fn on_count_submit_proc<C: Chip + 'static + serde::Serialize>(
    siv: &mut cursive::Cursive,
    name: &str,
) {
    if let Some(data) = siv.user_data::<Data<C>>() {
        if name.is_empty() {
            data.platform.process_count = 4_usize;
        } else if let Ok(count) = name.parse::<usize>() {
            data.platform.process_count = count;
        }

        siv.pop_layer();
        siv.add_layer(board_config_menu::<C>());
    }
}

/// Save the stack memory size to use in the JSON.
pub(crate) fn on_count_submit_stack<C: Chip + 'static + serde::Serialize>(
    siv: &mut cursive::Cursive,
    name: &str,
) {
    if let Some(data) = siv.user_data::<Data<C>>() {
        if name.is_empty() {
            // TODO: Safety comment
            unsafe {
                data.platform.stack_size = NonZeroUsize::new_unchecked(0x900_usize);
            }
        } else if let Some(number) = name.strip_prefix("0x") {
            if let Ok(count) = usize::from_str_radix(number, 16) {
                data.platform.update_stack_size(count);
            }
        } else if let Ok(count) = name.parse::<usize>() {
            data.platform.update_stack_size(count);
        }

        siv.pop_layer();
        siv.add_layer(board_config_menu::<C>());
    }
}

/// Write the contents of the inner Data to a JSON file
pub(crate) fn write_json<C: Chip + 'static + serde::ser::Serialize>(data: &mut Data<C>) {
    let board_config = serde_json::to_string_pretty(&data.platform).unwrap();
    let mut file = File::create(".config.json").unwrap();
    file.write_all(board_config.as_bytes()).unwrap();
}

/// Exit the current window and go back to the previous one.
pub(crate) fn on_save_submit<C: Chip + 'static + serde::ser::Serialize>(
    siv: &mut cursive::Cursive,
) {
    let ident = siv
        .call_on_name("save_name", |view: &mut EditView| view.get_content())
        .unwrap();
    on_name_submit::<C>(siv, &ident);
}
