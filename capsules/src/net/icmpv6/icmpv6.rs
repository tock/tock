//! This file contains the types, structs and methods associated with the
//! ICMPv6 header, including getter and setter methods and encode/decode
//! functionality necessary for transmission.
//!
//! - Author: Conor McAvity <cmcavity@stanford.edu>

use net::stream::SResult;
use net::stream::{decode_u16, decode_u32, decode_u8};
use net::stream::{encode_u16, encode_u32, encode_u8};

/// A struct representing an ICMPv6 header.
#[derive(Copy, Clone)]
pub struct ICMP6Header {
    pub code: u8,
    pub cksum: u16,
    pub options: ICMP6HeaderOptions,
    pub len: u16, // Not a real ICMP field, here for convenience
}

#[derive(Copy, Clone)]
pub enum ICMP6HeaderOptions {
    Type1 { unused: u32 },
    Type3 { unused: u32 },
    Type128 { id: u16, seqno: u16 },
    Type129 { id: u16, seqno: u16 },
}

#[derive(Copy, Clone)]
pub enum ICMP6Type {
    Type1, // Destination Unreachable
    Type3, // Time Exceeded
    Type128, // Echo Request
    Type129, // Echo Reply
}

impl ICMP6Header {
    pub fn new(icmp_type: ICMP6Type) -> ICMP6Header {
        let options = match icmp_type {
            ICMP6Type::Type1 => ICMP6HeaderOptions::Type1 { unused: 0 },
            ICMP6Type::Type3 => ICMP6HeaderOptions::Type3 { unused: 0 },
            ICMP6Type::Type128 => ICMP6HeaderOptions::Type128 { id: 0, seqno: 0 },
            ICMP6Type::Type129 => ICMP6HeaderOptions::Type129 { id: 0, seqno: 0 },
        };

        ICMP6Header {
            code: 0,
            cksum: 0,
            options: options,
            len: 0,
        }
    }

    pub fn set_type(&mut self, icmp_type: ICMP6Type) {
        match icmp_type {
            ICMP6Type::Type1 => self.set_options(ICMP6HeaderOptions::Type1 { unused: 0 }),
            ICMP6Type::Type3 => self.set_options(ICMP6HeaderOptions::Type3 { unused: 0 }),
            ICMP6Type::Type128 => self.set_options(ICMP6HeaderOptions::Type128 { id: 0, seqno: 0 }),
            ICMP6Type::Type129 => self.set_options(ICMP6HeaderOptions::Type129 { id: 0, seqno: 0 }),
        }
    }

    pub fn set_code(&mut self, code: u8) {
        self.code = code;
    }

    pub fn set_cksum(&mut self, cksum: u16) {
        self.cksum = cksum;
    }

    pub fn set_options(&mut self, options: ICMP6HeaderOptions) {
        self.options = options;
    }

    pub fn set_len(&mut self, len: u16) {
        self.len = len;
    }

    pub fn get_type(&self) -> ICMP6Type {
        match self.options {
            ICMP6HeaderOptions::Type1 { .. } => ICMP6Type::Type1,
            ICMP6HeaderOptions::Type3 { .. } => ICMP6Type::Type3,
            ICMP6HeaderOptions::Type128 { .. } => ICMP6Type::Type128,
            ICMP6HeaderOptions::Type129 { .. } => ICMP6Type::Type129,
        }
    }

    pub fn get_type_as_int(&self) -> u8 {
        match self.get_type() {
            ICMP6Type::Type1 => 1,
            ICMP6Type::Type3 => 3,
            ICMP6Type::Type128 => 128,
            ICMP6Type::Type129 => 129,
        }
    }

    pub fn get_code(&self) -> u8 {
        self.code
    }

    pub fn get_cksum(&self) -> u16 {
        self.cksum
    }

    pub fn get_options(&self) -> ICMP6HeaderOptions {
        self.options
    }

    pub fn get_len(&self) -> u16 {
        return self.len;
    }

    pub fn get_hdr_size(&self) -> usize {
        return 8;
    }

    /// Serializes an `ICMP6Header` into a buffer.
    ///
    /// # Arguments
    ///
    /// `buf` - A buffer to serialize the `ICMP6Header` into
    /// `offset` - The current offset into the provided buffer
    ///
    /// # Return Value
    ///
    /// This function returns the new offset into the buffer,
    /// wrapped in an SResult
    pub fn encode(&self, buf: &mut [u8], offset: usize) -> SResult<usize> {
        let mut off = offset;

        off = enc_consume!(buf, off; encode_u8, self.get_type_as_int());
        off = enc_consume!(buf, off; encode_u8, self.code);
        off = enc_consume!(buf, off; encode_u16, self.cksum);

        match self.options {
            ICMP6HeaderOptions::Type1 { unused } |
            ICMP6HeaderOptions::Type3 { unused } => {
                off = enc_consume!(buf, off; encode_u32, unused);
            }
            ICMP6HeaderOptions::Type128 { id, seqno } |
            ICMP6HeaderOptions::Type129 { id, seqno } => {
                off = enc_consume!(buf, off; encode_u16, id);
                off = enc_consume!(buf, off; encode_u16, seqno);
            }
        }

        stream_done!(off, off);
    }

    /// Deserializes an `ICMP6Header` from a buffer.
    ///
    /// # Arguments
    ///
    /// `buf` - The byte array corresponding to the serialized `ICMP6Header`
    ///
    /// # Return Value
    ///
    /// This function returns the `ICMP6Header`, wrapped in an SResult
    pub fn decode(buf: &[u8]) -> SResult<ICMP6Header> {
        let off = 0;
        let (off, type_num) = dec_try!(buf, off; decode_u8);

        let icmp_type = match type_num {
            1 => ICMP6Type::Type1,
            3 => ICMP6Type::Type3,
            128 => ICMP6Type::Type128,
            129 => ICMP6Type::Type129,
            _ => return SResult::Error(()),
        };

        let mut icmp_header = Self::new(icmp_type);

        let (off, code) = dec_try!(buf, off; decode_u8);
        icmp_header.set_code(code);
        let (off, cksum) = dec_try!(buf, off; decode_u16);
        icmp_header.set_cksum(u16::from_be(cksum));

        match icmp_type {
            ICMP6Type::Type1 => {
                let (_off, unused) = dec_try!(buf, off; decode_u32);
                let unused = u32::from_be(unused);
                icmp_header.set_options(ICMP6HeaderOptions::Type1 { unused });
            }
            ICMP6Type::Type3 => {
                let (_off, unused) = dec_try!(buf, off; decode_u32);
                let unused = u32::from_be(unused);
                icmp_header.set_options(ICMP6HeaderOptions::Type3 { unused });
            }
            ICMP6Type::Type128 => {
                let (_off, id) = dec_try!(buf, off; decode_u16);
                let id = u16::from_be(id);
                let (_off, seqno) = dec_try!(buf, off; decode_u16);
                let seqno = u16::from_be(seqno);
                icmp_header.set_options(ICMP6HeaderOptions::Type128 { id, seqno });
            }
            ICMP6Type::Type129 => {
                let (_off, id) = dec_try!(buf, off; decode_u16);
                let id = u16::from_be(id);
                let (_off, seqno) = dec_try!(buf, off; decode_u16);
                let seqno = u16::from_be(seqno);
                icmp_header.set_options(ICMP6HeaderOptions::Type129 { id, seqno });
            }
        }

        stream_done!(off, icmp_header);
    }
}
