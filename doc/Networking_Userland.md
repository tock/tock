Tock Userland Networking Design Document
========================================

_NOTE: This document is a work in progress._

This document describes the current userland interface for the networking stack
on Tock. This document should serve as a description of the abstraction
provided by libTock - what the exact system call interface looks like or how
libTock or the kernel implements this functionality is out-of-scope of this
document.

TODO: Authors/contributors

## Overview
The Tock networking stack and libTock should attempt to expose a networking
interface that is similar to the POSIX networking interface. The primary
motivation for this design choice is that application programmers are used
to the POSIX networking interface design, and significant amounts of code
have already been written for POSIX-style network interfaces. By designing
the libTock networking interface to be as similar to POSIX as possible, we
hope to improve developer experience while enabling the easy transition of
networking code to Tock.

## Design
In order to 

### POSIX Socket API Functions
Below is a fairly comprehensive overview of the POSIX networking socket
interface. Note that much of this functionality pertains to TCP or connection-
based protocols, which we currently do not handle.

- `socket(domain, type, protocol) -> int fd`  
    - `domain`: AF\_INET, AF\_INET6, AF\_UNIX  
    - `type`: SOCK\_STREAM (TCP), SOCK\_DGRAM (UDP), SOCK\_SEQPACKET (?), SOCK\_RAW  
    - `protocol`: IPPROTO\_TCP, IPPROTO\_SCTP, IPPROTO\_UDP, IPPROTO\_DCCP

- `bind(socketfd, my_addr, addrlen) -> int success`
    - `socketfd`: Socket file descriptor to bind to
    - `my_addr`: Address to bind on
    - `addrlen`: Length of address

- `listen(socketfd, backlog) -> int success`
    - `socketfd`: Socket file descriptor
    - `backlog`: Number of pending connections to be queued

    Only necessary for stream-oriented data modes

- `connect(socketfd, addr, addrlen) -> int success`
    - `socketfd`: Socket file descriptor to connect with
    - `addr`: Address to connect to (server protocol address)
    - `addrlen`: Length of address

    When used with connectionless protocols, defines the remote address for
    sending and receiving data, allowing the use of functions such as `send()`
    and `recv()` and preventing the reception of datagrams from other sources.

- `accept(socketfd, cliaddr, addrlen) -> int success`
    - `socketfd`: Socket file descriptor of the listening socket that has the
    connection queued
    - `cliaddr`: A pointer to an address to receive the client's address information
    - `addrlen`: Specifies the size of the client address structure

- `send(socketfd, buffer, length, flags) -> int success`
    - `socketfd`: Socket file descriptor to send on
    - `buffer`: Buffer to send
    - `length`: Length of buffer to send
    - `flags`: Various flags for the transmission

    Note that the `send()` function will only send a message when the `socketfd`
    is connected (including for connectionless sockets)

- `sendto(socketfd, buffer, length, flags, dst_addr, addrlen) -> int success`
    - `socketfd`: Socket file descriptor to send on
    - `buffer`: Buffer to send
    - `length`: Length of buffer to send
    - `flags`: Various flags for the transmission
    - `dst_addr`: Address to send to (ignored for connection type sockets)
    - `addrlen`: Length of `dst_addr`

    Note that if the socket is a connection type, dst_addr will be ignored.

- `recv(socketfd, buffer, length, flags)`
    - `socketfd`: Socket file descriptor to receive on
    - `buffer`: Buffer where the message will be stored
    - `length`: Length of buffer
    - `flags`: Type of message reception

    Typically used with connected sockets as it does not permit the application
    to retrieve the source address of received data.

- `recvfrom(socketfd, buffer, length, flags, address, addrlen)`
    - `socketfd`: Socket file descriptor to receive on
    - `buffer`: Buffer to store the message
    - `length`: Length of the buffer
    - `flags`: Various flags for reception
    - `address`: Pointer to a structure to store the sending address
    - `addrlen`: Length of address structure

    Normally used with connectionless sockets as it permits the application to
    retrieve the source address of received data

