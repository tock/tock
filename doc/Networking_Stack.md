Tock Networking Stack Design Document
=====================================

_NOTE: This document is a work in progress._

This document describes the design of the Networking stack on Tock.

The design described in this document is based off of ideas contributed by
Phil Levis, Amit Levy, Paul Crews, Hubert Teo, Mateo Garcia, Daniel Giffin, and
Hudson Ayers.

### Table of Contents

This document is split into several sections. These are as follows:

1. Principles - Describes the main principles which the design of
   this stack intended to meet,
   along with some justification of why these principles matter. Ultimately,
   the design should follow from these principles.

2. Stack Diagram - Graphically depicts the layout of the stack

3. Explanation of queuing - Describes where packets are queued prior to
   transmission.

4. List of Traits - Describes the traits which will exist at each layer of the
   stack. For traits that may seem surprisingly complex, provide examples of
   specific messages that require this more complex trait as opposed to the
   more obvious, simpler trait that might be expected.

5. Explanation of Queuing - Describe queueing principles for this stack

6. Description of rx path

7. Description of the userland interface to the networking stack

8. Implementation Details - Describes how certain implementations of these
   traits will work, providing some examples with pseudocode or commented
   explanations of functionality

9. Example Message Traversals - Shows how different example messages (Thread or
   otherwise) will traverse the stack

## Principles

1. Keep the simple case simple
   - Sending an IP packet via an established network should not
     require a more complicated interface than send(destination, packet)
   - If functionality were added to allow for the transmission of IP packets over
     the BLE interface, this IP send function should not have to deal with any
     options or MessageInfo structs that include 802.15.4 layer information.
   - This principle reflects a desire to limit the complexity of Thread/the
     tock networking stack to the capsules that implement the stack. This
     prevents the burden of this complexity from being passed up to whatever
     applications use Thread

2. Layering is separate from encapsulation
   - Libraries that handle encapsulation should not be contained within any
     specific layering construct. For example, If the Thread control unit wants
     to encapsulate a UDP payload inside of a UDP packet inside of an IP packet,
     it should be able to do so using encapsulation libraries and get the
     resulting IP packet without having to pass through all of the protocol layers
   - Accordingly, implementations of layers can take advantage of these
     encapsulation libraries, but are not required to.

3. Dataplane traits are Thread-independent
   - For example, the IP trait should not make any assumption that send()
     will be called for a message that will be passed down to the 15.4 layer, in
     case this IP trait is used on top of an implementation that passes IP
     packets down to be sent over a BLE link layer. Accordingly the IP trait
     can not expose any arguments regarding 802.15.4 security parameters.
   - Even for instances where the only implementation of a trait in the near
     future will be a Thread-based implementation, the traits should not
     require anything that limit such a trait to Thread-based implementations

4. Transmission and reception APIs are decoupled
   - This allows for instances where receive and send\_done callbacks should
     be delivered to different clients (ex: Server listening on all addresses
     but also sending messages from specific addresses)
   - Prevents send path from having to navigate the added complexity required
     for Thread to determine whether to forward received messages up the stack

## Stack Diagram

```
IPv6 over ethernet:      Non-Thread 15.4:   Thread Stack:                                       Encapsulation Libraries
+-------------------+-------------------+----------------------------+
|                         Application                                |-------------------\
----------------------------------------+-------------+---+----------+                    \
|TCP Send| UDP Send |TCP Send| UDP Send |  | TCP Send |   | UDP Send |--\                  v
+--------+----------+--------+----------+  +----------+   +----------+   \               +------------+  +------------+
|     IP Send       |     IP Send       |  |         IP Send         |    \      ----->  | UDP Packet |  | TCP Packet |
|                   |                   |  +-------------------------+     \    /        +------------+  +------------+
|                   |                   |                            |      \  /         +-----------+
|                   |                   |                            |       -+------->  | IP Packet |
|                   |                   |       THREAD               |       /           +-----------+
| IP Send calls eth | IP Send calls 15.4|                   <--------|------>            +-------------------------+
| 6lowpan libs with | 6lowpan libs with |                            |       \ ------->  | 6lowpan compress_Packet |
| default values    | default values    |                            |        \          +-------------------------+
|                   |                   |                            |         \         +-------------------------+
|                   |                   +                +-----------|          ------>  | 6lowpan fragment_Packet |
|                   |                   |                | 15.4 Send |                   +-------------------------+
|-------------------|-------------------+----------------------------+
|     ethernet      |          IEEE 802.15.4 Link Layer              |
+-------------------+------------------------------------------------+
```

