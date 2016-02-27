#include <firestorm.h>

CB_TYPE timer_cb(int, int, int, void*);

void main(void)
{
	gpio_enable(LED_0);
        timer_repeating_subscribe(timer_cb, NULL);
}

CB_TYPE timer_cb(int arg0, int arg2, int arg3, void* userdata) {
  gpio_toggle(LED_0);
  return 0;
}

