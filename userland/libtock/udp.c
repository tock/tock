#include <udp.h>
#include <tock.h>

int udp_socket(sock_handle_t *handle, ipv6_addr_t *my_addr, udp_port_t my_port) {
  return -1;
}

int udp_close(sock_handle_t *handle) {
  return -1;
}

int udp_send_to(sock_handle_t *handle, ipv6_addr_t *dst_addr, udp_port_t dst_port) {
  return -1;
}

int udp_recv_from(sock_handle_t *handle, ipv6_addr_t *dst_addr, udp_port_t dst_port) {
  return -1;
}

int udp_list_ifaces(void) {
  return -1;
} 
