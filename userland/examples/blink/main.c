#include <firestorm.h>
#include <gpio.h>
#include <stdio.h>

int main(void) {
    printf("Blinkin...\n");
    gpio_enable_output(LED_0);

    while(1) {
      gpio_toggle(LED_0);
      delay_ms(500);
    }
}

