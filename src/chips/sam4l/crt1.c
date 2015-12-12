/*
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 * 
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 * 
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <http://www.gnu.org/licenses/>.
 * 
 * Copyright 2014, Michael Andersen <m.andersen@eecs.berkeley.edu>
 */

#include <stdint.h>
#include <string.h>
 
// Symbols defined in the storm linker file
extern uint32_t _sfixed;
extern uint32_t _efixed;
extern uint32_t _etext;
extern uint32_t _srelocate;
extern uint32_t _erelocate;
extern uint32_t _szero;
extern uint32_t _ezero;
extern uint32_t _sstack;
extern uint32_t _estack;

int main(void);

void Dummy_Handler(void)
{
	while (1) {
	}
}

void Reset_Handler(void);
void NMI_Handler(void)          __attribute__ ((weak, alias("Dummy_Handler")));
void HardFault_Handler(void)    __attribute__ ((weak, alias("Dummy_Handler")));
void MemManage_Handler(void)    __attribute__ ((weak, alias("Dummy_Handler")));
void BusFault_Handler(void)     __attribute__ ((weak, alias("Dummy_Handler")));
void UsageFault_Handler(void)   __attribute__ ((weak, alias("Dummy_Handler")));
void SVC_Handler(void)          __attribute__ ((weak, alias("Dummy_Handler")));
void DebugMon_Handler(void)     __attribute__ ((weak, alias("Dummy_Handler")));
void PendSV_Handler(void)       __attribute__ ((weak, alias("Dummy_Handler")));
void SysTick_Handler(void)      __attribute__ ((weak, alias("Dummy_Handler")));

