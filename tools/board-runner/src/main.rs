pub mod opentitan;

fn main() {
    println!("Tock board-runner starting...");
    println!();
    println!("Running opentitan tests...");
    opentitan::all_opentitan_tests();
    println!("opentitan SUCCESS.");
}

// p.exp_eof()?;