Notes on the stack:
- IP messages sent via Thread networks are sent through Thread using an IP Send
  method that exposes only the parameters specified in the IP\_Send trait.
  Other parameters of the message (6lowpan decisions, link layer parameters,
  many IP header options) are decided by Thread.
- The stack provides an interface for the application layer to send
  raw IPv6 packets over Thread.
- When the Thread control plane generates messages (MLE messages etc.), they are
  formatted using calls to the encapsulation libraries and then delivered to the
  802.15.4 layer using the 15.4 send function
- This stack design allows Thread to control header elements from transport down
  to link layer, and to set link layer security parameters and more as required
  for certain packers
- The application can either directly send IP messages using the IP Send
  implementation exposed by the Thread stack or it can use the UDP Send
  and TCP send implementation exposed by the Thread stack. If the application
  uses the TCP or UDP send implementations it must use the transport packet library
  to insert its payload inside a packet and set certain header fields.
  The transport send method uses the IP Packet library to set certain
  IP fields before handing the packet off to Thread. Thread then sets other
  parameters at other layers as needed before sending the packet off via the
  15.4 send function implemented for Thread.
- Note that currently this design leaves it up to the application layer to
  decide what interface any given packet will be transmitted from. This is
  because currently we are working towards a minimum functional stack.
  However, once this is working we intend to add a layer below the application
  layer that would handle interface multiplexing by destination address via a
  forwarding table. This should be straightforward to add in to our current
  design.
- This stack does not demonstrate a full set of functionality we are planning to
  implement now. Rather it demonstrates how this setup allows for multiple
  implementations of each layer based off of traits and libraries such that a
  flexible network stack can be configured, rather than creating a network
  stack designed such that applications can only use Thread.


## Explanation of Queuing

Queuing happens at the application layer in this stack.
The userland interface to the
networking stack (described in greater detail in Networking\_Userland.md)
already handles queueing multiple packets sent from userland apps.
In the kernel, any application which wishes to send multiple UDP packets must
handle queueing itself, waiting for a send\_done to return from the radio
before calling send on the next packet in a series of packets.

## List of Traits

This section describes a number of traits which must be implemented by any
network stack. It is expected that multiple implementations of some of these
traits may exist to allow for Tock to support more than just Thread networking.

Before discussing these traits - a note on buffers:

>    Prior implementations of the tock networking stack passed around references
>    to 'static mut [u8] to pass packets along the stack. This is not ideal from a
>    standpoint of wanting
>    to be able to prevent as many errors as possible at compile time. The next iteration
>    of code will pass 'typed' buffers up and down the stack. There are a number
>    of packet library traits defined below (e.g. IPPacket, UDPPacket, etc.).
>    Transport Layer traits will be implemented by a struct that will contain at least one field -
>    a [u8] buffer with lifetime 'a. Lower level traits will simply contain
>    payload fields that are Transport Level packet traits (thanks to a
>    TransportPacket enum). This design allows for all buffers passed to
>    be passed as type 'UDPPacket', 'IPPacket', etc. An added advantage of this
>    design is that each buffer can easily be operated on using the library
>    functions associated with this buffer type.


The traits below are organized by the network layer they would typically be
associated with.

### Transport Layer

Thus far, the only transport layer protocol implemented in Tock is UDP.

Documentation describing the structs and traits that define the UDP layer can
be found in capsules/src/net/udp/(udp.rs, udp\_send.rs, udp\_recv.rs)

Additionally, a driver exists that provides a userland interface via which
udp packets can be sent and received. This is described in greater detail in
Networking\_Userland.md


### Network Stack Receive Path

