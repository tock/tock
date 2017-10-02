#include "button.h"

int button_subscribe(subscribe_cb callback, void *ud) {
  return subscribe(DRIVER_NUM_BUTTON, 0, callback, ud);
}

int button_count(void) {
  return command(DRIVER_NUM_BUTTON, 0, 0, 0);
}

int button_enable_interrupt(int pin_num) {
  return command(DRIVER_NUM_BUTTON, 1, pin_num, 0);
}

int button_disable_interrupt(int pin_num) {
  return command(DRIVER_NUM_BUTTON, 2, pin_num, 0);
}

int button_read(int pin_num) {
  return command(DRIVER_NUM_BUTTON, 3, pin_num, 0);
}

