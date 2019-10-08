/// fmt contains formatting routines for LowLevelDebug's console messages as
/// well as the buffer size necessary for the messages.
use super::DebugEntry;

// Messages that may be emitted:
//   1. LowLevelDebug: Dropped ## entries for app ##\n
//   2. LowLevelDebug: App ## status code ##\n
//   3. LowLevelDebug: App ## prints ##\n
//   4. LowLevelDebug: App ## prints ## ##\n
//
// Each ## above is a usize printed in hexadecimal, with a leading 0x.

// The longest message is either 1 or 4, depending on the size of a usize.
pub const BUF_LEN: usize = max(45 + 2 * USIZE_DIGITS, 35 + 3 * USIZE_DIGITS);

// Formats the given DebugEntry using the provided buffer. Returns the length of
// the message.
pub(crate) fn format_entry(app_num: usize, entry: DebugEntry, buffer: &mut [u8]) -> usize {
    use core::fmt::write;
    use DebugEntry::{Dropped, Print1, Print2, StatusCode};
    let mut adapter = WriteAdapter::new(buffer);
    let _ = match entry {
        Dropped(count) => write(
            &mut adapter,
            format_args!(
                "LowLevelDebug: Dropped 0x{:x} entries for app 0x{:x}\n",
                count, app_num
            ),
        ),
        StatusCode(code) => write(
            &mut adapter,
            format_args!(
                "LowLevelDebug: App 0x{:x} status code 0x{:x}\n",
                app_num, code
            ),
        ),
        Print1(num) => write(
            &mut adapter,
            format_args!("LowLevelDebug: App 0x{:x} prints 0x{:x}\n", app_num, num),
        ),
        Print2(num1, num2) => write(
            &mut adapter,
            format_args!(
                "LowLevelDebug: App 0x{:x} prints 0x{:x} 0x{:x}\n",
                app_num, num1, num2
            ),
        ),
    };
    adapter.finish()
}

// The length of a hex-formatted usize, excluding the leading 0x.
const USIZE_DIGITS: usize = 2 * core::mem::size_of::<usize>();

// const implementation of max
const fn max(a: usize, b: usize) -> usize {
    [a, b][(b > a) as usize]
}

// Adapter to allow core::fmt::write to write into a u8 slice.
struct WriteAdapter<'b> {
    buffer: &'b mut [u8],
    used: usize,
}

impl<'b> WriteAdapter<'b> {
    pub fn new(buffer: &'b mut [u8]) -> WriteAdapter<'b> {
        WriteAdapter { buffer, used: 0 }
    }

    pub fn finish(self) -> usize {
        self.used
    }
}

impl<'b> core::fmt::Write for WriteAdapter<'b> {
    fn write_str(&mut self, msg: &str) -> core::fmt::Result {
        if let Some(slice) = self.buffer.get_mut(self.used..(self.used + msg.len())) {
            slice.copy_from_slice(msg.as_bytes());
            self.used += msg.len();
        };
        Ok(())
    }
}
