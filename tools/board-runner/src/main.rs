// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

use std::env;

pub mod artemis_nano;
pub mod earlgrey_cw310;
pub mod esp32_c3;

fn main() {
    let args: Vec<String> = env::args().collect();

    println!("Tock board-runner starting...");

    for arg in args.iter() {
        if arg == "earlgrey_cw310" {
            println!();
            println!("Running earlgrey_cw310 tests...");
            earlgrey_cw310::all_earlgrey_cw310_tests();
            println!("earlgrey_cw310 SUCCESS.");
        } else if arg == "artemis_nano" {
            println!();
            println!("Running Redboard tests...");
            artemis_nano::all_artemis_nano_tests();
            println!("artemis_nano SUCCESS.");
        } else if arg == "esp32_c3" {
            println!();
            println!("Running ESP32-C3 tests...");
            esp32_c3::all_tests();
            println!("esp32_c3 SUCCESS.");
        }
    }
}
