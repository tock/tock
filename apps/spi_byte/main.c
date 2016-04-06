#include <firestorm.h>
#include <gpio.h>

void main(void)
{
        int i;
	gpio_enable_output(LED_0);

	for (i = 0;; i++) {
		gpio_clear(LED_0);

		spi_write_byte((unsigned char)i & 0xff);
		delay_ms(25);

		gpio_set(LED_0);

		delay_ms(25);
	}
}
