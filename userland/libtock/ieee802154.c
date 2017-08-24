#include "ieee802154.h"
#include "t"

const int RADIO_DRIVER = 154;

const int ALLOW_RX = 0;
const int ALLOW_TX = 1;
const int ALLOW_CFG = 2;

const int COMMAND_STATUS = 0;
const int COMMAND_SET_ADDR = 1;
const int COMMAND_SET_ADDR_LONG = 2;
const int COMMAND_SET_PAN = 3;
const int COMMAND_SET_CHANNEL = 4;
const int COMMAND_SET_POWER = 5;
const int COMMAND_CONFIG_COMMIT = 6;
const int COMMAND_GET_ADDR = 7;
const int COMMAND_GET_ADDR_LONG = 8;
const int COMMAND_GET_PAN = 9;
const int COMMAND_GET_CHANNEL = 10;
const int COMMAND_GET_POWER = 11;

// Temporary buffer used for some commands where the system call interface
// parameters / return codes are not enough te contain the required data.
unsigned char BUF_CFG[8];

int ieee802154_up(void) {
  // Spin until radio is on. Maybe this can be done with a callback?
  while (!ieee802154_is_up()) {
    delay_ms(10);
  }
  return TOCK_SUCCESS;
}

int ieee802154_down(void) {
  // Currently unsupported: there is no way to implement this with the existing
  // radio interface.
  return TOCK_ENOSUPPORT;
}

bool ieee802154_is_up(void) {
  return command(RADIO_DRIVER, COMMAND_STATUS, 0) == TOCK_SUCCESS;
}

int ieee802154_set_address(unsigned short addr) {
  return command(RADIO_DRIVER, COMMAND_SET_ADDR, (unsigned int) addr);
}

int ieee802154_set_address_long(unsigned char *addr_long) {
  if (!addr_long) return TOCK_EINVAL;
  int err = allow(RADIO_DRIVER, ALLOW_CFG, (void *) addr_long, 8);
  if (err < 0) return err;
  return command(RADIO_DRIVER, COMMAND_SET_ADDR_LONG, 0);
}

int ieee802154_set_pan(unsigned short pan) {
  return command(RADIO_DRIVER, COMMAND_SET_PAN, (unsigned int) pan);
}

int ieee802154_set_channel(unsigned char channel) {
  return command(RADIO_DRIVER, COMMAND_SET_CHANNEL, (unsigned int) channel);
}

int ieee802154_set_power(char power) {
  // Cast the signed char to an unsigned char before zero-padding it.
  return command(RADIO_DRIVER, COMMAND_SET_POWER, (unsigned int) (unsigned char) power);
}

int ieee802154_config_commit(void) {
  return command(RADIO_DRIVER, COMMAND_CONFIG_COMMIT, 0);
}

int ieee802154_get_address(unsigned short *addr) {
  if (!addr) return TOCK_EINVAL;
  int res = command(RADIO_DRIVER, COMMAND_GET_ADDR, 0);
  if (res >= 0) {
    // Driver adds 1 to make the value positive.
    *addr = (unsigned short) (res - 1);
  }
  return res;
}

int ieee802154_get_address_long(unsigned char *addr_long) {
  if (!addr_long) return TOCK_EINVAL;
  int err = allow(RADIO_DRIVER, ALLOW_CFG, (void *) addr_long, 8);
  if (err < 0) return err;
  return command(RADIO_DRIVER, COMMAND_GET_ADDR_LONG, 0);
}

int ieee802154_get_pan(unsigned short *pan) {
  if (!pan) return TOCK_EINVAL;
  int res = command(RADIO_DRIVER, COMMAND_GET_PAN, 0);
  if (res >= 0) {
    // Driver adds 1 to make the value positive.
    *pan = (unsigned short) (res - 1);
  }
  return res;
}

int ieee802154_get_channel(unsigned char *channel) {
  if (!channel) return TOCK_EINVAL;
  int res = command(RADIO_DRIVER, COMMAND_GET_PAN, 0);
  if (res >= 0) {
    // Driver adds 1 to make the value positive.
    *channel = (unsigned char) (res - 1);
  }
  return res;
}

int ieee802154_get_power(char *power) {
  if (!power) return TOCK_EINVAL;
  int res = command(RADIO_DRIVER, COMMAND_GET_POWER, 0);
  if (res >= 0) {
    // Driver adds 1 to the power after casting it to unsigned, so this works
    *power = (char) (res - 1);
  }
  return res;
}
