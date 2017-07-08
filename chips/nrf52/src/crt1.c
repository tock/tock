/*
 * This file has been relicensed by the original author from GPLv3 to Apache 2.0
 * The original version of the file, under GPL can be found at
 * https://github.com/SoftwareDefinedBuildings/stormport/blob/rebase0/tos/platforms/storm/stormcrt1.c
 * 
 * Copyright 2016, Michael Andersen <m.andersen@eecs.berkeley.edu>
 */

/* https://github.com/NordicSemiconductor/nrf52-hardware-startup-hands-on/blob/master/pca10040/s132/arm5_no_packs/RTE/Device/nRF52832_xxAA/arm_startup_nrf52.s */
/* https://github.com/NordicSemiconductor/nRF52-ble-app-lbs/blob/master/pca10040/s132/arm5_no_packs/RTE/Device/nRF52832_xxAA/system_nrf52.c */

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
     2015-12-11 nRF52832 Errata v1.2
     http://infocenter.nordicsemi.com/pdf/nRF52832_Errata_v1.2.pdf 
  */

    /* Workaround for Errata 12 "COMP: Reference ladder not correctly callibrated" found at the Errate doc */
        *(volatile uint32_t *)0x40013540 = (*(uint32_t *)0x10000324 & 0x00001F00) >> 8;
    
    /* Workaround for Errata 16 "System: RAM may be corrupt on wakeup from CPU IDLE" found at the Errata doc */
        *(volatile uint32_t *)0x4007C074 = 3131961357ul;

    /* Workaround for Errata 31 "CLOCK: Calibration values are not correctly loaded from FICR at reset" found at the Errata doc */
        *(volatile uint32_t *)0x4000053C = ((*(volatile uint32_t *)0x10000244) & 0x0000E000) >> 13;

    /* Workaround for Errata 32 "DIF: Debug session automatically enables TracePort pins" found at the Errata doc
        CoreDebug->DEMCR &= ~CoreDebug_DEMCR_TRCENA_Msk; */

    /* Workaround for Errata 36 "CLOCK: Some registers are not reset when expected" found at the Errata doc
        NRF_CLOCK->EVENTS_DONE = 0;
        NRF_CLOCK->EVENTS_CTTO = 0;
        NRF_CLOCK->CTIV = 0;
    }
    */

    /* Workaround for Errata 37 "RADIO: Encryption engine is slow by default" found at the Errata document doc */
        *(volatile uint32_t *)0x400005A0 = 0x3;

    /* Workaround for Errata 57 "NFCT: NFC Modulation amplitude" found at the Errata doc */
        *(volatile uint32_t *)0x40005610 = 0x00000005;
        *(volatile uint32_t *)0x40005688 = 0x00000001;
        *(volatile uint32_t *)0x40005618 = 0x00000000;
        *(volatile uint32_t *)0x40005614 = 0x0000003F;

    /* Workaround for Errata 66 "TEMP: Linearity specification not met with default settings" found at the Errata doc
        NRF_TEMP->A0 = NRF_FICR->TEMP.A0;
        NRF_TEMP->A1 = NRF_FICR->TEMP.A1;
        NRF_TEMP->A2 = NRF_FICR->TEMP.A2;
        NRF_TEMP->A3 = NRF_FICR->TEMP.A3;
        NRF_TEMP->A4 = NRF_FICR->TEMP.A4;
        NRF_TEMP->A5 = NRF_FICR->TEMP.A5;
        NRF_TEMP->B0 = NRF_FICR->TEMP.B0;
        NRF_TEMP->B1 = NRF_FICR->TEMP.B1;
        NRF_TEMP->B2 = NRF_FICR->TEMP.B2;
        NRF_TEMP->B3 = NRF_FICR->TEMP.B3;
        NRF_TEMP->B4 = NRF_FICR->TEMP.B4;
        NRF_TEMP->B5 = NRF_FICR->TEMP.B5;
        NRF_TEMP->T0 = NRF_FICR->TEMP.T0;
        NRF_TEMP->T1 = NRF_FICR->TEMP.T1;
        NRF_TEMP->T2 = NRF_FICR->TEMP.T2;
        NRF_TEMP->T3 = NRF_FICR->TEMP.T3;
        NRF_TEMP->T4 = NRF_FICR->TEMP.T4;
    }
    */

    /* Workaround for Errata 108 "RAM: RAM content cannot be trusted upon waking up from System ON Idle or System OFF mode" found at the Errata doc */
        *(volatile uint32_t *)0x40000EE4 = *(volatile uint32_t *)0x10000258 & 0x0000004F;

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

