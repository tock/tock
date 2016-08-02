/*
 * This file has been relicensed by the original author from GPLv3 to Apache 2.0
 * The original version of the file, under GPL can be found at
 * https://github.com/SoftwareDefinedBuildings/
 *     stormport/blob/rebase0/tos/platforms/storm/stormcrt1.c
 * 
 * Copyright 2016, Michael Andersen <m.andersen@eecs.berkeley.edu>
 */

#include <stdint.h>
#include <string.h>

#include "peripheral_interrupts.h"

/* Symbols defined in the linker file */
extern uint32_t _estack;
extern uint32_t _etext;
extern uint32_t _szero;
extern uint32_t _ezero;
extern uint32_t _srelocate;
extern uint32_t _erelocate;

void Dummy_Handler(void)
{
	while (1) {
	}
}

void reset_handler(void);
void NMI_Handler(void) __attribute__ ((weak, alias("Dummy_Handler")));
void HardFault_Handler(void)
    __attribute__ ((weak, alias("Dummy_Handler")));
void SVC_Handler(void) __attribute__ ((weak, alias("Dummy_Handler")));
void PendSV_Handler(void) __attribute__ ((weak, alias("Dummy_Handler")));
void SysTick_Handler(void) __attribute__ ((weak, alias("Dummy_Handler")));
PERIPHERAL_INTERRUPT_HANDLERS

typedef void (*interrupt_function_t) (void);

__attribute__ ((section(".vectors")))
interrupt_function_t interrupt_table[] = {
	(interrupt_function_t) (&_estack),
	reset_handler,
	NMI_Handler,
	HardFault_Handler,
	0, 0, 0, 0, 0, 0, 0,	/* Reserved */
	SVC_Handler,
	0, 0,			/* Reserved */
	PendSV_Handler,
	SysTick_Handler,
	PERIPHERAL_INTERRUPT_VECTORS
};

void init(void)
{
	uint32_t *pSrc, *pDest;

	/* Apply early initialization workarounds for anomalies documented on
	 * nRF51822-PAN v2.4. Note that they have been validated only for xxAA
	 * variant. For other variants, please refer to the applicable
	 * nRF51822-PAN. */

	/* Power on RAM blocks manually (PAN #16). Note that xxAA/xxAB variants
	 * have only two RAM blocks. For xxAC, change to 0x0000000F. */
	*((uint32_t volatile * ) 0x40000524) = 0x00000003;

	/* Setup peripherals manually (PAN #26) */
	*((uint32_t volatile * ) 0x40000504) = 0xC007FFDF;
	*((uint32_t volatile * ) 0x40006C18) = 0x00008000;

	/* Move the relocate segment
	 * This assumes it is located after the
	 * text segment, which is where the storm
	 * linker file puts it
	 */
	pSrc = &_etext;
	pDest = &_srelocate;

	if (pSrc != pDest) {
		for (; pDest < &_erelocate;) {
			*pDest++ = *pSrc++;
		}
	}

	/* Clear the zero segment */
	for (pDest = &_szero; pDest < &_ezero;) {
		*pDest++ = 0;
	}
}

