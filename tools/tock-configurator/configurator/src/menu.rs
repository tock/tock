// Copyright OxidOS Automotive 2024.

use crate::items::ToMenuItem;
use crate::state::{on_count_submit_proc, on_count_submit_stack};
use crate::state::{on_quit_submit, on_scheduler_submit, on_syscall_filter_submit};
use crate::views::main_dialog;
use crate::{items, state, views};

use cursive::theme::ColorStyle;
use cursive::view::{Nameable, Resizable};
use cursive::views::{Checkbox, Dialog, EditView, LinearLayout, ListView, ScrollView, TextView};
use cursive::Cursive;
use cursive_aligned_view::Alignable;

use parse::peripherals::chip::Chip;
use parse::peripherals::{DefaultPeripherals, Gpio};
use parse::scheduler::SchedulerType;
use parse::syscall_filter::SyscallFilterType;

use state::PinFunction;

/// Select menu of supported chips.
pub(crate) fn chip_select() -> cursive::views::SelectView<items::SupportedChip> {
    views::select_menu::<items::SupportedChip, (), String, _>(
        vec![items::ToMenuItem::to_menu_item(
            items::SupportedChip::MicroBit,
        )],
        crate::state::on_chip_submit,
    )
}

/// Menu for configuring the **capsules** the board will implement.
pub(crate) fn capsules_menu<C: Chip + 'static + serde::ser::Serialize>(
) -> cursive::views::ResizedView<cursive::views::LinearLayout> {
    // List of capsules that could be configured for our board.
    views::main_dialog(
        LinearLayout::vertical().child(cursive::views::ScrollView::new(views::select_menu(
            vec![
                items::SupportedCapsule::ALARM.to_menu_item(),
                items::SupportedCapsule::SPI.to_menu_item(),
                items::SupportedCapsule::I2C.to_menu_item(),
                items::SupportedCapsule::BLE.to_menu_item(),
                items::SupportedCapsule::FLASH.to_menu_item(),
                items::SupportedCapsule::LSM303AGR.to_menu_item(),
                items::SupportedCapsule::CONSOLE.to_menu_item(),
                items::SupportedCapsule::TEMPERATURE.to_menu_item(),
                items::SupportedCapsule::RNG.to_menu_item(),
                items::SupportedCapsule::GPIO.to_menu_item(),
            ],
            state::on_capsule_submit::<C>,
        ))),
        Some(state::on_exit_submit::<C>),
        Some(state::on_quit_submit::<C>),
    )
    .full_width()
}

/// Menu for configuring a capsule.
pub(crate) fn capsule_popup<
    C: Chip + 'static + serde::ser::Serialize,
    V: cursive::view::IntoBoxedView + 'static,
>(
    view: V,
) -> cursive::views::LinearLayout {
    views::dialog(
        "capsule",
        "Arrow keys navigate the menu. <Enter> selects submenus.",
        view,
        Some(state::on_exit_submit::<C>),
        Some(state::on_quit_submit::<C>),
    )
}

/// A popup with a checkbox.
pub fn checkbox_popup<
    V: cursive::view::IntoBoxedView + 'static,
    F: 'static + Fn(&mut cursive::Cursive),
    G: 'static + Fn(&mut cursive::Cursive),
>(
    view: V,
    submit_callback: F,
    quit_callback: G,
) -> cursive::views::LinearLayout {
    #![allow(unused)]
    views::dialog(
        "capsule",
        "Arrow keys navigate the menu. <Enter> selects submenus.",
        view,
        Some(submit_callback),
        Some(quit_callback),
    )
}

/// Popup in case of a peripheral not being supported.
pub(crate) fn no_support(peripheral: &'static str) -> cursive::views::TextView {
    TextView::new(format!(
        "The chip does not have support for the {} peripheral.",
        peripheral,
    ))
}

/// Popup in case of a dependency capsule not being configured.
#[allow(unused)]
pub(crate) fn capsule_not_configured(capsule: &'static str) -> cursive::views::TextView {
    TextView::new(format!(
        "This capsule depends on the {} capsule. Please enable it to configure this capsule.",
        capsule,
    ))
}

/// A checkbox list that has disabled entries if they can't be used.
pub(crate) fn pin_list_disabled<C: Chip>(
    pin_list: Vec<(
        <<<C as Chip>::Peripherals as DefaultPeripherals>::Gpio as Gpio>::PinId,
        PinFunction,
    )>,
    gpio_use: PinFunction,
    name: &str,
) -> ScrollView<LinearLayout> {
    let mut list = ListView::new();
    for entry in pin_list {
        if entry.1 == PinFunction::None {
            list.add_child(format!("{:?}", entry.0).as_str(), Checkbox::new());
        } else if entry.1 == gpio_use {
            list.add_child(format!("{:?}", entry.0).as_str(), Checkbox::new().checked());
        } else {
            list.add_child(
                format!("{:?} - used by {:?}", entry.0, entry.1).as_str(),
                Checkbox::new().disabled(),
            );
        }
    }
    ScrollView::new(LinearLayout::vertical().child(list.with_name(name)))
}

/// Menu for configuring the **kernel resources** the board will use.
pub(crate) fn kernel_resources_menu<C: Chip + 'static + serde::ser::Serialize>(
) -> cursive::views::ResizedView<cursive::views::LinearLayout> {
    // List of capsules that could be configured for our board.
    views::main_dialog(
        LinearLayout::vertical().child(views::select_menu(
            vec![items::ToMenuItem::to_menu_item(
                items::KernelResources::Scheduler,
            )],
            state::on_kernel_resource_submit::<C>,
        )),
        Some(state::on_exit_submit::<C>),
        Some(state::on_quit_submit::<C>),
    )
    .full_width()
}