/* Peripherals handlers */
void ABDACB_Handler(void)       __attribute__ ((weak, alias("Dummy_Handler")));
void ACIFC_Handler(void)        __attribute__ ((weak, alias("Dummy_Handler")));
void ADCIFE_Handler(void)       __attribute__ ((weak, alias("Dummy_Handler")));
void AESA_Handler(void)         __attribute__ ((weak, alias("Dummy_Handler")));
void AST_ALARM_Handler(void)    __attribute__ ((weak, alias("Dummy_Handler")));
void AST_CLKREADY_Handler(void) __attribute__ ((weak, alias("Dummy_Handler")));
void AST_OVF_Handler(void)      __attribute__ ((weak, alias("Dummy_Handler")));
void AST_PER_Handler(void)      __attribute__ ((weak, alias("Dummy_Handler")));
void AST_READY_Handler(void)    __attribute__ ((weak, alias("Dummy_Handler")));
void BPM_Handler(void)          __attribute__ ((weak, alias("Dummy_Handler")));
void BSCIF_Handler(void)        __attribute__ ((weak, alias("Dummy_Handler")));
void CATB_Handler(void)         __attribute__ ((weak, alias("Dummy_Handler")));
void CRCCU_Handler(void)        __attribute__ ((weak, alias("Dummy_Handler")));
void DACC_Handler(void)         __attribute__ ((weak, alias("Dummy_Handler")));
void EIC_1_Handler(void)        __attribute__ ((weak, alias("Dummy_Handler")));
void EIC_2_Handler(void)        __attribute__ ((weak, alias("Dummy_Handler")));
void EIC_3_Handler(void)        __attribute__ ((weak, alias("Dummy_Handler")));
void EIC_4_Handler(void)        __attribute__ ((weak, alias("Dummy_Handler")));
void EIC_5_Handler(void)        __attribute__ ((weak, alias("Dummy_Handler")));
void EIC_6_Handler(void)        __attribute__ ((weak, alias("Dummy_Handler")));
void EIC_7_Handler(void)        __attribute__ ((weak, alias("Dummy_Handler")));
void EIC_8_Handler(void)        __attribute__ ((weak, alias("Dummy_Handler")));
void FREQM_Handler(void)        __attribute__ ((weak, alias("Dummy_Handler")));
void GPIO_0_Handler(void)       __attribute__ ((weak, alias("Dummy_Handler")));
void GPIO_1_Handler(void)       __attribute__ ((weak, alias("Dummy_Handler")));
void GPIO_10_Handler(void)      __attribute__ ((weak, alias("Dummy_Handler")));
void GPIO_11_Handler(void)      __attribute__ ((weak, alias("Dummy_Handler")));
void GPIO_2_Handler(void)       __attribute__ ((weak, alias("Dummy_Handler")));
void GPIO_3_Handler(void)       __attribute__ ((weak, alias("Dummy_Handler")));
void GPIO_4_Handler(void)       __attribute__ ((weak, alias("Dummy_Handler")));
void GPIO_5_Handler(void)       __attribute__ ((weak, alias("Dummy_Handler")));
void GPIO_6_Handler(void)       __attribute__ ((weak, alias("Dummy_Handler")));
void GPIO_7_Handler(void)       __attribute__ ((weak, alias("Dummy_Handler")));
void GPIO_8_Handler(void)       __attribute__ ((weak, alias("Dummy_Handler")));
void GPIO_9_Handler(void)       __attribute__ ((weak, alias("Dummy_Handler")));
void HFLASHC_Handler(void)      __attribute__ ((weak, alias("Dummy_Handler")));
void IISC_Handler(void)         __attribute__ ((weak, alias("Dummy_Handler")));
void LCDCA_Handler(void)        __attribute__ ((weak, alias("Dummy_Handler")));
void PARC_Handler(void)         __attribute__ ((weak, alias("Dummy_Handler")));
void PDCA_0_Handler(void)       __attribute__ ((weak, alias("Dummy_Handler")));
void PDCA_1_Handler(void)       __attribute__ ((weak, alias("Dummy_Handler")));
void PDCA_10_Handler(void)      __attribute__ ((weak, alias("Dummy_Handler")));
void PDCA_11_Handler(void)      __attribute__ ((weak, alias("Dummy_Handler")));
void PDCA_12_Handler(void)      __attribute__ ((weak, alias("Dummy_Handler")));
void PDCA_13_Handler(void)      __attribute__ ((weak, alias("Dummy_Handler")));
void PDCA_14_Handler(void)      __attribute__ ((weak, alias("Dummy_Handler")));
void PDCA_15_Handler(void)      __attribute__ ((weak, alias("Dummy_Handler")));
void PDCA_2_Handler(void)       __attribute__ ((weak, alias("Dummy_Handler")));
void PDCA_3_Handler(void)       __attribute__ ((weak, alias("Dummy_Handler")));
void PDCA_4_Handler(void)       __attribute__ ((weak, alias("Dummy_Handler")));
void PDCA_5_Handler(void)       __attribute__ ((weak, alias("Dummy_Handler")));
void PDCA_6_Handler(void)       __attribute__ ((weak, alias("Dummy_Handler")));
void PDCA_7_Handler(void)       __attribute__ ((weak, alias("Dummy_Handler")));
void PDCA_8_Handler(void)       __attribute__ ((weak, alias("Dummy_Handler")));
void PDCA_9_Handler(void)       __attribute__ ((weak, alias("Dummy_Handler")));
void PEVC_OV_Handler(void)      __attribute__ ((weak, alias("Dummy_Handler")));
void PEVC_TR_Handler(void)      __attribute__ ((weak, alias("Dummy_Handler")));
void PM_Handler(void)           __attribute__ ((weak, alias("Dummy_Handler")));
void SCIF_Handler(void)         __attribute__ ((weak, alias("Dummy_Handler")));
void SPI_Handler(void)          __attribute__ ((weak, alias("Dummy_Handler")));
void TC00_Handler(void)        __attribute__ ((weak, alias("Dummy_Handler")));
void TC01_Handler(void)        __attribute__ ((weak, alias("Dummy_Handler")));
void TC02_Handler(void)        __attribute__ ((weak, alias("Dummy_Handler")));
void TC10_Handler(void)        __attribute__ ((weak, alias("Dummy_Handler")));
void TC11_Handler(void)        __attribute__ ((weak, alias("Dummy_Handler")));
void TC12_Handler(void)        __attribute__ ((weak, alias("Dummy_Handler")));
void TRNG_Handler(void)         __attribute__ ((weak, alias("Dummy_Handler")));
void TWIM0_Handler(void)        __attribute__ ((weak, alias("Dummy_Handler")));
void TWIM1_Handler(void)        __attribute__ ((weak, alias("Dummy_Handler")));
void TWIM2_Handler(void)        __attribute__ ((weak, alias("Dummy_Handler")));
void TWIM3_Handler(void)        __attribute__ ((weak, alias("Dummy_Handler")));
void TWIS0_Handler(void)        __attribute__ ((weak, alias("Dummy_Handler")));
void TWIS1_Handler(void)        __attribute__ ((weak, alias("Dummy_Handler")));
void USART0_Handler(void)       __attribute__ ((weak, alias("Dummy_Handler")));
void USART1_Handler(void)       __attribute__ ((weak, alias("Dummy_Handler")));
void USART2_Handler(void)       __attribute__ ((weak, alias("Dummy_Handler")));
void USART3_Handler(void)       __attribute__ ((weak, alias("Dummy_Handler")));
void USBC_Handler(void)         __attribute__ ((weak, alias("Dummy_Handler")));
void WDT_Handler(void)          __attribute__ ((weak, alias("Dummy_Handler")));

