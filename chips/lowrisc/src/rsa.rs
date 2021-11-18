//! RSA Implemented on top of the OTBN

use crate::virtual_otbn::VirtualMuxAccel;
use core::cell::Cell;
use kernel::hil::public_key_crypto::rsa_math::{Client, RsaCryptoBase};
use kernel::hil::public_key_crypto::rsa_math::{MutableClient, MutableRsaCryptoBase};
use kernel::utilities::cells::OptionalCell;
use kernel::utilities::cells::TakeCell;
use kernel::utilities::leasable_buffer::LeasableBuffer;
use kernel::utilities::mut_imut_buffer::MutImutBuffer;
use kernel::ErrorCode;

pub struct AppAddresses {
    pub imem_start: usize,
    pub imem_size: usize,
    pub dmem_start: usize,
    pub dmem_size: usize,
}

#[derive(Clone, Copy, PartialEq)]
enum Operation {
    None,
    LoadBinary,
    LoadData,
    SetMode,
    SetLength,
    SetModulus,
    SetMessage,
    SetEXponent,
    Run,
}

#[derive(Clone, Copy)]
enum DynamicClient<'a> {
    Immutable (&'a dyn Client<'a>),
    Mutable (&'a dyn MutableClient<'a>),
}

pub struct OtbnRsa<'a> {
    otbn: &'a VirtualMuxAccel<'a>,
    client: OptionalCell<DynamicClient<'a>>,

    internal: TakeCell<'static, [u8]>,

    message: TakeCell<'static, [u8]>,
    modulus: OptionalCell<MutImutBuffer<'static, u8>>,
    exponent: OptionalCell<MutImutBuffer<'static, u8>>,
    result: TakeCell<'static, [u8]>,

    rsa: AppAddresses,
    op: Cell<Operation>,
}

impl<'a> OtbnRsa<'a> {
    pub fn new(
        otbn: &'a VirtualMuxAccel<'a>,
        rsa: AppAddresses,
        internal_buffer: &'static mut [u8],
    ) -> Self {
        OtbnRsa {
            otbn,
            client: OptionalCell::empty(),
            internal: TakeCell::new(internal_buffer),
            message: TakeCell::empty(),
            modulus: OptionalCell::empty(),
            exponent: OptionalCell::empty(),
            result: TakeCell::empty(),
            rsa,
            op: Cell::new(Operation::None),
        }
    }

    fn report_error(&self, error: ErrorCode) {
        self.client.map(|dynamic| {
            match dynamic {
                DynamicClient::Mutable(client) => {
                    let modulus = self.modulus.take().unwrap();
                    let exponent = self.modulus.take().unwrap();
                    if let MutImutBuffer::Mutable(mbuf) = modulus {
                        if let MutImutBuffer::Mutable(ebuf) = exponent {
                            client.mod_exponent_done(
                                Err(error),
                                self.message.take().unwrap(),
                                mbuf,
                                ebuf,
                                self.result.take().unwrap(),
                            );
                        } else {
                            panic!("RSA ebuf changed mutablity, can't complete");
                        }
                    } else {
                        panic!("RSA mbuf changed mutablity, can't complete");
                    }
                },
                DynamicClient::Immutable(client) => {
                    let modulus = self.modulus.take().unwrap();
                    let exponent = self.modulus.take().unwrap();
                    if let MutImutBuffer::Immutable(mbuf) = modulus {
                        if let MutImutBuffer::Immutable(ebuf) = exponent {
                            client.mod_exponent_done(
                                Err(error),
                                self.message.take().unwrap(),
                                mbuf,
                                ebuf,
                                self.result.take().unwrap(),
                            );
                        } else {
                            panic!("RSA ebuf changed mutablity, can't complete");
                        }
                    } else {
                        panic!("RSA mbuf changed mutablity, can't complete");
                    }
                }
            }});
    }
}

