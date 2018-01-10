#include <led.h>
#include <timer.h>
#include <stdio.h>

int serial_read(char* buf, size_t len);

char buf[128];

int main(void) {
  printf("Starting uart_echo.\r\n");
  while (1) {
    int rval = serial_read(buf, 20);
    printf("Read returned %i.\r\n", rval);
    buf[rval] = 0;
    serial_write(buf, rval);
  }
}
