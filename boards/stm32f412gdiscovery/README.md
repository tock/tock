STM32F4 Discovery Kit with STM32F412G MCU
======================================================

For more details [visit STM32F412G Discovery Kit
website](https://www.st.com/en/evaluation-tools/32f412gdiscovery.html).

## Flashing the kernel

The kernel can be programmed using OpenOCD. `cd` into `boards/discovery_f412g`
directory and run:

```bash
$ make flash

(or)

$ make flash-debug
```

> **Note:** Unlike other Tock platforms, the default kernel image for this
> board will clear flashed apps when the kernel is loaded. This is to support
> the non-tockloader based app flash procedure below. To preserve loaded apps,
> comment out the `APP_HACK` variable in `src/main.rs`.

## Flashing app

Apps are built out-of-tree. Once an app is built, you can add the path to it in the Makefile (APP variable), then run:
```bash
$ make program
```

or you can define the APP variable in the command line

```bash
$ make program APP=path_to_tbf
```


## OpenOCD Note
The release version of openocd does not fully support stm32412g discovery kit. Uploading seems to work
with the setup for nucelo429zi. The openocd.cfg file contains both setups, one being commented.

To install an openocd that full supports stm32f412g you have to build openocd.

```bash
$ git clone --recursive https://git.code.sf.net/p/openocd/code openocd-code
$ cd openocd-code
$ git fetch http://openocd.zylin.com/openocd refs/changes/21/4321/7 && git cherry-pick FETCH_HEAD
$ ./bootstrap
$ ./configure --disable-werror
$ make
# optionally use sudo make install
```

> Please note that you may have some conflicts in a file containing a list of 
> sources when patching. Accept both changes.
