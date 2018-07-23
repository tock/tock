#Userland IP Stack Config

##This file contains notes on the configuration of the IP stack.

###Current design (where each of the following values is configuredm and stored):

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
