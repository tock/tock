/* This is a test file being used to test various interface implementations
 * to see if they compile */

use net::ip::{IPAddr};
use kernel::ReturnCode;
use net::ieee802154::{MacAddress};
use net::lowpan_fragment::FragState;
use kernel::hil::time;

/* ============= Transport Layer ========== */

struct UDPHeader {
    pub src_port: u16,
    pub dst_port: u16,
    pub len: u16,
    pub cksum: u16,
}

struct UDPSocketExample { /* Example UDP socket implementation */
    pub src_ip: IPAddr,
    pub src_port: u16,
}

trait UDPSocket:UDPSend {
    fn bind(&self, src_ip: IPAddr, src_port: u16) -> ReturnCode;
    fn send(&self, dest: IPAddr, udp_packet: &'static mut UDPPacket) -> ReturnCode;
    fn send_done(&self, udp_packet: &'static mut UDPPacket, result: ReturnCode);
}

struct UDPPacketExample<'a> { /* Example UDP Packet struct */
    pub head: UDPHeader,
    pub payload: &'a [u8], //Not mutable!
    pub len: u16, // length of payload
}

pub trait UDPPacket {
    fn reset(&self); //Sets fields to appropriate defaults    
    fn get_offset(&self) -> usize; //Always returns 8

    fn set_dest_port(&self, port: u16); 
    fn set_src_port(&self, port: u16);
    fn set_len(&self, len: u16);
    fn set_cksum(&self, cksum: u16);
    fn get_dest_port(&self) -> u16;
    fn get_src_port(&self) -> u16;
    fn get_len(&self) -> u16;
    fn get_cksum(&self) -> u16;

    fn set_payload<'a>(&self, payload: &'a [u8]);

    /* Note that no UDP cksum function is required here. It is my belief that
       it makes more sense to require the IP layer implement a function to calculate
       the UDP cksum, as the IP layer is what should have access to all of the fields
       neccessary to construct the IPv6 Pseudoheader. Further, openThread uses
       this same approach*/
}

