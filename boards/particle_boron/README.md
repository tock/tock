Platform-Specific Instructions: Particle-Boron
===================================

## Particle Boron

<img src="https://user-images.githubusercontent.com/36925352/188639324-e93fe4d7-9673-4bb3-a97e-1c08d330e683.png" width="720">
</br> </br>

The [Particle_Boron](https://docs.particle.io/reference/datasheets/b-series/boron-datasheet/) is development board based on the `nRF52840 SOC`. "The Boron is a powerful LTE Cat M1 or 2G/3G enabled development kit that supports cellular networks and Bluetooth LE (BLE). It is based on the Nordic nRF52840 and has built-in battery charging circuitry so it’s easy to connect a Li-Po and deploy your local network in minutes."

To program the [Particle_Boron](https://docs.particle.io/reference/datasheets/b-series/boron-datasheet/) with Tock, you will need a JLink JTAG device and the
appropriate cables.

Then, follow the [Tock Getting Started guide](../../../doc/Getting_Started.md)

## Programming the kernel

Once you have all software installed, you should be able to simply run
`make install` in this directory to install a fresh kernel. Make sure to install 
the [tockloader](https://github.com/tock/tockloader).

## Hardware Setup

Required: 

* Segger JLink 
* USB Cable for power & Debugging

Ensure JLink is connected and the Particle Boron is powered using either mUSB cable or a Li-Po battery. 

## Software Setup

To flash using JLink, make sure the segger software is installed on your machine. See [here](https://www.segger.com/downloads/jlink). You can test that it is installed properly by running the following command. 

```bash
$ JLinkExe
```

## Programming user-level applications

You can program an application via JTAG using `tockloader`: 
 
First clone libtock-c, more instructions [here](https://github.com/tock/libtock-c).

Note: We currently use the `nrf52dk` (until tockloader supports) board alias for the Particle Boron to flash using the tockloader. 

```bash
$ git clone git@github.com:tock/libtock-c.git  
$ cd libtock-c/examples/<app>
$ make
$ tockloader install --board nrf52dk --jlink blink/build/blink.tab
```
You should see the rgb-led light show on the board now!

## Board Reset

A reset on the baord can be performed by the **_RESET** push button, to reboot the kernel. 

## Console output

Console is interfaced using `USB cdc_acm`, so additional hardware is not required. Once the kernel is flashed, simply open up a serial port

```bash
$ screen /dev/ttyACM0 115200 
.
.
.
> tock$ help
> Valid commands are: help status list stop start fault process kernel
```

OR

```bash
$ tockloader listen
.
.
.
[INFO   ] No device name specified. Using default name "tock".
[INFO   ] Using "/dev/ttyACM0 - Boron - TockOS".
[INFO   ] Listening for serial output.

tock$ help
Welcome to the process console.
Valid commands are: help status list stop start fault process kernel
tock$ list
 PID    Name                Quanta  Syscalls  Dropped Upcalls  Restarts    State  Grants
  0	buttons                  0        22                0         0  Yielded    1/11
```

## Tested LibTock-C Samples

```
* adc                       --  [OK]
* ble_advertising           --  [OK]
* ble_passive_scanning      --  [OK]
* blink                     --  [OK]
* buttons                   --  [OK]
* c_hello                   --  [OK]
```
## Currently Unsupported Board Features

* LTE [u-blox SARA-R410M-02B or R410M-03] or 2G/3G [u-blox SARA-U201 (2G/3G)] modem(s).
* PMIC/Fuel Gauge (I2C device).
