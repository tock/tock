#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include <timer.h>
#include <usb.h>

int main(void) {
  int r;

  if (!usb_exists()) {
    printf("USB driver is not present\n");
    exit(1);
  }

  r = usb_enable_and_attach();

  if (r == TOCK_SUCCESS) {
    printf("Enabled and attached\n");
  } else {
    printf("Attach failed with status %d\n", r);
  }
}
