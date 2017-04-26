#include <stdio.h>
#include <gpio.h>
#include <led.h>
#include <timer.h>

/** #define RECEIVER */



int main(void)
{
	for(;;)
	{
		led_toggle(3);
		delay_ms(1000);
		led_toggle(3);
		delay_ms(1000);
	}
	printf("Done\r\n");
  return 0;
}
