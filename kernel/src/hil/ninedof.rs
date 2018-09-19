//! Interface for chips that provide 9DOF functionality.
//!
//! This trait file provides a standard interface for chips that implement
//! some or all of a nine degrees of freedom (accelerometer, magnetometer,
//! gyroscope) sensor. Any interface functions that a chip cannot implement
//! can be ignored by the chip capsule and an error will automatically be
//! returned.

use returncode::ReturnCode;

/// A basic interface for a 9-DOF compatible chip.
/// Not all functions must be implemented if not all features are supported
/// (for instance some chips may not include a gyroscope).
pub trait NineDof<'a> {
    /// Set the client to be notified when the capsule has data ready or
    /// has finished some command. This is likely called in a board's main.rs
    /// and is set to the virtual_ninedof.rs driver.
    fn set_client(&self, client: &'a NineDofClient);

    /// Get a single instantaneous reading of the acceleration in the
    /// X,Y,Z directions.
    fn read_accelerometer(&self) -> ReturnCode {
        ReturnCode::ENODEVICE
    }

    /// Get a single instantaneous reading from the magnetometer in all
    /// three directions.
    fn read_magnetometer(&self) -> ReturnCode {
        ReturnCode::ENODEVICE
    }

    /// Get a single instantaneous reading from the gyroscope of the rotation
    /// around all three axes.
    fn read_gyroscope(&self) -> ReturnCode {
        ReturnCode::ENODEVICE
    }
}

/// Client for receiving done events from the chip.
pub trait NineDofClient {
    /// Signals a command has finished. The arguments will most likely be passed
    /// over the syscall interface to an application.
    ///
    /// The arguments to the callback specify the sensor reading along the x, y
    /// and z axis. The values are expressed in mg (thounsandths of standard
    /// gravity) for acceleration and uT (micro-Teslas) for megnetic field.
    fn callback(&self, x: usize, y: usize, z: usize);
}
