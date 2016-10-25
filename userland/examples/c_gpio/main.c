/**
 * This application is for testing GPIO interrupts in the nRF51822 EK.
 * To run this application, hook up a button connected to VDD to GPIO pin 1
 * (the top right pin on the top left header).
 *
 * When it boots, you should see one of the two LEDs blink 5 times, then
 * go silent. This is to show that the app has booted correctly.
 *
 * Then, when you push the button, the other LED should blink.
 */

#include "led.h"
#include "gpio.h"

/* Delay for for the given microseconds (approximately).
 *
 * For a 16 MHz CPU, 1us == 16 instructions (assuming each instruction takes
 * one cycle). */
static void busy_delay_us(int duration)
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
static void busy_delay_ms(int duration) {
	while (duration-- != 0) {
		busy_delay_us(1000);
	}
}

void interrupt_callback() {
    led_toggle(1);
}

int main(void) {
    led_on(1);
    // Application pin 0 is Button 1
    gpio_enable_input(0, PullDown);
    gpio_enable_interrupt(0, PullDown, RisingEdge);
    gpio_interrupt_callback(interrupt_callback, NULL);

    int i;
    for (i = 0; i < 10; i++) {
      led_toggle(0);
      busy_delay_ms(200);
    }
    return 1;
}

