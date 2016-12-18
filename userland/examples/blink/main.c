#include <stdio.h>

#include <led.h>
#include <timer.h>

int main(void) {
  int num_leds = led_count();
  for (int count = 0; ; count++) {
    for (int i = 0; i < num_leds; i++) {
      if (count & (1 << i)) {
        led_on(i);
      } else {
        led_off(i);
      }
    }
    delay_ms(250);
  }
}
