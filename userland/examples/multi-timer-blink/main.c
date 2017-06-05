#include <led.h>
#include <timer.h>
#include <virtual_timer.h>

typedef struct {
  int interval;
  int led_num;
} led_event;

void event_cb(int, int, int, void*);
void event_cb(int now,
             __attribute__ ((unused)) int expiration,
             __attribute__ ((unused)) int unused, void* ud) {
  led_event *ev = (led_event*)ud;
  led_toggle(ev->led_num);
  virtual_timer_start(now + ev->interval, event_cb, (void*)ev);
}

led_event slow_ev = {
  .interval = 0,
  .led_num = 0,
};
led_event fast_ev = {
  .interval = 0,
  .led_num = 1,
};

int main(void) {
  int frequency = timer_frequency();
  int now = timer_read();

  slow_ev.interval = frequency;
  fast_ev.interval = 333 * frequency / 1000;

  virtual_timer_start(now + slow_ev.interval, event_cb, (void*)&slow_ev);
  virtual_timer_start(now + fast_ev.interval, event_cb, (void*)&fast_ev);
}
