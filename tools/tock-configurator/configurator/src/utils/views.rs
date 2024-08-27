// Copyright OxidOS Automotive 2024.

use cursive::align::HAlign;
use cursive::view::{Nameable, Resizable};
use cursive::views::{Dialog, LinearLayout, RadioGroup, SelectView, TextView};
use cursive_aligned_view::Alignable;

/// Create a select menu generic over the options.
pub(crate) fn select_menu<T, R, S, F>(items: Vec<(S, T)>, on_submit: F) -> SelectView<T>
where
    T: 'static,
    S: Into<String>,
    F: Fn(&mut cursive::Cursive, &T) -> R + 'static,
{
    let mut select_view = SelectView::new().h_align(HAlign::Left);

    for item in items {
        select_view.add_item(item.0, item.1);
    }

    select_view.set_on_submit(on_submit);

    select_view.with_inactive_highlight(false)
}

/// Create a list of radio buttons with the `None` option (checked).
pub(crate) fn radio_group_with_null<T, F>(items: Vec<T>, on_change: F) -> LinearLayout
where
    T: 'static + std::fmt::Display,
    F: 'static + Fn(&mut cursive::Cursive, &Option<T>),
{
    let mut radio_group: RadioGroup<Option<T>> = RadioGroup::new();
    let mut list = LinearLayout::vertical();

    list.add_child(radio_group.button(None, "None"));

    for item in items {
        let label = format!("{}", item);
        list.add_child(radio_group.button(Some(item), label));
    }

    radio_group.set_on_change(on_change);

    list
}

/// Create a list of radio buttons with the `None` option.
/// This variant has one of the other options checked.
pub(crate) fn radio_group_with_null_known<T, F, U>(
    items: Vec<T>,
    on_change: F,
    known: U,
) -> LinearLayout
where
    T: 'static + std::fmt::Display,
    U: 'static + std::fmt::Display,
    F: 'static + Fn(&mut cursive::Cursive, &Option<T>),
{
    let mut radio_group: RadioGroup<Option<T>> = RadioGroup::new();
    let mut list = LinearLayout::vertical();

    list.add_child(radio_group.button(None, "None"));

    let known_label = format!("{}", known);

    for item in items {
        let label = format!("{}", item);
        if label == known_label {
            list.add_child(radio_group.button(Some(item), label).selected());
        } else {
            list.add_child(radio_group.button(Some(item), label));
        }
    }

    radio_group.set_on_change(on_change);

    list
}

/// Create a list of radio buttons.
/// This variant has one of the other options checked.
pub(crate) fn radio_group_with_known<T, F, I, U>(items: I, on_change: F, known: U) -> LinearLayout
where
    T: 'static + std::fmt::Display,
    U: 'static + std::fmt::Display,
    F: 'static + Fn(&mut cursive::Cursive, &T),
    I: IntoIterator<Item = T>,
{
    let mut radio_group: RadioGroup<T> = RadioGroup::new();
    let mut list = LinearLayout::vertical();

    let known_label = format!("{}", known);

    for item in items {
        let label = format!("{}", item);
        if known_label == label {
            list.add_child(radio_group.button(item, label).selected());
        } else {
            list.add_child(radio_group.button(item, label));
        }
    }

    radio_group.set_on_change(on_change);

    list
}

/// Create a dialog window with a `Quit` button.
pub(crate) fn dialog<
    V: cursive::view::IntoBoxedView + 'static,
    F: 'static + Fn(&mut cursive::Cursive),
    Q: 'static + Fn(&mut cursive::Cursive),
>(
    name: &'static str,
    prompt: &'static str,
    child_view: V,
    exit_cb: Option<F>,
    quit_cb: Option<Q>,
) -> LinearLayout {
    let mut dialog = Dialog::around(child_view);

    if let Some(callback) = quit_cb {
        dialog.add_button("Quit", callback);
    }

    if let Some(callback) = exit_cb {
        dialog.add_button("Back", callback);
    }

    LinearLayout::vertical()
        .child(TextView::new(prompt).align_center())
        .child(dialog.with_name(name).min_height(15))
}

/// The main dialog component that will be reused for multiple layers.
pub(crate) fn main_dialog<
    V: cursive::view::IntoBoxedView + 'static,
    F: 'static + Fn(&mut cursive::Cursive),
    Q: 'static + Fn(&mut cursive::Cursive),
>(
    child_view: V,
    exit_cb: Option<F>,
    quit_cb: Option<Q>,
) -> LinearLayout {
    dialog(
        "main_dialog",
        "Arrow keys navigate the menu. <Enter> selects submenus and submits the forms.",
        child_view,
        exit_cb,
        quit_cb,
    )
}
