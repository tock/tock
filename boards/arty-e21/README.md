Approved for public release. Distribution is unlimited.

This material is based upon work supported by the Under Secretary of Defense for Research and Engineering under Air Force Contract No. FA8702-15-D-0001. Any opinions, findings, conclusions or recommendations expressed in this material are those of the author(s) and do not necessarily reflect the views of the Under Secretary of Defense for Research and Engineering.

© 2019 Massachusetts Institute of Technology.

The software/firmware is provided to you on an As-Is basis

Delivered to the U.S. Government with Unlimited Rights, as defined in DFARS Part 252.227-7013 or 7014 (Feb 2014). Notwithstanding any copyright notice, U.S. Government rights in this work are defined by DFARS 252.227-7013 or DFARS 252.227-7014 as detailed above. Use of this work other than as specifically authorized by the U.S. Government may violate any copyrights that exist in this work.

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

