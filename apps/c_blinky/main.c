#include <firestorm.h>
#include <gpio.h>

void main(void) {
    gpio_enable_output(LED_0);

    while(1) {
      gpio_toggle(LED_0);
      delay_ms(500);
    }
}

