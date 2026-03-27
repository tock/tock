# ST Micro STM32U545 MCU (Skeleton)

This crate is a skeleton for STM32U545 support in Tock.

Current status:
- Architecture placeholder (`cortexm33`) is present.
- Peripheral drivers and interrupt mapping are not implemented yet.

Next steps:
1. Add the STM32U5 family shared crate (`chips/stm32u5xx`) or implement
   STM32U545 drivers directly here.
2. Implement the chip type and interrupt service.
3. Add NVIC constants and the IRQ table.
4. Add RCC/clock, GPIO, USART, EXTI, timer, and other required peripherals.
