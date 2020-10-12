use std::env;

pub mod artemis_nano;
pub mod opentitan;

fn main() {
    let args: Vec<String> = env::args().collect();

    println!("Tock board-runner starting...");

    for arg in args.iter() {
        if arg == "opentitan" {
            println!();
            println!("Running opentitan tests...");
            opentitan::all_opentitan_tests();
            println!("opentitan SUCCESS.");
        } else if arg == "artemis_nano" {
            println!();
            println!("Running Redboard tests...");
            artemis_nano::all_artemis_nano_tests();
            println!("artemis_nano SUCCESS.");
        }
    }
}
