The Configurator crate
======================

The `configurator/` crate contains the TUI (Terminal User Interface) menu used for visually configuring a platform.

This part of the configuration process is meant to be as agnostic as possible to the Tock-specific implementations.

The application saves the configuration into a JSON file named `.config.json`.

The TUI library that was chosen for the configurator is [`cursive`](https://github.com/gyscos/cursive) because it has great Linux compatibility and flexibility.

## Current status

The menu items are currently: 
- capsules (configuration menus for the Tock capsules)
- kernel resources (configuration menus for the resources of the Tock kernel)
- syscall filter (configuration menu to choose whether to use a syscall filter or not)
- processes (configuration menu for the number of processes)
- stack memory (configuration menu for the stack memory size)

## File structure

- `main.rs`: entry point of the configurator. It starts the TUI.
- `lib.rs`: exposes the modules.
- `menu.rs`: provides general (as in not for capsules) menus to be used in the configuration of Tock.
- `state.rs`: has the functions that handle the internal state of the configurator.
- The `capsule` module: contains the configuration menus and logic for each Tock capsule.
- The `utils` module: contains different macros and items used for the TUI.

