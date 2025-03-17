Tock Syscalls
=============

This folder contains the detailed documentation for the interfaces between
userspace and the kernel. It includes details of the ABI interface, the kernel
provided syscalls, and the driver specific interfaces (using `allow`,
`schedule`, and `command`). For more information on the general syscalls, see
[here](https://book.tockos.org/doc/syscalls).

<!-- toc -->

- [Syscall Binary Interface](#syscall-binary-interface)
- [Core Kernel Provided Syscalls](#core-kernel-provided-syscalls)
- [Capsule Provided Drivers](#capsule-provided-drivers)
  * [Base](#base)
  * [Kernel](#kernel)
  * [Hardware Access](#hardware-access)
  * [Networking](#networking)
  * [Cryptography](#cryptography)
  * [Storage](#storage)
  * [Sensors](#sensors)
  * [Sensor ICs](#sensor-ics)
  * [Other ICs](#other-ics)
  * [Display](#display)
  * [Miscellaneous](#miscellaneous)

<!-- tocstop -->

## Syscall Binary Interface

Details of the [application binary interface](../Syscalls.md).

## Core Kernel Provided Syscalls

- [`memop`](memop.md): Memory-related operations.

## Capsule Provided Drivers

Each driver type that has been allocated a permanent driver number is listed in
the tables below. The "2.0" column indicates whether the driver has been
[stabilized](../Maintenance.md#stabilizing-a-syscall-driver) or not (a "✓" indicates stability) in the Tock 2.0 release.

### Base

|2.0| Driver Number | Driver                      | Description                                |
|---|---------------|-----------------------------|--------------------------------------------|
| ✓ | 0x00000       | [Alarm](00000_alarm.md)     | Used for timers in userspace               |
| ✓ | 0x00001       | [Console](00001_console.md) | UART console                               |
| ✓ | 0x00002       | [LED](00002_leds.md)        | Control LEDs on board                      |
| ✓ | 0x00003       | [Button](00003_buttons.md)  | Get interrupts from buttons on the board   |
|   | 0x00008       | [Low-Level Debug](00008_low_level_debug.md) | Low-level debugging tools  |

### Kernel

|2.0| Driver Number | Driver           | Description                                |
|---|---------------|------------------|--------------------------------------------|
|   | 0x00009       | [ROS](00009_ros.md) | Read Only State, access system information |
|   | 0x10000       | IPC              | Inter-process communication                |

### Hardware Access

|2.0| Driver Number | Driver           | Description                                |
|---|---------------|------------------|--------------------------------------------|
|   | 0x00004       | [GPIO](00004_gpio.md) | Set and read GPIO pins                |
| ✓ | 0x00005       | [ADC](00005_adc.md)| Sample analog-to-digital converter pins  |
|   | 0x00006       | DAC              | Digital to analog converter                |
|   | 0x00007       | [AnalogComparator](00007_analog_comparator.md) | Analog Comparator |
|   | 0x00010       | [PWM](00010_pwm.md)| Control PWM pins                         |
|   | 0x20000       | UART             | UART                                       |
|   | 0x20001       | SPI              | Raw SPI Master interface                   |
|   | 0x20002       | SPI Slave        | Raw SPI slave interface                    |
|   | 0x20003       | I2C Master       | Raw I2C Master interface                   |
|   | 0x20004       | I2C Slave        | Raw I2C Slave interface                    |
|   | 0x20005       | USB              | Universal Serial Bus interface             |
|   | 0x20007       | [CAN](20007_can.md)| Controller Area Network interface        |

_Note:_ GPIO is slated for re-numbering in Tock 2.0.

### Networking

|2.0| Driver Number | Driver           | Description                                |
|---|---------------|------------------|--------------------------------------------|
|   | 0x30000       | BLE              | Bluetooth Low Energy                       |
|   | 0x30001       | 802.15.4         | IEEE 802.15.4                              |
|   | 0x30002       | [UDP](30002_udp.md)  | UDP / 6LoWPAN Interface                |

### Cryptography

|2.0| Driver Number | Driver           | Description                                |
|---|---------------|------------------|--------------------------------------------|
|   | 0x40000       | AES              | AES Symmetric Key Cryptography             |
|   | 0x40001       | RNG              | Random number generator                    |
|   | 0x40002       | CRC              | Cyclic Redundancy Check computation        |

### Storage

|2.0| Driver Number | Driver           | Description                                |
|---|---------------|------------------|--------------------------------------------|
|   | 0x50000       | App Flash        | Allow apps to write their own flash        |
|   | 0x50001       | Nonvolatile Storage | Generic interface for persistent storage |
|   | 0x50002       | SDCard           | Raw block access to an SD card             |
|   | 0x50003       | [Key-Value](50003_key_value.md) | Access to a key-value storage database |

### Sensors

|2.0| Driver Number | Driver                                        | Description                                |
|---|---------------|-----------------------------------------------|--------------------------------------------|
| ✓ | 0x60000       | [Ambient Temp.](60000_ambient_temperature.md) | Ambient temperature (centigrate)           |
| ✓ | 0x60001       | [Humidity](60001_humidity.md)                 | Humidity Sensor (percent)                  |
| ✓ | 0x60002       | [Luminance](60002_luminance.md)               | Ambient Light Sensor (lumens)              |
|   | 0x60003       | Pressure                                      | Pressure sensor                            |
|   | 0x60004       | Ninedof                                       | Virtualized accelerometer/magnetometer/gyroscope |
|   | 0x60005       | Proximity                                     | Proximity Sensor                           |
|   | 0x60006       | SoundPressure                                 | Sound Pressure Sensor                      |
|   | 0x90002       | [Touch](90002_touch.md)                       | Multi Touch Panel                          |
|   | 0x60009       | [Distance](60009_distance.md)                 | Distance Sensor                            |

### Sensor ICs

|2.0| Driver Number | Driver                            | Description                                               |
|---|---------------|-----------------------------------|-----------------------------------------------------------|
|   | 0x70000       | TSL2561                           | Light sensor                                              |
|   | 0x70001       | TMP006                            | Temperature sensor                                        |
|   | 0x70004       | LPS25HB                           | Pressure sensor                                           |
|   | 0x70005       | [L3GD20](70005_l3gd20.md)         | 3 axis gyroscope and temperature sensor                   |
|   | 0x70006       | [LSM303DLHC](70006_lsm303dlhc.md) | 3 axis accelerometer, magnetometer and temperature sensor |

### Other ICs

|2.0| Driver Number | Driver           | Description                                |
|---|---------------|------------------|--------------------------------------------|
|   | 0x80000       | LTC294X          | Battery gauge IC                           |
|   | 0x80001       | MAX17205         | Battery gauge IC                           |
|   | 0x80002       | PCA9544A         | I2C address multiplexing                   |
|   | 0x80003       | GPIO Async       | Asynchronous GPIO pins                     |
|   | 0x80004       | nRF51822         | nRF serialization link to nRF51822 BLE SoC |

### Display

|2.0| Driver Number | Driver                                  | Description                                |
|---|---------------|-----------------------------------------|--------------------------------------------|
|   | 0x90001       | [Screen](90001_screen.md)               | Graphic Screen                             |
|   | 0x90003       | [Text Screen](90003_text_screen.md)     | Text Screen                                |

### Miscellaneous

|2.0| Driver Number | Driver                                  | Description                                |
|---|---------------|-----------------------------------------|--------------------------------------------|
|   | 0x90000       | Buzzer                                  | Buzzer                                     |
|   | 0x90009       | [Servo](90009_servo.md)                |                  |
Servo