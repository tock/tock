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

      // SPI Configuration from rf232 app
  ;                    // spi_init();
  command(4,3,0);      // spi_set_chip_select(3);
  command(4,8,0);      // spi_set_polarity(0);
  command(4,6,0);      // spi_set_phase(0);
  command(4,4,400000); //spi_set_rate(400000);
      /*
int spi_init() {return 0;}
int spi_set_chip_select(unsigned char cs) {return command(4, 2, cs);}
int spi_get_chip_select()                 {return command(4, 3, 0);}
int spi_set_rate(int rate)                {return command(4, 4, rate);}
int spi_get_rate()                        {return command(4, 5, 0);} 
int spi_set_phase(bool phase)             {return command(4, 6, (unsigned char)phase);} 
int spi_get_phase()                       {return command(4, 7, 0);} 
int spi_set_polarity(bool pol)            {return command(4, 8, (unsigned char)pol);} 
int spi_get_polarity()                    {return command(4, 9, 0);} 
int spi_hold_low()                        {return command(4, 10, 0);}
int spi_release_low()                     {return command(4, 11, 0);}
      */

      // RESULT: 39.4 us

      // Clear
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

