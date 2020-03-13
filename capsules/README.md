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

- **[Analog Sensors](src/analog_sensor.rs)**: Single ADC pin sensors.
- **[FXOS8700CQ](src/fxos8700cq.rs)**: Accelerometer and magnetometer.
- **[ISL29035](src/isl29035.rs)**: Light sensor.
- **[LPS25HB](src/lps25hb.rs)**: Pressure sensor.
- **[SI7021](src/si7021.rs)**: Temperature and humidity sensor.
- **[TMP006](src/tmp006.rs)**: Infrared temperature sensor.
- **[TSL2561](src/tsl2561.rs)**: Light sensor.

These drivers provide support for various ICs.

- **[FM25CL](src/fm25cl.rs)**: FRAM chip.
- **[LTC294X](src/ltc294x.rs)**: LTC294X series of coulomb counters.
- **[MAX17205](src/max17205.rs)**: Battery fuel gauge.
- **[MCP230xx](src/mcp230xx.rs)**: I2C GPIO extender.
- **[MX25r6435F](src/mx25r6435f.rs)**: SPI flash chip.
- **[PCA9544A](src/pca9544a.rs)**: Multiple port I2C selector.
- **[SD Card](src/sdcard.rs)**: Support for SD cards.


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
- **[USB](src/usb.rs)**: USB 2.0.
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
- **[I2C_MASTER_SLAVE](src/i2c_master_slave_driver.rs)**: I2C master and slave access.
- **[RNG](src/rng.rs)**: Random number generation.
- **[SPI](src/spi.rs)**: SPI master and slave.


### Helpful Userspace Capsules

These provide common and better abstractions for userspace.

- **[Ambient Light](src/ambient_light.rs)**: Query light sensors.
- **[App Flash](src/app_flash_driver.rs)**: Allow applications to write their
  own flash.
- **[Button](src/button.rs)**: Detect button presses.
- **[Buzzer](src/buzzer_driver.rs)**: Simple buzzer.
- **[Console](src/console.rs)**: UART console support.
- **[Humidity](src/humidity.rs)**: Query humidity sensors.
- **[LED](src/led.rs)**: Turn on and off LEDs.
- **[Temperature](src/temperature.rs)**: Query temperature sensors.


### Virtualized Sensor Capsules for Userspace

These provide virtualized (i.e. multiple applications can use them
simultaneously) support for generic sensor interfaces.

- **[Asynchronous GPIO](src/gpio_async.rs)**: GPIO pins accessed by split-phase
  calls.
- **[9DOF](src/ninedof.rs)**: 9DOF sensors (acceleration, magnetometer, gyroscope).
- **[Nonvolatile Storage](src/nonvolatile_storage_driver.rs)**: Persistent storage for
  userspace.


### Virtualized Hardware Resources

These allow for multiple users of shared hardware resources in the kernel.

- **[Virtual Alarm](src/virtual_alarm.rs)**: Shared alarm resource.
- **[Virtual Flash](src/virtual_flash.rs)**: Shared flash resource.
- **[Virtual I2C](src/virtual_i2c.rs)**: Shared I2C and fixed addresses.
- **[Virtual PWM](src/virtual_pwm.rs)**: Shared PWM hardware.
- **[Virtual SPI](src/virtual_spi.rs)**: Shared SPI and fixed chip select pins.
- **[Virtual UART](src/virtual_uart.rs)**: Shared UART bus.


### Utility Capsules

Other capsules that implement reusable logic.

- **[Nonvolatile to Pages](src/nonvolatile_to_pages.rs)**: Map arbitrary reads
  and writes to flash pages.
- **[AES Encryption](src/aes_ccm.rs)**: AES-CCM encryption.
- **[Log Storage](src/log_storage.rs)**: Log storage abstraction on top of flash devices.


### Debugging Capsules

These are selectively included on a board to help with testing and debugging
various elements of Tock.

- **[Debug Process Restart](src/debug_process_restart.rs)**: Force all processes
  to enter a fault state when a button is pressed.
- **[Low-Level Debug](src/low_level_debug)**: Provides system calls for
  low-level debugging tasks, such as debugging toolchain and relocation issues.
- **[Process Console](src/process_console.rs)**: Provide a UART console to
  inspect the status of process and stop/start them.