- The radio in the kernel has a single `RxClient`, which is set as the mac layer (awake_mac, typically)
- The mac layer (i.e. `AwakeMac`) has a single `RxClient`, which is the mac_device(`ieee802154::Framer::framer`)
- The Mac device has a single receive client - `MuxMac` (virtual MAC device).
- The `MuxMac` can have multiple "users" which are of type `MacUser`
- Any received packet is passed to ALL MacUsers, which are expected to filter packets themselves accordingly.
- Right now, we initialize two MacUsers in the kernel (in main.rs/components). These are the 'radio_mac', which is the MacUser for the RadioDriver that enables the userland interface to directly send 802154 frames, and udp_mac, the mac layer that is ultimately associated with the udp userland interface.
- The udp_mac MacUser has a single receive client, which is the `sixlowpan_state` struct
- `sixlowpan_state` has a single rx_client, which in our case is a single struct that implements the `ip_receive ` trait.
- the `ip_receive` implementing struct (`IP6RecvStruct`) has a single client, which is udp_recv, a `UDPReceive` struct.
- The UDPReceive struct is a field of the UDPDriver, which ultimately passes the packets up to userland.

So what are the implications of all this?

1) Currently, any userland app could receive udp packets intended for
anyone else if the app implmenets 6lowpan itself on the received raw frames.

2) Currently, packets are only muxed at the Mac layer.

3) Right now the IPReceive struct receives all IP packets sent to the MAC address of this device, and soon will drop all packets sent to non-local addresses. Right now, the device effectively only has one address anyway, as we only support 6lowpan over 15.4, and as we haven't implemented a loopback interface on the IP_send path. If, in the future, we implement IP forwarding on Tock, we will need to add an IPSend object to the IPReceiver which would then retransmit any packets received that were not destined for local addresses.

## Explanation of Configuration

This section describes how the IP stack is currently configured, and previews how this
configuration will change soon.

###Current design (where each of the following values is configured and stored):

* Source IP address: stored in IPSend struct, set in main.rs

* Destination IP address: Stored in IPPacket on a per packet basis. Can be set
individually for each packet sent from the userland UDP interface. For packets
sent from userland via the UDP example app, this value is pulled from the
INTERFACES array in net/udp/driver.rs.

* src MAC address: stored in the sixlowpan_tx object, currently passed in as the
SRC_MAC_ADDR constant in ipv6_send.rs. This is for sent packets. However, the
src mac is also stored in a register in the radio, which it is loaded into
from the rf233 object when config_commit is called. Right now, the address
known by the radio can be set by calling ieee802154_set_address() from
userland, or by calling set_address on whatever implements the Mac trait.

* dst MAC address: stored in the sixlowpan_tx object, currently passed in as the
DST_MAC_ADDR constant in ipv6_send.rs

* src pan: Stored in three places -- the rf233 object, a register on the rf233
(pulled from rf233.pan when config_commit() is called), and in the
sixlowpan_tx object. The sixlowpant_tx init() call takes in a parameter
radio_pan, which is set by calling the getter to obtain the pan from the
radio. The pan for the radio therefore must be set before init() is
called on sixlowpan_tx. The pan for the radio is set by calling
ieee802154_set_pan() from userspace, or by calling set_pan() on
whatever implements the Mac trait in the kernel.

* dst pan: Stored in the sixlowpan\_tx object, then passed to the
prepare\_data\_frame() 15.4 function to be set for each frame. Set by main.rs.

* radio channel: stored in the radio object (rf233.rs), pulled from a constant
in rf233\_const.rs (PHY\_CHANNEL: u8 = 26;)


### Future Design (where we think each of these should be set):

* Source IP address: Clearly needs to be changed, as it should use the
Interfaces array defined in net/udp/driver.rs if that array is
actually where we are going to store the available interfaces. Worth noting
that this array isn't used when adding anything to the kernel that uses
the IP stack, so it probably doesn't make a lot of sense for the interfaces
to be stored in this file. Instead, the interfaces should be stored somewhere
like ip\_utils.rs or ipv6.rs, and perhaps referenced by udp/driver.rs

* Destination IP address: The current implementation is probably correct.

