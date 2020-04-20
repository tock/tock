//! Chip specific configuration.

// Chip configuration based on the target device.
pub struct Config<'a> {
    pub name: &'a str,
    pub chip_freq: u32,
    pub uart_baudrate: u32,
}

#[cfg(not(feature = "config_disable_default"))]
pub const CONFIG: Config = Config {
    name: &"default",
    chip_freq: 50_000_000,
    uart_baudrate: 230400,
};

#[cfg(feature = "config_fpga_nexysvideo")]
pub const CONFIG: Config = Config {
    name: &"fpga_nexysvideo",
    chip_freq: 50_000_000,
    uart_baudrate: 230400,
};

#[cfg(feature = "config_sim_verilator")]
pub const CONFIG: Config = Config {
    name: &"sim_verilator",
    chip_freq: 500_000,
    uart_baudrate: 9600,
};
