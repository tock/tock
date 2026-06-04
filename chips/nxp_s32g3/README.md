NXP S32G3 Chip Crate
====================

This crate provides S32G3 Cortex-M7 peripheral support for the SAIL board's
M7_0. Source presence does not imply hardware validation.

## Memory and deployment context

The SAIL board links the kernel at `0x34200000`. Writable kernel sections use
L2 SRAM from `0x34000000` through `0x341fffff`; DTCM is deliberately unused
because 56 KiB cannot hold the writable image without specializing Tock's
default layout. The board README documents normal build, generic SRAM image
construction, and the ephemeral XMODEM workflow. An unsigned Tock payload must
not be deployed to NOR; no signed/containerized NOR flow is supplied here.

## UART contract

LINFlexD supports asynchronous 8N1 buffer HIL only. Polling output is limited
to panic and early boot. Word, DMA, wider-word, parity, additional-stop-bit,
and flow-control modes are unsupported.

On the SAIL board, LF0 is the debug/process console on TX `PC9` (MSCR41) and
RX `PC10`, nominal `115200` baud. LF1 is the userspace console on TX `PC8` and
RX `PC4`, nominal `921600` baud. At a 40 MHz LIN_BAUD_CLK, LF1 divisor
`2 + 11/16` yields `930232` baud, an `8632`-baud error below 1%.

## Validation matrix

| Area | Status | Evidence / limitation |
|---|---|---|
| LINFlexD UART | functional HIL-tested | S03 host tests cover 8N1 and baud; S10 hardware verifies LF1 TX plus immediate TX/RX aborts. LF1 RX success is unproven. |
| STM_1 | functional HIL-tested | S12 verifies minimum/10 ms/wrap/disarm plus 100 sequential one-second callbacks; source arithmetic is 516419 Hz and corrected hardware deltas are +3.873 ppm. |
| M7 clocks | boot-path hardware-exercised | S06/S10 exercise retained M7 clocks; CM7_0 is `396610169` Hz. A53 remains reset and A53 cooperation is unsupported. |
| MC_ME / MSCM | boot-path hardware-exercised | M7 partition enable and LF0/LF1/STM1 routing execute at startup; S04 has typed host tests. |
| SIUL2 pinmux | boot-path hardware-exercised | LF0/LF1 pinmux is exercised by S10; S12 proves LF0 process-console `help\r` receive. GPIO helpers are preparatory-only; no GPIO HIL or interrupt support. |
| XRDC_0 policy patching | boot-path hardware-exercised | Required at startup; S07 has typed register-backed/DMB coverage. |
| XRDC_1 standby-SRAM / SSRAMC | preparatory-only | Retained policy/ECC setup; no functional consumer is claimed. |
| Reset and memory layout | boot-path hardware-exercised | S08 map/listing and normal/test boot paths prove vector/reset placement. |
| SWT watchdog | boot-path hardware-exercised | Explicitly disabled by the board; no watchdog protection or servicing is provided. |
| Generic/uninvoked drivers | compiled | No hardware claim for A53, DDR, ACCEL, CAN, SPI, QSPI, USDHC, Ethernet, or unused channels. |
