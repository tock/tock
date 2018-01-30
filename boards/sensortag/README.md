## Platform specific instructions

### Flashing
Download and use [uniflash](http://processors.wiki.ti.com/index.php/Category:CCS_UniFlash) to flash.

### Debugging
You need to use openocd together with gdb in order to debug the sensortag board using JTAG. Once flashed, simply launch openocd

```bash
$> openocd -f jtag/sensortag_openocd.cfg
```

And then launch gdb

```bash
$> arm-none-eabi-gdb -x jtag/gdbinit
```

and it will automatically connect to, and reset, the board.