* src MAC address: This should just be a constant, but should probably be
stored somewhere associated with the MAC layer, not the sixlowpan layer,
as all packets sent by the radio should share the same src MAC.
It doesn't make sense that the src\_mac used for outgoing packets can be
different from the src\_mac loaded to the radio. I propose that the
src\_mac should be stored in a single constant in, perhaps, net/ieee802154.rs,
and that config\_commit() should simply pull that constant into the radio at
runtime. Alternatively, for a more flexible interface, we could still allow
for calls to set\_address on whatever implements the Mac trait, but we could
add a set\_address method to the sixlowpan\_tx object, and have the call
sixlowpan\_tx::set\_address() set the address for the sixlowpan\_tx object
and call Mac::set\_address().

* dst MAC address: This constant could simply be moved to wherever the constant
for the SRC Mac address is moved, but that still doesnt really make sense.
Instead, some method needs to exist to correctly pick the Mac address for
each packet sent. In a typical networking stack, this would occur via
an ARP table or something similar. However, we cannot expect other nodes to
implement ARP. A more realistic and basic implementation might simply allow
for the UDP userland interface to require that a Mac address be passed into
each call to send along with the IP address, or might require that the UDP
userland interface to provide a setter to set the destination IP address
for each packet until the address is changed. Another alternative would
involve a table mapping some set of IP addresses to MAC addresses, and
would allow each userland app to add rows to this table. This method
also is imperfect, as it has indefinite memory requirements given the
absence of dynamic allocation etc. Perhaps each userland app
could be allowed some set number of known addresses (5?) and the IP->mac
mapping for each could be stored in the grant region for that app. If
a given app wanted to talk to more than 5 external addresses, it would have to
add and remove mappings from the list or something?

* src pan: This setup also does not make sense for the same reasons the src MAC
address setup does not make sense. Whatever changes are made for the src MAC
address should also be made for the src pan.

* dst pan: It seems as though the src\_pan and dst\_pan should always match
except in scenarios where the dst\_pan should be broadcast. Perhaps the dst\_pan
field should be replaced with a boolean send\_broadcast field which is set to
1 whenever packets should be sent broadcast, and set to 0 when packets should
be sent with the src\_pan set to the dst\_pan. This would remove any ability for
cross PAN support, but I dont expect us to require such support anyway, and
prevents the possibility of packets being sent with mismatched PAN due to poor
configuration. Would have to make sure that the send\_broadcast field can be
safely set independently by each app, which could be difficult.

* radio channel: Probably fine for now, but I think constants like this and the
radio power should simply be made parameters to new() once the IP stack is
moved over to the component interface.

## Tock Userland Networking Design

This section describes the current userland interface for the networking stack
on Tock. This section should serve as a description of the abstraction
provided by libTock - what the exact system call interface looks like or how
libTock or the kernel implements this functionality is out-of-scope of this
document.

### Overview
The Tock networking stack and libTock should attempt to expose a networking
interface that is similar to the POSIX networking interface. The primary
motivation for this design choice is that application programmers are used
to the POSIX networking interface design, and significant amounts of code
have already been written for POSIX-style network interfaces. By designing
the libTock networking interface to be as similar to POSIX as possible, we
hope to improve developer experience while enabling the easy transition of
networking code to Tock.

### Design

udp.c and udp.h in libtock-c/libtock define the userland interface to the
Tock networking stack. These files interact with capsules/src/net/udp/driver.rs
in the main tock repository. driver.rs implements an interface for sending
and receiving UDP messages. It also exposes a list of interace addresses to
the application layer. The primary functionality embedded in the UDP driver
is within the allow(), subscribe(), and command() calls which can be made to
the driver.

Details of this driver can be found in `doc/syscalls` folder

udp.c and udp.h in libtock-c make it easy to interact with this driver interface.
Important functions available to userland apps written in c include:

`udp_socket()` - sets the port on which the app will receive udp packets,
                 and sets the `src_port` of outgoing packets sent via that socket. Once socket
                 binding is implemented in the kernel, this function will handle reserving ports
                 to listen on and send from.

`udp_close()` - currently just returns success, but once socket binding has been
                implemented in the kernel, this function will handle freeing bound ports.

