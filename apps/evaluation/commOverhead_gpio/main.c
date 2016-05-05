#include <firestorm.h>
#include <gpio.h>

void main(void) {
    gpio_enable_output(P3);

    while(1) {
      /*
         base = 0x400E1000
         base+0x54: Set
         base+0x58: Clear
         P3 -> PA12
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
      /* RESULT: 5.12 us */
      gpio_clear(P3);
      /* RESULT: 4.72 us */

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

      delay_ms(1);
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
      delay_ms(2);
    }
}

