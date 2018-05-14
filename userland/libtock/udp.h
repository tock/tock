#pragma once

#include "tock.h"

#ifdef __cplusplus
extern "C" {
#endif

typedef uint16_t udp_port_t;

typedef struct ipv6_addr {
  uint8_t addr[16];
} ipv6_addr_t;

typedef struct sock_handle {
  ipv6_addr_t addr;
  udp_port_t port;
} sock_handle_t;

int udp_socket(sock_handle_t *handle, ipv6_addr_t *my_addr, udp_port_t my_port);
int udp_close(sock_handle_t *handle);
int udp_send_to(sock_handle_t *handle, ipv6_addr_t *dst_addr, udp_port_t dst_port);
int udp_recv_from(sock_handle_t *handle, ipv6_addr_t *dst_addr, udp_port_t dst_port);
int udp_list_ifaces(void); 

#ifdef __cplusplus
}
#endif