`udp_send_to()` - Sends a udp packet to a specified addr/port pair, returns the result
                  of the tranmission once the radio has transmitted it (or once a failure has occured).

`udp_recv_from_sync()` - Pass an interface to listen on and an incoming source address
                         to listen for. Sets up a callback to wait for a received packet, and yeilds until that
                         callback is triggered. This function never returns if a packet is not received.

`udp_recv_from()` - Pass an interface to listen on and an incoming source address to
                    listen for. However, this takes in a buffer to which the received packet should be placed,
                    and returns the callback that will be triggered when a packet is received.

`udp_list_ifaces()` - Populates the passed pointer of ipv6 addresses with the available
                      ipv6 addresses of the interfaces on the device. Right now this merely returns a constant
                      hardcoded into the UDP driver, but should change to return the source IP addresses held
                      in the network configuration file once that is created. Returns up to `len` addresses.

Other design notes:

The current design of the driver has a few limitations, these include:

- Currently, any app can listen on any address/port pair

- The current tx implementation allows for starvation, e.g. an app with an earlier app ID can
  starve a later ID by sending constantly.

#### POSIX Socket API Functions
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

#### Tock Userland API
Below is a list of desired functionality for the libTock userland API.

- `struct sock_addr_t`
    `ipv6_addr_t`: IPv6 address (single or ANY)
    `port_t`: Transport level port (single or ANY)

- `struct sock_handle_t`
    Opaque to the user; allocated in userland by malloc (or on the stack)

- `list_ifaces() -> iface[]`
    `ifaces`: A list of `ipv6_addr_t, name` pairs corresponding to all
    interfaces available

- `udp_socket(sock_handle_t, sock_addr_t) -> int socketfd`
    `socketfd`: Socket object to be initialized as a UDP socket with the given
    address information
    `sock_addr_t`: Contains an IPv6 address and a port

- `udp_close(sock_handle_t)`
    `sock_handle_t`: Socket to close

- `send_to(sock_handle_t, buffer, length, sock_addr_t)`
    - `sock_handle_t`: Socket to send using
    - `buffer`: Buffer to send
    - `length`: Length of buffer to send
    - `sock_addr_t`: Address struct (IPv6 address, port) to send the packet from

- `recv_from(sock_handle_t, buffer, length, sock_addr_t)`
    - `sock_handle_t`: Receiving socket
    - `buffer`: Buffer to receive into
    - `length`: Length of buffer
    - `sock_addr_t`: Struct where the kernel writes the received packet's sender
    information

#### Differences Between the APIs

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

An example use of the userland networking stack can be found in libtock-c/examples/ip\_sense

## Implementation Details for potential future Thread implementation

This section was written when the networking stack was incomplete, and aspects
may be outdated. This goes for all sections following this point in the document.

This section was written to include pseudocode examples of how different
implementation of these traits should look for different Thread messages that
might be sent, and for other messages (non-thread) that might be sent using
this messaging stack.


One Example Implementation of IP6Send:

