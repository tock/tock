// Generic interface for CRC computation

use returncode::ReturnCode;

pub trait CRC {
    // Call this method exactly once before any other calls
    fn init(&mut self) -> ReturnCode;

    fn get_version(&self) -> u32;

    // Initiate a CRC calculation
    fn compute(&mut self, data: &[u8]) -> ReturnCode;
}

pub trait Client {
    // Receive the successful result of a CRC calculation
    fn receive_result(&self, u32);

    fn receive_err(&self);

    // For debugging only: We got some CRCCU interrupt
    fn interrupt(&self);
}
