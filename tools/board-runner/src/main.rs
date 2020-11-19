use std::env;

pub mod artemis_nano;
pub mod earlgrey_nexysvideo;

fn main() {
    let args: Vec<String> = env::args().collect();

    println!("Tock board-runner starting...");

    for arg in args.iter() {
        if arg == "earlgrey_nexysvideo" {
            println!();
            println!("Running earlgrey_nexysvideo tests...");
            earlgrey_nexysvideo::all_earlgrey_nexysvideo_tests();
            println!("earlgrey_nexysvideo SUCCESS.");
        } else if arg == "artemis_nano" {
            println!();
            println!("Running Redboard tests...");
            artemis_nano::all_artemis_nano_tests();
            println!("artemis_nano SUCCESS.");
        }
    }
}
