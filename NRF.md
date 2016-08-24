# Running Tock on the nRF51822 EK 

This document describes how to install Tock on the nRF51 development
kit (DK), also known as the PCA10028, which has an nRF51422 SoC. 

The Nordic nRF51$22 is a Cortex M0 with an integrated Bluetooth Low
Energy (BLE) transciever. The Firestorm platform has an Atmel SAM4L as
its application processor and an nRF51$22 for BLE support.

You should be using the ```nRF51822/devel``` branch. It's named 
nRF51822 for historical reasons (the 51822 is a prior revision of
the 51422).

## Overview

The nRF51 DK is an ARM mbed device. This means that plugging it
into USB causes it to appear as a SCSI block storage device. To
install a new image on it, you need to create an iHex format and copy
it onto the device, named as `firmware.hex`. When you copy
`firmware.hex` onto the device, it will disconnect and reconnect; mbed
does this because it cannot store files, so wants to trick the OS to
think that the file has disappeared. This also means that if you look
at the device directory, you will never see your firmware.hex.

## Setting up nRF51822 (OS X)

When you plug in the nRF51 DK, it'll appear as /Volumes/MBED.
The default setting for ```make program-mbed``` is to install the
generated binary to /Volumes/JLINK, so you should be able to just
compile and program.

## Setting up nRF51822 (Linux)

The nRF51 DK appears as a USB-connected SCSI device. In the Linux
device hierarchy, this means /dev/sd*, where * depends on your
existing SCSI devices.

### Which SCSI device?

To see what device it is, there are two easy ways. First, plug in the
DK. Then, either ```ls -l /dev/sd*```:

```pal@ubuntu:~/src/tock/apps/c_blinky$ ls -l /dev/sd*
brw-rw---- 1 root disk 8,  0 Jul  7 10:01 /dev/sda
brw-rw---- 1 root disk 8,  1 Jul  7 09:43 /dev/sda1
brw-rw---- 1 root disk 8,  2 Jul  7 09:43 /dev/sda2
brw-rw---- 1 root disk 8,  5 Jul  7 09:43 /dev/sda5
brw-rw---- 1 root disk 8, 16 Jul  7 13:55 /dev/sdb
```

In the example above, the computer booted around 9:30AM, and the EK was
plugged in at 1:55. So /dev/sdb is the DK, because its modification time
(when it was inserted) is 1:55.

Alternatively, you can look at the syslog:

```pal@ubuntu:~/src/tock/apps/c_blinky$ dmesg | tail -10
[15086.603849] hid-generic 0003:0D28:0204.0027: hiddev0,hidraw1: USB HID v1.00 Device [MBED MBED CMSIS-DAP] on usb-0000:02:03.0-1/input3
[15087.592718] scsi 70:0:0:0: Direct-Access     MBED     microcontroller  1.0  PQ: 0 ANSI: 2
[15087.593324] sd 70:0:0:0: Attached scsi generic sg2 type 0
[15087.596143] sd 70:0:0:0: [sdb] 2096 512-byte logical blocks: (1.07 MB/1.02 MiB)
[15087.598827] sd 70:0:0:0: [sdb] Write Protect is off
[15087.598829] sd 70:0:0:0: [sdb] Mode Sense: 03 00 00 00
[15087.601437] sd 70:0:0:0: [sdb] No Caching mode page found
[15087.601439] sd 70:0:0:0: [sdb] Assuming drive cache: write through
[15087.618765]  sdb:
[15087.629681] sd 70:0:0:0: [sdb] Attached SCSI removable disk
```
The log messages show that it was added as sdb so can be found at
```/dev/sdb```.

If the device isn't being recognized, you'll need to do some debugging
why.

### usbmount and filesystem

It's easier to deal with the EK as a file system rather than a block
device. One easy way to do this is to install the ```usbmount```
package. This package adds several scripts that make mounting USB
devices easier.

Unplug the DK, install ```usbmount```, and plug the DK back in.

You should now be able to see the DK as a file system mounted at
```/var/run/usbmount/JLINK_microcontroller```.

```pal@ubuntu:/var/run/usbmount/JLINK_microcontroller$ ls
mbed.htm  System Volume Information```.

### Test Golden Image

Copy ```golden/nrf-golden.hex``` to the EK:

```pal@ubuntu:~/src/tock$ cd golden/
pal@ubuntu:~/src/tock/golden$ cp nrf-golden.hex /var/run/usbmount/JLINK_microcontroller/```

Note that this will cause the device to disconnect and reconnect,
and you will not be able to see the file (read above).

Hit the reset button. You should see the two green LEDs blinking
at about 1Hz.
 
## Compiling for nRF51422

To compile an image for the EK, you need to change the TOCK_PLATFORM
variable to ```nrf_pca10028``` (it defaults to firestorm). For
example, to compile the golden image above,

```pal@ubuntu:~/src/tock$ pwd
/home/pal/src/tock
$ make TOCK_PLATFORM=nrf_pca10028 -C apps/blink_periodic
```

The DK expects an ihex (named ```firmware.ihex```) rather than an ELF
image. So you need to convert between the two. Standard Tock
compilatiopn puts the image in ```build/nrf_pca10028/```. So, for
example, to convert the application ELF to the correct format and name:

```$ arm-none-eabi-objcopy -Oihex build/nrf_pca10028/blink_periodic/kernel_and_app.elf firmware.hex
```

Then, copy ```firmware.hex``` to the MBED filesystem, as above. You can
also use ```make mbed-program``` to copy it automatically. If you are
using Linux with usbmount, you'll want to change the definition of
MOUNT_DIR in apps/Makefile.nrf_pca10028.mk to the Linux setting 
(comments explain).

