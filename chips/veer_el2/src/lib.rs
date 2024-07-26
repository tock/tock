// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.
// Copyright (c) 2024 Antmicro <www.antmicro.com>

#![no_std]
#![crate_name = "veer_el2"]
#![crate_type = "rlib"]

pub mod chip;
pub mod io;
pub mod machine_timer;
pub mod pic;
pub mod uart;
