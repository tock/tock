//! These structs define permissions for network related capabilities
/*

const MAX_ADDR_SET_SIZE: usize = 16;
const MAX_PORT_SET_SIZE: usize = 16;
const MAX_DATA_SEGMENT_SIZE: usize = 1024;
const KEY_BYTES: usize = 32;

pub enum AddrRange {
    // TODO: provide netmask option?
    Any,     // Any address
    NoAddrs, // Is this one necessary?
    AddrSet([u32; MAX_ADDR_SET_SIZE]),
    Range(u32, u32),
    Addr(u32),
}

pub enum PortRange {
    Any,
    NoPorts,
    PortSet([u16; MAX_PORT_SET_SIZE]),
    Range(u16, u16),
    Port(u16),
}

// Should these structs be unsafe?
pub struct IpPermissions {
    remote_addrs: AddrRange, // local vs. remote
                             //recv_addrs: AddrRange, // AddrRange is for remote
}

pub struct UdpPermissions {
    remote_ports: PortRange,
    local_ports: PortRange,
}

pub struct UnencryptedDataPermission {}

pub enum DataSegment {
    MsgKey([u8; MAX_DATA_SEGMENT_SIZE], [u8; KEY_BYTES]),
    MsgPermission([u8; MAX_DATA_SEGMENT_SIZE], UnencryptedDataPermission),
}
*/
