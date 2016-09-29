pub enum Error {
}

pub type FlashResult = Result<(), Error>;

/// A block of readable persistent flash memory.
pub trait FlashReadable {
    type Word;

    /// Gets the size of the flash memory in words.
    fn len(&self) -> usize;

    /// Reads into a buffer from flash memory.
    fn read(&self, offset: usize, buf: &mut [Self::Word]) -> FlashResult;
}

/// A block of writable persistent flash memory.
pub trait FlashWritable: FlashReadable {
    /// Writes the contents of a buffer to flash memory.
    fn write(&self, offset: usize, buf: &[Self::Word]) -> FlashResult;

    /// Erases count words.
    fn erase(&self, offset: usize, count: usize) -> FlashResult;
}