impl<'a> crate::otbn::Client<'a> for OtbnRsa<'a> {
    fn binary_load_done(&'a self, result: Result<(), ErrorCode>, _input: &'static mut [u8]) {
        // Binary load is finished, now let's load the main data.

        if let Err(e) = result {
            self.op.set(Operation::None);

            self.report_error(e);
        } else {
            self.op.set(Operation::LoadData);
            // BAD! This is not actually mutable!!
            // This is stored in flash which is not mutable.
            // Once https://github.com/tock/tock/pull/2852 is merged this should be fixed
            let slice = unsafe {
                core::slice::from_raw_parts_mut(self.rsa.dmem_start as *mut u8, self.rsa.dmem_size)
            };
            if let Err(e) = self.otbn.load_data(0, LeasableBuffer::new(slice)) {
                self.report_error(e.0);
            }
        }
    }

    fn data_load_done(&'a self, result: Result<(), ErrorCode>, data: &'static mut [u8]) {
        if let Err(e) = result {
            self.report_error(e);
            return;
        }

        match self.op.get() {
            Operation::None | Operation::LoadBinary | Operation::Run => {
                unreachable!()
            }
            Operation::LoadData => {
                self.op.set(Operation::SetMode);

                // Set the mode to decryption
                if let Some(buf) = self.internal.take() {
                    buf[0] = 2;
                    buf[1] = 0;
                    buf[2] = 0;
                    buf[3] = 0;
                    let mut lease_buf = LeasableBuffer::new(buf);
                    lease_buf.slice(0..4);

                    if let Err(e) = self.otbn.load_data(0, lease_buf) {
                        self.internal.replace(e.1);

                        self.report_error(e.0);
                    }
                } else {
                    self.report_error(ErrorCode::NOMEM);
                }
            }
            Operation::SetMode => {
                self.op.set(Operation::SetLength);

                if let Some(modulus) = self.modulus.take() {
                    let length = modulus.len();
                    self.modulus.replace(modulus);

                    data[0] = (length / 32) as u8;
                    data[1] = 0;
                    data[2] = 0;
                    data[3] = 0;
                    let mut lease_buf = LeasableBuffer::new(data);
                    lease_buf.slice(0..4);

                    if let Err(e) = self.otbn.load_data(4, lease_buf) {
                        self.internal.replace(e.1);
                        self.report_error(e.0);
                    }
                } else {
                    self.report_error(ErrorCode::NOMEM);
                }
            }
            Operation::SetLength => {
                self.op.set(Operation::SetModulus);

                if let Some(modulus) = self.modulus.take() {
                    match modulus {
                        MutImutBuffer::Mutable(ref buf) => {
                            data.copy_from_slice(buf);
                        }
                        MutImutBuffer::Immutable(buf) => {
                            data.copy_from_slice(buf);
                        }
                    }
                    self.modulus.replace(modulus);

                    // We were passed BE data and the OTBN expects LE
                    // so reverse the order.
                    data.reverse();

                    if let Err(e) = self.otbn.load_data(0x420, LeasableBuffer::new(data)) {
                        self.internal.replace(e.1);
                        self.report_error(e.0);
                    }
                } else {
                    self.report_error(ErrorCode::NOMEM);
                }
            }
            Operation::SetModulus => {
                self.op.set(Operation::SetEXponent);

                if let Some(exponent) = self.exponent.take() {
                    let len = exponent.len();
                    
                    match exponent {
                        MutImutBuffer::Mutable(ref buf) => {
                            data[0..len].copy_from_slice(buf);
                        }
                        MutImutBuffer::Immutable(buf) => {
                            data[0..len].copy_from_slice(buf);
                        }
                    }
                    self.exponent.replace(exponent);

                    // We were passed BE data and the OTBN expects LE
                    // so reverse the order.
                    data.reverse();

                    let mut lease_buf = LeasableBuffer::new(data);
                    lease_buf.slice(0..len);

                    if let Err(e) = self.otbn.load_data(0x620, lease_buf) {
                        self.internal.replace(e.1);
                        self.report_error(e.0);
                    }
                } else {
                    self.report_error(ErrorCode::NOMEM);
                }
            }
            Operation::SetEXponent => {
                self.op.set(Operation::SetMessage);

                self.internal.replace(data);

                if let Some(message) = self.message.take() {
                    if let Err(e) = self.otbn.load_data(0x820, LeasableBuffer::new(message)) {
                        self.message.replace(e.1);
                        self.report_error(e.0);
                    }
                } else {
                    self.report_error(ErrorCode::NOMEM);
                }
            }
            Operation::SetMessage => {
                self.op.set(Operation::Run);
                self.message.replace(data);

                if let Some(result) = self.result.take() {
                    if let Err(e) = self.otbn.run(0x288, result) {
                        self.result.replace(e.1);
                        self.report_error(e.0);
                    }
                } else {
                    self.report_error(ErrorCode::NOMEM);
                }
            }
        }
    }

    fn op_done(&'a self, result: Result<(), ErrorCode>, output: &'static mut [u8]) {
        self.op.set(Operation::None);

        if let Err(e) = result {
            self.report_error(e);
            return;
        }

        // We want to return BE data
        output.reverse();

        self.client.map(|client| {
            match client {
                DynamicClient::Mutable(m) => {
                    let modulus = self.modulus.take().unwrap();
                    let exponent = self.modulus.take().unwrap();
                    if let MutImutBuffer::Mutable(mbuf) = modulus {
                        if let MutImutBuffer::Mutable(ebuf) = exponent {
                            m.mod_exponent_done(
                                Ok(true),
                                self.message.take().unwrap(),
                                mbuf,
                                ebuf,
                                output,
                            );
                        } else {
                            panic!("RSA ebuf changed mutablity, can't complete");
                        }
                    } else {
                        panic!("RSA mbuf changed mutablity, can't complete");
                    }
                },
                DynamicClient::Immutable(i) => {
                    let modulus = self.modulus.take().unwrap();
                    let exponent = self.modulus.take().unwrap();
                    if let MutImutBuffer::Immutable(mbuf) = modulus {
                        if let MutImutBuffer::Immutable(ebuf) = exponent {
                            i.mod_exponent_done(
                                Ok(true),
                                self.message.take().unwrap(),
                                mbuf,
                                ebuf,
                                self.result.take().unwrap(),
                            );
                        } else {
                            panic!("RSA ebuf changed mutablity, can't complete");
                        }
                    } else {
                        panic!("RSA mbuf changed mutablity, can't complete");
                    }
                }
            }});
    }
}

