pub mod icmpv6_send;

// Reexport the exports of the [`icmpv6`] module, to avoid redundant
// module paths (e.g. `capsules::net::icmpv6::icmpv6::ICMP6Header`)
mod icmpv6;
pub use icmpv6::ICMP6Header;
pub use icmpv6::ICMP6HeaderOptions;
pub use icmpv6::ICMP6Type;
