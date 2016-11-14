#include <led.h>

/* Delay for for the given microseconds (approximately).
 *
 * For a 16 MHz CPU, 1us == 16 instructions (assuming each instruction takes
 * one cycle). */
static void busy_delay_us(int duration) {
  // The inner loop instructions are: 14 NOPs + 1 SUBS/ADDS + 1 CMP
  while (duration-- != 0) {
    __asm volatile (
      "nop\n"
      "nop\n"
      "nop\n"
      "nop\n"
      "nop\n"
      "nop\n"
      "nop\n"
      "nop\n"
      "nop\n"
      "nop\n"
      "nop\n"
      "nop\n"
      "nop\n"
      "nop\n"
    );
  }
}

/* Delay for for the given milliseconds (approximately).
 *
 * Note that this is not precise as there are 2 extra instructions on the inner
 * loop. Therefore, there is 1us added every 8 iterations. */
static void busy_delay_ms(int duration) {
  while (duration-- != 0) {
    busy_delay_us(1000);
  }
}

int main(void) {
  while (1) {
    led_toggle(0);
    busy_delay_ms(500);
  }
}
