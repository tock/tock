#include <stdio.h>

#include "firestorm.h"
#include "gpio.h"
#include "timer.h"

int main(void) {
    gpio_enable_output(LED_0);

    while(1) {
      gpio_toggle(LED_0);
      delay_ms(500);
    }
}

