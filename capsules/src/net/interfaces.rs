/* This is a test file being used to test various interface implementations
 * to see if they compile */

use net::ip::{IPAddr};
use kernel::ReturnCode;
use net::ieee802154::{MacAddress};
use net::lowpan_fragment::FragState;
use kernel::hil::time;

/* ============= Transport Layer ========== */

pub struct UDPHeader {
    pub src_port: u16,
    pub dst_port: u16,
    pub len: u16,
    pub cksum: u16,
}

pub struct UDPSocketExample { /* Example UDP socket implementation */
    pub src_ip: IPAddr,
    pub src_port: u16,
}

pub trait UDPSocket:UDPSend {
    fn bind(&self, src_ip: IPAddr, src_port: u16) -> ReturnCode;
    fn send(&self, dest: IPAddr, udp_packet: &'static mut UDPPacket) -> ReturnCode;
    fn send_done(&self, udp_packet: &'static mut UDPPacket, result: ReturnCode);
}

pub struct UDPPacket<'a> { /* Example UDP Packet struct */
    pub head: UDPHeader,
    pub payload: &'a mut [u8], 
    pub len: u16, // length of payload
}

impl<'a> UDPPacket<'a> {
    pub fn reset(&self){} //Sets fields to appropriate defaults    
    pub fn get_offset(&self) -> usize{8} //Always returns 8

    pub fn set_dest_port(&self, port: u16){} 
    pub fn set_src_port(&self, port: u16){}
    pub fn set_len(&self, len: u16){}
    pub fn set_cksum(&self, cksum: u16){}
    pub fn get_dest_port(&self) -> u16{0}
    pub fn get_src_port(&self) -> u16{0}
    pub fn get_len(&self) -> u16{0}
    pub fn get_cksum(&self) -> u16{0}

    pub fn set_payload(&self, payload: &'a [u8]){}

}

pub trait UDPSend {
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

pub enum TransportPacket<'a> { 
    UDP(UDPPacket<'a>),
    /* TCP(TCPPacket), // NOTE: TCP,ICMP,RawIP traits not yet detailed in this
                     // document, but follow logically from UDPPacket trait. 
    ICMP(ICMPPacket),
    Raw(RawIPPacket), */
}

pub struct IP6Packet<'a> {
    pub header: IP6Header,
    pub payload: TransportPacket<'a>,
} 


impl<'a> IP6Packet<'a> {
    pub fn reset(&self){} //Sets fields to appropriate defaults
    pub fn get_offset(&self) -> usize{40} //Always returns 40 until we add options support
    
    // Remaining functions are just getters and setters for the header fields
    pub fn set_tf(&self, tf: u8){}
    pub fn set_flow_label(&self, flow_label: u8){}
    pub fn set_len(&self, len: u16){}
    pub fn set_protocol(&self, proto: u8){}
    pub fn set_dest_addr(&self, dest: IPAddr){}
    pub fn set_src_addr(&self, src: IPAddr){}
    pub fn get_tf(&self) -> u8{0}
    pub fn get_flow_label(&self)-> u8{0}
    pub fn get_len(&self) -> u16{0}
    pub fn get_protocol(&self) -> u8{0}
    pub fn get_dest_addr(&self) -> IPAddr{
        self.header.dst_addr
    }
    pub fn get_src_addr(&self) -> IPAddr{
        self.header.src_addr
    }

    pub fn set_transpo_cksum(&self){} //Looks at internal buffer assuming
    // it contains a valid IP packet, checks the payload type. If the payload
    // type requires a cksum calculation, this function calculates the 
    // psuedoheader cksum and calls the appropriate transport packet function
    // using this pseudoheader cksum to set the transport packet cksum

}

pub trait IP6Send {
    fn send_to(&self, dest: IPAddr, ip6_packet: IP6Packet); //Convenience fn, sets dest addr, sends
    fn send(&self, ip6_packet: IP6Packet); //Length can be determined from IP6Packet
    fn send_done(&self, ip6_packet: IP6Packet, result: ReturnCode);
}

/* ======== 6lowpan Layer ======== */

//TODO: Change this to reference Paul's 6lowpan implementation


pub trait SixlowpanFragment {
    
    fn fragment<'a, A: time::Alarm>(frag_state: FragState<'a, A>, //some frag_state struct must exist
                    ip6_packet: &'a IP6Packet, //Length can be extracted
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

pub trait SixlowpanCompress {

    fn compress(ctx_store: &ContextStore, 
                ip6_datagram: &'static IP6Packet, 
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
    fn send_to(&self, dest: IPAddr, ip6_packet: IP6Packet) {
        ip6_packet.set_dest_addr(dest);
        self.send(ip6_packet);
    }
    
    fn send(&self, ip6_packet: IP6Packet) {
        ip6_packet.set_transpo_cksum(); //If packet is UDP, this sets the cksum

        //Calls to sixlowpan library would go here
    }

    fn send_done(&self, ip6_packet: IP6Packet, result: ReturnCode) {
        //...
    }

}