typedef void (*interrupt_function_t) (void);

__attribute__ ((section(".vectors"))) interrupt_function_t interrupt_table[] = 
{
	//The first few are defined in the Cortex M4 manual:
    //see: http://infocenter.arm.com/help/topic/com.arm.doc.dui0553a/DUI0553A_cortex_m4_dgug.pdf
    //Section 2.3.4
	(interrupt_function_t) (&_estack),
	Reset_Handler,
	NMI_Handler,
	HardFault_Handler,
	MemManage_Handler,
	BusFault_Handler,
	UsageFault_Handler,
	0, 0, 0, 0,        /* Reserved */
	SVC_Handler,
	DebugMon_Handler,
	0,                 /* Reserved  */
	PendSV_Handler,
	SysTick_Handler,

	//These are defined by ATMEL, see:
    //http://www.atmel.com/images/atmel-42023-arm-microcontroller-atsam4l-low-power-lcd_datasheet.pdf
    //Section 4.7
	HFLASHC_Handler,      // 0
	PDCA_0_Handler,       // 1
	PDCA_1_Handler,       // 2
	PDCA_2_Handler,       // 3
	PDCA_3_Handler,       // 4
	PDCA_4_Handler,       // 5
	PDCA_5_Handler,       // 6
	PDCA_6_Handler,       // 7
	PDCA_7_Handler,       // 8
	PDCA_8_Handler,       // 9
	PDCA_9_Handler,       // 10
	PDCA_10_Handler,      // 11
	PDCA_11_Handler,      // 12
	PDCA_12_Handler,      // 13
	PDCA_13_Handler,      // 14
	PDCA_14_Handler,      // 15
	PDCA_15_Handler,      // 16
	CRCCU_Handler,        // 17
	USBC_Handler,         // 18
	PEVC_TR_Handler,      // 19
	PEVC_OV_Handler,      // 20
	AESA_Handler,         // 21
	PM_Handler,           // 22
	SCIF_Handler,         // 23
	FREQM_Handler,        // 24
	GPIO_0_Handler,       // 25
	GPIO_1_Handler,       // 26
	GPIO_2_Handler,       // 27
	GPIO_3_Handler,       // 28
	GPIO_4_Handler,       // 29
	GPIO_5_Handler,       // 30
	GPIO_6_Handler,       // 31
	GPIO_7_Handler,       // 32
	GPIO_8_Handler,       // 33
	GPIO_9_Handler,       // 34
	GPIO_10_Handler,      // 35
	GPIO_11_Handler,      // 36
	BPM_Handler,          // 37
	BSCIF_Handler,        // 38
	AST_ALARM_Handler,    // 39
	AST_PER_Handler,      // 40
	AST_OVF_Handler,      // 41
	AST_READY_Handler,    // 42
	AST_CLKREADY_Handler, // 43
	WDT_Handler,          // 44
	EIC_1_Handler,        // 45
	EIC_2_Handler,        // 46
	EIC_3_Handler,        // 47
	EIC_4_Handler,        // 48
	EIC_5_Handler,        // 49
	EIC_6_Handler,        // 50
	EIC_7_Handler,        // 51
	EIC_8_Handler,        // 52
	IISC_Handler,         // 53
	SPI_Handler,          // 54
	TC00_Handler,         // 55
	TC01_Handler,         // 56
	TC02_Handler,         // 57
	TC10_Handler,         // 58
	TC11_Handler,         // 59
	TC12_Handler,         // 60
	TWIM0_Handler,        // 61
	TWIS0_Handler,        // 62
	TWIM1_Handler,        // 63
	TWIS1_Handler,        // 64
	USART0_Handler,       // 65
	USART1_Handler,       // 66
	USART2_Handler,       // 67
	USART3_Handler,       // 68
	ADCIFE_Handler,       // 69
	DACC_Handler,         // 70
	ACIFC_Handler,        // 71
	ABDACB_Handler,       // 72
	TRNG_Handler,         // 73
	PARC_Handler,         // 74
	CATB_Handler,         // 75
	Dummy_Handler,        // one not used
	TWIM2_Handler,        // 77
	TWIM3_Handler,        // 78
	LCDCA_Handler         // 79
};