trait UDPSend {
    fn send(dest: IPAddr, udp_packet: &'static mut UDPPacket); // dest rqrd
    fn send_done(buf: &'static mut UDPPacket, result: ReturnCode);
}

/* Notes on this UDP implementation:
  - Want to require a socket be used to call UDPSend at the application level
  - May not want this requirement within Thread, so allow Thread to directly
    call UDPSend (this is the reason for seperation between UDPSocket and
    UDPSend traits) */

/* ======== Network Layer ======== */

pub struct IP6Header {
    pub version_class_flow: [u8; 4],
    pub payload_len: u16,
    pub next_header: u8,
    pub hop_limit: u8,
    pub src_addr: IPAddr,
    pub dst_addr: IPAddr,
}


//BELOW: One idea was to use an enum, but I think there are issues with this.
/*
pub enum TransportPacket { //doesnt work, cant have traits as fields of enum
    UDP(UDPPacket),
    TCP(TCPPacket), // NOTE: TCP,ICMP,RawIP traits not yet detailed in this
                     // document, but follow logically from UDPPacket trait. 
    ICMP(ICMPPacket),
    Raw(RawIPPacket),
}


pub struct IP6Packet_ex {
    pub header: IP6Header;
    pub payload: TransportPacket; //yeah this doesnt work
} */



/* Idea: have IP6_packet_udp_payload, IP6_packet_tcp_payload ... structs. This
   would work if the IP6packet trait had fns like returnUDPpayloadifUDP() ->
   UDPPacket etc. */

pub struct IP6PacketUdpPayload { //impl IP6Packet
    pub header: IP6Header,
    pub payload: UDPPacket,
}
/* //Commented out bc TCPPacket trait not created yet
pub struct IP6Packet_tcp_payload { //impl IP6Packet
    pub header: IP6Header;
    pub payload: TCPPacket;
}
*/




trait IP6Packet {
    fn new_udp<T: IP6Packet>(payload: UDPPacket) -> T;
//    fn new_tcp(payload: TCPPacket) -> IP6Packet;
    fn reset(&self); //Sets fields to appropriate defaults
    fn get_offset(&self) -> usize; //Always returns 40 until we add options support
    
    // Remaining functions are just getters and setters for the header fields
    fn set_tf(&self, tf: u8);
    fn set_flow_label(&self, flow_label: u8);
    fn set_len(&self, len: u16);
    fn set_protocol(&self, proto: u8);
    fn set_dest_addr(&self, dest: IPAddr);
    fn set_src_addr(&self, src: IPAddr);
    fn get_tf(&self) -> u8;
    fn get_flow_label(&self)-> u8;
    fn get_len(&self) -> u16;
    fn get_protocol(&self) -> u8;
    fn get_dest_addr(&self) -> IPAddr;
    fn get_src_addr(&self) -> IPAddr;

    fn calc_and_set_udp_cksum(&self) -> u16; //Looks at internal buffer assuming
    // it contains a valid IP packet, checks if it contains a UDP packet. If so, it
    // calculates the UDP cksum using the IP and UDP fields and returns it.

    // Now, functions to determine which type of IP6Packet this is..
    fn get_payload_if_udp<T: UDPPacket>(&self) -> Option<T>; //Returns UDPPacket if the IP payload is a UDP packet. Otherwise returns None.
  /*fn get_payload_if_tcp(&self) -> Option<TCPPacket>; */
    // etc..
}

trait IP6Send {
    fn send_to<T: IP6Packet>(&self, dest: IPAddr, ip6_packet: &'static mut T); //Convenience fn, sets dest addr, sends
    fn send<T: IP6Packet>(&self, ip6_packet: &'static mut T); //Length can be determined from IP6Packet
    fn send_done<T: IP6Packet>(&self, ip6_packet: &'static mut T, result: ReturnCode);
}

/* ======== 6lowpan Layer ======== */


trait SixlowpanFragment {
    
    fn fragment<'a, A: time::Alarm, T: IP6Packet>(frag_state: FragState<'a, A>, //some frag_state struct must exist
                    ip6_packet: &'a T, //Length can be extracted
                    ) -> Result<ReturnCode, &'static mut [u8]>; // Returned buffer
                                                                // is the link layer frame.
//Where will the frame buffer come from? Passed in? 
}

pub trait ContextStore {
    /* fn get_context_from_addr(&self, ip_addr: IPAddr) -> Option<Context>;
    fn get_context_from_id(&self, ctx_id: u8) -> Option<Context>;
    fn get_context_0(&self) -> Context;
    fn get_context_from_prefix(&self, prefix: &[u8], prefix_len: u8) -> Option<Context>;*/
}

trait SixlowpanCompress {

    fn compress<T: IP6Packet>(ctx_store: &ContextStore, 
                ip6_datagram: &'static T, 
                src_mac_addr: MacAddress,
                dst_mac_addr: MacAddress, 
                buf: &mut [u8]) -> Result<(usize, usize), ()>;
// Question: What is the purpose of buf here?
}

/*=================================================================================================*/

/* Now, some code to test these interfaces... */

pub struct ThreadMLEIP6Send {
    pub id: u16, //Placeholder
}
/*
pub struct SixlowFrag {
    pub tmp: u16, //placeholder
}

impl SixlowpanFragment for SixlowFragm {
    pub fn fragment <'a, A: time::Alarm. T: IP6Packet>(frag_state: FragState<'a, A>, 
                    ip6_packet: &'a T) -> Result<ReturnCode, &'static mut [u8]> {

        //How do I return a static array here...?
    }
}

pub struct SixlowComp {
    pub tmp: u16,
} */

impl IP6Send for ThreadMLEIP6Send{ /* MLE Based example */
    fn send_to<T: IP6Packet>(&self, dest: IPAddr, ip6_packet: T) {
        ip6_packet.set_dest_addr(dest);
        self.send(ip6_packet);
    }
    
    fn send<T: IP6Packet>(&self, ip6_packet: T) {
        ip6_packet.calc_and_set_udp_checksum(); //If packet is UDP, this sets the cksum

        //Calls to sixlowpan library would go here
    }

    fn send_done<T: IP6Packet>(&self, ip6_packet: &'static mut T, result: ReturnCode) {
        //...
    }

}
