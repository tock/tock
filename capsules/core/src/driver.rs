// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Mapping of capsules to their syscall driver number.

use enum_primitive::cast::FromPrimitive;
use enum_primitive::enum_from_primitive;

enum_from_primitive! {
#[derive(Debug, PartialEq)]
// syscall driver numbers
pub enum NUM {
    // Base
    Alarm                 = 0x00000,
    Console               = 0x00001,
    Led                   = 0x00002,
    Button                = 0x00003,
    Gpio                  = 0x00004,
    Adc                   = 0x00005,
    Dac                   = 0x00006,
    AnalogComparator      = 0x00007,
    LowLevelDebug         = 0x00008,
    ReadOnlyState         = 0x00009,
    Pwm                   = 0x00010,

    // Kernel
    Ipc                   = 0x10000,
    AppLoader             = 0x10001,
    ProcessInfo           = 0x10002,

    // HW Buses
    Spi                   = 0x20001,
    SpiPeripheral         = 0x20002,
    I2cMaster             = 0x20003,
    UsbUser               = 0x20005,
    I2cMasterSlave        = 0x20006,
    Can                   = 0x20007,

    // Networking
    BleAdvertising        = 0x30000,
    Ieee802154            = 0x30001,
    Udp                   = 0x30002,
    LoRaPhySPI            = 0x30003,
    LoRaPhyGPIO           = 0x30004,
    Thread                = 0x30005,
    Eui64                 = 0x30006,
    EthernetTap           = 0x30007,

    // Cryptography
    Rng                   = 0x40001,
    Crc                   = 0x40002,
    Hmac                  = 0x40003,
    CtapHid               = 0x40004,
    Sha                   = 0x40005,
    Aes                   = 0x40006,

    // Storage
    AppFlash              = 0x50000,
    NvmStorage            = 0x50001,
    SdCard                = 0x50002,
    Kv                    = 0x50003,
    IsolatedNvmStorage    = 0x50004,

    // Sensors
    Temperature           = 0x60000,
    Humidity              = 0x60001,
    AmbientLight          = 0x60002,
    NINEDOF               = 0x60004,
    Proximity             = 0x60005,
    SoundPressure         = 0x60006,
    AirQuality            = 0x60007,
    Pressure              = 0x60008,
    Distance              = 0x60009,
    Moisture              = 0x6000A,
    RainFall              = 0x6000B,

    // Sensor ICs
    Tsl2561               = 0x70000,
    Tmp006                = 0x70001,
    Lps25hb               = 0x70004,
    L3gd20                = 0x70005,
    Lsm303dlch            = 0x70006,
    Mlx90614              = 0x70007,
    Lsm6dsoxtr            = 0x70008,

    // Other ICs
    Ltc294x               = 0x80000,
    Max17205              = 0x80001,
    Pca9544a              = 0x80002,
    GpioAsync             = 0x80003,
    Nrf51822Serialization = 0x80004,

    // Misc
    Buzzer                = 0x90000,
    Screen                = 0x90001,
    Touch                 = 0x90002,
    TextScreen            = 0x90003,
    SevenSegment          = 0x90004,
    KeyboardHid           = 0x90005,
    DateTime              = 0x90007,
    CycleCount            = 0x90008,
    Servo                 = 0x90009,
}
}
