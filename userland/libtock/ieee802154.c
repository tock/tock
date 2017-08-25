#include "ieee802154.h"
#include "timer.h"

const int RADIO_DRIVER = 154;

const int ALLOW_RX  = 0;
const int ALLOW_TX  = 1;
const int ALLOW_CFG = 2;

const int SUBSCRIBE_RX = 0;
const int SUBSCRIBE_TX = 1;

const int COMMAND_STATUS        = 0;
const int COMMAND_SET_ADDR      = 1;
const int COMMAND_SET_ADDR_LONG = 2;
const int COMMAND_SET_PAN       = 3;
const int COMMAND_SET_CHANNEL   = 4;
const int COMMAND_SET_POWER     = 5;
const int COMMAND_CONFIG_COMMIT = 6;

const int COMMAND_GET_ADDR      = 7;
const int COMMAND_GET_ADDR_LONG = 8;
const int COMMAND_GET_PAN       = 9;
const int COMMAND_GET_CHANNEL   = 10;
const int COMMAND_GET_POWER     = 11;

const int COMMAND_MAX_NEIGHBORS = 12;
const int COMMAND_NUM_NEIGHBORS = 13;
const int COMMAND_GET_NEIGHBOR_ADDR      = 14;
const int COMMAND_GET_NEIGHBOR_ADDR_LONG = 15;
const int COMMAND_ADD_NEIGHBOR           = 16;
const int COMMAND_REMOVE_NEIGHBOR        = 17;

const int COMMAND_MAX_KEYS      = 18;
const int COMMAND_NUM_KEYS      = 19;
const int COMMAND_GET_KEY_LEVEL = 20;
const int COMMAND_GET_KEY_ID    = 21;
const int COMMAND_GET_KEY       = 22;
const int COMMAND_ADD_KEY       = 23;
const int COMMAND_REMOVE_KEY    = 24;

const int COMMAND_SEND = 25;

// Temporary buffer used for some commands where the system call interface
// parameters / return codes are not enough te contain the required data.
unsigned char BUF_CFG[27];

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
  int err = command(RADIO_DRIVER, COMMAND_GET_ADDR, 0);
  if (err > 0) {
    // Driver adds 1 to make the value positive.
    *addr = (unsigned short) (err - 1);
  }
  return err;
}

int ieee802154_get_address_long(unsigned char *addr_long) {
  if (!addr_long) return TOCK_EINVAL;
  int err = allow(RADIO_DRIVER, ALLOW_CFG, (void *) addr_long, 8);
  if (err < 0) return err;
  return command(RADIO_DRIVER, COMMAND_GET_ADDR_LONG, 0);
}

int ieee802154_get_pan(unsigned short *pan) {
  if (!pan) return TOCK_EINVAL;
  int err = command(RADIO_DRIVER, COMMAND_GET_PAN, 0);
  if (err > 0) {
    // Driver adds 1 to make the value positive.
    *pan = (unsigned short) (err - 1);
  }
  return err;
}

int ieee802154_get_channel(unsigned char *channel) {
  if (!channel) return TOCK_EINVAL;
  int err = command(RADIO_DRIVER, COMMAND_GET_PAN, 0);
  if (err > 0) {
    // Driver adds 1 to make the value positive.
    *channel = (unsigned char) (err - 1);
  }
  return err;
}

int ieee802154_get_power(char *power) {
  if (!power) return TOCK_EINVAL;
  int err = command(RADIO_DRIVER, COMMAND_GET_POWER, 0);
  if (err > 0) {
    // Driver adds 1 to the power after casting it to unsigned, so this works
    *power = (char) (err - 1);
  }
  return err;
}

int ieee802154_max_neighbors(void) {
  int err = command(RADIO_DRIVER, COMMAND_MAX_NEIGHBORS, 0);
  // Driver adds 1 to ensure it is positive, but on error we want to return 0
  return (err > 0) ? (err - 1) : 0;
}

int ieee802154_num_neighbors(void) {
  int err = command(RADIO_DRIVER, COMMAND_NUM_NEIGHBORS, 0);
  // Driver adds 1 to ensure it is positive, but on error we want to return 0
  return (err > 0) ? (err - 1) : 0;
}

int ieee802154_get_neighbor_address(unsigned index, unsigned short *addr) {
  if (!addr) return TOCK_EINVAL;
  int err = command(RADIO_DRIVER, COMMAND_GET_NEIGHBOR_ADDR, (unsigned int) index);
  if (err > 0) {
    // Driver adds 1 to ensure it is positive.
    *addr = (unsigned short) (err - 1);
  }
  return err;
}

