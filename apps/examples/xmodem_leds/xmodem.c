#include <tock.h>
#include <timer.h>
#include <led.h>
#include <console.h>

// Set the buffer that xmodem should fill with a transfer.
void xmodem_set_buffer(char* buf, size_t len);

// The callback that indicates an xmodem transfer completed.
// After this callback is issued, the xmodem library will wait
// until a new buffer is set with xmodem_set_buffer  before accepting
// a new transfer.
typedef void xmodem_cb(char* buf, int len, int error);

void xmodem_set_callback(xmodem_cb buffer_filled);
void xmodem_restart_transfer(void);
void xmodem_restart_block(void);
int xmodem_write(uint8_t byte);
int xmodem_init(void);

void xmodem_read_callback(__attribute__ ((unused)) int unused0,
                          __attribute__ ((unused)) int unused1,
                          __attribute__ ((unused)) int unused2,
                          __attribute__ ((unused))void* ud);
void xmodem_write_callback(__attribute__ ((unused)) int unused0,
                          __attribute__ ((unused)) int unused1,
                          __attribute__ ((unused)) int unused2,
                          __attribute__ ((unused))void* ud);
void xmodem_timer_callback(__attribute__ ((unused)) int unused0,
                          __attribute__ ((unused)) int unused1,
                          __attribute__ ((unused)) int unused2,
                          __attribute__ ((unused))void* ud);

typedef enum {
  STOP,
  NEW_BLOCK,
  BLOCK_NUMBER,
  BLOCK_INVERSE,
  DATA,
  CHECKSUM
} xmodem_state_t;

enum {
        SOH = 0x01,   // Start Of Header
        ACK = 0x06,   // Acknowledge (positive)
        NAK = 0x15,   // Acknowledge (negative)
        EOT = 0x04,   // End of transmission
        PAYLOAD_SIZE = 128,
        ARMBASE = 0x8000
};


static xmodem_state_t xmodem_state = STOP;
static uint8_t xmodem_write_busy = false;
static const uint32_t XMODEM_TIMEOUT = 4000;
static uint8_t xmodem_byte_count;
static uint8_t xmodem_recv;
static uint8_t xmodem_send;
static uint8_t xmodem_block_number;
static uint8_t xmodem_checksum;
static tock_timer_t xmodem_timer;
static char* xmodem_buffer = NULL;
static size_t xmodem_buffer_len = 0;
static xmodem_cb* xmodem_callback = NULL;

void xmodem_set_buffer(char* buf, size_t len) {
  xmodem_buffer = buf;
  xmodem_buffer_len = len;
  xmodem_block_number = 1;
}

void xmodem_set_callback(xmodem_cb buffer_filled) {
  xmodem_callback = buffer_filled;
}

void xmodem_restart_transfer(void) {
  xmodem_state = NEW_BLOCK;
  xmodem_write(NAK);
  xmodem_block_number = 1;
  xmodem_byte_count = 0;
  xmodem_checksum = 0;
}

void xmodem_restart_block(void) {
  xmodem_state = NEW_BLOCK;
  xmodem_write(NAK);
  xmodem_checksum = 0;
}

void xmodem_read_callback(__attribute__ ((unused)) int unused0,
                          __attribute__ ((unused)) int unused1,
                          __attribute__ ((unused)) int unused2,
                          __attribute__ ((unused))void* ud) {
  // Restart the NAK read timeout
  timer_cancel(&xmodem_timer);
  timer_in(XMODEM_TIMEOUT, xmodem_timer_callback, NULL, &xmodem_timer);
  // issue another read
  int ret = command(DRIVER_NUM_CONSOLE, 2, sizeof(uint8_t), 0);
  if (ret < 0 ) {
    xmodem_restart_transfer();
  } 
  switch (xmodem_state) {
  case NEW_BLOCK:
    switch (xmodem_recv) {
    case EOT:
      xmodem_write(ACK);
      if (xmodem_callback != NULL) {
        uint32_t size = (xmodem_block_number - 1) * PAYLOAD_SIZE ;
        xmodem_callback(xmodem_buffer, size, 0);
      }
      xmodem_block_number = 1;
      break;
    case SOH:
      xmodem_state = BLOCK_NUMBER;
      xmodem_checksum = 0;
      break;
    default:
      xmodem_restart_block();
      break;
    }
    break;
  case BLOCK_NUMBER:
    if (xmodem_recv == xmodem_block_number) {
      xmodem_state = BLOCK_INVERSE;
    } else { // Go back to beginning of block
      xmodem_restart_transfer();
    }
    break;
  case BLOCK_INVERSE:
    if (xmodem_recv == (0xff - xmodem_block_number)) {
      xmodem_state = DATA;
      xmodem_byte_count = 0;
    } else { // Go back to beginning of block
      xmodem_restart_transfer();
    }
    break;
  case DATA: {
    unsigned pos = ((xmodem_block_number - 1) * PAYLOAD_SIZE);
    pos += xmodem_byte_count;

    if (pos >= xmodem_buffer_len) {     // Wrote past end of buffer -- abort
      xmodem_restart_transfer();
      xmodem_callback(xmodem_buffer, 0, -1);
    } else {
      if (xmodem_buffer != NULL) {
          xmodem_buffer[pos] = xmodem_recv;
      }
      xmodem_checksum += xmodem_recv;
      xmodem_byte_count++;
      // Completed the block
      if (xmodem_byte_count == PAYLOAD_SIZE) {
        xmodem_state = CHECKSUM;
      }
    }
    break;
	     }
  case CHECKSUM:
    if (xmodem_recv != xmodem_checksum) {
      xmodem_restart_transfer();
    } else {
      xmodem_write(ACK);
      xmodem_block_number++;
      xmodem_state = NEW_BLOCK;
    }
    break;
  case STOP:
  default:
    xmodem_restart_transfer();
    // Should never happen
    break;
  }
}

