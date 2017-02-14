#ifndef _RADIO_H
#define _RADIO_H

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

int radio_receive(const char* packet, unsigned char len);
// Set local 16-bit short address
int radio_set_addr(unsigned short addr);

// PAN is the personal area network identifier: it allows multiple
// networks using the same channel to remain logically distinct
int radio_set_pan(unsigned short pan);
// Valid channels are 10-26
int radio_set_channel(unsigned char channel);


#ifdef __cplusplus
}
#endif

#endif // _RADIO_H