```rust
/* Implementation of IP6Send Specifically for sending MLE messages. This
implementation is incomplete and not entirely syntactically correct. However it
is useful in that it provides insight into the benefit of having IP6Send
merely be implemented as a trait instead of a layer. This function assumes
that the buffer passed in contains an already formatted IP message. (A
previous function would have been used to create the IP Header and place a UDP
message with an MLE payload inside of it). This message then uses the
appropriate 6lowpan trait implementation to compress/fragment this IP message,
then sets the 15_4 link layer headers and settings as required. Accordingly
this function reveals how an implementation of IP6Send could give control to
Thread at the IP layer, 6lowpan layer, and 15.4 layer. */

impl IP6Send for ThreadMLEIP6Send{
    fn sendTo(&self, dest: IP6Addr, ip6_packet: IP6Packet) {
        ip6_packet.setDestAddr(dest);
        self.send(ip6_packet);
    }

    fn send(&self, ip6_packet: IP6Packet) {
        ip6_packet.setTranspoCksum(); //If packet is UDP etc., this sets the cksum
        ctx_store = sixlowpan_comp::ContextStore::new();
        fragState = sixlowpan_frag::fragState::new(ip6_packet);

        /* Note: the below loop should be replaced with repetitions on callbacks, but
        you get the idea - multiple calls to the frag library are required to
        send all of the link layer frames */

        while(fragState != Done) {
            let fragToSend: 15_4_frag_buf = 15_4_6lowpan_frag(&ip6_packet, fragState);
            fragToSend.setSrcPANID(threadPANID);
            if(ip6_packet.is_disc_request()) { // One example of a thread
                                               // decision that affects link layer parameters
                fragToSend.setSrcMAC(MAC::generate_random());
            }
            // etc.... (More Thread decision making)
            let security = securityType::MLESecurity;
            15_4_link_layer_send(fragToSend, security, len);
        }
    }
}

/* Implementation of IP6Send for an application sitting on top of Thread which
simply wants to send an IP message through Thread. For such an instance the
user does not need to worry about setting parameters below the IP layer, as
Thread handles this. This function reflects Thread making those decisions in
such a scenario */
impl IP6Send for IP6SendThroughThread {
    fn sendTo(&self, dest: IP6Addr, ip6_packet: IP6Packet) {
        setDestAddr(ip6_packet, dest);
        self.send(ip6_packet);
    }

    fn send(&self, ip6_packet: IP6Packet) {
        ip6_packet.setTranspoCksum(); //If packet is UDP, this sets the cksum
        fragState = new fragState(ip6_packet);
        while(fragState != Done) {
            let fragToSend: 15_4_frag_buf = 15_4_6lowpan_frag(&ip6_packet, fragState);
            fragToSend.setSrcPANID(threadPANID);
            fragToSend.setSrcMAC(getSrcMacFromSrcIPaddr(ip6_packet.getSrcIP));
            // etc....
            let security = securityType::LinkLayerSec;
            15_4_link_layer_send(fragToSend, security, len);
        }
    }
}

/* Implementation of UDPSend for an application sitting on top of Thread which
simply wants to send a UDP message through Thread. This simply calls on the
appropriate implementation of IP6Send sitting beneath it. Recall that this
function assumes it is passed an already formatted UDP Packet. Also recall the
assumption that the IPSend function will calculate and set the UDP cksum. */

impl UDPSend for UDPSendThroughThread {
    fn send(&self, dest, udp_packet: UDPPacket) {

        let trans_pkt = TransportPacket::UDP(udp_packet);

        ip6_packet = IPPacket::new(trans_pkt);

        /* First, library calls to format IP Packet */
        ip6_packet.setDstAddr(dest);
        ip6_packet.setSrcAddr(THREAD_GLOBAL_SRC_IP_CONST); /* this fn only
          called for globally destined packets sent over Thread network */
        ip6_packet.setTF(0);
        ip6_packet.setHopLimit(64);
        ip6_packet.setProtocol(UDP_PROTO_CONST);
        ip6_packet.setLen(40 + trans_pkt.get_len());
        /* Now, send the packet */
        IP6SendThroughThread.sendTo(dest, ip6_packet);
    }
}
```

The above implementations are not meant to showcase accurate code, but rather
give an example as to how multiple implementations of a given trait can be
useful in the creation of a flexible network stack. Right now this section
does not contain much, as actually writing all of this example code seems less
productive than simply writing and testing actual code in Tock. These examples
are merely intended to give an idea of how traits will be used in this stack,
so please don't bother nitpicking the examples (for instance, I realize it
doesn't make sense that the function doesn't set all of the IP Header fields,
and that there should be decision making occurring to set the source address,
etc.)

## Example Message Traversals

The Thread specification determines an entire control plane that spans many
different layers in the OSI networking model. To adequately understand the
interactions and dependencies between these layers' behaviors, it might help to
trace several types of messages and see how each layer processes the different
types of messages. Let's trace carefully the way OpenThread handles messages.

We begin with the most fundamental message: a data-plane message that does not
interact with the Thread control plane save for passing through a
Thread-defined network interface. Note that some of the procedures in the below
traces will not make sense when taken independently: the responsibility-passing
will only make sense when all the message types are taken as a whole.
Additionally, no claim is made as to whether or not this sequence of callbacks
is the optimal way to express these interactions: it is just OpenThread's way
of doing it.

