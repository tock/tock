#include "timer.h"

static void delay_cb( __attribute__ ((unused)) int unused0,
                      __attribute__ ((unused)) int unused1,
                      __attribute__ ((unused)) int unused2,
                      void* ud) {
  *((bool*)ud) = true;
}

void delay_ms(uint32_t ms) {
  bool cond = false;
  timer_subscribe(delay_cb, &cond);
  timer_oneshot(ms);
  yield_for(&cond);
}

int timer_subscribe(subscribe_cb cb, void *userdata) {
  return subscribe(3, 0, cb, userdata);
}

int timer_oneshot(uint32_t interval_ms) {
  return command(3, 1, (int)interval_ms);
}

int timer_start_repeating(uint32_t interval_ms) {
  return command(3, 2, (int)interval_ms);
}

int timer_stop(void) {
  return command(3, 3, 0);
}

unsigned int timer_read(void) {
  return (unsigned int) command(3, 4, 0);
}
