/**
 * This application is for testing GPIO interrupts in the nRF51822 EK.
 * To run this application, hook up a button connected to VDD to GPIO pin 1
 * (the top right pin on the top left header).
 *
 * When it boots, you should see one of the two LEDs blink 5 times, then
 * go silent. This is to show that the app has booted correctly.
 *
 * Then, when you push the button, the other LED should blink.
 */

#include <button.h>
#include <led.h>

void interrupt_callback(int pin_num, int val) {
  if (val == 0) {
    led_toggle(pin_num);
  }
}

int main(void) {
  button_subscribe(interrupt_callback, 0);
  int j = 0;
  for (int i = 0; i < 4 && j >= 0; i++) {
    j = button_enable_interrupt(i);
  }

  return 0;
}

