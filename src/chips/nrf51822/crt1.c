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

/* Symbols defined in the linker file */
extern uint32_t _estack;
extern uint32_t _etext;
extern uint32_t _szero;
extern uint32_t _ezero;
extern uint32_t _srelocate;
extern uint32_t _erelocate;

int main(void);

void Dummy_Handler(void)
{
	while (1) {
	}
}

void Reset_Handler(void);
void NMI_Handler(void) __attribute__ ((weak, alias("Dummy_Handler")));
void HardFault_Handler(void)
    __attribute__ ((weak, alias("Dummy_Handler")));
void SVC_Handler(void) __attribute__ ((weak, alias("Dummy_Handler")));
void PendSV_Handler(void) __attribute__ ((weak, alias("Dummy_Handler")));
void SysTick_Handler(void) __attribute__ ((weak, alias("Dummy_Handler")));

typedef void (*interrupt_function_t) (void);

__attribute__ ((section(".vectors")))
interrupt_function_t interrupt_table[] = {
	(interrupt_function_t) (&_estack),
	Reset_Handler,
	NMI_Handler,
	HardFault_Handler,
	0, 0, 0, 0, 0, 0, 0,	/* Reserved */
	SVC_Handler,
	0, 0,			/* Reserved */
	PendSV_Handler,
	SysTick_Handler,
};

void Reset_Handler(void)
{
	uint32_t *pSrc, *pDest;

	/* Power on RAM blocks manually (see nRF51822-PAN v2.4, PAN #16). Note
	 * that xxAA/xxAB variants have only two RAM blocks. For xxAC, change
	 * to 0x0F. */
	*((uint32_t volatile * ) 0x40000524) = 0x03;

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

	/* Branch to main function */
	main();
}

// IMPORTANT!! __aeabi_memset has count and value arguments reversed from ANSI
// memset. TODO(alevy): Why does arm-none-eabi's libc not have __aeabi_memset?
__attribute__ ((weak))
void __aeabi_memset(void *dest, size_t count, int value)
{
	memset(dest, value, count);
}

__attribute__ ((weak))
extern void __aeabi_memcpy(void *dest, void *src, unsigned int n)
{
	memcpy(dest, src, n);
}

__attribute__ ((weak))
extern void __aeabi_memcpy4(void *dest, void *src, unsigned int n)
{
	memcpy(dest, src, n);
}

__attribute__ ((weak))
extern void __aeabi_memcpy8(void *dest, void *src, unsigned int n)
{
	memcpy(dest, src, n);
}

__attribute__ ((weak))
extern void __aeabi_memclr(void *dest, size_t n)
{
	memset(dest, 0, n);
}

__attribute__ ((weak))
extern void __aeabi_memclr4(void *dest, size_t n)
{
	memset(dest, 0, n);
}

__attribute__ ((weak))
extern void __aeabi_memclr8(void *dest, size_t n)
{
	memset(dest, 0, n);
}

/* Based on reference code from GCC documentation, see "Legacy __sync Built-in
 * Functions for Atomic Memory Access" */

__attribute__ ((weak))
extern uint32_t __sync_fetch_and_add_4(uint32_t * ptr, uint32_t val)
{
	uint32_t tmp = *ptr;
	*ptr += val;
	return tmp;
}

__attribute__ ((weak))
extern uint32_t __sync_fetch_and_sub_4(uint32_t * ptr, uint32_t val)
{
	uint32_t tmp = *ptr;
	*ptr -= val;
	return tmp;
}
