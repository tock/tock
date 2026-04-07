# NUCLEO-U545RE-Q

The [NUCLEO-U545RE-Q](https://www.st.com/en/evaluation-tools/nucleo-u545re-q.html) 
is a development board based on the STM32U545RE microcontroller.

## Building

To build the kernel for this board, run `make` in this directory:

```bash
cd boards/nucleo_u545re_q
make
```

## Flashing

This board can be flashed using `probe-rs`.

## Console

The kernel console is available on USART1 via the ST-LINK USB connection at 
115,200 baud.
