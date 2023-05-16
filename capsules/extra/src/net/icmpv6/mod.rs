// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

pub mod icmpv6_send;

// Reexport the exports of the [`icmpv6`] module, to avoid redundant
// module paths (e.g. `capsules::net::icmpv6::icmpv6::ICMP6Header`)
mod icmpv6;
pub use icmpv6::ICMP6Header;
pub use icmpv6::ICMP6HeaderOptions;
pub use icmpv6::ICMP6Type;
