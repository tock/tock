#include <stdio.h>
#include <timer.h>
#include <ambient_light.h>
#include <temperature.h>
#include <humidity.h>
#include <ninedof.h>

int main(void)
{
  while (1) {
    int lux = ambient_light_read_intensity();
    printf("Light: %d lux\n", lux);

    int temp;
    temperature_read_sync(&temp);
    printf("Temperature: %d degrees C\n", temp/100);

    unsigned humi;
    humidity_read_sync(&humi);
    printf("Relative humidity: %u%%\n", humi/100);

    int ax, ay, az;
    ninedof_read_acceleration_sync(&ax, &ay, &az);
    printf("Acceleration: %dg X, %dg Y, %dg Z\n", ax, ay, az);

    int mx, my, mz;
    ninedof_read_magenetometer_sync(&mx, &my, &mz);
    printf("Magnetic field: %duT X, %duT Y, %duT Z\n", mx, my, mz);

    printf("\n");
    delay_ms(2000);
  }
}
