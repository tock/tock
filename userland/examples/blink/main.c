#include <stdio.h>

#include <led.h>
#include <timer.h>

int main(void) {
  // Ask the kernel how many LEDs are on this board.
  int num_leds = led_count();

  // Blink the LEDs in a binary count pattern and scale
  // to the number of LEDs on the board.
  for (int count = 0; ; count++) {
    for (int i = 0; i < num_leds; i++) {
      if (count & (1 << i)) {
        led_on(i);
      } else {
        led_off(i);
      }
    }

    // This delay uses an underlying timer in the kernel.
    delay_ms(250);
  }
}
