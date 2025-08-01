# Tock Board Runner

This is a Rust program that uses [`rexpect`][rexpect] to test Tock on different
boards.  The goal of this is to automate the testing, currently it still
requires manual steps though.

## Supported Boards

### OpenTitan

This can be used to perform Tock release testing on the OpenTitan board.

This assumes that the OpenTitan serial console is available on the machines
first serial port (`/dev/ttyUSB0` for Unix systems). The tests can be run from
the top level of the Tock direcotry with the following command

```shell
OPENTITAN_TREE=<opentitan_repo> LIBTOCK_C_TREE=<libtock_c_repo> TARGET=earlgrey_nexysvideo make board-release-test
```

Where `opentitan_repo` and `libtock_c_repo` point to the top level directory
of the corresponding repos. You will need to make sure that the OpenTitan
spiflash command has been built in the OpenTitan repo and that the c apps have
been built in the libtock-c repo.

### Redboard Artemis Nano

This can be used to perform Tock release testing on the Sparkfun Redboard
Artemis Nano board.

This assumes that the ARtemis Nano serial console is available on the machines
first serial port (`/dev/ttyUSB0` for Unix systems). The tests can be run from
the top level of the Tock directory with the following command

```shell
LIBTOCK_C_TREE=<libtock_c_repo> TARGET=artemis_nano make board-release-test
```

Where `libtock_c_repo` points to the top level directory of the corresponding
libtock-c repo.

### ESP32-C3

This can be used to perform Tock release testing on the Espressif ESP32-C3
board.

This assumes that the ESP32-C3 serial console is available on the machine's
first serial port (`/dev/ttyUSB0` for Unix systems). The tests can be run from
the top level of the Tock directory with the following command

```shell
LIBTOCK_C_TREE=<libtock_c_repo> TARGET=esp32_c3 make board-release-test
```

Where `libtock_c_repo` points to the top level directory of the corresponding
libtock-c repo.


[rexpect]: https://docs.rs/rexpect/latest/rexpect/