int ieee802154_get_neighbor_address_long(unsigned index, unsigned char *addr_long) {
  if (!addr_long) return TOCK_EINVAL;
  int err = allow(RADIO_DRIVER, ALLOW_CFG, (void *) addr_long, 8);
  if (err < 0) return err;
  return command(RADIO_DRIVER, COMMAND_GET_NEIGHBOR_ADDR_LONG, (unsigned int) index);
}

int ieee802154_get_neighbor(unsigned index,
                            unsigned short *addr,
                            unsigned char *addr_long) {
  int err = ieee802154_get_neighbor_address(index, addr);
  if (err < 0) return err;
  return ieee802154_get_neighbor_address_long(index, addr_long);
}

int ieee802154_add_neighbor(unsigned short addr, unsigned char *addr_long, unsigned *index) {
  if (!addr_long) return TOCK_EINVAL;
  int err = allow(RADIO_DRIVER, ALLOW_CFG, (void *) addr_long, 8);
  if (err < 0) return err;
  err = command(RADIO_DRIVER, COMMAND_ADD_NEIGHBOR, (unsigned int) addr);
  if (err > 0 && index) {
    // Driver adds 1 to ensure it is positive.
    *index = (unsigned) (err - 1);
  }
  return err;
}

int ieee802154_remove_neighbor(unsigned index) {
  return command(RADIO_DRIVER, COMMAND_REMOVE_NEIGHBOR, (unsigned int) index);
}

int ieee802154_max_keys(void) {
  int err = command(RADIO_DRIVER, COMMAND_MAX_KEYS, 0);
  // Driver adds 1 to ensure it is positive, but on error we want to return 0
  return (err > 0) ? (err - 1) : 0;
}

int ieee802154_num_keys(void) {
  int err = command(RADIO_DRIVER, COMMAND_NUM_KEYS, 0);
  // Driver adds 1 to ensure it is positive, but on error we want to return 0
  return (err > 0) ? (err - 1) : 0;
}

int ieee802154_get_key_security_level(unsigned index, security_level_t *level) {
  if (!level) return TOCK_EINVAL;
  int err = command(RADIO_DRIVER, COMMAND_GET_KEY_LEVEL, (unsigned int) index);
  if (err > 0) {
    // Driver adds 1 to ensure it is positive.
    *level = (security_level_t) (err - 1);
  }
  return err;
}

int ieee802154_key_id_bytes(key_id_mode_t key_id_mode) {
  switch (key_id_mode) {
    default:
    case KEY_ID_IMPLICIT:
      return 0;
    case KEY_ID_INDEX:
      return 1;
    case KEY_ID_SRC_4_INDEX:
      return 5;
    case KEY_ID_SRC_8_INDEX:
      return 9;
  }
}

int ieee802154_get_key_id(unsigned index,
                          key_id_mode_t *key_id_mode,
                          unsigned char *key_id) {
  if (!key_id_mode || !key_id) return TOCK_EINVAL;
  int err = allow(RADIO_DRIVER, ALLOW_CFG, (void *) BUF_CFG, 10);
  if (err < 0) return err;
  err = command(RADIO_DRIVER, COMMAND_GET_KEY_ID, (unsigned int) index);
  if (err == TOCK_SUCCESS) {
    *key_id_mode = (key_id_mode_t) (BUF_CFG[0]);
    memcpy(key_id, BUF_CFG + 1, ieee802154_key_id_bytes(*key_id_mode));
  }
  return err;
}

int ieee802154_get_key(unsigned index, unsigned char *key) {
  if (!key) return TOCK_EINVAL;
  int err = allow(RADIO_DRIVER, ALLOW_CFG, (void *) key, 16);
  if (err < 0) return err;
  return command(RADIO_DRIVER, COMMAND_GET_KEY, (unsigned int) index);
}

int ieee802154_get_key_desc(unsigned index,
                            security_level_t *level,
                            key_id_mode_t *key_id_mode,
                            unsigned char *key_id,
                            unsigned char *key) {
  int err = ieee802154_get_key_security_level(index, level);
  if (err < 0) return err;
  err = ieee802154_get_key_id(index, key_id_mode, key_id);
  if (err < 0) return err;
  return ieee802154_get_key(index, key);
}

int ieee802154_add_key(security_level_t level,
                       key_id_mode_t key_id_mode,
                       unsigned char *key_id,
                       unsigned char *key,
                       unsigned *index) {
  if (!key) return TOCK_EINVAL;
  int err = allow(RADIO_DRIVER, ALLOW_CFG, (void *) BUF_CFG, 27);
  if (err < 0) return 0;
  BUF_CFG[0] = level;
  BUF_CFG[1] = key_id_mode;
  int bytes = ieee802154_key_id_bytes(key_id_mode);
  if (bytes > 0) {
    memcpy(BUF_CFG + 2, key_id, bytes);
  }
  memcpy(BUF_CFG + 2 + 9, key, 16);
  err = command(RADIO_DRIVER, COMMAND_ADD_KEY, 0);
  if (err > 0 && index) {
    // Driver adds 1 to ensure it is positive.
    *index = (unsigned) (err - 1);
  }
  return err;
}

