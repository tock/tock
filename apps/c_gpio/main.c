/**
 * This application is for testing GPIO interrupts in the nRF51822 EK.
 * To run this application, hook up a button connected to VDD to GPIO pin 1
 * (the top right pin on the top left header).
 *  
 * When it boots, you should see one of the two LEDs blink 5 times, then
 * go silent. This is to show that the app has booted correctly.
 *  
 * Then, when you push the button, the other LED should blink.
 *
 * Note that to an application, pin 0 is LED0 (GPIO pin 18), pin 1 is 
 * LED1 (GPIO pin 19), and pin 3 is GPIO pin 0.
 */

#include <firestorm.h>
#include <gpio.h>

#define LED_1 1

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
    gpio_toggle(LED_1); 
}

int main(void) {
    gpio_enable_output(LED_0);
    gpio_enable_output(LED_1);
    gpio_set(LED_1);
    // Application pin 3 is HW GPIO pin 0, see nrf_pc10001/lib.rs
    gpio_enable_input(3, PullDown);
    gpio_enable_interrupt(3, PullDown, RisingEdge);
    gpio_interrupt_callback(interrupt_callback, NULL);

    int i;
    for (i = 0; i < 10; i++) {
      gpio_toggle(LED_0);
      busy_delay_ms(200);
    }
    return 1;
}

