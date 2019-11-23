pub(crate) struct Config {
    // Whether the kernel should trace syscalls to the debug output.
    pub(crate) strace: bool,
}

pub(crate) const CONFIG: Config = Config { strace: false };
