#[derive(Copy, Clone, Debug)]
pub enum Syscall {
    YIELD = 0,
    SUBSCRIBE = 1,
    COMMAND = 2,
    ALLOW = 3,
    MEMOP = 4,
}
