#include <firestorm.h>
#include <gpio.h>

int main(void) {
    gpio_set(0);
    while(1) {
      gpio_toggle(0);
//      printf("Before delay.\n");
      delay_ms(1000);
//      printf("After delay.\n");
//      gpio_toggle(1);
    }
}