void Reset_Handler(void)
{
	uint32_t *pSrc, *pDest;

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

	/* Set the vector table base address */
	//XTAG we don't have to do this, the bootloader did this for us
	//pSrc = (uint32_t *) &_sfixed;
	//SCB->VTOR = ((uint32_t) pSrc & SCB_VTOR_TBLOFF_Msk);

	/* Initialize the C library */
	//XTA
//	__libc_init_array();

    //Workaround for SB.02 hardware bug
    *((uint32_t volatile * ) 0x400E1004) = 1 << 14; //GPER
    *((uint32_t volatile * ) 0x400E1054) = 1 << 14; //OVR
    *((uint32_t volatile * ) 0x400E1044) = 1 << 14; //ODER

	/* Branch to main function */
	main();

}

// IMPORTANT!! __aeabi_memset has count and value arguments reversed from ANSI
// memset. TODO(alevy): Why does arm-none-eabi's libc not have __aeabi_memset?
__attribute__ ((weak)) void __aeabi_memset(void* dest, size_t count, int value) {
  memset(dest, value, count);
}

__attribute__ ((weak)) extern void __aeabi_memcpy(void* dest, void* src, unsigned int n) {
  memcpy(dest, src, n);
}

__attribute__ ((weak)) extern void __aeabi_memcpy4(void* dest, void* src, unsigned int n) {
  memcpy(dest, src, n);
}

__attribute__ ((weak)) extern void __aeabi_memcpy8(void* dest, void* src, unsigned int n) {
  memcpy(dest, src, n);
}


__attribute__ ((weak)) extern void __aeabi_memclr (void *dest, size_t n)
{
	  memset (dest, 0, n);
}

__attribute__ ((weak)) extern void __aeabi_memclr4 (void *dest, size_t n)
{
	  memset (dest, 0, n);
}

__attribute__ ((weak)) extern void __aeabi_memclr8 (void *dest, size_t n)
{
	  memset (dest, 0, n);
}