int ieee802154_remove_key(unsigned index) {
  return command(RADIO_DRIVER, COMMAND_REMOVE_KEY, (unsigned int) index);
}

// Internal callback for transmission
static int tx_result;
static int tx_acked;
static void tx_done_callback(int result,
                             int acked,
                             __attribute__ ((unused)) int arg3,
                             void* ud) {
  tx_result     = result;
  tx_acked      = acked;
  *((bool*) ud) = true;
}

int ieee802154_send(unsigned short addr,
                    security_level_t level,
                    key_id_mode_t key_id_mode,
                    unsigned char *key_id,
                    const char *payload,
                    unsigned char len) {
  // Setup parameters in ALLOW_CFG and ALLOW_TX
  int err = allow(RADIO_DRIVER, ALLOW_CFG, (void *) BUF_CFG, 11);
  if (err < 0) return err;
  BUF_CFG[0] = level;
  BUF_CFG[1] = key_id_mode;
  int bytes = ieee802154_key_id_bytes(key_id_mode);
  if (bytes > 0) {
    memcpy(BUF_CFG + 2, key_id, bytes);
  }
  err = allow(RADIO_DRIVER, ALLOW_TX, (void *) payload, len);
  if (err < 0) return err;

  // Subscribe to the transmit callback
  bool tx_done = false;
  err = subscribe(RADIO_DRIVER, SUBSCRIBE_TX,
                  tx_done_callback, (void *) &tx_done);
  if (err < 0) return err;

  // Issue the send command and wait for the transmission to be done.
  err = command(RADIO_DRIVER, COMMAND_SEND, (unsigned int) addr);
  if (err < 0) return err;
  yield_for(&tx_done);
  return tx_result;
}

// Internal callback for receive
static void rx_done_callback(__attribute__ ((unused)) int pans,
                             __attribute__ ((unused)) int dst_addr,
                             __attribute__ ((unused)) int src_addr,
                             void* ud) {
  *((bool*) ud) = true;
}

int ieee802154_receive_sync(const char *frame, unsigned char len) {
  // Provide the buffer to the kernel
  int err = allow(RADIO_DRIVER, ALLOW_RX, (void *) frame, len);
  if (err < 0) return err;

  // Subscribe to the received callback
  bool rx_done = false;
  err = subscribe(RADIO_DRIVER, SUBSCRIBE_RX, rx_done_callback, (void *) &rx_done);
  if (err < 0) return err;

  // Wait for a frame
  yield_for(&rx_done);
  return TOCK_SUCCESS;
}

int ieee802154_receive(subscribe_cb callback,
                       const char *frame,
                       unsigned char len) {
  // Provide the buffer to the kernel
  int err = allow(RADIO_DRIVER, ALLOW_RX, (void *) frame, len);
  if (err < 0) return err;
  return subscribe(RADIO_DRIVER, SUBSCRIBE_RX, callback, NULL);
}

int ieee802154_frame_get_length(const char *frame) {
  if (!frame) return 0;
  // data_offset + data_len - 2 header bytes
  return frame[0] + frame[1] - 2;
}

int ieee802154_frame_get_payload_offset(const char *frame) {
  if (!frame) return 0;
  return frame[0];
}

int ieee802154_frame_get_payload_length(const char *frame) {
  if (!frame) return 0;
  return frame[1];
}

addr_mode_t ieee802154_frame_get_dst_addr(__attribute__ ((unused)) const char *frame,
                                          __attribute__ ((unused)) unsigned short *short_addr,
                                          __attribute__ ((unused)) unsigned char *long_addr) {
  // TODO: Inspect the frame and find the offset of the dst address
  return ADDR_NONE;
}

addr_mode_t ieee802154_frame_get_src_addr(__attribute__ ((unused)) const char *frame,
                                          __attribute__ ((unused)) unsigned short *short_addr,
                                          __attribute__ ((unused)) unsigned char *long_addr) {
  // TODO: Inspect the frame and find the offset of the dst address
  return ADDR_NONE;
}

bool ieee802154_frame_get_dst_pan(__attribute__ ((unused)) const char *frame,
                                  __attribute__ ((unused)) unsigned short *pan) {
  // TODO: Actually get the pan
  return false;
}

bool ieee802154_frame_get_src_pan(__attribute__ ((unused)) const char *frame,
                                  __attribute__ ((unused)) unsigned short *pan) {
  // TODO: Actually get the pan
  return false;
}
