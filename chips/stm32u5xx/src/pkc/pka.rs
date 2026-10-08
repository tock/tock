// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OxidOS Automotive 2026.

use core::cell::Cell;

use kernel::deferred_call::{DeferredCall, DeferredCallClient};
use kernel::hil::crypto::ecc::ecc_constants::{Curve, NistP256Constants, P_256_P_SIZE};
use kernel::hil::crypto::ecc::ecc_math::{EccClient, EccCrypto, VerifyEccPoint};
use kernel::hil::crypto::modular_arithmetic::{MathClient, ModularArithmetic};
use kernel::hil::public_key_crypto::rsa_math::{Client, RsaCryptoBase};
use kernel::utilities::StaticRef;
use kernel::utilities::cells::{OptionalCell, TakeCell};
use kernel::utilities::registers::FieldValue;
use kernel::utilities::registers::interfaces::{ReadWriteable, Readable, Writeable};
use kernel::{ErrorCode, debug};

use crate::pkc::constants::{
    ARITH_OP1_IDX, ARITH_OP2_IDX, ARITH_RESULT_IDX, CLRFR, CR, ECC_A_ABS_IDX, ECC_A_SIGN_IDX,
    ECC_ADD_OUT_X_IDX, ECC_ADD_OUT_Y_IDX, ECC_ADD_P_IDX, ECC_ADD_PT1_X_IDX, ECC_ADD_PT1_Y_IDX,
    ECC_ADD_PT1_Z_IDX, ECC_ADD_PT2_X_IDX, ECC_ADD_PT2_Y_IDX, ECC_ADD_PT2_Z_IDX, ECC_B_IDX,
    ECC_MUL_IN_X_IDX, ECC_MUL_IN_Y_IDX, ECC_MUL_K_IDX, ECC_N_IDX, ECC_N_LEN_BITS_IDX,
    ECC_OUT_X_IDX, ECC_OUT_Y_IDX, ECC_P_IDX, ECC_P_LEN_BITS_IDX, ECC_P_R2_IDX, ECC_RESULT_OK,
    EXP_LEN_BITS_IDX, FPCHECK_RESULT_IDX, FPCHECK_X_IDX, FPCHECK_Y_IDX, INV_RED_MODULUS_IDX,
    MODEXP_BASE_IDX, MODEXP_EXPONENT_IDX, MODEXP_RESULT_IDX, MODULUS_IDX, MONT_R2_OUT_IDX,
    OPERAND_LEN_BITS_IDX, P256_R2_MOD_P, PkaRegisters, SR, SupportedOp,
};

/// Size of the chunks exchanged with the math client (bytes).
const MATH_CHUNK: usize = 64;

#[derive(Copy, Clone, Debug, PartialEq)]
enum State {
    Idle,
    Rsa,
    ScalarMul,
    PointAddition,
    ProjToAffinePass1,
    ProjToAffinePass2,
    ProjToAffinePass3,
    VerifyPoint,
    MathAddition,
    MathInvert,
    MathComputeR2,
    MathComputeAR,
    MathComputeAB,
    MathModulus,
}

/// Which number to request from the math client.
#[derive(Copy, Clone)]
enum Operand {
    Modulus,
    First,
    Second,
}

pub struct Pka<'a> {
    registers: StaticRef<PkaRegisters>,
    deferred_call: DeferredCall,

    rsa_client: OptionalCell<&'a dyn Client<'a>>,
    ecc_client: OptionalCell<&'a dyn EccClient>,
    math_client: OptionalCell<&'a dyn MathClient<SupportedOp>>,

    modulus: OptionalCell<&'static [u8]>,
    exponent: OptionalCell<&'static [u8]>,

    message: TakeCell<'static, [u8]>,
    result: TakeCell<'static, [u8]>,

    math_len: Cell<usize>,

    state: Cell<State>,
}

