#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Error {
    PageBoundary,
    WordBoundary,
}

/// A block of writable persistent flash memory.
pub trait Flash {
    /// Set the client for this flash peripheral. The client will be called
    /// when operations complete.
    fn set_client(&self, client: &'static Client);

    /// Read data
    fn read(&self, offset: usize, buf: &'static mut [u8]);

    /// Write data
    fn write(&self, offset: usize, buf: &'static mut [u8]);

    /// Erase flash
    fn erase(&self, offset: usize, len: usize);
}

/// Implement Client to receive callbacks from Flash
pub trait Client {
    /// Flash read complete
    fn read_complete(&self, read_buffer: &'static mut [u8], error: Error);

    /// Flash write complete
    fn write_complete(&self, write_buffer: &'static mut [u8], error: Error);

    /// Flash erase complete
    fn erase_complete(&self, error: Error);
}
