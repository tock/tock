"Extra" Tock Capsules
=====================

This crate contains miscellaneous capsules which do not fit into any other, more
specific category, and which do not require any external (non-vendored and
unvetted) dependencies.

For more information on capsules, see [the top-level README](../README.md).

The remainder of this document contains a list of capsules in this crate, along
with a short description.

Sensor and other IC Drivers
---------------------------

These implement a driver to setup and read various physical sensors.

- **[ADC Microphone](src/adc_microphone.rs)**: Single ADC pin microphone.
- **[Analog Sensors](src/analog_sensor.rs)**: Single ADC pin sensors.
- **[APDS9960](src/apds9960.rs)**: Proximity sensor.
- **[ATECC508A](src/atecc508a.rs)**: Cryptographic Co-Processor Breakout.
- **[BME280](src/bme280.rs)**: Humidity and air pressure sensor.
- **[BMM150](src/bmm150.rs)**: Geomagnetic sensor.
- **[BMP280](src/bmp280.rs)**: Temperature (and air pressure) sensor.
- **[CCS811](src/ccs811.rs)**: VOC gas sensor.
- **[Chirp I2C Moisture](src/chirp_i2c_moisture.rs)**: I2C moisture sensor
    from Chirp project.
- **[DFRobot Rainfall Sensor](src/dfrobot_rainfall_sensor.rs)**: Rainfall sensor.
- **[FXOS8700CQ](src/fxos8700cq.rs)**: Accelerometer and magnetometer.
- **[HS3003](src/hs3003.rs)**: Temperature and humidity sensor.
- **[HTS221](src/hts221.rs)**: Temperature and humidity sensor.
- **[ISL29035](src/isl29035.rs)**: Light sensor.
- **[L3GD20](src/l3gd20.rs)**: MEMS 3 axys digital gyroscope and temperature
  sensor.
- **[LSM303xx Support](src/lsm303xx.rs)**: Shared files.
  - **[LSM303AGR](src/lsm303agr.rs)**: 3D accelerometer and 3D magnetometer
    sensor.
  - **[LSM303DLHC](src/lsm303dlhc.rs)**: 3D accelerometer and 3D magnetometer
    sensor.
- **[LSM6DSOXTR](src/lsm6dsoxtr.rs)**: 3D accelerometer and 3D magnetometer
    sensor.
- **[LPS22HB](src/lps22hb.rs)**: Pressure sensor.
- **[LPS25HB](src/lps25hb.rs)**: Pressure sensor.
- **[MLX90614](src/mlx90614.rs)**: Infrared temperature sensor.
- **[RP2040 Temperature](src/temperature_rp2040.rs)**: Analog RP2040 temperature
  sensor.
- **[SHT3x](src/sht3x.rs)**: Temperature and humidity sensor.
- **[SHT4x](src/sht4x.rs)**: Temperature and humidity sensor.
- **[SI7021](src/si7021.rs)**: Temperature and humidity sensor.
- **[STM32 Temperature](src/temperature_stm.rs)**: Analog STM32 temperature
  sensor.
- **[TSL2561](src/tsl2561.rs)**: Light sensor.
- **[HC-SR04](src/hc_sr04.rs)**: Ultrasonic distance sensor

These drivers provide support for various ICs.

- **[AT24C32/64](src/at24c_eeprom.rs)**: EEPROM chip.
- **[FM25CL](src/fm25cl.rs)**: FRAM chip.
- **[FT6x06](src/ft6x06.rs)**: FT6x06 touch panel.
- **[HD44780 LCD](src/hd44780.rs)**: HD44780 LCD screen.
- **[LPM013M126](src/lpm013m126.rs)**: LPM013M126 LCD screen.
- **[LTC294X](src/ltc294x.rs)**: LTC294X series of coulomb counters.
- **[MAX17205](src/max17205.rs)**: Battery fuel gauge.
- **[MCP230xx](src/mcp230xx.rs)**: I2C GPIO extender.
- **[MX25r6435F](src/mx25r6435f.rs)**: SPI flash chip.
- **[PCA9544A](src/pca9544a.rs)**: Multiple port I2C selector.
- **[SD Card](src/sdcard.rs)**: Support for SD cards.
- **[Seven Segment Display](src/seven_segment.rs)**: Seven segment displays.
- **[SH1106](src/sh1106.rs)**: SH1106 OLED screen driver.
- **[SSD1306](src/ssd1306.rs)**: SSD1306 OLED screen driver.
- **[ST77xx](src/st77xx.rs)**: ST77xx IPS screen.


Wireless and Networking
--------

Support for wireless radios, network stacks and related infrastructure.

- **[nRF51822 Serialization](src/nrf51822_serialization.rs)**: Kernel support
  for using the nRF51 serialization library.
- **[RF233](src/rf233.rs)**: Driver for RF233 radio.
- **[BLE Advertising](src/ble_advertising_driver.rs)**: Driver for sending BLE
  advertisements.
- **[LoRa Phy]**: Support for exposing Semtech devices to userspace
  See the lora_things_plus board for an example
- **[Ethernet Tap Driver](src/ethernet_tap.rs)**: Forwarding raw IEEE
  802.3 Ethernet frames from / to userspace. Useful for running
  network stacks in userspace.

Libraries
---------

Protocol stacks and other libraries.

