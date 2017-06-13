#include <alarm.h>
#include <led.h>

typedef struct {
  int led_num;
} led_event;

void event_cb(int, int, int, void*);
void event_cb(__attribute__ ((unused)) int now,
              __attribute__ ((unused)) int expiration,
              __attribute__ ((unused)) int unused, void* ud) {
  int led_num = (int)ud;
  led_toggle(led_num);
}

int main(void) {
  int slow_interval = 1000;
  int fast_interval = 333;

  alarm_every(slow_interval, event_cb, (void*)0);
  alarm_every(fast_interval, event_cb, (void*)1);
}
