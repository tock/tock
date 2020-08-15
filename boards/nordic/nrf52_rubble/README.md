# Support for using the Rubble stack on nRF52* boards

This repository contains a small amount of glue code to allow loading the
`tock_rubble` integration crate as a provider for the `kernel::hil::rubble`
interface, enabling the `rubble` capsule.

This is a separate repository so that it can be easily disabled without using
feature flags, and to keep `rubble` out of the dependency tree for Tock unless
it's currently being used.

To enable the `rubble` capsule, you'll need to add it to the `Platform`
of the board you're using in that board crate's `main.rs`, and remove
`capsules::ble_advertising_driver::BLE` if it's present, as it and Rubble
both try to control the same BLE Radio driver.

This requires four changes.

First, add `nrf52_rubble` as a dependency of your `nrf52*` board crate. In
`Cargo.toml`, add:

```toml
[dependencies]
# ...

nrf52_rubble = { path = "../nrf52_rubble" }
```

Second, add the rubble capsule as a field of `Platform` like so:

```rust
pub struct Platform {
    // ...
    ble_radio: &'static capsules::rubble::BLE<
        'static,
        VirtualMuxAlarm<'static, nrf52840::rtc::Rtc<'static>>,
        nrf52_rubble::Nrf52RubbleImplementation<
            'static,
            VirtualMuxAlarm<'static, nrf52840::rtc::Rtc<'static>>,
        >,
    >,
    // ...
```

Second, add it under its known driver number in the `match` statement in the
`Platform::with_driver` function:

```rust
impl kernel::Platform for Platform {
    fn with_driver<F, R>(&self, driver_num: usize, f: F) -> R
    where
        F: FnOnce(Option<&dyn kernel::Driver>) -> R,
    {
        match driver_num {
            // ...
            capsules::rubble::DRIVER_NUM => f(Some(self.ble_radio)),
            // ...
        }
    }
}
```

Finally, add code to initialize the rubble driver to the `reset_handler` function:

```rust

#[no_mangle]
pub unsafe fn reset_handler() {
    // ...

    let ble_radio =
        nrf52_rubble::RubbleComponent::new(board_kernel, &nrf52840::ble_radio::RADIO, mux_alarm)
            .finalize(());

    // ...

    let platform = Platform {
        // ...
        ble_radio,
        // ...
    };

    // ...
}
```

In each of these places, be sure to remove or replace the
`capsules::ble_advertising_driver::BLE` capsule if it's present.