impl<'a> Pka<'a> {
    pub fn new(registers: StaticRef<PkaRegisters>) -> Pka<'a> {
        Pka {
            registers,
            deferred_call: DeferredCall::new(),

            rsa_client: OptionalCell::empty(),
            ecc_client: OptionalCell::empty(),
            math_client: OptionalCell::empty(),

            modulus: OptionalCell::empty(),
            exponent: OptionalCell::empty(),

            message: TakeCell::empty(),
            result: TakeCell::empty(),

            math_len: Cell::new(0),

            state: Cell::new(State::Idle),
        }
    }

    fn write_slice(&self, idx: usize, data: &[u8]) {
        for (i, chunk) in data.rchunks(4).enumerate() {
            let Some(cell) = self.registers.ram.get(idx + i) else {
                break;
            };
            let mut word = [0u8; 4];
            word[4 - chunk.len()..].copy_from_slice(chunk);
            cell.set(u32::from_be_bytes(word));
        }
    }

    fn read_slice(&self, idx: usize, buffer: &mut [u8]) {
        for (i, chunk) in buffer.rchunks_mut(4).enumerate() {
            let Some(cell) = self.registers.ram.get(idx + i) else {
                break;
            };
            let bytes = cell.get().to_be_bytes();
            chunk.copy_from_slice(&bytes[4 - chunk.len()..]);
        }
    }

    fn copy_ram(&self, from: usize, to: usize, len: usize) {
        for i in 0..len.div_ceil(4) {
            let word = self.registers.ram[from + i].get();
            self.registers.ram[to + i].set(word);
        }
    }

    fn set_len(&self, idx: usize, value: u32) {
        self.registers.ram[idx].set(value);
        self.registers.ram[idx + 1].set(0);
    }

    fn clear_ram(&self) {
        for cell in self.registers.ram.iter() {
            cell.set(0);
        }
    }

    fn wait_init_ok(&self) {
        while !self.registers.sr.is_set(SR::INITOK) {}
    }

    fn enable_peripheral(&self) -> Result<(), ErrorCode> {
        if self.registers.sr.is_set(SR::BUSY) {
            return Err(ErrorCode::BUSY);
        }
        self.registers.cr.modify(CR::EN::SET);
        self.wait_init_ok();
        Ok(())
    }

    fn start_operation(&self, mode: FieldValue<u32, CR::Register>) {
        self.registers.cr.modify(
            mode + CR::PROCENDIE::SET
                + CR::ADDERRIE::SET
                + CR::RAMERRIE::SET
                + CR::OPERRIE::SET
                + CR::EN::SET,
        );
        self.registers.cr.modify(CR::START::SET);
    }

    fn load_p256_parameters(&self) {
        let bits = (P_256_P_SIZE as u32) << 3;
        self.set_len(ECC_N_LEN_BITS_IDX, bits);
        self.set_len(ECC_P_LEN_BITS_IDX, bits);
        self.set_len(ECC_A_SIGN_IDX, 0);

        self.write_slice(ECC_A_ABS_IDX, &NistP256Constants::EQ_PARAMS.0);
        self.write_slice(ECC_B_IDX, &NistP256Constants::EQ_PARAMS.1);
        self.write_slice(ECC_N_IDX, &NistP256Constants::N);
        self.write_slice(ECC_P_R2_IDX, &P256_R2_MOD_P);
        self.write_slice(ECC_P_IDX, &NistP256Constants::P);
        self.write_slice(ECC_ADD_P_IDX, &NistP256Constants::P);
    }

    fn load_point(&self, x_idx: usize, y_idx: usize) {
        let mut point = [0u8; 2 * P_256_P_SIZE];
        self.ecc_client.map(|client| client.read_point(&mut point));
        self.write_slice(x_idx, &point[..P_256_P_SIZE]);
        self.write_slice(y_idx, &point[P_256_P_SIZE..]);
    }

    fn start_projective_to_affine(&self) {
        self.start_operation(CR::MODE::ECCProjectiveToAffine);
    }

    fn feed_affine_to_projective(&self) -> ([u8; P_256_P_SIZE], [u8; P_256_P_SIZE]) {
        let mut x = [0u8; P_256_P_SIZE];
        let mut y = [0u8; P_256_P_SIZE];

        self.read_slice(ECC_OUT_X_IDX, &mut x);
        self.read_slice(ECC_OUT_Y_IDX, &mut y);
        self.write_slice(ECC_ADD_OUT_X_IDX, &x);
        self.write_slice(ECC_ADD_OUT_Y_IDX, &y);

        (x, y)
    }

    fn ecc_done(&self, result: Result<(), ErrorCode>) {
        self.state.set(State::Idle);
        self.ecc_client.map(|client| client.operation_done(result));
    }

    fn begin_math(&self, state: State, len: usize) {
        self.state.set(state);
        self.math_len.set(len);
        self.set_len(OPERAND_LEN_BITS_IDX, (len as u32) * 8);
    }

    fn read_from_client(&self, which: Operand, chunk: &mut [u8]) {
        self.math_client.map(|client| match which {
            Operand::Modulus => {
                if client.read_modulus(chunk).is_err() {
                    self.math_fail();
                }
            }
            Operand::First => {
                client.read_number(chunk);
            }
            Operand::Second => {
                client.read_second_number(chunk);
            }
        });
    }

    fn load_operand(&self, idx: usize, len: usize, which: Operand) {
        let mut buf = [0u8; MATH_CHUNK];
        let mut offset = 0;
        while offset < len {
            let n = MATH_CHUNK.min(len - offset);
            let chunk = &mut buf[..n];
            chunk.fill(0);
            self.read_from_client(which, chunk);
            let word = (len - offset - n) / 4;
            self.write_slice(idx + word, chunk);
            offset += n;
        }
    }

    fn math_fail(&self) {
        self.state.set(State::Idle);
        self.math_client
            .map(|client| client.computation_completed(Err(ErrorCode::FAIL)));
    }

    fn math_finish(&self, success: bool) {
        if !success {
            self.math_fail();
            return;
        }
        self.state.set(State::Idle);

        let len = self.math_len.get();
        let mut buf = [0u8; MATH_CHUNK];
        let mut offset = 0;
        while offset < len {
            let n = MATH_CHUNK.min(len - offset);
            let word = (len - offset - n) / 4;
            self.read_slice(ARITH_RESULT_IDX + word, &mut buf[..n]);
            if self
                .math_client
                .map_or(Ok(()), |client| client.write_number(&buf[..n]))
                .is_err()
            {
                self.math_fail();
            }

            offset += n;
        }
        self.math_client
            .map(|client| client.computation_completed(Ok(())));
    }

    pub fn handle_interrupt(&self) {
        let sr = self.registers.sr.extract();
        if sr.is_set(SR::OPERRF) {
            self.registers.clrfr.write(CLRFR::OPERRFC::SET);
        }
        if sr.is_set(SR::ADDRERRF) {
            self.registers.clrfr.write(CLRFR::ADDERRFC::SET);
        }
        if sr.is_set(SR::RAMERRF) {
            self.registers.clrfr.write(CLRFR::RAMERRFC::SET);
        }
        let success = sr.is_set(SR::PROCENDF);
        if success {
            self.registers.clrfr.write(CLRFR::PROCENDFC::SET);
        }

        match self.state.get() {
            State::Idle => {}

            State::Rsa => {
                let modulus = self.modulus.take().unwrap();
                let exponent = self.exponent.take().unwrap();
                let message = self.message.take().unwrap();
                let result = self.result.take().unwrap();
                self.state.set(State::Idle);

                let outcome = if success {
                    self.read_slice(MODEXP_RESULT_IDX, result);
                    Ok(true)
                } else {
                    Err(ErrorCode::FAIL)
                };
                self.rsa_client.map(|client| {
                    client.mod_exponent_done(outcome, message, modulus, exponent, result)
                });
            }

            State::ScalarMul => {
                if !success {
                    self.ecc_done(Err(ErrorCode::FAIL));
                    return;
                }
                let mut res = [0u8; 2 * P_256_P_SIZE];
                let (x, y) = res.split_at_mut(P_256_P_SIZE);
                self.read_slice(ECC_OUT_X_IDX, x);
                self.read_slice(ECC_OUT_Y_IDX, y);
                self.state.set(State::Idle);
                self.ecc_client.map(|client| {
                    client.operation_done(client.write_point(&res));
                });
            }

            State::PointAddition => {
                if success {
                    self.state.set(State::ProjToAffinePass1);
                    self.start_projective_to_affine();
                } else {
                    self.ecc_done(Err(ErrorCode::FAIL));
                }
            }

            State::ProjToAffinePass1 => {
                self.state.set(State::ProjToAffinePass2);
                self.feed_affine_to_projective();
                self.start_projective_to_affine();
            }

            State::ProjToAffinePass2 => {
                self.state.set(State::ProjToAffinePass3);
                let (x_out, _) = self.feed_affine_to_projective();
                self.ecc_client.map(|client| {
                    if client.write_point(&x_out).is_err() {
                        self.ecc_client.map(|client| {
                            client.operation_done(Err(ErrorCode::FAIL));
                        });
                    }
                });
                self.start_projective_to_affine();
            }

            State::ProjToAffinePass3 => {
                self.state.set(State::Idle);
                let mut y_out = [0u8; P_256_P_SIZE];
                self.read_slice(ECC_OUT_Y_IDX, &mut y_out);
                self.ecc_client.map(|client| {
                    client.operation_done(client.write_point(&y_out));
                });
            }

            State::VerifyPoint => {
                let code = self.registers.ram[FPCHECK_RESULT_IDX].get();
                debug!("GOT CODE: {:08x?}", code);
                let result = if code == ECC_RESULT_OK {
                    Ok(())
                } else {
                    Err(ErrorCode::INVAL)
                };
                self.ecc_done(result);
            }

            State::MathAddition | State::MathModulus | State::MathInvert | State::MathComputeAB => {
                self.math_finish(success);
            }

            State::MathComputeR2 => {
                if !success {
                    self.math_fail();
                    return;
                }
                let len = self.math_len.get();
                // R^2 mod n becomes the first operand; the client's A the second.
                self.copy_ram(MONT_R2_OUT_IDX, ARITH_OP1_IDX, len);
                self.load_operand(ARITH_OP2_IDX, len, Operand::First);

                self.state.set(State::MathComputeAR);
                self.start_operation(CR::MODE::MontgomeryMultiplication);
            }

            State::MathComputeAR => {
                if !success {
                    self.math_fail();
                    return;
                }
                let len = self.math_len.get();
                // A*R becomes the second operand; the client's B the first.
                self.copy_ram(ARITH_RESULT_IDX, ARITH_OP2_IDX, len);
                self.load_operand(ARITH_OP1_IDX, len, Operand::Second);

                self.state.set(State::MathComputeAB);
                self.start_operation(CR::MODE::MontgomeryMultiplication);
            }
        }
    }
}

