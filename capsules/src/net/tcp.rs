/* Though TCP has not yet been implemented for the Tock Networking stackm
   this file defines the structure of the TCPHeader and TCPPacket structs
   so that TCPPacket can be included for clarity as part of the
   TransportPacket enum */

pub struct TCPHeader {
    pub src_port: u16,
    pub dst_port: u16,
    pub seq_num: u32,
    pub ack_num: u32,
    pub offset_and_control: u16,
    pub window: u16,
    pub cksum: u16,
    pub urg_ptr: u16,
}

/*
impl<'a> TCPPacket<'a> {
    pub fn new(buf: &mut [u8]) -> TCPPacket<'a> {
        let header = TCPHeader {
            src_port: 0,
            dst_port: 0,
            seq_num: 0,
            ack_num: 0,
            offset_and_control: 0,
            window: 0,
            cksum: 0,
            urg_ptr: 0,
        };
        TCPPacket {
            head: header,
            payload: buf,
            len: 0,
        }
    }
}
*/
