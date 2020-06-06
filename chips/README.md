Tock Chips
==========

The `/chips` folder contains the list of microcontrollers supported by Tock.
Each MCU folder contains the hardware peripheral drivers for that MCU.



HIL Support
-----------

<!--START OF HIL SUPPORT-->

| HIL                                     | apollo3  | arty_e21 | cc26x2 | e310x | imxrt1052 | lowrisc | nrf52832 | nrf52840 | sam4l | stm32f3xx | stm32f4xx | 
|-----------------------------------------|----------|----------|--------|-------|-----------|---------|----------|----------|-------|-----------|-----------|
| adc::Adc                                |          |          |        |       |           |         | ✓        | ✓        | ✓     |           |           |
| adc::AdcHighSpeed                       |          |          |        |       |           |         |          |          | ✓     |           |           |
| analog_comparator::AnalogComparator     |          |          |        |       |           |         |          |          | ✓     |           |           |
| ble_advertising::BleAdvertisementDriver |          |          |        |       |           |         | ✓        | ✓        |       |           |           |          |
| ble_advertising::BleConfig              |          |          |        |       |           |         | ✓        | ✓        |       |           |           |
| crc::CRC                                |          |          |        |       |           |         |          |          | ✓     |           |           |
| dac::DacChannel                         |          |          |        |       |           |         |          |          | ✓     |           |           |
| eic::ExternalInterruptController        |          |          |        |       |           |         |          |          | ✓     |           |           |
| entropy::Entropy32                      |          |          | ✓      |       |           |         | ✓        | ✓        | ✓     |           |           |
| flash::Flash                            |          |          |        |       |           |         | ✓        | ✓        | ✓     |           |           |
| gpio::Input                             |          | ✓        | ✓      | ✓     | ✓         | ✓       | ✓        | ✓        | ✓     | ✓         | ✓         |
| gpio::Interrupt                         |          | ✓        | ✓      | ✓     |           | ✓       | ✓        | ✓        | ✓     | ✓         | ✓         |
| gpio::InterruptPin                      | ✓        | ✓        | ✓      | ✓     |           | ✓       | ✓        | ✓        | ✓     | ✓         | ✓         |
| gpio::Output                            | ✓        | ✓        | ✓      | ✓     | ✓         | ✓       | ✓        | ✓        | ✓     | ✓         | ✓         |
| gpio::Pin                               | ✓        | ✓        | ✓      | ✓     | ✓         | ✓       | ✓        | ✓        | ✓     | ✓         | ✓         |
| i2c::I2CMaster                          |          |          | ✓      |       | ✓         |         | ✓        | ✓        | ✓     | ✓         |           |
| i2c::I2CMasterSlave                     |          |          |        |       |           |         |          |          | ✓     |           |           |
| i2c::I2CSlave                           |          |          |        |       |           |         |          |          | ✓     |           |           |
| mod::Controller                         |          |          |        |       |           |         | ✓        | ✓        | ✓     |           |           |
| pwm::Pwm                                |          |          |        |       |           |         | ✓        | ✓        |       |           |           |
| radio::Radio                            |          |          |        |       |           |         | ✓        | ✓        |       |           |           |
| radio::RadioConfig                      |          |          |        |       |           |         | ✓        | ✓        |       |           |           |
| radio::RadioData                        |          |          |        |       |           |         | ✓        | ✓        |       |           |           |
| sensors::TemperatureDriver              |          |          |        |       |           |         | ✓        | ✓        |       |           |           |
| spi::SpiMaster                          |          |          |        |       |           |         | ✓        | ✓        | ✓     | ✓         | ✓         |
| spi::SpiSlave                           |          |          |        |       |           |         |          |          | ✓     |           |           |
| symmetric_encryption::AES128            |          |          |        |       |           |         | ✓        | ✓        | ✓     |           |           |
| symmetric_encryption::AES128CBC         |          |          |        |       |           |         | ✓        | ✓        | ✓     |           |           |
| symmetric_encryption::AES128CCM         |          |          |        |       |           |         | ✓        | ✓        |       |           |           |
| symmetric_encryption::AES128Ctr         |          |          |        |       |           |         | ✓        | ✓        | ✓     |           |           |
| time::Alarm                             |          |          | ✓      |       | ✓         | ✓       | ✓        | ✓        | ✓     | ✓         | ✓         |
| time::Frequency                         |          |          | ✓      |       |           | ✓       | ✓        | ✓        |       |           |           |
| time::Time                              |          |          | ✓      |       | ✓         | ✓       | ✓        | ✓        | ✓     | ✓         | ✓         |
| uart::Configure                         | ✓        | ✓        | ✓      | ✓     | ✓         | ✓       | ✓        | ✓        | ✓     | ✓         | ✓         |
| uart::Receive                           |          | ✓        | ✓      | ✓     | ✓         | ✓       | ✓        | ✓        | ✓     | ✓         | ✓         |
| uart::ReceiveAdvanced                   |          |          |        |       |           |         |          |          | ✓     |           |           |
| uart::Transmit                          | ✓        | ✓        | ✓      | ✓     | ✓         | ✓       | ✓        | ✓        | ✓     | ✓         | ✓         |
| uart::Uart                              | ✓        | ✓        | ✓      | ✓     | ✓         | ✓       | ✓        | ✓        | ✓     | ✓         | ✓         |
| uart::UartAdvanced                      |          |          |        |       |           |         |          |          | ✓     |           |           |
| uart::UartData                          | ✓        | ✓        | ✓      | ✓     | ✓         | ✓       | ✓        | ✓        |       | ✓         | ✓         |
| usb::UsbController                      |          |          |        |       |           |         | ✓        | ✓        | ✓     |           |           |
| watchdog::Watchdog                      |          |          |        |       |           |         |          |          | ✓     |           |           |

<!--END OF HIL SUPPORT-->