void xmodem_write_callback(__attribute__ ((unused)) int unused0,
                           __attribute__ ((unused)) int unused1,
                           __attribute__ ((unused)) int unused2,
                           __attribute__ ((unused))void* ud) {
  xmodem_write_busy = false;
}

void xmodem_timer_callback(__attribute__ ((unused)) int unused0,
                           __attribute__ ((unused)) int unused1,
                           __attribute__ ((unused)) int unused2,
                           __attribute__ ((unused)) void* ud) {
  xmodem_write(NAK);
  led_toggle(0);
  timer_in(XMODEM_TIMEOUT, xmodem_timer_callback, NULL, &xmodem_timer);
}



// Non-blocking write, just issues the command, protected by a
// busy flag so a write doesn't occur while one is pending.
int xmodem_write(uint8_t byte) {
  if (xmodem_write_busy == false) {
    xmodem_send = byte;
    int ret = allow(DRIVER_NUM_CONSOLE, 1, &xmodem_send, sizeof(uint8_t));
    if (ret < 0) return ret;
    ret = subscribe(DRIVER_NUM_CONSOLE, 1, xmodem_write_callback, NULL);
    if (ret < 0) return ret;
    ret = command(DRIVER_NUM_CONSOLE, 1, sizeof(uint8_t), 0);
    if (ret == 0) {
      xmodem_write_busy = true;
      return 0;
    }
  }
  return -1;
}

int xmodem_init(void) {
  xmodem_state = NEW_BLOCK;
  xmodem_block_number = 1;
  xmodem_byte_count = 0;

  led_on(0);

  // Start reading
  int ret = allow(DRIVER_NUM_CONSOLE, 0, &xmodem_recv, sizeof(uint8_t));
  if (ret < 0)  return ret;
  ret = subscribe(DRIVER_NUM_CONSOLE, 0, xmodem_read_callback, NULL);
  if (ret < 0)  return ret;
  ret = command(DRIVER_NUM_CONSOLE, 2, sizeof(uint8_t), 0);
  if (ret < 0)  return ret;


  // Set the timeout
  timer_in(XMODEM_TIMEOUT, xmodem_timer_callback, NULL, &xmodem_timer);
  return 0;
}

//void notmain ( void ) {

        /*
         * 132 byte packet.  All fields are 1 byte except for the 128 byte data
         * payload.
         *              +-----+------+----------+--....----+-----+
         *              | SOH | blk# | 255-blk# | ..data.. | cksum |
         *              +-----+------+----------+--....----+-----+
         * Protocol:
         *      - first block# = 1.
         *  - CRC is over the whole packet
         *  - after all packets sent, sender transmits a single EOT (must ACK).
        unsigned char block = 1;
        unsigned addr = ARMBASE;
        while (1) {
                unsigned char b;

                // We received an EOT, send an ACK, jump to beginning of code
                if((b = getbyte()) == EOT) {
                        uart_send(ACK);
                        BRANCHTO(ARMBASE);
                        return; // NOTREACHED
                }

         */
                /*
                 * if first byte is not SOH, or second byte is not the
                 * expected block number or the third byte is not its
                 * negation, send a nak for a resend of this block.
                 
                if(b != SOH
                || getbyte() != block
                || getbyte() != (0xFF - block)) {
                        uart_send(NAK);
                        continue;
                }

                // get the data bytes
                int i;
                unsigned char cksum;
                for(cksum = i = 0; i < PAYLOAD_SIZE; i++) {
                        cksum += (b = getbyte());
                        PUT8(addr+i, b);
                }

                // Checksum failed: NAK the block
                if(getbyte() != cksum)
                        uart_send(NAK);
                // Commit our addr pointer and go to next block.
                else {
                        uart_send(ACK);
                        addr += PAYLOAD_SIZE;
                        block++;
                }
        }
} */
