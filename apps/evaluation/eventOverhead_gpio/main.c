/* vim: set sw=2 expandtab tw=80: */

#include <string.h>
#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>

#include <tock.h>
#include <firestorm.h>
#include <gpio.h>

void gpio_cb (int pin_num, int arg2, int arg3, void* userdata) {

  /*
  // Event Overhead, GPIO, Process
  // set P3 as low to end test
  asm ("\
      movw r3, 0x1058    \n\
      movt r3, 0x400E    \n\
      movs r4, 0x1000    \n\
      str  r4, [r3]      \n\
      "
      :               // output
      :               // input
      : "r3", "r4"    // clobbers
      );
    */

  // test complete!
}

void main() {
  // event overhead GPIO test


  //*** Test Setup ***

  // wait for a bit for everything to be happy
  for (volatile int i=0; i<1000000; i++);

  // enable P3 as output
  gpio_enable_output(P3);

  // set P3 as low
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

  // set callback for GPIO interrupts
  gpio_interrupt_callback(gpio_cb, NULL);

  // set P2 as interrupt input, active high
  gpio_enable_interrupt(P2, PullNone, RisingEdge);

  // wait a bit more just in case
  for (volatile int i=0; i<1000000; i++);


  //*** Begin Test ***

  // set P3 as high to begin test
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

  // waiting for interrupt
}

