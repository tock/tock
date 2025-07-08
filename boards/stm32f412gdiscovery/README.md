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
