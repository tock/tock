// Crash on button press.

#include <button.h>

volatile int* nullptr = 0;

static void button_callback(int btn_num,
                            int val,
                            __attribute__ ((unused)) int arg2,
                            __attribute__ ((unused)) void *ud) {
  volatile int k = *nullptr;
}

int main(void) {
  button_subscribe(button_callback, NULL);
  button_enable_interrupt(0);

  return 0;
}
