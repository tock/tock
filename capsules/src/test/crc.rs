//! Test the CRC hardware.

use kernel::debug;
use kernel::hil::crc::{Client, Crc, CrcAlgorithm, CrcOutput};
use kernel::utilities::cells::TakeCell;
use kernel::utilities::leasable_buffer::LeasableBuffer;
use kernel::ErrorCode;

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

    pub fn run_test(&self, algorithm: CrcAlgorithm) {
        let res = self.crc.set_algorithm(algorithm);
        if res.is_err() {
            debug!("CrcTest ERROR: failed to set algorithm to Crc32: {:?}", res);
            return;
        }
        let leasable: LeasableBuffer<'static, u8> = LeasableBuffer::new(self.data.take().unwrap());

        let res = self.crc.input(leasable);
        if let Err((error, _buffer)) = res {
            debug!(
                "CrcTest ERROR: failed to start input processing: {:?}",
                error
            );
        }
    }

    pub fn run(&self) {
        self.run_test(CrcAlgorithm::Crc32);
    }
}

impl<'a, C: Crc<'a>> Client for TestCrc<'a, C> {
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
            }
        } else {
            let res = self.crc.input(buffer);
            if let Err((error, _buffer)) = res {
                debug!(
                    "CrcTest ERROR: failed to start input processing: {:?}",
                    error
                );
            }
        }
    }

    /// Called when the CRC computation is finished.
    fn crc_done(&self, result: Result<CrcOutput, ErrorCode>) {
        if let Err(code) = result {
            debug!("CrcTest ERROR: failed to compute CRC: {:?}", code);
        } else {
            if let Ok(output) = result {
                match output {
                    CrcOutput::Crc32(x) => {
                        debug!("CRC32: {:#x}", x);
                        self.run_test(CrcAlgorithm::Crc32C);
                    }
                    CrcOutput::Crc32C(x) => {
                        debug!("CRC32C: {:#x}", x);
                        self.run_test(CrcAlgorithm::Crc16CCITT);
                    }
                    CrcOutput::Crc16CCITT(x) => {
                        debug!("CRC16CCITT: {:#x}", x);
                    }
                }
            }
        }
    }
}