### Data plane: IPv6 datagram

1. Upper layer (application) wants to send a payload
  - Provides payload
  - Specifies the IP6 interface to send it on (via some identifier)
  - Specifies protocol (IP6 next header field)
  - Specifies destination IP6 address
  - Possibly doesn't specify source IP6 address
2. IP6 interface dispatcher (with knowledge of all the interfaces) fills in the
  IP6 header and produces an IP6 message
  - Payload, protocol, and destination address used directly from the upper layer
  - Source address is more complicated
    - If the address is specified and is not multicast, it is used directly
    - If the address is unspecified or multicast, source address is determined
      from the specific IP6 selected AND the destination address via a matching scheme on
      the addresses associated with the interface.
  - Now that the addresses are determined, the IP6 layer computes the pseudoheader
    checksum.
    - If the application layer's payload has a checksum that includes the pseudoheader
      (UDP, ICMP6), this partial checksum is now used to update the checksum field in the payload.
3. The actual IP6 interface (Thread-controlled) tries to send that message
  - First step is to determine whether the message can be sent immediately or not (sleepy child or not).
       This passes the message to the scheduler. This is important for sleepy children where there is a
       control scheme that determines when messages are sent.
  - Next, determine the MAC src/dest addresses.
    - If this is a direct transmission, there is a source matching scheme to determine if the destination address
      used should be short or long. The same length is used for the source MAC address, obtained from the MAC interface.
  - Notify the MAC layer to notify you that your message can be sent.
4. The MAC layer schedules its transmissions and determines that it can send the above message
  - MAC sets the transmission power
  - MAC sets the channel differently depending on the message type
5. The IP6 interface fills up the frame. This is the chance for the IP6 interface to do things like
  fragmentation, retransmission, and so on. The MAC layer just wants a frame.
  - XXX: The IP6 interface fills up the MAC header. This should really be the responsibility of the MAC layer.
    Anyway, here is what is done:
    - Channel, source PAN ID, destination PAN ID, and security modes are determined by message type.
      Note that the channel set by the MAC layer is sometimes overwritten.
    - A mesh extension header is added for some messages. (eg. indirect transmissions)
  - The IP6 message is then 6LoWPAN-compressed/fragmented into the payload section of the frame.
6. The MAC layer receives the raw frame and tries to send it
  - MAC sets the sequence number of the frame (from the previous sequence number for the correct link neighbor),
    if it is not a retransmission
  - The frame is secured if needed. This is another can of worms:
    - Frame counter is dependent on the link neighbor and whether or not the frame is a retransmission
    - Key is dependent on which key id mode is selected, and also the link neighbor's key sequence
    - Key sequence != frame counter
    - One particular mode requires using a key, source and frame counter that is a Thread-defined constant.
  - The frame is transmitted, an ACK is waited for, and the process completes.

As you can see, the data dependencies are nowhere as clean as the OSI model
dictates. The complexity mostly arises because

- Layer 4 checksum can include IPv6 pseudoheader
- IP6 source address (mesh local? link local? multicast?) is determined by
  interface and destination address
- MAC src/dest addresses are dependent on the next device on the route to the
  IP6 destination address
- Channel, src/dest PAN ID, security is dependent on message type
- Mesh extension header presence is dependent on message type
- Sequence number is dependent on message type and destination

Note that all of the MAC layer dependencies in step 5 can be pre-decided so
that the MAC layer is the only one responsible for writing the MAC header.

This gives a pretty good overview of what minimally needs to be done to even be
able to send normal IPv6 datagrams, but does not cover all of Thread's
complexities. Next, we look at some control-plane messages.

### Control plane: MLE messages

1. The MLE layer encapsulates its messages in UDP on a constant port
  - Security is determined by MLE message type. If MLE-layer security is
    required, the frame is secured using the same CCM* encryption scheme used
    in the MAC layer, but with a different key discipline.
  - MLE key sequence is global across a single Thread device
  - MLE sets IP6 source address to the interface's link local address
