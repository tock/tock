SiFive E21 Core on ARTY FPGA Board
=================

- SiFive E21 Core IP v1p0
- Digilent Arty Artix-7 35T Development Board


Required Tools
--------------

- OpenOCD


Setup
-----

The first step is to flash the SiFive E21 core on to the FPGA. To do this,
connect the ARTY board over USB and run:

```
$ make flash-e21
```

It may display that many memory locations are incorrect. This didn't seem
to matter when I tried this.

After that has finished, press the `PROG` red button on the top left of the
board. After a few seconds, one of the RGB LEDs should start pulsing colors.
It also prints over the serial connection. To see that, run:

```
$ tockloader listen
```

and select the option with the larger number.


Programming
-----------

To load a new kernel on to the board, run:

```
$ make flash
```

