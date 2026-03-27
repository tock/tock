STM32 Nucleo development board with STM32U545RE-Q MCU (Skeleton)
=================================================================

This board directory is a skeleton to bootstrap STM32U545 support in Tock.

Current status:
- Board crate exists and is wired into the workspace.
- Chip crate exists as a placeholder (`chips/stm32u545`).
- Peripheral bring-up in `src/main.rs` is intentionally minimal.

Before this can boot on hardware, you must implement:
1. STM32U545 chip support (NVIC, clock tree, GPIO, USART, timer, EXTI).
2. Board pin mux for the selected UART/LED/button pins.
3. Real memory map in `chip_layout.ld`.
4. Working flash/debug configuration for your probe setup.

Build
-----

```bash
make -C boards/nucleo_u545re_q
```

Flash (template)
----------------

```bash
make -C boards/nucleo_u545re_q flash
```

The OpenOCD board script in the Makefile is a placeholder and may need to be
changed for your setup.
