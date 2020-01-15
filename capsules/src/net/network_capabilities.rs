use core::cell::Cell;
use kernel::capabilities::{UdpVisCap, IpVisCap, NetCapCreateCap};
use crate::net::ipv6::ip_utils::IPAddr;

const MAX_ADDR_SET_SIZE: usize = 16;
const MAX_PORT_SET_SIZE: usize = 16;
const MAX_NUM_CAPAB: usize = 16;
const MAX_NUM_CAPSULES: usize = 32;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AddrRange { // TODO: change u32 to IPAddr type (inclusion weirdness?)
    Any, // Any address
    NoAddrs,
    AddrSet([IPAddr; MAX_ADDR_SET_SIZE]),
    Addr(IPAddr),
    // TODO: add range for IP addrs.
}

impl AddrRange {
    pub fn is_addr_valid(&self, addr: IPAddr) -> bool {
        match self {
            AddrRange::Any => true,
            AddrRange::NoAddrs => false,
            AddrRange::AddrSet(allowed_addrs) =>
                allowed_addrs.iter().any(|&a| a == addr),
            AddrRange::Addr(allowed_addr) => addr == *allowed_addr, //TODO: refs?
        }
    }
}


#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PortRange {
    Any,
    NoPorts,
    PortSet([u16; MAX_PORT_SET_SIZE]),
    Range(u16, u16),
    Port(u16),
}

impl PortRange {
    pub fn is_port_valid(&self, port: u16) -> bool {
        match self {
            PortRange::Any => true,
            PortRange::NoPorts => false,
            PortRange::PortSet(allowed_ports) =>
                allowed_ports.iter().any(|&p| p == port), // TODO: check refs
            PortRange::Range(low, high) => (*low <= port && port <= *high),
            PortRange::Port(allowed_port) => port == *allowed_port,
        }
    }
}


// Make the structs below implement an unsafe trait to make them only
// constructable in trusted code.

// TODO: remove copy eventually!!!!
#[derive(Clone, Copy, PartialEq)]
pub struct NetworkCapability {
    // can potentially add more
    remote_addrs: AddrRange,
    remote_ports: PortRange, // dst
    local_ports: PortRange, // src

}

impl NetworkCapability {
    pub fn new(remote_addrs: AddrRange, remote_ports: PortRange,
        local_ports: PortRange, create_net_cap: & dyn NetCapCreateCap)
        -> NetworkCapability {
            NetworkCapability {
                remote_addrs: remote_addrs,
                remote_ports: remote_ports,
                local_ports: local_ports,
            }
    }

    pub fn get_range(&self, ip_cap: & dyn IpVisCap) -> AddrRange {
        self.remote_addrs
    }

    pub fn remote_addr_valid(&self, remote_addr: IPAddr, ip_cap: & dyn IpVisCap)
        -> bool {
        self.remote_addrs.is_addr_valid(remote_addr)
    }

    pub fn get_remote_ports(&self, udp_cap: & dyn UdpVisCap) -> PortRange {
        self.remote_ports
    }

    pub fn get_local_ports(&self, udp_cap: & dyn UdpVisCap) -> PortRange {
        self.local_ports
    }

    pub fn remote_port_valid(&self, remote_port: u16, udp_cap: & dyn UdpVisCap) -> bool {
        self.remote_ports.is_port_valid(remote_port)
    }

    pub fn local_port_valid(&self, local_port: u16, udp_cap: & dyn UdpVisCap) -> bool {
        self.local_ports.is_port_valid(local_port)
    }
    
}