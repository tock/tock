use std::env;

pub mod artemis_nano;
pub mod earlgrey_cw310;

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
        }
    }
}
