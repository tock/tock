NXP S32G3 SAIL Tock board
=========================

This target name uses SAIL to mean “Safety Island”; it is not the name of a
publicly discoverable NXP board product. It is a bring-up target for the Main
SoC S32G3 M7_0 with current support focused on the safety-island kernel.
Future capsules and board support are welcome when backed by hardware evidence
and normal Tock review.

## Build, image layout, and deployment

From the Tock repository root, build the normal kernel with:

```sh
make -C boards/nxp_s32g3_sail
```

The Cargo release binary is `target/thumbv7em-none-eabihf/release/nxp_s32g3_sail.bin`.
It is linked for and must be loaded at `0x34200000`. Platform tooling must
perform the Cortex-M vector fetch from that address: word 0 supplies the
initial stack pointer and word 1 supplies the Thumb reset entry.
The linker reserves `0x34220000..0x3429ffff` for TBF applications.
Writable sections start in L2 SRAM at `0x34000000` shared between kernel and app.
L2 SRAM is `0x34000000..0x341fffff`.
DTCM is deliberately unused: 56 KiB cannot hold the writable image without specializing Tock's default layout.

A raw kernel/application image can be constructed as follows:

```sh
KERNEL_BIN=target/thumbv7em-none-eabihf/release/nxp_s32g3_sail.bin
APP_TBF=/path/to/application.tbf
COMBINED_BIN=/path/to/nxp_s32g3_sail-with-app.bin
cat -- "$KERNEL_BIN" "$APP_TBF" > "$COMBINED_BIN"
cmp -s -n "$(wc -c < "$KERNEL_BIN")" "$KERNEL_BIN" "$COMBINED_BIN" \
  && echo "combined image begins with the kernel binary"
```

The normal kernel BIN excludes the `.apps` section; its 128 KiB kernel interval
places the appended TBF at `0x34220000`. This verifies image construction only.

The Tock repository does not currently provide a programmer, loader, signing
flow, or `make flash` target for this hardware. The image-construction command
above verifies the raw artifact only; deployment requires platform tooling
outside this repository.

## UART topology and manual receive procedure

- LF0 is the debug and process console: TX `PC9` (MSCR41), RX `PC10`, nominal
  `115200` baud.
- LF1 is the userspace console: TX `PC8`, RX `PC4`, nominal `921600` baud.
- LINFlexD implements asynchronous 8N1 buffer HIL only. Word, DMA,
  wider-word, parity, additional-stop-bit, and flow-control modes are
  unsupported. Polling output is reserved for panic and early boot.

## Validation matrix

| Area | Status | Evidence / limitation |
|---|---|---|
| LINFlexD UART | functional HIL-tested | Manual host tests verify 8N1 and baud; automated test suite verifies LF1 TX plus immediate TX/RX abort contracts. LF1 RX success is unproven. |
| STM_1 | functional HIL-tested | test suite verifies minimum/10 ms/wrap/disarm plus 100 sequential one-second callbacks; source arithmetic is 516419 Hz and corrected hardware deltas are +3.873 ppm. |
| M7 clocks | boot-path hardware-exercised |  A53 cooperation is unsupported. |
| MC_ME / MSCM | boot-path hardware-exercised | Board enables M7 partitions and routes LF0, LF1, and STM1 IRQs. |
| SIUL2 pinmux | boot-path hardware-exercised | GPIO helpers are preparatory-only; no GPIO HIL or interrupt support. |
| XRDC_0 policy patching | boot-path hardware-exercised | |
| XRDC_1 standby-SRAM / SSRAMC | preparatory-only | Policy grant and ECC initialization are retained; no functional consumer is claimed. |
| Reset and memory layout | boot-path hardware-exercised | |
| SWT watchdog | boot-path hardware-exercised | Board explicitly leaves hardware watchdog disabled; `DisabledWatchdog` is a no-op, not protection. |
