#include "udp.h"
#include "tock.h"

const int UDP_DRIVER = 0x30002;

static const int ALLOW_RX     = 0;
static const int ALLOW_TX     = 1;
static const int ALLOW_CFG    = 2;
static const int ALLOW_RX_CFG = 3;

static const int SUBSCRIBE_RX = 0;
static const int SUBSCRIBE_TX = 1;

// COMMAND 0 is driver existence check
static const int COMMAND_GET_IFACES = 1;
static const int COMMAND_SEND = 2;

static unsigned char BUF_TX_CFG[2 * sizeof(sock_addr_t)];
static unsigned char BUF_RX_CFG[2 * sizeof(sock_addr_t)];

int udp_socket(sock_handle_t *handle, sock_addr_t *addr) {
  memcpy(&(handle->addr), addr, sizeof(sock_addr_t));
  return TOCK_SUCCESS;
}

int udp_close(__attribute__ ((unused)) sock_handle_t *handle) {
  return TOCK_SUCCESS;
}

static int tx_result;
static void tx_done_callback(int result,
                             __attribute__ ((unused)) int arg2,
                             __attribute__ ((unused)) int arg3,
                             void *ud) {
  tx_result = result;
  *((bool *) ud) = true;
}

ssize_t udp_send_to(sock_handle_t *handle, void *buf, size_t len,
                    sock_addr_t *dst_addr) {

  // Set up source and destination address/port pairs
  int bytes = sizeof(sock_addr_t);
  int err = allow(UDP_DRIVER, ALLOW_CFG, (void *) BUF_TX_CFG, 2 * bytes);
  if (err < 0) return err;

  memcpy(BUF_TX_CFG, &(handle->addr), bytes);
  memcpy(BUF_TX_CFG + bytes, dst_addr, bytes);

  // Set message buffer
  err = allow(UDP_DRIVER, ALLOW_TX, buf, len);
  if (err < 0) return err;

  bool tx_done = false;
  err = subscribe(UDP_DRIVER, SUBSCRIBE_TX, tx_done_callback, (void *) &tx_done);
  if (err < 0) return err;

  err = command(UDP_DRIVER, COMMAND_SEND, 0, 0);
  if (err < 0) return err;
  yield_for(&tx_done);
  return tx_result;
}

static int rx_result;
static void rx_done_callback(int result,
                             __attribute__ ((unused)) int arg2,
                             __attribute__ ((unused)) int arg3,
                             void *ud) {
  rx_result = result;
  *((bool *) ud) = true;
}

ssize_t udp_recv_from_sync(sock_handle_t *handle, void *buf, size_t len,
                           sock_addr_t *dst_addr) {
  int err = allow(UDP_DRIVER, ALLOW_RX, (void *) buf, len);
  if (err < 0) return err;

  // Pass interface to listen on and incoming source address to listen for
  int bytes = sizeof(sock_addr_t);
  err = allow(UDP_DRIVER, ALLOW_RX_CFG, (void *) BUF_RX_CFG, 2 * bytes);
  if (err < 0) return err;

  memcpy(BUF_RX_CFG, &(handle->addr), bytes);
  memcpy(BUF_RX_CFG + bytes, dst_addr, bytes);

  bool rx_done = false;
  err = subscribe(UDP_DRIVER, SUBSCRIBE_RX, rx_done_callback, (void *) &rx_done);
  if (err < 0) return err;

  yield_for(&rx_done);
  return rx_result; 
}

ssize_t udp_recv_from(subscribe_cb callback, sock_handle_t *handle, void *buf,
                      size_t len, sock_addr_t *dst_addr) {

  int err = allow(UDP_DRIVER, ALLOW_RX, (void *) buf, len);
  if (err < 0) return err;

  // Pass interface to listen on and incoming source address to listen for
  int bytes = sizeof(sock_addr_t);
  err = allow(UDP_DRIVER, ALLOW_RX_CFG, (void *) BUF_RX_CFG, 2 * bytes);
  if (err < 0) return err;

  memcpy(BUF_RX_CFG, &(handle->addr), bytes);
  memcpy(BUF_RX_CFG + bytes, dst_addr, bytes);

  return subscribe(UDP_DRIVER, SUBSCRIBE_RX, callback, NULL);
}

int udp_list_ifaces(ipv6_addr_t *ifaces, size_t len) {

  if (!ifaces) return TOCK_EINVAL;

  int err = allow(UDP_DRIVER, ALLOW_CFG, (void *)ifaces, len * sizeof(ipv6_addr_t));
  if (err < 0) return err;

  return command(UDP_DRIVER, COMMAND_GET_IFACES, len, 0);
}

