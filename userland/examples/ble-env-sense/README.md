BLE Environmental Sensing
=========================

This app demonstrates providing the
[environmental sensing service](https://www.bluetooth.com/specifications/assigned-numbers/environmental-sensing-service-characteristics)
over BLE.


nRF Serialization
-----------------

This app uses Nordic's BLE Serialization format where BLE commands can be
communicated over UART to the nRF51822 radio. This serialization protocol allows
for otherwise normal nRF51822 BLE apps to be executed on a different
microcontroller. All function calls that would have hit the BLE stack are
instead shuttled over UART to the nRF51822 radio.
