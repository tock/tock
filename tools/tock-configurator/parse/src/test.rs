// Copyright OxidOS Automotive 2024.

use uuid::Uuid;

#[test]
fn gen_json() {
    println!("{:?}", Uuid::new_v4());
}
