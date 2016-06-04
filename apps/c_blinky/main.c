#include <firestorm.h>
#include <gpio.h>

#define LED_1 1

void main(void) {
    gpio_enable_output(LED_0);
    gpio_enable_output(LED_1);

    while(1) {
      gpio_set(LED_0);
      gpio_clear(LED_1);
      delay_ms(500);
      gpio_set(LED_1);
      gpio_clear(LED_0);
      delay_ms(500);
    }
}

