#include <stdio.h>
#include <temperature.h>
#include <timer.h>

static void callback(int temp, int not_used2,
    __attribute__ ((unused)) int arg2,
    __attribute__ ((unused)) void *ud){

  printf("CALLBACK TEMP: %d\r\n", temp);
}


int main(void)
{
  printf("Temperature Sensor App\r\n");

  temp_init(callback, NULL);
  for(;;) {
    temp_measure();
    delay_ms(1000);
  }
  return 0;
}
