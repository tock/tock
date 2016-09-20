BLE Serialization App
=====================

This app makes it possible to provide a BLE interface on the Sam4l.
It does this by using Nordic's BLE Serialization format where BLE commands
can be communicated over UART to the nRF51822 radio. This serialization
protocol allows for otherwise normal nRF51822 BLE apps to be executed on
a different microcontroller. All function calls that would have hit the BLE
stack are instead shuttled over UART to the nRF51822 radio.


Usage
-----

1. Flash the
[Firestorm BLE serialization app](https://github.com/helena-project/storm-ble/tree/master/nrf51822/apps/firestorm-ble-serialization-uart-peripheral)
to the nRF51822.

2. Make sure the submodule in the `ble_example` application is up to date.

3. Install this app on Storm.




TODO
----

1. Add ability to set BLE address.
2. Implement app_timer for the Nordic stack.
3. Verify that services/connections work after adding timers.
