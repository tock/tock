//! ICMP layer that works with IPv4.
//!
//! - Author: Conor McAvity <cmcavity@stanford.edu>

use net::stream::{encode_u32, encode_u16, encode_u8};
use net::stream::{decode_u32, decode_u16, decode_u8};
use net::stream::SResult;

#[derive(Copy, Clone)]
pub struct ICMP4Header {
    pub code: u8,
    pub cksum: u16,
    pub options: ICMP4HeaderOptions,
}

#[derive(Copy, Clone)]
pub enum ICMP4HeaderOptions {
    Type0 { id: u16, seqno: u16 },
    Type3 { unused: u16, next_mtu: u16 },
    Type11 { unused: u32 },
}

#[derive(Copy, Clone)]
pub enum ICMP4Type {
    Type0,
    Type3,
    Type11,
}

impl ICMP4Header {
    pub fn new(icmp_type: ICMP4Type) -> ICMP4Header {
        let options = match icmp_type {
            ICMP4Type::Type0 => ICMP4HeaderOptions::Type0 { id: 0, seqno: 0 },
            ICMP4Type::Type3 => ICMP4HeaderOptions::Type3 { unused: 0, 
                next_mtu: 0 },
            ICMP4Type::Type11 => ICMP4HeaderOptions::Type11 { unused: 0 },
        };
        
        ICMP4Header {
            code: 0,
            cksum: 0,
            options: options,
        }
    }

    pub fn set_type(&mut self, icmp_type: ICMP4Type) {
        match icmp_type {
            ICMP4Type::Type0 => self.set_options(ICMP4HeaderOptions::Type0 { 
                id: 0, seqno: 0 }),
            ICMP4Type::Type3 => self.set_options(ICMP4HeaderOptions::Type3 { 
                unused: 0, next_mtu: 0 }),
            ICMP4Type::Type11 => self.set_options(ICMP4HeaderOptions::Type11 {
                unused: 0 }),
        }
    }

    pub fn set_code(&mut self, code: u8) {
        self.code = code;
    }

    pub fn set_cksum(&mut self, cksum: u16) {
        self.cksum = cksum;
    }

    pub fn set_options(&mut self, options: ICMP4HeaderOptions) {
        self.options = options;
    }

    pub fn get_type(&self) -> ICMP4Type {
        match self.options {
            ICMP4HeaderOptions::Type0 { id, seqno } => ICMP4Type::Type0,
            ICMP4HeaderOptions::Type3 { unused, next_mtu } => ICMP4Type::Type3,
            ICMP4HeaderOptions::Type11 { unused } => ICMP4Type::Type11,
        }
    }

    pub fn get_type_as_int(&self) -> u8 {
        match self.get_type() {
            ICMP4Type::Type0 => 0,
            ICMP4Type::Type3 => 3,
            ICMP4Type::Type11 => 11,
        }
    }

    pub fn get_code(&self) -> u8 {
        self.code
    }

    pub fn get_cksum(&self) -> u16 {
        self.cksum
    }

    pub fn get_options(&self) -> ICMP4HeaderOptions {
        self.options
    }

    pub fn encode(&self, buf: &mut [u8], offset: usize) -> SResult<usize> {
        let mut off = offset;  

        off = enc_consume!(buf, off; encode_u8, self.get_type_as_int());
        off = enc_consume!(buf, off; encode_u8, self.code);
        off = enc_consume!(buf, off; encode_u16, self.cksum);

        match self.options {
             ICMP4HeaderOptions::Type0 { id, seqno } => {
                off = enc_consume!(buf, off; encode_u16, id);
                off = enc_consume!(buf, off; encode_u16, seqno);
             },
             ICMP4HeaderOptions::Type3 { unused, next_mtu } => {
                off = enc_consume!(buf, off; encode_u16, unused);
                off = enc_consume!(buf, off; encode_u16, next_mtu);
             },
             ICMP4HeaderOptions::Type11 { unused } => {
                off = enc_consume!(buf, off; encode_u32, unused);
             },
        }
        
        stream_done!(off, off);
    }

    pub fn decode(buf: &[u8]) -> SResult<ICMP4Header> {
        let off = 0;
        
        let (off, type_num) = dec_try!(buf, off; decode_u8);
        
        // placeholder value
        let mut icmp_type = ICMP4Type::Type0;

        match type_num {
            0 => icmp_type = ICMP4Type::Type0,
            3 => icmp_type = ICMP4Type::Type3,
            11 => icmp_type = ICMP4Type::Type11,
            _ => return SResult::Error(()),
        }

        let mut icmp_header = Self::new(icmp_type);
        
        let (off, code) = dec_try!(buf, off; decode_u8);
        icmp_header.set_code(code); 
        let (off, cksum) = dec_try!(buf, off; decode_u16);
        icmp_header.set_cksum(u16::from_be(cksum));
       
        match icmp_type {
            ICMP4Type::Type0 => {
                let (off, id) = dec_try!(buf, off; decode_u16);
                let id = u16::from_be(id);
                let (off, seqno) = dec_try!(buf, off; decode_u16);
                let seqno = u16::from_be(seqno);
                icmp_header.set_options(ICMP4HeaderOptions::Type0 { id, 
                    seqno });
            },
            ICMP4Type::Type3 => {
                let (off, unused) = dec_try!(buf, off; decode_u16);
                let unused = u16::from_be(unused);
                let (off, next_mtu) = dec_try!(buf, off; decode_u16);
                let next_mtu = u16::from_be(next_mtu);
                icmp_header.set_options(ICMP4HeaderOptions::Type3 { unused, 
                    next_mtu });
            },
            ICMP4Type::Type11 => {
                let (off, unused) = dec_try!(buf, off; decode_u32);
                let unused = u32::from_be(unused);
                icmp_header.set_options(ICMP4HeaderOptions::Type11 { unused });
            },
        }

        stream_done!(off, icmp_header);
    }
}