fn get_bitlen(data: &[u8]) -> u32 {
    data.iter().position(|&b| b != 0).map_or(0, |i| {
        (8 - data[i].leading_zeros()) + ((data.len() - 1 - i) as u32) * 8
    })
}

impl<'a> RsaCryptoBase<'a> for Pka<'a> {
    fn set_client(&'a self, client: &'a dyn Client<'a>) {
        self.rsa_client.set(client);
    }

    fn clear_data(&self) {
        self.clear_ram();
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
        if self.registers.sr.is_set(SR::BUSY) || self.state.get() != State::Idle {
            return Err((ErrorCode::BUSY, message, modulus, exponent, result));
        }

        if result.len() < modulus.len() || exponent.is_empty() || message.is_empty() {
            return Err((ErrorCode::SIZE, message, modulus, exponent, result));
        }

        let exp_bits = get_bitlen(exponent);
        let op_bits = get_bitlen(modulus);

        if exp_bits == 0 || op_bits == 0 {
            return Err((ErrorCode::INVAL, message, modulus, exponent, result));
        }

        self.registers.cr.modify(CR::EN::SET);
        self.wait_init_ok();

        self.state.set(State::Rsa);
        self.clear_ram();

        self.set_len(EXP_LEN_BITS_IDX, exp_bits);
        self.set_len(OPERAND_LEN_BITS_IDX, op_bits);

        self.write_slice(MODEXP_EXPONENT_IDX, exponent);
        self.write_slice(MODULUS_IDX, modulus);
        self.write_slice(MODEXP_BASE_IDX, message);

        self.message.replace(message);
        self.modulus.set(modulus);
        self.exponent.set(exponent);
        self.result.replace(result);

        self.start_operation(CR::MODE::MontgomeryModularExp);
        Ok(())
    }
}

