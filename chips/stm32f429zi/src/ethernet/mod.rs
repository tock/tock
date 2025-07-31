// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright 2023 OxidOS Automotive SRL
//
// Author: Ioan-Cristian CÎRSTEA <ioan.cirstea@oxidos.io>

pub mod ethernet;
mod receive_descriptor;
mod transmit_descriptor;
pub mod utils;

pub use ethernet::{Ethernet, RX_PACKET_LENGTH};
