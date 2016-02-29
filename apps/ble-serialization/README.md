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

2. Clone [nrf5x-base](https://github.com/lab11/nrf5x-base) in this folder.

3. Install this app on Storm.


