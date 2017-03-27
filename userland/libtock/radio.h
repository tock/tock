#pragma once

#include <tock.h>

#ifdef __cplusplus
extern "C" {
#endif

/* System calls for an 802.15.4 radio. */
int radio_init(void);

/* Returns 1 if radio is ready, 0 otherwise. */
int radio_ready(void);

// packet contains the payload of the 802.15.4 packet
int radio_send(unsigned short addr, const char* packet, unsigned char len);

// Blocking radio receive
int radio_receive(const char* packet, unsigned char len);

// Issue a callback when a packet is received;
// not usable simultaneously with radio_receive.
int radio_receive_callback(subscribe_cb callback,
                           const char* packet,
                           unsigned char len);

// Calls to configure the radio don't take full effect
// until you call radio_commit()

// Set local 16-bit short address
int radio_set_addr(unsigned short addr);
// PAN is the personal area network identifier: it allows multiple
// networks using the same channel to remain logically distinct
int radio_set_pan(unsigned short pan);
// Valid channels are 10-26
int radio_set_channel(unsigned char channel);
// Specify power in dBm. Typical range is -20 -- 4.
int radio_set_power(char power);
// Commit the channel, PAN, addr, and transmit power
int radio_commit(void);

#ifdef __cplusplus
}
#endif
