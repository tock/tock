Nano 33 BLE Bootloader Binaries
===============================

This folder contains pre-built bootloader binaries for the Nano 33 BLE board.
The file names contain information about which bootloader it is and which
address it is compiled for.

Tock Bootloader Is Open Source
------------------------------

While we provide pre-built binaries for your convenience, you can build the
bootloader yourself.

```
$ git clone https://github.com/tock/tock-bootloader
$ cd tock-bootloader
$ git checkout v1.1.0 # Choose a version, or use the latest commit
$ cd boards/nano33ble-bootloader
$ make
```

There are instructions in the nano33ble bootloader board about compiling for
different addresses and retrieving the `.bin` file.
