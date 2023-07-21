// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

pub mod driver;
pub mod udp_port_table;
pub mod udp_recv;
pub mod udp_send;

pub use self::driver::UDPDriver;
pub use self::driver::DRIVER_NUM;

// Reexport the exports of the [`udp`] module, to avoid redundant
// module paths (e.g. `capsules::net::udp::udp::UDPHeader`)
mod udp;
pub use udp::UDPHeader;