impl<'a> RsaCryptoBase<'a> for OtbnRsa<'a> {
    fn set_client(&'a self, client: &'a dyn Client<'a>) {
        self.client.set(DynamicClient::Immutable(client));
    }

    fn clear_data(&self) {
        self.otbn.clear_data();
    }

    fn mod_exponent(
        &self,
        message: &'static mut [u8],
        modulus: &'static [u8],
        exponent: &'static [u8],
        result: &'static mut [u8],
    ) -> Result<
            (),
        (
            ErrorCode,
            &'static mut [u8],
            &'static [u8],
            &'static [u8],
            &'static mut [u8],
        ),
        > {
        let correct_client = self.client.map_or(false, |c| {
            match c {
                DynamicClient::Mutable(_c) => false,
                DynamicClient::Immutable(_c) => true,
            }
        });
        
        if !correct_client {
            return Err((ErrorCode::INVAL, message, modulus, exponent, result));
        }
        
        if self.op.get() != Operation::None {
            return Err((ErrorCode::BUSY, message, modulus, exponent, result));
        }
        
        self.op.set(Operation::LoadBinary);

        self.message.replace(message);
        self.modulus.replace(MutImutBuffer::Immutable(modulus));
        self.exponent.replace(MutImutBuffer::Immutable(exponent));
        self.result.replace(result);

        // BAD! This is not actually mutable!!
        // This is stored in flash which is not mutable.
        // Once https://github.com/tock/tock/pull/2852 is merged this should be fixed
        let slice = unsafe {
            core::slice::from_raw_parts_mut(self.rsa.imem_start as *mut u8, self.rsa.imem_size)
        };
        let buf = LeasableBuffer::new(slice);

        if let Err(e) = self.otbn.load_binary(buf) {
            let modulus = self.modulus.take().unwrap();
            let exponent = self.modulus.take().unwrap();
            if let MutImutBuffer::Immutable(mbuf) = modulus {
                if let MutImutBuffer::Immutable(ebuf) = exponent {
                    return Err((
                        e.0,
                        self.message.take().unwrap(),
                        mbuf,
                        ebuf,
                        self.result.take().unwrap(),
                    ));
                }
            }
            panic!("RSA buffers disappeared, can't return an error");
        }

        Ok(())
    }
}

impl<'a> MutableRsaCryptoBase<'a> for OtbnRsa<'a> {
    fn set_client(&'a self, client: &'a dyn MutableClient<'a>) {
        self.client.set(DynamicClient::Mutable(client));
    }

    fn clear_data(&self) {
        self.otbn.clear_data();
    }

    fn mod_exponent(
        &self,
        message: &'static mut [u8],
        modulus: &'static mut [u8],
        exponent: &'static mut [u8],
        result: &'static mut [u8],
    ) -> Result<
            (),
        (
            ErrorCode,
            &'static mut [u8],
            &'static mut [u8],
            &'static mut [u8],
            &'static mut [u8],
        ),
    > {
        self.op.set(Operation::LoadBinary);

        self.message.replace(message);
        self.modulus.replace(MutImutBuffer::Mutable(modulus));
        self.exponent.replace(MutImutBuffer::Mutable(exponent));
        self.result.replace(result);

        // BAD! This is not actually mutable!!
        // This is stored in flash which is not mutable.
        // Once https://github.com/tock/tock/pull/2852 is merged this should be fixed
        let slice = unsafe {
            core::slice::from_raw_parts_mut(self.rsa.imem_start as *mut u8, self.rsa.imem_size)
        };
        let buf = LeasableBuffer::new(slice);

        if let Err(e) = self.otbn.load_binary(buf) {
            let modulus = self.modulus.take().unwrap();
            let exponent = self.modulus.take().unwrap();
            if let MutImutBuffer::Mutable(mbuf) = modulus {
                if let MutImutBuffer::Mutable(ebuf) = exponent {
                    return Err((
                        e.0,
                        self.message.take().unwrap(),
                        mbuf,
                        ebuf,
                        self.result.take().unwrap(),
                    ));
                }
            }
            panic!("RSA buffers disappeared, can't return an error");
        }

        Ok(())
    }
}
