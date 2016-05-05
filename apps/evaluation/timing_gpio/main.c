#include <firestorm.h>
#include <gpio.h>

void main(void) {
    gpio_enable_output(LED_0);

    while(1) {
      /*
         base = 0x400E1000
         base+0x54: Set
         base+0x58: Clear
         LED_0 -> PC10 -> PC10=0
      */

      // Set pin using direct MMIO
      asm ("\
          movw r3, 0x1054    \n\
          movt r3, 0x400E    \n\
          movs r4, 0x1000    \n\
          str  r4, [r3]      \n\
          "
          :               /* output */
          :               /* input */
          : "r3", "r4"    /* clobbers */
          );

      // Clear using interface path
      gpio_toggle(LED_0);

      // Set pin using direct MMIO
      asm ("\
          movw r3, 0x1054    \n\
          movt r3, 0x400E    \n\
          movs r4, 0x1000    \n\
          str  r4, [r3]      \n\
          "
          :               /* output */
          :               /* input */
          : "r3", "r4"    /* clobbers */
          );

      delay_ms(500);
      // Clear to start fresh timing round
      asm ("\
          movw r3, 0x1058    \n\
          movt r3, 0x400E    \n\
          movs r4, 0x1000    \n\
          str  r4, [r3]      \n\
          "
          :               /* output */
          :               /* input */
          : "r3", "r4"    /* clobbers */
          );
    }
}

