#include <stdbool.h>
#include <stdio.h>

#include <timer.h>
#include <ambient_light.h>
#include <temperature.h>
#include <humidity.h>
#include <led.h>

int main (void) {
  while (1) {
    int lux;
    ambient_light_read_intensity_sync(&lux);
    printf("Light: %d lux\n", lux);

    /* Turn on the red LED in low light conditions */
    if (lux < 30) {
      led_on(0);
    }
    else {
      led_off(0);
    }

    int temp;
    temperature_read_sync(&temp);
    printf("Temperature: %d degrees C\n", temp/100);

    unsigned humi;
    humidity_read_sync(&humi);
    printf("Relative humidity: %u%%\n", humi/100);

    printf("\n");
    delay_ms(2000);
  }
}

