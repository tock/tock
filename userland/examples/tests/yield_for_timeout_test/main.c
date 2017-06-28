#include <led.h>
#include <timer.h>

static void timer_cb(__attribute__ ((unused)) int now,
                     __attribute__ ((unused)) int expiration,
                     __attribute__ ((unused)) int unused, void* ud) {
  *((bool*)ud) = true;
}

int main(void) {
  
  while(1) {
    bool done = false;
    timer_in(1500, timer_cb, &done);
    
    int ret = yield_for_timeout(&done, 1000);
    if(ret == TOCK_SUCCESS) {
      led_on(0);
    } else {
      led_off(0);
    }

    ret = yield_for_timeout(&done, 2000);
    if(ret == TOCK_SUCCESS) {
      led_on(0);
    } else {
      led_off(0);
    }
  }
}
