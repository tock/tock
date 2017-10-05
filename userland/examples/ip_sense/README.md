IP Sensor App
=============

An example app for platforms with sensors and an 802.15.4 radio that broadcasts
periodic sensor readings over the network. Currently, it sends raw 802.15.4
packets with statically configured PAN, source and destination addresses, but
as support is added for 6lowpan, Thread, etc, this app will evolve to use those
instead.

## Running

Program the kernel on two imixs. On one, program the `radio_rx` app in
`userland/examples/tests/ieee802154/radio_rx` with the `PRINT_PAYLOAD` and
`PRINT_STRING` options enabled (this app simply prints received 802.15.4
packets to the console). On the other, program the `ip_sense` app.

You'll see packets printed on the console of the form:

```
Packet destination PAN ID: 0xabcd
Packet destination address: 0x0802
Packet source PAN ID: 0xabcd
Packet source address: 0x1540
Received packet with payload of 28 bytes from offset 11
2848 deg C; 3457%; 500 lux;

Packet destination PAN ID: 0xabcd
Packet destination address: 0x0802
Packet source PAN ID: 0xabcd
Packet source address: 0x1540
Received packet with payload of 28 bytes from offset 11
2848 deg C; 3456%; 500 lux;
```
