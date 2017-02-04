#include "radio.h"
#include "gpio.h"
/*
 * Userland library for sending and receiving 802.15.4 packets.
 *
 * Author: Philip Levis
 * Date: Jan 12 2017
 */


const int SYS_RADIO = 154;

const int BUF_RX = 0;
const int BUF_TX = 1;

const int COM_ADDR = 1;
const int COM_PAN = 2;
const int COM_CHAN = 3;
const int COM_POWER = 4;
const int COM_TX = 5;
const int COM_READY = 6;
const int COM_COMMIT = 7;

const int EVT_TX = 0;
const int EVT_RX = 1;
const int EVT_CFG = 2;

int radio_init() {
  while (!radio_ready()) {}
  return 0;
} // Do nothing for now

static void cb_tx( __attribute__ ((unused)) int unused0,
                __attribute__ ((unused)) int unused1,
                __attribute__ ((unused)) int unused2,
                void* ud) {
  *((bool*)ud) = true;
}

static void cb_rx( __attribute__ ((unused)) int unused0,
                __attribute__ ((unused)) int unused1,
                __attribute__ ((unused)) int unused2,
                void* ud) {
  *((bool*)ud) = true;
}

static void cb_config( __attribute__ ((unused)) int unused0,
                       __attribute__ ((unused)) int unused1,
                       __attribute__ ((unused)) int unused2,
                       void* ud) {
  *((bool*)ud) = true;
}

// packet contains the payload of the 802.15.4 packet; this will
// be copied into a packet buffer with header space within the kernel.
int radio_send(unsigned short addr, const char* packet, unsigned char len) {
  bool cond = false;
  int err = allow(SYS_RADIO, BUF_TX, (void*)packet, len);
  if (err < 0) {
    return err;
  }
  err = subscribe(SYS_RADIO, EVT_TX, cb_tx, &cond);
  if (err < 0) {
    return err;
  }
  // The send system call packs the length and destination address in
  // the 32-bit argument.
  unsigned int param = addr;
  param |= (len << 16);
  err = command(SYS_RADIO, COM_TX, param);
  if (err != 0) {
    gpio_toggle(0);
    return err; // yield here too?
  } else {
    yield_for(&cond); // This should return -1, but for some reason
  }                   // successful calls don't return 0! radio.rs:208
  return 0;
}

// Set local 16-bit short address.
int radio_set_addr(unsigned short addr) {
  return command(SYS_RADIO, COM_ADDR, (unsigned int)addr);
}

// PAN is the personal area network identifier: it allows multiple
// networks using the same channel to be logically separated.
int radio_set_pan(unsigned short pan) {
  return command(SYS_RADIO, COM_PAN, (unsigned int)pan);
}

int radio_set_power(char power) {
  return command(SYS_RADIO, COM_POWER, (unsigned int) (power + 128));
}

int radio_commit() {
  bool cond = false;
  int err = subscribe(SYS_RADIO, EVT_CFG, cb_config, &cond);
  if (err != SUCCESS) {
    return err;
  }
  err = command(SYS_RADIO, COM_COMMIT, 0);
  if (err != SUCCESS) {
    return err;
  }
  yield_for(&cond);
  return SUCCESS;
}

// Valid channels are 10-26
int radio_set_channel(unsigned char channel) {
  return command(SYS_RADIO, COM_CHAN, (unsigned int)channel);
}

int radio_receive(const char* packet, unsigned char len) {
  bool cond = false;
  int err = allow(SYS_RADIO, BUF_RX, (void*)packet, len);
  if (err < 0) {
    return err;
  }
  err = subscribe(SYS_RADIO, EVT_RX, cb_rx, &cond);
  if (err < 0) {
    return err;
  }
  yield_for(&cond);
  return (int)packet[1];
}

int radio_ready() {
  return command(SYS_RADIO, COM_READY, 0) == SUCCESS;
}