impl<'a> EccCrypto<'a, P_256_P_SIZE, NistP256Constants> for Pka<'a> {
    fn set_client(&self, client: &'a dyn EccClient) {
        self.ecc_client.replace(client);
    }

    fn point_addition(&self) -> Result<(), ErrorCode> {
        self.enable_peripheral()?;
        self.state.set(State::PointAddition);
        self.load_p256_parameters();
        self.deferred_call.set();
        Ok(())
    }

    fn scalar_multiplication(&self) -> Result<(), ErrorCode> {
        self.enable_peripheral()?;
        self.state.set(State::ScalarMul);
        self.load_p256_parameters();
        self.deferred_call.set();
        Ok(())
    }
}

impl<'a> VerifyEccPoint<'a, P_256_P_SIZE, NistP256Constants> for Pka<'a> {
    fn verify_point(&self) -> Result<(), ErrorCode> {
        self.enable_peripheral()?;
        self.state.set(State::VerifyPoint);
        self.load_p256_parameters();
        self.load_point(FPCHECK_X_IDX, FPCHECK_Y_IDX);

        self.start_operation(CR::MODE::FpCheck);
        Ok(())
    }
}

impl<'a> ModularArithmetic<'a, SupportedOp> for Pka<'a> {
    fn set_client(&self, client: &'a dyn MathClient<SupportedOp>) {
        self.math_client.replace(client);
    }

    fn start_computation(
        &self,
        modulus_len: usize,
        operation: SupportedOp,
    ) -> Result<(), ErrorCode> {
        if modulus_len == 0 || !modulus_len.is_multiple_of(4) {
            return Err(ErrorCode::SIZE);
        }
        self.enable_peripheral()?;

        let len = modulus_len;
        match operation {
            SupportedOp::Addition => {
                self.begin_math(State::MathAddition, len);
            }
            SupportedOp::Multiplication => {
                self.begin_math(State::MathComputeR2, len);
            }
            SupportedOp::Inverse => {
                self.begin_math(State::MathInvert, len);
            }
            SupportedOp::Modulus => {
                self.begin_math(State::MathModulus, len);
            }
        }
        self.deferred_call.set();
        Ok(())
    }

    fn clear_data(&self) {
        self.clear_ram();
    }
}