/// Scheduler configuration menu.
pub(crate) fn scheduler_menu<C: Chip + 'static + serde::ser::Serialize>(
    current_scheduler: SchedulerType,
) -> cursive::views::ResizedView<cursive::views::LinearLayout> {
    static SCHEDULERS: [SchedulerType; 2] = [SchedulerType::Cooperative, SchedulerType::RoundRobin];
    views::main_dialog(
        views::radio_group_with_known(SCHEDULERS, on_scheduler_submit::<C>, current_scheduler),
        Some(state::on_exit_submit::<C>),
        Some(state::on_quit_submit::<C>),
    )
    .full_width()
}

/// Syscall filter configuration menu.
pub(crate) fn syscall_filter_menu<C: Chip + 'static + serde::ser::Serialize>(
    current_filter: SyscallFilterType,
) -> cursive::views::ResizedView<cursive::views::LinearLayout> {
    static FILTERS: [SyscallFilterType; 2] = [
        SyscallFilterType::None,
        SyscallFilterType::TbfHeaderFilterDefaultAllow,
    ];
    views::main_dialog(
        views::radio_group_with_known(FILTERS, on_syscall_filter_submit::<C>, current_filter),
        Some(state::on_exit_submit::<C>),
        Some(state::on_quit_submit::<C>),
    )
    .full_width()
}

/// Process count configuration menu.
pub(crate) fn processes_menu<C: Chip + 'static + serde::ser::Serialize>(
    proc_count: usize,
) -> cursive::views::Dialog {
    Dialog::around(
        EditView::new()
            .content(format!("{}", proc_count))
            .on_submit(on_count_submit_proc::<C>)
            .with_name("proc_count"),
    )
    .title("Number of processes")
    .button("Save", |siv| {
        let count = siv
            .call_on_name("proc_count", |view: &mut EditView| view.get_content())
            .unwrap();
        on_count_submit_proc::<C>(siv, &count);
    })
}

/// Stack memory size configuration menu.
pub(crate) fn stack_menu<C: Chip + 'static + serde::ser::Serialize>(
    current_stack_size: usize,
) -> cursive::views::Dialog {
    Dialog::around(
        EditView::new()
            .content(format!("0x{:x}", current_stack_size))
            .on_submit(on_count_submit_stack::<C>)
            .with_name("stack_count"),
    )
    .title("Stack memory size")
    .button("Save", |siv| {
        let count = siv
            .call_on_name("stack_count", |view: &mut EditView| view.get_content())
            .unwrap();
        on_count_submit_stack::<C>(siv, &count);
    })
}

/// Status bar at top.
pub(crate) fn status_bar() -> LinearLayout {
    cursive::views::LinearLayout::vertical()
        .child(
            TextView::new(".config.json - TockOS Kernel Configuration")
                .style(ColorStyle::new(
                    cursive::theme::PaletteColor::Tertiary,
                    cursive::theme::PaletteColor::Background,
                ))
                .with_name("status"),
        )
        .child(
            TextView::new("")
                .style(ColorStyle::new(
                    cursive::theme::PaletteColor::Tertiary,
                    cursive::theme::PaletteColor::Background,
                ))
                .with_name("status"),
        )
}

/// Board configuration menu.
pub(crate) fn board_config_menu<C: Chip + 'static + serde::ser::Serialize>(
) -> cursive::views::ResizedView<cursive::views::LinearLayout> {
    let child_view = views::select_menu(
        vec![
            items::ToMenuItem::to_menu_item(items::ConfigurationField::Capsules),
            items::ToMenuItem::to_menu_item(items::ConfigurationField::KernelResources),
            items::ToMenuItem::to_menu_item(items::ConfigurationField::SysCallFilter),
            items::ToMenuItem::to_menu_item(items::ConfigurationField::Processes),
            items::ToMenuItem::to_menu_item(items::ConfigurationField::StackMem),
        ],
        state::on_config_submit::<C>,
    );
    main_dialog(
        child_view,
        None::<fn(&mut cursive::Cursive)>,
        Some(on_quit_submit::<C>),
    )
    .full_width()
}

/// Build the configurator by adding the layers defined in [`crate::menu::layers`]
/// and initalizing [`crate::menu::builder::CONFIGURATION_BUILDER`].
pub fn init_configurator() -> cursive::CursiveRunnable {
    // Init configurator with the default.
    let mut configurator = cursive::default();

    configurator.set_theme(cursive::theme::Theme::retro());

    // Status bar layer.
    configurator.screen_mut().add_transparent_layer_at(
        cursive::XY::new(
            cursive::view::Offset::Absolute(2),
            cursive::view::Offset::Absolute(0),
        ),
        status_bar(),
    );

    // First layer of the chip select.
    configurator.screen_mut().add_layer(
        views::main_dialog(
            chip_select(),
            None::<fn(&mut cursive::Cursive)>,
            Some(|siv: &mut Cursive| siv.quit()),
        )
        .full_width(),
    );
    configurator
}

/// Menu used for saving the configuration to a JSON file.
pub fn save_dialog<C: parse::peripherals::Chip + 'static + serde::ser::Serialize>(
) -> cursive::views::LinearLayout {
    let child_view = ListView::new().child(
        "Board name: ",
        EditView::new()
            .max_content_width(20)
            .on_submit(state::on_name_submit::<C>)
            .with_name("save_name"),
    );

    let dialog = Dialog::around(child_view)
        .button("Save", state::on_save_submit::<C>)
        .button("Quit without saving", |siv| siv.quit());

    LinearLayout::vertical()
        .child(TextView::new("Set a board name before saving the configuration").align_center())
        .child(dialog.min_height(15))
}
