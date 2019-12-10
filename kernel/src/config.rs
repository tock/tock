pub(crate) struct Config {
    // Whether the kernel should trace syscalls to the debug output.
    pub(crate) trace_syscalls: bool,
}

pub(crate) const CONFIG: Config = Config {
    trace_syscalls: false,
};
