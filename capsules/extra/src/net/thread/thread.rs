pub struct ThreadState {
    pending_parent_req: bool,
}

impl ThreadState {
    pub fn new() -> ThreadState {
        ThreadState {
            pending_parent_req: false,
        }
    }
}
