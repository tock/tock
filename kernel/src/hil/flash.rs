#[derive(Copy, Clone, PartialEq, Eq)]
pub enum Error {
}

/// A block of writable persistent flash memory.
pub trait Flash {
    /// Integer type the size of a word.
    type Word;

    /// Set the client for this flash peripheral. The client will be called
    /// when operations complete.
    fn set_client(&self, client: &'static Client);

    /// Read data
    fn read(&self, offset: usize, buf: &mut [Self::Word]);

    /// Write data
    fn write(&self, offset: usize, buf: &mut [Self::Word]);
}

/// Implement Client to receive callbacks from Flash
pub trait Client {
    /// Flash read complete
    fn read_complete(&self, read_buffer: &'static mut [u8], error: Error);

    /// Flash write complete
    fn write_complete(&self, write_buffer: &'static mut [u8], error: Error);
}