- `close(socketfd)`
    - `socketfd`: Socket file descriptor to delete

- `gethostbyname()/gethostbyaddr()`
    Legacy interfaces for resolving host names and addresses

- `select(nfds, readfds, writefds, errorfds, timeout)`
    - `nfds`: The range of file descriptors to be tested (0..nfds)
    - `readfds`: On input, specifies file descriptors to be checked to see if they
    are ready to be read. On output, indicates which file descriptors are ready
    to be read
    - `writefds`: Same as readfds, but for writing
    - `errorfds`: Same as readfds, writefds, but for errors
    - `timeout`: A structure that indicates the max amount of time to block if
    no file descriptors are ready. If None, blocks indefinitely

- `poll(fds, nfds, timeout)`
    - `fds`: Array of structures for file descriptors to be checked. The array
    members are structures which contain the file descriptor, and events
    to check for plus areas to write which events occurred
    - `nfds`: Number of elements in the fds array
    - `timeout`: If 0 return immediately, or if -1 block indefinitely. Otherwise,
    wait at least `timeout` milliseconds for an event to occur

- `getsockopt()/setsockopt()`

### Tock Userland API
Below is a list of desired functionality for the libTock userland API.

- `struct sock_addr_t`  
    `ipv6_addr_t`: IPv6 address (single or ANY)  
    `port_t`: Transport level port (single or ANY)

- `socket() -> int fd`  
    Returns some integer representing a socket structure.

- `list_ifaces() -> ifaces[]`  
    ifaces = (ip, string name [8 char])
    This is a stateless function for listing all current interfaces.  
    TODO: Do we need any arguments to this function

- `udp_socket(socketfd, sock_addr_t)`  
    `socketfd`: Socket object to be initialized as a UDP socket with the given
    address information  
    `sock_addr_t`: Contains an IPv6 address and a port

- `udp_close(socketfd)`  
    `socketfd`: Socket to close

- `send_to(socketfd, buffer, length, sock_addr_t)`  
    - `socketfd`: Socket to send using  
    - `buffer`: Buffer to send  
    - `length`: Length of buffer to send  
    - `sock_addr_t`: Address struct (IPv6 address, port) to send the packet from

- `recv_from(socketfd, buffer, length, sock_addr_t)`  
    - `socketfd`: Receiving socket  
    - `buffer`: Buffer to receive into  
    - `length`: Length of buffer  
    - `sock_addr_t`: Struct where the kernel writes the received packet's sender
    information

### Differences Between the APIs

There are two major differences between the proposed Tock APIs and the standard
POSIX APIs. First, the POSIX APIs must support connection-based protocols such
as TCP, whereas the Tock API is only concerned with connectionless, datagram
based protocols. Second, the POSIX interface has a concept of the `sock_addr_t`
structure, which is used to encapsulate information such as port numbers to
bind on and interface addresses. This makes `bind_to_port` redundant in POSIX,
as we can simply set the port number in the `sock_addr_t` struct when binding.
I think one of the major questions is whether to adopt this convention, or to
use the above definitions for at least the first iteration.

### Example: `ip_sense` 

Given the work-in-progress API design above, here's what the `examples/ip_sense` application would look like.

    #include <ip.h>

    int main(void) {

      // initialization
      // ...
      char packet[64];

      // IP configuration
      int sock = socket();
      bind(sock, list_ifaces()[0]); // ??? TODO no idea here

      while (1) {
        
        // read sensors
        // ...

        // prepare data packet
        int len = snprintf(packet, sizeof(packet), "packet payload");

        // send packet
        int err = send_to(sock, packet, len, "10.0.0.6"); // TODO address fmt

        switch (err) {
          case TOCK_SUCCESS:
            printf("Sent and acknowledged\n");
            break;
          case TOCK_ENOACK:
            printf("Sent but not acknowledged\n");
            break;
          default:
            printf("Error sending packet %d\n", err);
        }

        delay_ms(1000);
      }
    }
