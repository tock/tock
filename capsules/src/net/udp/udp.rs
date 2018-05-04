//! This file contains the structs and methods associated with the UDP header.
//! This includes getters and setters for the various header fields, as well
//! as the standard encode/decode functionality required for serializing
//! the struct for transmission.

use net::stream::decode_u16;
use net::stream::encode_u16;
use net::stream::SResult;

/// The `UDPHeader` struct follows the layout for the UDP packet header.
/// Note that the implementation of this struct provides getters and setters
/// for the various fields of the header, to avoid confusion with endian-ness.
#[derive(Copy, Clone)]
pub struct UDPHeader {
    pub src_port: u16,
    pub dst_port: u16,
    pub len: u16,
    pub cksum: u16,
}

impl Default for UDPHeader {
    fn default() -> UDPHeader {
        UDPHeader {
            src_port: 0,
            dst_port: 0,
            len: 8,
            cksum: 0,
        }
    }
}

impl UDPHeader {
    pub fn new() -> UDPHeader {
        UDPHeader::default()
    }
    // TODO: Always returns size of UDP header
    pub fn get_offset(&self) -> usize {
        8
    }

    pub fn set_dst_port(&mut self, port: u16) {
        self.dst_port = port;
    }
    pub fn set_src_port(&mut self, port: u16) {
        self.src_port = port;
    }

    pub fn set_len(&mut self, len: u16) {
        self.len = len;
    }

    pub fn set_cksum(&mut self, cksum: u16) {
        self.cksum = cksum;
    }

    pub fn get_src_port(&self) -> u16 {
        self.src_port
    }

    pub fn get_dst_port(&self) -> u16 {
        self.dst_port
    }

    pub fn get_len(&self) -> u16 {
        self.len
    }

    pub fn get_cksum(&self) -> u16 {
        self.cksum
    }

    pub fn get_hdr_size(&self) -> usize {
        // TODO
        8
    }

    /// This function serializes the `UDPHeader` into the provided buffer.
    ///
    /// # Arguments
    ///
    /// `buf` - A mutable buffer to serialize the `UDPHeader` into
    /// `offset` - The current offset into the provided buffer
    ///
    /// # Return Value
    ///
    /// This function returns the new offset into the buffer wrapped in an
    /// SResult.
    pub fn encode(&self, buf: &mut [u8], offset: usize) -> SResult<usize> {
        stream_len_cond!(buf, self.get_hdr_size() + offset);

        let mut off = offset;
        off = enc_consume!(buf, off; encode_u16, self.src_port);
        off = enc_consume!(buf, off; encode_u16, self.dst_port);
        off = enc_consume!(buf, off; encode_u16, self.len);
        off = enc_consume!(buf, off; encode_u16, self.cksum);
        stream_done!(off, off);
    }

    /// This function deserializes the `UDPHeader` from the provided buffer.
    ///
    /// # Arguments
    ///
    /// `buf` - The byte array corresponding to a serialized `UDPHeader`
    ///
    /// # Return Value
    ///
    /// This function returns a `UDPHeader` struct wrapped in an SResult
    // TODO: Decode has not been tested
    pub fn decode(buf: &[u8]) -> SResult<UDPHeader> {
        stream_len_cond!(buf, 8);
        let mut udp_header = Self::new();
        let off = 0;
        let (off, src_port) = dec_try!(buf, off; decode_u16);
        udp_header.src_port = u16::from_be(src_port);
        let (off, dst_port) = dec_try!(buf, off; decode_u16);
        udp_header.dst_port = u16::from_be(dst_port);
        let (off, len) = dec_try!(buf, off; decode_u16);
        udp_header.len = u16::from_be(len);
        let (off, cksum) = dec_try!(buf, off; decode_u16);
        udp_header.cksum = u16::from_be(cksum);
        stream_done!(off, udp_header);
    }
}
