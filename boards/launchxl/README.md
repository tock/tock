## Platform specific instructions

### Flashing
Download and use [uniflash](http://processors.wiki.ti.com/index.php/Category:CCS_UniFlash) to flash. Follow the guide
[here](http://processors.wiki.ti.com/index.php/UniFlash_v4_Quick_Guide#Standalone_Command_line_tool) in order to generate
a standalone command line tool to ease the flashing process.

The standalone CLI has been extracted, set an environment variable named `UNIFLASH_CLI_BASE` in your shell profile:

```bash
$> echo UNIFLASH_CLI_BASE="<path to extracted uniflash CLI>" >> ~/.bash_profile
$> source ~/.bash_profile
```

Now you're able to use the Makefile targets `flash` in order to load the kernel onto the launchxl board.

```bash
$> make flash       # make and flash the kernel
```

### Flashing apps
You can flash apps by navigating to their directory, and by invoking `make flash` once again.

### Debugging
You need to use openocd together with gdb in order to debug the launchxl board using JTAG. However, you'll need to build OpenOCD with extra applied patches until the next version has been released. 

Clone the repository and apply the patches:

```bash
$> git clone https://git.code.sf.net/p/openocd/code openocd 
$> cd openocd
$> git pull http://openocd.zylin.com/openocd refs/changes/22/4322/2 
$> git pull http://openocd.zylin.com/openocd refs/changes/58/4358/1
```

Once cloned and the patches has been applied, read the readme in order to build and install openocd.

Once flashed, launch openocd with the configuration specified at jtag/openocd.cfg:

```bash
$> openocd -f jtag/openocd.cfg
```

And then launch gdb

```bash
$> arm-none-eabi-gdb -x jtag/gdbinit
```

and it will automatically connect to the board.

**NOTE**: There is currently a problem using the `cortex_m SYSRESETREQ` command in openocd in order to reset the board. This is also the default way we want to reset..
