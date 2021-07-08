//! Test the CRC hardware.

use kernel::common::cells::TakeCell;
use kernel::common::leasable_buffer::LeasableBuffer;
use kernel::debug;
use kernel::ErrorCode;
use kernel::hil::crc::{Client, Crc, CrcAlgorithm, CrcOutput};

pub struct TestCrc<'a, C: 'a> {
    crc: &'a C,
    data: TakeCell<'static, [u8]>,
}

impl<'a, C: Crc<'a>> TestCrc<'a, C> {
    pub fn new(crc: &'a C, data: &'static mut [u8]) -> Self {
        TestCrc {
            crc: crc,
            data: TakeCell::new(data),
        }
    }

    pub fn run(&self) {
        let res = self.crc.set_algorithm(CrcAlgorithm::Crc32);
        if res.is_err() {
            debug!("CrcTest ERROR: failed to set algorithm to Crc32: {:?}", res);
            return;
        }
        let leasable: LeasableBuffer<'static, u8> = LeasableBuffer::new(self.data.take().unwrap());

        let res = self.crc.input(leasable);
        if let Err((error, _buffer)) = res {
            debug!("CrcTest ERROR: failed to start input processing: {:?}", error);
            return;
        }
    }
}

impl<'a, C: Crc<'a>>  Client for TestCrc<'a, C> {
    fn input_done(&self, result: Result<(), ErrorCode>, buffer: LeasableBuffer<'static, u8>) {
        if result.is_err() {
            debug!("CrcTest ERROR: failed to process input: {:?}", result);
            return;
        }

        if buffer.len() == 0 {
            self.data.replace(buffer.take());
            let res = self.crc.compute();
            if res.is_err() {
                debug!("CrcTest ERROR: failed to start CRC computation: {:?}", res);
                return;
            }
        } else {
            let res = self.crc.input(buffer);
            if let Err((error, _buffer)) = res {
                debug!("CrcTest ERROR: failed to start input processing: {:?}", error);
                return;
            }
        }
    }

    /// Called when the CRC computation is finished.
    fn crc_done(&self, result: Result<CrcOutput, ErrorCode>) {
        if let Err(code) = result {
            debug!("CrcTest ERROR: failed to compute CRC: {:?}", code);
            return;
        } else {
            if let Ok(output) = result {
                match output {
                    CrcOutput::Crc32(x) => {debug!("CRC32: {}", x);},
                    CrcOutput::Crc32C(x) => {debug!("CRC32C: {}", x);},
                    CrcOutput::Crc16CCITT(x) => {debug!("CRC17CCITT: {}", x);},
                }
            }
        }
    }
}
