#include <led.h>
#include <timer.h>

static int interval;

static void toggle(int led_num) {
  led_on(led_num);
  delay_ms(300);
  led_off(led_num);
}

static void event_cb(__attribute__ ((unused)) int now,
                     __attribute__ ((unused)) int expiration,
                     __attribute__ ((unused)) int unused, void* ud) {
  toggle((int)ud);
}

static void start_cb(__attribute__ ((unused)) int now,
                     __attribute__ ((unused)) int expiration,
                     __attribute__ ((unused)) int unused, void* ud) {
  timer_every(interval, event_cb, ud);
  toggle((int)ud);
}

int main(void) {
  int spacing  = 1000; // 1 second between each led
  int num_leds = led_count();
  interval = spacing * num_leds;

  for (int i = 0; i < num_leds; i++) {
    timer_in(spacing * (i + 1), start_cb, (void*)i);
  }
}
