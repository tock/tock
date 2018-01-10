#pragma once
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

// Initialize the XMODEM receiver and start receiving data.
// Note that no data will be received until a buffer is installed
// with xmodem_set_buffer.
int xmodem_init(void);

// Install a buffer to receive into, and its length. XMODEM will
// not write past the length. If the transfer goes past the length
// of the buffer, XMODEM will trigger reception via the callback
// but issue NAKs to the sender. So this will appear as a successful
// transmission of len bytes to the process but will throw an error
// to the sender.
//
// Pass a buffer of NULL or size 0 to stop future receptions.
void xmodem_set_buffer(char* buf, size_t len);

// Install the callback to issue when a transfer completes. This
// occurs either on successful reception of an EOT from the sender,
// or if the end of the buffer is reached. EOT has an error code of
// 0, while end of the buffer has an error code of -1.
typedef void xmodem_cb(char* buf, int len, int error);
void xmodem_set_callback(xmodem_cb buffer_filled);

#ifdef __cplusplus
}
#endif
