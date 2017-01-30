BLE Serialization App
=====================

This app makes it possible to provide a BLE interface on the SAM4L.
It does this by using Nordic's BLE Serialization format where BLE commands
can be communicated over UART to the nRF51822 radio. This serialization
protocol allows for otherwise normal nRF51822 BLE apps to be executed on
a different microcontroller. All function calls that would have hit the BLE
stack are instead shuttled over UART to the nRF51822 radio.


Usage
-----

1. Flash the
[BLE serialization app](https://github.com/helena-project/tock-nrf-serialization/tree/master/nrf51822/apps/tock-nrf51822-serialization-sdk11-s130-uart-conn)
to the nRF51822, following the instructions in that folder to specify the platform you are on.

2. Install this Tock app on Storm.

