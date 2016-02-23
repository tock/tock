#include <stdbool.h>
#include <firestorm.h>

#define LED_1 19

#define BUF_SIZE 200
char rbuf[BUF_SIZE];
char wbuf[BUF_SIZE];
bool toggle = true;

CB_TYPE write_cb(int arg0, int arg2, int arg3, void* userdata) {
    gpio_toggle(LED_0);
    if (toggle) { 
        spi_read_write(rbuf, wbuf, BUF_SIZE, write_cb);
    } else {
        spi_read_write(wbuf, rbuf, BUF_SIZE, write_cb);
    }
    toggle = !toggle;
}

// This function can operate in one of two modes. Either
// a periodic timer triggers an SPI operation, or SPI
// operations are performed back-to-back (callback issues
// the next one.) The periodic one writes 6 byte messages,
// the back-to-back writes a 10 byte message, followed by
// 6 byte ones.
//
// In both cases, the calls alternate on which of two
// buffers is used as the write buffer. The first call
// uses the buffer initialized to 0..199. The
// 2n calls use the buffer initialized to 0. 
//
// If you use back-to-back operations, the calls
// both read and write. Periodic operations only
// write. Therefore, if you set SPI to loopback
// and use back-to-back // loopback, then the read buffer 
// on the first call will read in the data written. As a 
// result, you can check if reads work properly: all writes 
// will be 0..n rather than all 0s.

void main(void) {
        int i;
	gpio_enable(LED_0);
	gpio_enable(LED_1);

	gpio_enable(7);
	gpio_set(7);

	for (i = 0; i < 200; i++) {
		wbuf[i] = i;
	}

        spi_read_write(wbuf, rbuf, BUF_SIZE, write_cb);
}
