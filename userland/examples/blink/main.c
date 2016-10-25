#include <stdio.h>

#include "led.h"
#include "timer.h"

int main(void) {
    while(1) {
      led_toggle(0);
      delay_ms(500);
    }
}
