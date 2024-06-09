IEEE 802.15.4 Stack
===================

Tock supports two different implementations of an IEEE 802.15.4 stack in the
kernel. The first version implements packet framing and a MAC layer, virtualizes
the 15.4 interface, and provides a multi-programmable userspace interface. The second version
provides userspace with the ability to send/receive raw 802.15.4 frames as well as directly control the radio, and is appropriate for access by one process at a time.

In-Kernel Stack
---------------

Stack overview:

```text
┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄ Syscall Interface
┌──────────────────────┐
│     RadioDriver      │
└──────────────────────┘
┄┄ ieee802154::device::MacDevice ┄┄
┌──────────────────────┐
│      VirtualMac      │
└──────────────────────┘
┄┄ ieee802154::device::MacDevice ┄┄
┌──────────────────────┐
│        Framer        │
└──────────────────────┘
┄┄ ieee802154::mac::Mac ┄┄
┌──────────────────────┐
│  MAC (ex: AwakeMac)  │
└──────────────────────┘
┄┄ hil::radio::Radio ┄┄
┌──────────────────────┐
│    802.15.4 Radio    │
└──────────────────────┘
```


Raw Stack
---------

Stack overview:

```text
┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄ Syscall Interface
┌─────────────────────────┐
│ phy_driver::RadioDriver │
└─────────────────────────┘
┄┄ hil::radio::Radio ┄┄
┌─────────────────────────┐
│     802.15.4 Radio      │
└─────────────────────────┘
```
