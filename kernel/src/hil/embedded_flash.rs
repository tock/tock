use crate::returncode::ReturnCode;

/// Embedded flash API.
///
/// The purpose of this HIL is to provide low-level control of the embedded flash to allow
/// applications to implement flash-efficient data-structures. The API is blocking since most flash
/// either halt the CPU during write and erase operations or ask the application to wait until the
/// operation is finished. A blocking API is also simpler to reason and less error-prone.
pub trait EmbeddedFlash {
    /// Returns the size of a word in bytes.
    fn word_size(&self) -> usize;

    /// Returns the size of a page in bytes.
    fn page_size(&self) -> usize;

    /// Returns how many times a word can be written between page erasures.
    fn max_word_writes(&self) -> usize;

    /// Returns how many times a page can be erased in the lifetime of the flash.
    fn max_page_erases(&self) -> usize;

    /// Writes a word-aligned slice at a word-aligned address.
    ///
    /// Words are written only if necessary, i.e. if writing the new value would change the current
    /// value. This can be used to simplify recovery operations (e.g. if power is lost during a
    /// write operation). The application doesn't need to check which prefix has already been
    /// written and may repeat the complete write that was interrupted.
    ///
    /// # Safety
    ///
    /// The slice starting at `ptr` of length `slice.len()` must be a valid flash range (this should
    /// be checked by the capsule). The words in this range must have been written less than
    /// `max_word_writes()` since the last erasure of their page.
    ///
    /// # Errors
    ///
    /// Fails with `EINVAL` if `ptr` or `slice.len()` are not word-aligned.
    fn write_slice(&self, ptr: usize, slice: &[u8]) -> ReturnCode;

    /// Erases a page at a page-aligned address.
    ///
    /// # Safety
    ///
    /// The slice starting at `ptr` of length `page_size()` must be a valid flash range (this should
    /// be checked by the capsule).
    ///
    /// # Errors
    ///
    /// Fails with `EINVAL` if `ptr` is not page-aligned.
    fn erase_page(&self, ptr: usize) -> ReturnCode;
}