impl DeferredCallClient for Pka<'_> {
    fn handle_deferred_call(&self) {
        match self.state.get() {
            State::ScalarMul => {
                let mut scalar = [0u8; P_256_P_SIZE];
                self.ecc_client
                    .map(|client| client.read_scalar(&mut scalar));
                self.write_slice(ECC_MUL_K_IDX, &scalar);
                self.load_point(ECC_MUL_IN_X_IDX, ECC_MUL_IN_Y_IDX);
                self.start_operation(CR::MODE::MontgomeryECC);
            }
            State::PointAddition => {
                let mut point_q = [0u8; 2 * P_256_P_SIZE];
                let mut z_coord = [0u8; P_256_P_SIZE];
                z_coord[P_256_P_SIZE - 1] = 1;
                self.load_point(ECC_ADD_PT1_X_IDX, ECC_ADD_PT1_Y_IDX);
                self.write_slice(ECC_ADD_PT1_Z_IDX, &z_coord);
                self.ecc_client
                    .map(|client| client.read_second_point(&mut point_q));
                self.write_slice(ECC_ADD_PT2_X_IDX, &point_q[..P_256_P_SIZE]);
                self.write_slice(ECC_ADD_PT2_Y_IDX, &point_q[P_256_P_SIZE..]);
                self.write_slice(ECC_ADD_PT2_Z_IDX, &z_coord);
                self.start_operation(CR::MODE::ECCCompleteAddition);
            }
            State::MathAddition => {
                let len = self.math_len.get();
                self.load_operand(MODULUS_IDX, len, Operand::Modulus);
                self.load_operand(ARITH_OP1_IDX, len, Operand::First);
                self.load_operand(ARITH_OP2_IDX, len, Operand::Second);
                self.start_operation(CR::MODE::ModularAddition);
            }
            State::MathInvert => {
                let len = self.math_len.get();
                self.load_operand(INV_RED_MODULUS_IDX, len, Operand::Modulus);
                self.load_operand(ARITH_OP1_IDX, len, Operand::First);
                self.start_operation(CR::MODE::ModularInversion);
            }
            State::MathComputeR2 => {
                let len = self.math_len.get();
                self.load_operand(MODULUS_IDX, len, Operand::Modulus);
                self.start_operation(CR::MODE::MontgomeryOnly);
            }
            State::MathModulus => {
                let len = self.math_len.get();
                self.set_len(EXP_LEN_BITS_IDX, (len as u32) * 8);
                self.load_operand(INV_RED_MODULUS_IDX, len, Operand::Modulus);
                self.load_operand(ARITH_OP1_IDX, len, Operand::First);
                self.start_operation(CR::MODE::ModularReduction);
                self.state.set(State::MathAddition);
            }
            _ => {}
        }
    }

    fn register(&'static self) {
        self.deferred_call.register(self);
    }
}
