#include <stdint.h>

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
