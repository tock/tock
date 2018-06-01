#pragma once

#include "tock.h"

#ifdef __cplusplus
extern "C" {
#endif

typedef uint16_t udp_port_t;

typedef struct ipv6_addr {
  uint8_t addr[16];
} ipv6_addr_t;

typedef struct sock_addr {
  ipv6_addr_t addr;
  udp_port_t port;
} sock_addr_t;

typedef struct sock_handle {
  sock_addr_t addr;
} sock_handle_t;

// Creates a new datagram socket bound to an address.
// Returns 0 on success, negative on failure.
int udp_socket(sock_handle_t *handle, sock_addr_t *addr);

// Closes a socket.
// Returns 0 on success, negative on failure.
int udp_close(sock_handle_t *handle);

// Sends data on a socket.
// Returns 0 on success, negative on failure.
ssize_t udp_send_to(sock_handle_t *handle, void *buf, size_t len,
                    sock_addr_t *dst_addr);

// Receives message from a socket asynchronously. The number of bytes 
// received will be passed to the first argument of the supplied callback. 
// To receive more messages, subscribe again after processing a message.
// Returns 0 on success, negative on failure.
ssize_t udp_recv_from(subscribe_cb callback, sock_handle_t *handle, void *buf,
                      size_t len, sock_addr_t *dst_addr);

// Receives data from a socket.
// Returns number of bytes received, negative on failure.
ssize_t udp_recv_from_sync(sock_handle_t *handle, void *buf, size_t len,
                           sock_addr_t *dst_addr);

// Lists `len` interfaces at the array pointed to by `ifaces`. 
// Returns the _total_ number of interfaces, negative on failure.
int udp_list_ifaces(ipv6_addr_t *ifaces, size_t len); 

#ifdef __cplusplus
}
#endif