2. This UDP-encapsulated MLE message is sent to the IP6 dispatch again
3. The actual IP6 interface (Thread-controlled) tries to send that message
4. The MAC layer schedules the transmission
5. The IP6 interface fills up the frame.
  - MLE messages disable link-layer security when MLE-layer security is
    present. However, if link-layer security is disabled and the MLE message
    doesn't fit in a single frame, link-layer security is enabled so that
    fragmentation can proceed.
6. The MAC layer receives the raw frame and tries to send it

The only cross-layer dependency introduced by the MLE layer is the dependency
between MLE-layer security and link-layer security. Whether or not the MLE
layer sits atop an actual UDP socket is an implementation detail.

### Control plane: Mesh forwarding

If Thread REED devices are to be eventually supported in Tock, then we must
also consider this case. If a frame is sent to a router which is not its final
destination, then the router must forward that message to the next hop.

1. The MAC layer receives a frame, decrypts it and passes it to the IP6 interface
2. The IP6 reception reads the frame and realizes that it is an indirect
   transmission that has to be forwarded again
  - The frame must contain a mesh header, and the HopsLeft field in it should
    be decremented
  - The rest of the payload remains the same
  - Hence, the IP6 interface needs to send a raw 6LoWPAN-compressed frame
3. The IP6 transmission interface receives a raw 6LoWPAN-compressed frame to be
   transmitted again
  - This frame must still be scheduled: it might be destined for a sleepy
    device that is not yet awake
4. The MAC layer schedules the transmission
5. The IP6 transmission interface copies the frame to be retransmitted
   verbatim, but with the modified mesh header and a new MAC header
6. The MAC layer receives the raw frame and tries to send it

This example shows that the IP6 transmission interface may need to handle more
message types than just IP6 datagrams: there is a case where it is convenient
to be able to handle a datagram that is already 6LoWPAN compressed.

### Control plane: MAC data polling

From time to time, a sleepy edge device will wake up and begin polling its
parent to check if any frames are available for it. This is done via a MAC
command frame, which must still be sent through the transmission pipeline with
link security enabled (Key ID mode 1).  OpenThread does this by routing it
through the IP6 transmission interface, which arguably isn't the right choice.

1. Data poll manager send a data poll message directly to the IP6 transmission
   interface, skipping the IP6 dispatch
2. The IP6 transmission interface notices the different type of message, which
   always warrants a direct transmission.
3. The MAC layer schedules the transmission
4. The IP6 transmission interface fills in the frame
  - The MAC dest is set to the parent of this node and the MAC src is set to be
    the same length as the address of the parent
  - The payload is filled up to contain the Data Request MAC command
  - The MAC security level and key ID mode is also fixed for MAC commands under
    the Thread specification
5. The MAC layer secures the frame and sends it out

We could imagine giving the data poll manager direct access as a client of the
MAC layer to avoid having to shuffle data through the IP6 transmission
interface. This is only justified because MAC command frames are never
6LoWPAN-compressed or fragmented, nor do they depend on the IP6 interface in
any way.

### Control plane: Child supervision

This type of message behaves similarly to the MAC data polls. The message is
essentially and empty MAC frame, but OpenThread chooses to also route it
through the IP6 transmission interface. It would be far better to allow a child
supervision implementation to be a direct client of the MAC interface.

### Control plane: Joiner entrust and MLE announce

These two message types are also explicitly marked, because they require a
specific Key ID Mode to be selected when producing the frame for the MAC
interface.

### Caveat about MAC layer security

So far, it seems like we can expect the MAC layer to have no cross-layer
dependencies: it receives frames with a completely specified description of how
they are to be secured and transmitted, and just does so. However, this is not
entirely the case.

When the frame is being secured, the key ID mode has been set by the upper
layers as described above, and this key ID mode is used to select between a few
different key disciplines. For example, mode 0 is only used by Joiner entrust
messages and uses the Thread KEK sequence. Mode 1 uses the MAC key sequence and
Mode 2 is a constant key used only in MLE announce messages. Hence, this key ID
mode selection is actually enabling an upper layer to determine the specific
key being used in the link layer.

Note that we cannot just reduce this dependency by allowing the upper layer to
specify the key used in MAC encryption. During frame reception, the MAC layer
itself has to know which key to use in order to decrypt the frames correctly.
