pub mod alarm;
pub mod isl29035;
pub mod nonvolatile_storage;
pub mod si7021;
pub mod spi;

pub use self::alarm::AlarmDriverComponent;
pub use self::isl29035::Isl29035Component;
pub use self::nonvolatile_storage::NonvolatileStorageComponent;
pub use self::si7021::{HumidityComponent,SI7021Component,TemperatureComponent};
pub use self::spi::SpiComponent;
pub use self::spi::SpiSyscallComponent;
