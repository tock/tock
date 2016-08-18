/**
 *  This application runs a while(1) loop in which it starts
 *  a one-shot timer then waits for it to fire. The main loop
 *  toggles GPIO pin 0, while the event toggles GPIO pin 1.
 *  If this application is working properly, you should see
 *  GPIO pins 0 and 1 toggle (with one being on, one being off)
 *  at approximately 2Hz.
 *
 *  Author: Philip Levis <pal@cs.stanford.edu>
 *  Date: August 18, 2016
 */

#include <firestorm.h>
#include <gpio.h>

/* Delay for for the given microseconds (approximately).
 *
 * For a 16 MHz CPU, 1us == 16 instructions (assuming each instruction takes
 * one cycle). */
static void busy_delay_us(int duration) {
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
 * Note that this is not precise as there are 2 extra instructions on the in    ner
 * loop. Therefore, there is 1us added every 8 iterations. */
static void busy_delay_ms(int duration)
{
	while (duration-- != 0) {
		busy_delay_us(1000);
	}
}

static void delay_cb( __attribute__ ((unused)) int unused0,
		__attribute__ ((unused)) int unused1,
		__attribute__ ((unused)) int unused2,
		void* ud) {
	gpio_toggle(1);
	bool* c = (bool*)ud;
	*c = true;
}

void mywait() {
}

int main(void) {
	for (int i = 0; i < 4; i++) {
		gpio_enable_output(i);
		gpio_set(i);
	}
	bool c = false;
	while (1) {
		gpio_toggle(0);
		timer_subscribe(delay_cb, &c);
		timer_oneshot(500);
		wait_for(&c);
		c = false;
	}
}

