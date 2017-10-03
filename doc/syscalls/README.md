Tock Syscalls
=============

This folder contains the detailed documentation for the interfaces between
userspace and kernel drivers (using `allow`, `schedule`, and `command`).

<!-- toc -->

- [Allocated Driver Numbers](#allocated-driver-numbers)
  * [Base](#base)
  * [Kernel](#kernel)
  * [HW Buses](#hw-buses)
  * [Radio](#radio)
  * [Cryptography](#cryptography)
  * [Storage](#storage)
  * [Sensors](#sensors)
  * [Sensor ICs](#sensor-ics)
  * [Other ICs](#other-ics)

<!-- tocstop -->

## Allocated Driver Numbers

Each driver type that has been allocated a permanent driver number is listed in
the tables below. The "1.0" column indicates whether the driver has been
stabilized or not (a "✓" indicates stability) in the Tock 1.0 release.

### Base

|1.0| Driver Number | Driver                      | Description                                |
|---|---------------|-----------------------------|--------------------------------------------|
| ✓ | 0x0           | [Alarm](00000_alarm.md)     | Used for timers in userspace               |
| ✓ | 0x1           | [Console](00001_console.md) | UART console                               |
| ✓ | 0x2           | [LED](00002_leds.md)        | Control LEDs on board                      |
| ✓ | 0x3           | [Button](00003_buttons.md)  | Get interrupts from buttons on the board   |
|   | 0x4           | [GPIO](00004_gpio.md)       | Set and read GPIO pins                     |
| ✓ | 0x5           | [ADC](00005_adc.md)         | Sample analog-to-digital converter pins    |
|   | 0x6           | DAC                         | Digital to analog converter                |

### Kernel

|1.0| Driver Number | Driver           | Description                                |
|---|---------------|------------------|--------------------------------------------|
|   | 0x10000       | IPC              | Inter-process communication                |

### HW Buses

|1.0| Driver Number | Driver           | Description                                |
|---|---------------|------------------|--------------------------------------------|
|   | 0x20000       | UART             | UART                                       |
|   | 0x20001       | SPI              | Raw SPI Master interface                   |
|   | 0x20002       | SPI Slave        | Raw SPI slave interface                    |
|   | 0x20003       | I2C Master       | Raw I2C Master interface                   |
|   | 0x20004       | I2C Slave        | Raw I2C Slave interface                    |
|   | 0x20005       | USB              | Universal Serial Bus interface             |

### Radio

|1.0| Driver Number | Driver           | Description                                |
|---|---------------|------------------|--------------------------------------------|
|   | 0x30000       | BLE              | Bluetooth Low Energy                       |
|   | 0x30001       | 802.15.4         | IEEE 802.15.4                              |

### Cryptography

|1.0| Driver Number | Driver           | Description                                |
|---|---------------|------------------|--------------------------------------------|
|   | 0x40000       | AES              | AES Symmetric Key Cryptography             |
|   | 0x40001       | RNG              | Random number generator                    |
|   | 0x40002       | CRC              | Cyclic Redundancy Check computation        |

### Storage

|1.0| Driver Number | Driver           | Description                                |
|---|---------------|------------------|--------------------------------------------|
|   | 0x50000       | App Flash        | Allow apps to write their own flash        |
|   | 0x50001       | Nonvolatile Storage | Generic interface for persistent storage |
|   | 0x50002       | SDCard           | Raw block access to an SD card             |

### Sensors

|1.0| Driver Number | Driver                                        | Description                                |
|---|---------------|-----------------------------------------------|--------------------------------------------|
| ✓ | 0x60000       | [Ambient Temp.](60000_ambient_temperature.md) | Ambient temperature (centigrate)           |
| ✓ | 0x60001       | [Humidity](60001_humidity.md)                 | Humidity Sensor (percent)                  |
| ✓ | 0x60002       | [Luminance](60002_luminance.md)               | Ambient Light Sensor (lumens)              |
|   | 0x60003       | Pressure         | Pressure sensor                            |
|   | 0x60004       | Ninedof          | Virtualized accelerometer/magnetometer/gyroscope |

### Sensor ICs

|1.0| Driver Number | Driver           | Description                                |
|---|---------------|------------------|--------------------------------------------|
|   | 0x70000       | TSL2561          | Light sensor                               |
|   | 0x70001       | TMP006           | Temperature sensor                         |
|   | 0x70004       | LPS25HB          | Pressure sensor                            |

### Other ICs

|1.0| Driver Number | Driver           | Description                                |
|---|---------------|------------------|--------------------------------------------|
|   | 0x80000       | LTC294X          | Battery gauge IC                           |
|   | 0x80001       | MAX17205         | Battery gauge IC                           |
|   | 0x80002       | PCA9544A         | I2C address multiplexing                   |
|   | 0x80003       | GPIO Async       | Asynchronous GPIO pins                     |
|   | 0x80004       | nRF51822         | nRF serialization link to nRF51822 BLE SoC |
