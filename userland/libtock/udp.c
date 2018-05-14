#include <udp.h>
#include <tock.h>

int udp_socket(sock_handle_t *handle, sock_addr_t *addr) {
  // TODO
  return -1;
}

int udp_close(sock_handle_t *handle) {
  // TODO
  return -1;
}

ssize_t udp_send_to(sock_handle_t *handle, const void *buf, size_t len, sock_addr_t *dst_addr) {
  // TODO
  return -1;
}

ssize_t udp_recv_from(sock_handle_t *handle, void *buf, size_t len, sock_addr_t *dst_addr) {
  // TODO
  return -1;
}

int udp_list_ifaces(ipv6_addr_t *ifaces) {
  // TODO
  return -1;
}