- **[IEEE 802.15.4](src/ieee802154)**: 802.15.4 networking.
- **[Networking](src/net)**: Networking stack.
- **[USB](src/usb)**: USB 2.0.
- **[Symmetric Cryptography](src/symmetric_encryption)**: Symmetric
  encryption.
- **[Public Key Cryptography](src/public_key_crypto)**: Asymmetric
  encryption.


MCU Peripherals for Userspace
-----------------------------

These capsules provide a `Driver` interface for common MCU peripherals.

- **[Analog Comparator](src/analog_comparator.rs)**: Voltage comparison.
- **[CRC](src/crc.rs)**: CRC calculation.
- **[DAC](src/dac.rs)**: Digital to analog conversion.
- **[CAN](src/can.rs)**: CAN communication.


Helpful Userspace Capsules
--------------------------

These provide common and better abstractions for userspace.

- **[Air Quality](src/air_quality.rs)**: Query air quality sensors.
- **[Ambient Light](src/ambient_light.rs)**: Query light sensors.
- **[App Flash](src/app_flash_driver.rs)**: Allow applications to write their
  own flash.
- **[App Loader](src/app_loader.rs)**: Allow applications to request to 
  install and load new applications.
- **[Buzzer](src/buzzer_driver.rs)**: Simple buzzer.
- **[Servo](src/servo.rs)**: Servo motor.
- **[Date-Time](src/date_time.rs)**: Real time clock date/time support.
- **[EUI64](src/eui64.rs)**: Query device's extended unique ID.
- **[HMAC](src/hmac.rs)**: Hash-based Message Authentication Code support.
- **[Humidity](src/humidity.rs)**: Query humidity sensors.
- **[Key-Value Store](src/kv_driver.rs)**: Store key-value data.
- **[LED Matrix](src/led_matrix.rs)**: Control a 2D array of LEDs.
- **[Moisture](src/moisture.rs)**: Query moisture sensors.
- **[Pressure](src/pressure.rs)**: Pressure sensors.
- **[Proximity](src/proximity.rs)**: Proximity sensors.
- **[PWM](src/pwm.rs)**: Pulse-width modulation support.
- **[Rainfall](src/rainfall.rs)**: Query rainfall sensors.
- **[Read Only State](src/read_only_state.rs)**: Read-only state sharing.
- **[Screen](src/screen.rs)**: Displays and screens.
- **[Screen Shared](src/screen_shared.rs)**: App-specific screen windows.
- **[SHA](src/sha.rs)**: SHA hashes.
- **[Sound Pressure](src/sound_pressure.rs)**: Query sound pressure levels.
- **[Temperature](src/temperature.rs)**: Query temperature sensors.
- **[Text Screen](src/text_screen.rs)**: Text-based displays.
- **[Touch](src/touch.rs)**: User touch panels.
- **[Distance](src/distance.rs)**: Distance sensor.


Virtualized Sensor Capsules for Userspace
-----------------------------------------

These provide virtualized (i.e. multiple applications can use them
simultaneously) support for generic sensor interfaces.

- **[Asynchronous GPIO](src/gpio_async.rs)**: GPIO pins accessed by split-phase
  calls.
- **[9DOF](src/ninedof.rs)**: 9DOF sensors (acceleration, magnetometer,
  gyroscope).
- **[Nonvolatile Storage](src/nonvolatile_storage_driver.rs)**: Persistent
  storage for userspace.
- **[Isolated Nonvolatile Storage](src/isolated_nonvolatile_storage_driver.rs)**:
  Per-app isolated persistent storage for userspace.


Utility Capsules
----------------

Other capsules that implement reusable logic.

- **[Bus Adapters](src/bus.rs)**: Generic abstraction for SPI/I2C/8080.
- **[Buzzer PWM](src/buzzer_pwm.rs)**: Buzzer with a PWM pin.
- **[SG90 PWM](src/sg90.rs)**: SG90 servomotor.
- **[HMAC-SHA256](src/hmac_sha256.rs)**: HMAC using SHA-256.
- **[Key-Value Store with Permissions](src/kv_store_permissions.rs)**: Key-value
  interface that requires read/write permissions.
- **[Log Storage](src/log.rs)**: Log storage abstraction on flash devices.
- **[Nonvolatile to Pages](src/nonvolatile_to_pages.rs)**: Map arbitrary reads
  and writes to flash pages.
- **[SHA256](src/sha256.rs)**: SHA256 software hash.
- **[SipHash](src/sip_hash.rs)**: SipHash software hash.
- **[TicKV](src/tickv.rs)**: Key-value storage.
- **[TicKV KV Store](src/tickv_kv_store.rs)**: Provide `hil::kv::KV` with TickV.
- **[Virtual KV](src/virtual_kv.rs)**: Virtualize access to KV with permissions.


Debugging Capsules
------------------

These are selectively included on a board to help with testing and debugging
various elements of Tock.

- **[Cycle Counter](src/cycle_count.rs)**: Start, stop, reset, and read a hardware cycle
  counter from userspace.
- **[Debug Process Restart](src/debug_process_restart.rs)**: Force all processes
  to enter a fault state when a button is pressed.
- **[Panic Button](src/panic_button.rs)**: Use a button to force a `panic!()`.
- **[Process Info](src/process_info_driver.rs)**: Inspect and control processes.

