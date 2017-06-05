/// Helper code for Signbus

/// Signbus Constants
pub const I2C_MAX_LEN: usize = 255;
pub const HEADER_SIZE: usize = 8;
pub const I2C_MAX_DATA_LEN: usize = I2C_MAX_LEN - HEADER_SIZE;

/// Signbus Errors
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Error {
    CommandComplete,
    AddressNak,
    DataNak,
    ArbitrationLost,
}

/// Signbus Port Layer
#[derive(Clone,Copy,PartialEq)]
pub enum MasterAction {
    Read(u8),
    Write,
}

/// Signbus Packet
#[repr(C, packed)]
#[derive(Copy)]
pub struct SignbusNetworkFlags {
    pub is_fragment: u8, // full message contained multiple packets
    pub is_encrypted: u8,
    pub rsv_wire_bit5: u8,
    pub rsv_wire_bit4: u8,
    pub version: u8,
}

#[repr(C, packed)]
#[derive(Copy)]
pub struct SignbusNetworkHeader {
    pub flags: SignbusNetworkFlags,
    pub src: u8, // address of message
    pub sequence_number: u16, // specific to message not packet
    pub length: u16, // data length + header_size
    pub fragment_offset: u16, // offset of data
}

#[repr(C, packed)]
#[derive(Copy)]
pub struct Packet {
    pub header: SignbusNetworkHeader,
    pub data: [u8; I2C_MAX_DATA_LEN],
}

/// Signbus Packet Clone trait
impl Clone for Packet {
    fn clone(&self) -> Packet {
        *self
    }
}
impl Clone for SignbusNetworkHeader {
    fn clone(&self) -> SignbusNetworkHeader {
        *self
    }
}
impl Clone for SignbusNetworkFlags {
    fn clone(&self) -> SignbusNetworkFlags {
        *self
    }
}

/// Host to network short
pub fn htons(a: u16) -> u16 {
    (((a & 0x00FF) << 8) | ((a & 0xFF00) >> 8))
}

/// Signbus packet -> [u8]
pub fn serialize_packet(packet: Packet, data_len: usize, buf: &mut [u8]) {

    // Network Flags
    buf[0] = packet.header.flags.is_fragment | (packet.header.flags.is_encrypted << 1) |
             (packet.header.flags.rsv_wire_bit5 << 2) |
             (packet.header.flags.rsv_wire_bit4 << 3) |
             (packet.header.flags.version << 4);

    let seq_no = htons(packet.header.sequence_number);
    let length = htons(packet.header.length);
    let fragment_offset = htons(packet.header.fragment_offset);

    // Network Header
    buf[1] = packet.header.src;
    buf[2] = (seq_no & 0x00FF) as u8;
    buf[3] = ((seq_no & 0xFF00) >> 8) as u8;
    buf[4] = (length & 0x00FF) as u8;
    buf[5] = ((length & 0xFF00) >> 8) as u8;
    buf[6] = (fragment_offset & 0x00FF) as u8;
    buf[7] = ((fragment_offset & 0xFF00) >> 8) as u8;

    // Copy packet.data to buf
    for (i, c) in packet.data[0..data_len].iter().enumerate() {
        buf[i + HEADER_SIZE] = *c;
    }

}

/// [u8] -> Signbus packet
pub fn unserialize_packet(buf: &[u8]) -> Packet {
    // Network Flags
    let flags: SignbusNetworkFlags = SignbusNetworkFlags {
        is_fragment: buf[0] & 0x1,
        is_encrypted: (buf[0] >> 1) & 0x1,
        rsv_wire_bit5: (buf[0] >> 2) & 0x1,
        rsv_wire_bit4: (buf[0] >> 3) & 0x1,
        version: (buf[0] >> 4) & 0xF,
    };

    let seq_no = htons((buf[2] as u16) | ((buf[3] as u16) << 8));
    let length = htons((buf[4] as u16) | ((buf[5] as u16) << 8));
    let fragment_offset = htons((buf[6] as u16) | ((buf[7] as u16) << 8));

    // Network Header
    let header: SignbusNetworkHeader = SignbusNetworkHeader {
        flags: flags,
        src: buf[1],
        sequence_number: seq_no,
        length: length,
        fragment_offset: fragment_offset,
    };

    if header.flags.is_fragment == 1 {
        // Copy data from slice to fixed sized array to package into packet
        let mut data: [u8; I2C_MAX_DATA_LEN] = [0; I2C_MAX_DATA_LEN];
        for (i, c) in buf[HEADER_SIZE..I2C_MAX_LEN].iter().enumerate() {
            data[i] = *c;
        }

        // Packet
        Packet {
            header: header,
            data: data,
        }
    } else {
        // Copy data from slice to fixed size array to package into packet
        let end = (header.length - HEADER_SIZE as u16 - header.fragment_offset) as usize;
        let mut data: [u8; I2C_MAX_DATA_LEN] = [0; I2C_MAX_DATA_LEN];
        for (i, c) in buf[HEADER_SIZE..end].iter().enumerate() {
            data[i] = *c;
        }

        // Packet
        Packet {
            header: header,
            data: data,
        }
    }
}
