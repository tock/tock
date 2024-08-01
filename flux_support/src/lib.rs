#[allow(dead_code)]
#[flux::sig(fn(x: bool[true]))]
pub fn assert(_x: bool) {}

#[flux::sig(fn(b:bool) ensures b)]
pub fn assume(b: bool) {
    if !b {
        panic!("assume fails")
    }
}

