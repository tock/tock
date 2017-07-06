#include <led.h>
#include <timer.h>

int main(void)
{
  
  /** delay_ms(250); */
  for (int i = 0; i < led_count(); i++) {
    led_on(i);
  }

  delay_ms(250);
  // This delay uses an underlying timer in the kernel.
  /** delay_ms(250); */
  return 0;
}
