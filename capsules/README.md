Tock Capsules
=============

Capsules are drivers that live in the kernel and are written in Rust. They are
required to conform to Rust's type system (i.e. no `unsafe`). Capsules are
platform agnostic and provide a range of features:
- Drivers for sensors or other ICs
- Virtualization of hardware resources
- Syscall interfaces for userland applications

When using hardware resources, capsules must only use features provided by the
HIL (hardware interface layer). This ensures they can be used on multiple
microcontrollers and hardware platforms.

Capsules have some flexibility in how they present access to a sensor or
virtualized hardware resource. Some capsules directly implement the `Driver`
trait and can be used by userland applications. Others provide an internal
interface that can be used by other in-kernel capsules as well as a `Driver`
interface for applications.


List of Tock Capsules
---------------------

The list of Tock capsules and a brief description.

### Sensor and other IC Drivers

These implement a driver to setup and read various physical sensors.

- **[ADC Microphone](src/adc_microphone.rs)**: Single ADC pin microphone.
- **[Analog Sensors](src/analog_sensor.rs)**: Single ADC pin sensors.
- **[APDS9960](src/apds9960.rs)**: Proximity sensor.
- **[FXOS8700CQ](src/fxos8700cq.rs)**: Accelerometer and magnetometer.
- **[HTS221](src/hts221.rs)**: Temperature and humidity sensor.
- **[ISL29035](src/isl29035.rs)**: Light sensor.
- **[L3GD20](src/l3gd20.rs)**: MEMS 3 axys digital gyroscope and temperature
  sensor.
- **[LSM303xx Support](src/lsm303xx.rs)**: Shared files.
  - **[LSM303AGR](src/lsm303agr.rs)**: 3D accelerometer and 3D magnetometer
    sensor.
  - **[LSM303DLHC](src/lsm303dlhc.rs)**: 3D accelerometer and 3D magnetometer
    sensor.
- **[LPS25HB](src/lps25hb.rs)**: Pressure sensor.
- **[MLX90614](src/mlx90614.rs)**: Infrared temperature sensor.
- **[RP2040 Temperature](src/temperature_rp2040.rs)**: Analog RP2040 temperature
  sensor.
- **[SHT3x](src/sht3x.rs)**: SHT3x temperature and humidity sensor.
- **[SI7021](src/si7021.rs)**: Temperature and humidity sensor.
- **[STM32 Temperature](src/temperature_stm.rs)**: Analog STM32 temperature
  sensor.
- **[TSL2561](src/tsl2561.rs)**: Light sensor.

These drivers provide support for various ICs.

- **[FM25CL](src/fm25cl.rs)**: FRAM chip.
- **[FT6x06](src/ft6x06.rs)**: FT6x06 touch panel.
- **[HD44780 LCD](src/hd44780.rs)**: HD44780 LCD screen.
- **[LTC294X](src/ltc294x.rs)**: LTC294X series of coulomb counters.
- **[MAX17205](src/max17205.rs)**: Battery fuel gauge.
- **[MCP230xx](src/mcp230xx.rs)**: I2C GPIO extender.
- **[MX25r6435F](src/mx25r6435f.rs)**: SPI flash chip.
- **[PCA9544A](src/pca9544a.rs)**: Multiple port I2C selector.
- **[SD Card](src/sdcard.rs)**: Support for SD cards.
- **[ST77xx](src/st77xx.rs)**: ST77xx IPS screen.


### Wireless

Support for wireless radios.

- **[nRF51822 Serialization](src/nrf51822_serialization.rs)**: Kernel support
  for using the nRF51 serialization library.
- **[RF233](src/rf233.rs)**: Driver for RF233 radio.
- **[BLE Advertising](src/ble_advertising_driver.rs)**: Driver for sending BLE
  advertisements.

### Libraries

Protocol stacks and other libraries.

- **[IEEE 802.15.4](src/ieee802154)**: 802.15.4 networking.
- **[Networking](src/net)**: Networking stack.
- **[USB](src/usb)**: USB 2.0.
- **[Segger RTT](src/segger_rtt.rs)**: Segger RTT support. Provides `hil::uart`
  interface.


### MCU Peripherals for Userspace

These capsules provide a `Driver` interface for common MCU peripherals.

- **[ADC](src/adc.rs)**: Individual and continuous samples.
- **[Alarm](src/alarm.rs)**: Oneshot and periodic timers.
- **[Analog Comparator](src/analog_comparator.rs)**: Voltage comparison.
- **[CRC](src/crc.rs)**: CRC calculation.
- **[DAC](src/dac.rs)**: Digital to analog conversion.
- **[GPIO](src/gpio.rs)**: GPIO configuring and control.
- **[I2C_MASTER](src/i2c_master.rs)**: I2C master access only.
- **[I2C_MASTER_SLAVE](src/i2c_master_slave_driver.rs)**: I2C master and slave
  access.
