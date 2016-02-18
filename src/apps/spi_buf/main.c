#include <firestorm.h>

#define LED_1 19

/* FIXME: These delay functions are Cortex-M0 specific (and calibrated for a
 * 16MHz CPU clock), therefore should be moved to platform specific location.
 * */

/* Delay for for the given microseconds (approximately).
 *
 * For a 16 MHz CPU, 1us == 16 instructions (assuming each instruction takes
 * one cycle). */
static void delay_us(int duration)
{
	// The inner loop instructions are: 14 NOPs + 1 SUBS/ADDS + 1 CMP
	while (duration-- != 0) {
		__asm volatile (
			"nop\n"
			"nop\n"
			"nop\n"
			"nop\n"
			"nop\n"
			"nop\n"
			"nop\n"
			"nop\n"
			"nop\n"
			"nop\n"
			"nop\n"
			"nop\n"
			"nop\n"
			"nop\n"
		);
	}
}

/* Delay for for the given milliseconds (approximately).
 *
 * Note that this is not precise as there are 2 extra instructions on the inner
 * loop. Therefore, there is 1us added every 8 iterations. */
static void delay_ms(int duration)
{
	while (duration-- != 0) {
		delay_us(1000);
	}
}

char rbuf[200];
char wbuf[200];
int toggle = 0;

CB_TYPE timer_cb(int arg0, int arg2, int arg3, void* userdata) {
    gpio_toggle(LED_0);
    if (toggle == 0) { 
        spi_block_write(rbuf, 6, timer_cb);
    } else {
        spi_block_write(wbuf, 6, timer_cb);
    }
    toggle = toggle ^ 1;
}

CB_TYPE write_cb(int arg0, int arg2, int arg3, void* userdata) {
    gpio_toggle(LED_0);
    if (toggle == 0) { 
        spi_read_write(rbuf, wbuf, 6, write_cb);
    } else {
        spi_read_write(wbuf, rbuf, 6, write_cb);
    }
    toggle = toggle ^ 1;
}

// This function can operate in one of two modes. Either
// a periodic timer triggers an SPI operation, or SPI
// operations are performed back-to-back (callback issues
// the next one.) The periodic one writes 6 byte messages,
// the back-to-back writes a 10 byte message, followed by
// 6 byte ones.
//
// In both cases, the calls alternate on which of two
// buffers is used as the write buffer. The first call
// uses the buffer initialized to 0..199. The
// 2n calls use the buffer initialized to 0. 
//
// If you use back-to-back operations, the calls
// both read and write. Periodic operations only
// write. Therefore, if you set SPI to loopback
// and use back-to-back // loopback, then the read buffer 
// on the first call will read in the data written. As a 
// result, you can check if reads work properly: all writes 
// will be 0..n rather than all 0s.

void main(void) {
        int i;
	gpio_enable(LED_0);
	gpio_enable(LED_1);

	for (i = 0; i < 200; i++) {
                rbuf[i] = i + 10;
		wbuf[i] = i;
	}
//	timer_repeating_subscribe(timer_cb, NULL);
        spi_read_write(wbuf, rbuf, 10, write_cb, NULL);
}
