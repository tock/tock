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
* FDTI/Serial UART converter (Optional for debugging/Console)

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

Console is interfaced with UART. On the left side of the Particle Boron, you can see the marked UART RX/TX pins. Connect a serial converter to the appropriate pins (`UART1_RX->CONVERTER_TX` & `UART1_TX->CONVERTER_RX`), see the figure above for the pin-out.

If you are on a linux machine, it's a good idea to monitor the kernel message buffer with `dmesg -w` when plugging the serial converter in. This can quickly tell us which USB device to open.

```bash
$ dmesg -w

[26244.142183] usb 3-4: new full-speed USB device number 50 using xhci_hcd
[26244.295996] usb 3-4: New USB device found, idVendor=0403, idProduct=6001, bcdDevice= 6.00
[26244.296001] usb 3-4: New USB device strings: Mfr=1, Product=2, SerialNumber=3
[26244.296004] usb 3-4: Product: FT232R USB UART
[26244.296006] usb 3-4: Manufacturer: FTDI
[26244.296007] usb 3-4: SerialNumber: A50285BI
[26244.302027] ftdi_sio 3-4:1.0: FTDI USB Serial Device converter detected
[26244.302055] usb 3-4: Detected FT232R
[26244.309214] usb 3-4: FTDI USB Serial Device converter now attached to ttyUSB0
```
Based on the last kernel message, we are mapped to `ttyUSB0`, now, we can access the console with:
```bash
$ screen /dev/ttyUSB0 115200
.
..
Particle Boron: Initialization complete. Entering main loop
tock$
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