- **[RNG](src/rng.rs)**: Random number generation.
- **[SPI Controller](src/spi_controller.rs)**: SPI controller device (SPI
  master)
- **[SPI Peripheral](src/spi_peripheral.rs)**: SPI peripheral device (SPI slave)


### Helpful Userspace Capsules

These provide common and better abstractions for userspace.

- **[Ambient Light](src/ambient_light.rs)**: Query light sensors.
- **[App Flash](src/app_flash_driver.rs)**: Allow applications to write their
  own flash.
- **[Button](src/button.rs)**: Detect button presses.
- **[Buzzer](src/buzzer_driver.rs)**: Simple buzzer.
- **[Console](src/console.rs)**: UART console support.
- **[CTAP](src/ctap.rs)**: Client to Authenticator Protocol (CTAP) support.
- **[Humidity](src/humidity.rs)**: Query humidity sensors.
- **[LED](src/led.rs)**: Turn on and off LEDs.
- **[LED Matrix](src/led_matrix.rs)**: Control a 2D array of LEDs.
- **[Proximity](src/proximity.rs)**: Proximity sensors.
- **[Screen](src/screen.rs)**: Displays and screens.
- **[SHA](src/sha.rs)**: SHA hashes.
- **[Sound Pressure](src/sound_pressure.rs)**: Query sound pressure levels.
- **[Temperature](src/temperature.rs)**: Query temperature sensors.
- **[Text Screen](src/text_screen.rs)**: Text-based displays.
- **[Touch](src/touch.rs)**: User touch panels.


### Virtualized Sensor Capsules for Userspace

These provide virtualized (i.e. multiple applications can use them
simultaneously) support for generic sensor interfaces.

- **[Asynchronous GPIO](src/gpio_async.rs)**: GPIO pins accessed by split-phase
  calls.
- **[9DOF](src/ninedof.rs)**: 9DOF sensors (acceleration, magnetometer,
  gyroscope).
- **[Nonvolatile Storage](src/nonvolatile_storage_driver.rs)**: Persistent
  storage for userspace.


### Virtualized Hardware Resources

These allow for multiple users of shared hardware resources in the kernel.

- **[Virtual ADC](src/virtual_adc.rs)**: Shared single ADC channel.
- **[Virtual AES-CCM](src/virtual_aes_ccm.rs)**: Shared AES-CCM engine.
- **[Virtual Alarm](src/virtual_alarm.rs)**: Shared alarm resource.
- **[Virtual Digest](src/virtual_digest.rs)**: Shared digest resource.
- **[Virtual Flash](src/virtual_flash.rs)**: Shared flash resource.
- **[Virtual HMAC](src/virtual_hmac.rs)**: Shared HMAC resource.
- **[Virtual I2C](src/virtual_i2c.rs)**: Shared I2C and fixed addresses.
- **[Virtual PWM](src/virtual_pwm.rs)**: Shared PWM hardware.
- **[Virtual RNG](src/virtual_rng.rs)**: Shared random number generator.
- **[Virtual SHA](src/virtual_sha.rs)**: Shared SHA hashes.
- **[Virtual SPI](src/virtual_spi.rs)**: Shared SPI and fixed chip select pins.
- **[Virtual Timer](src/virtual_timer.rs)**: Shared timer.
- **[Virtual UART](src/virtual_uart.rs)**: Shared UART bus.


### Utility Capsules

Other capsules that implement reusable logic.

- **[Nonvolatile to Pages](src/nonvolatile_to_pages.rs)**: Map arbitrary reads
  and writes to flash pages.
- **[HMAC](src/hmac.rs)**: Hash-based Message Authentication Code (HMAC) digest
  engine.
- **[Log Storage](src/log.rs)**: Log storage abstraction on top of flash
  devices.
- **[Bus Adapters](src/bus.rs)**: Generic abstraction for SPI/I2C/8080.
- **[TicKV](src/tickv.rs)**: Key-value storage.


### Debugging Capsules

These are selectively included on a board to help with testing and debugging
various elements of Tock.

- **[Debug Process Restart](src/debug_process_restart.rs)**: Force all processes
  to enter a fault state when a button is pressed.
- **[Low-Level Debug](src/low_level_debug)**: Provides system calls for
  low-level debugging tasks, such as debugging toolchain and relocation issues.
- **[Panic Button](src/panic_button.rs)**: Use a button to force a `panic!()`.
- **[Process Console](src/process_console.rs)**: Provide a UART console to
  inspect the status of process and stop/start them.
