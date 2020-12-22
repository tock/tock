pub mod ip_utils;
pub mod ipv6_recv;
pub mod ipv6_send;

// Reexport the exports of the [`ipv6`] module, to avoid redundant
// module paths (e.g. `capsules::net::ipv6::ipv6::IP6Header`)
mod ipv6;
pub use ipv6::IP6Header;
pub use ipv6::IP6Packet;
pub use ipv6::IPPayload;
pub use ipv6::TransportHeader;
pub use ipv6::ICMP_HDR_LEN;
pub use ipv6::UDP_HDR_LEN;
