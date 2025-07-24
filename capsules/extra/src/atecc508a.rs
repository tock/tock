// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2023.

//! Capsule for interfacing with the ATECC508A CryptoAuthentication Device
//! using the I2C bus.
//!
//! <https://ww1.microchip.com/downloads/en/DeviceDoc/20005928A.pdf>
//!
//! The device requires at least 60us of the SDA pin being pulled low
//! to power on. So before any I2C commands can be issued the SDA pin
//! must be pulled low.
//!
//! The ATECC508A is shipped in an unlocked state. That is, the configuration
//! can be changed. The ATECC508A is practically useless while it's unlocked
//! though. Even the random number generator only returns
//! 0xFF, 0xFF, 0x00, 0x00 when the device is unlocked.
//!
//! Locking the device is permanent! Once the device is locked it can not be
//! unlocked. Be very careful about locking the configurations. In saying that
//! the device must be locked before it can be used.
//!
//! Look at the `setup_and_lock_tock_config()` function for an example of
//! setting up the device.

use core::cell::Cell;
use kernel::debug;
use kernel::hil::i2c::{self, I2CClient, I2CDevice};
use kernel::hil::public_key_crypto::keys;
use kernel::hil::public_key_crypto::signature::{ClientVerify, SignatureVerify};
use kernel::hil::{digest, entropy, entropy::Entropy32};
use kernel::utilities::cells::{MapCell, OptionalCell, TakeCell};
use kernel::utilities::leasable_buffer::{SubSlice, SubSliceMut, SubSliceMutImmut};
use kernel::ErrorCode;

/* Protocol + Cryptographic defines */
const RESPONSE_COUNT_SIZE: usize = 1;
const RESPONSE_SIGNAL_SIZE: usize = 1;
const RESPONSE_SHA_SIZE: usize = 32;
#[allow(dead_code)]
const RESPONSE_INFO_SIZE: usize = 4;
const RESPONSE_RANDOM_SIZE: usize = 32;
const CRC_SIZE: usize = 2;
#[allow(dead_code)]
const CONFIG_ZONE_SIZE: usize = 128;
#[allow(dead_code)]
const SERIAL_NUMBER_SIZE: usize = 10;

/* Protocol Indices */
const ATRCC508A_PROTOCOL_FIELD_COMMAND: usize = 0;
const ATRCC508A_PROTOCOL_FIELD_LENGTH: usize = 1;
const ATRCC508A_PROTOCOL_FIELD_OPCODE: usize = 2;
const ATRCC508A_PROTOCOL_FIELD_PARAM1: usize = 3;
const ATRCC508A_PROTOCOL_FIELD_PARAM2: usize = 4;
const ATRCC508A_PROTOCOL_FIELD_DATA: usize = 6;

/* Protocl Sizes */
const ATRCC508A_PROTOCOL_FIELD_SIZE_COMMAND: usize = 1;
const ATRCC508A_PROTOCOL_FIELD_SIZE_LENGTH: usize = 1;
const ATRCC508A_PROTOCOL_FIELD_SIZE_OPCODE: usize = 1;
const ATRCC508A_PROTOCOL_FIELD_SIZE_PARAM1: usize = 1;
const ATRCC508A_PROTOCOL_FIELD_SIZE_PARAM2: usize = 2;
const ATRCC508A_PROTOCOL_FIELD_SIZE_CRC: usize = CRC_SIZE;

const ZONE_CONFIG: u8 = 0x00;
#[allow(dead_code)]
const ZONE_OTP: u8 = 0x01;
#[allow(dead_code)]
const ZONE_DATA: u8 = 0x02;

const ADDRESS_CONFIG_READ_BLOCK_0: u16 = 0x0000; // 00000000 00000000 // param2 (byte 0), address block bits: _ _ _ 0  0 _ _ _
#[allow(dead_code)]
const ADDRESS_CONFIG_READ_BLOCK_1: u16 = 0x0008; // 00000000 00001000 // param2 (byte 0), address block bits: _ _ _ 0  1 _ _ _
const ADDRESS_CONFIG_READ_BLOCK_2: u16 = 0x0010; // 00000000 00010000 // param2 (byte 0), address block bits: _ _ _ 1  0 _ _ _
#[allow(dead_code)]
const ADDRESS_CONFIG_READ_BLOCK_3: u16 = 0x0018; // 00000000 00011000 // param2 (byte 0), address block bits: _ _ _ 1  1 _ _ _

/* configZone EEPROM mapping */
const CONFIG_ZONE_READ_SIZE: usize = 32;
#[allow(dead_code)]
const CONFIG_ZONE_SLOT_CONFIG: usize = 20;
const CONFIG_ZONE_OTP_LOCK: usize = 86;
const CONFIG_ZONE_LOCK_STATUS: usize = 87;
const CONFIG_ZONE_SLOTS_LOCK0: usize = 88;
const CONFIG_ZONE_SLOTS_LOCK1: usize = 89;
#[allow(dead_code)]
const CONFIG_ZONE_KEY_CONFIG: usize = 96;

// COMMANDS (aka "opcodes" in the datasheet)
#[allow(dead_code)]
const COMMAND_OPCODE_INFO: u8 = 0x30; // Return device state information.
const COMMAND_OPCODE_LOCK: u8 = 0x17; // Lock configuration and/or Data and OTP zones
const COMMAND_OPCODE_RANDOM: u8 = 0x1B; // Create and return a random number (32 bytes of data)
const COMMAND_OPCODE_READ: u8 = 0x02; // Return data at a specific zone and address.
#[allow(dead_code)]
const COMMAND_OPCODE_WRITE: u8 = 0x12; // Return data at a specific zone and address.
const COMMAND_OPCODE_SHA: u8 = 0x47; // Computes a SHA-256 or HMAC/SHA digest for general purpose use by the system.
#[allow(dead_code)]
const COMMAND_OPCODE_GENKEY: u8 = 0x40; // Creates a key (public and/or private) and stores it in a memory key slot
#[allow(dead_code)]
const COMMAND_OPCODE_NONCE: u8 = 0x16; //
#[allow(dead_code)]
const COMMAND_OPCODE_SIGN: u8 = 0x41; // Create an ECC signature with contents of TempKey and designated key slot
#[allow(dead_code)]
const COMMAND_OPCODE_VERIFY: u8 = 0x45; // takes an ECDSA <R,S> signature and verifies that it is correctly generated from a given message and public key

const VERIFY_MODE_EXTERNAL: u8 = 0x02; // Use an external public key for verification, pass to command as data post param2, ds pg 89
#[allow(dead_code)]
const VERIFY_MODE_STORED: u8 = 0b00000000; // Use an internally stored public key for verification, param2 = keyID, ds pg 89
const VERIFY_PARAM2_KEYTYPE_ECC: u8 = 0x0004; // When verify mode external, param2 should be KeyType, ds pg 89
#[allow(dead_code)]
const VERIFY_PARAM2_KEYTYPE_NONECC: u8 = 0x0007; // When verify mode external, param2 should be KeyType, ds pg 89
const NONCE_MODE_PASSTHROUGH: u8 = 0b00000011; // Operate in pass-through mode and Write TempKey with NumIn. datasheet pg 79

const LOCK_MODE_ZONE_CONFIG: u8 = 0b10000000;
const LOCK_MODE_ZONE_DATA_AND_OTP: u8 = 0b10000001;
const LOCK_MODE_SLOT0: u8 = 0b10000010;

#[allow(dead_code)]
const RANDOM_BYTES_BLOCK_SIZE: usize = 32;

const SHA_START: u8 = 0;
const SHA_UPDATE: u8 = 1;
const SHA_END: u8 = 2;

#[allow(dead_code)]
const SHA256_SIZE: usize = 32;
const PUBLIC_KEY_SIZE: usize = 64;
#[allow(dead_code)]
const SIGNATURE_SIZE: usize = 64;
#[allow(dead_code)]
const BUFFER_SIZE: usize = 128;

const RESPONSE_SIGNAL_INDEX: usize = RESPONSE_COUNT_SIZE;
const ATRCC508A_SUCCESSFUL_TEMPKEY: u8 = 0x00;
const ATRCC508A_SUCCESSFUL_VERIFY: u8 = 0x00;
const ATRCC508A_SUCCESSFUL_LOCK: u8 = 0x00;

const WORD_ADDRESS_VALUE_RESET: u8 = 0x00;
const WORD_ADDRESS_VALUE_IDLE: u8 = 0x02;
const WORD_ADDRESS_VALUE_COMMAND: u8 = 0x03;

const ATRCC508A_PROTOCOL_OVERHEAD: usize = ATRCC508A_PROTOCOL_FIELD_SIZE_COMMAND
    + ATRCC508A_PROTOCOL_FIELD_SIZE_LENGTH
    + ATRCC508A_PROTOCOL_FIELD_SIZE_OPCODE
    + ATRCC508A_PROTOCOL_FIELD_SIZE_PARAM1
    + ATRCC508A_PROTOCOL_FIELD_SIZE_PARAM2
    + ATRCC508A_PROTOCOL_FIELD_SIZE_CRC;

#[allow(dead_code)]
const GENKEY_MODE_PUBLIC: u8 = 0b00000000;
const GENKEY_MODE_NEW_PRIVATE: u8 = 0b00000100;

#[derive(Clone, Copy, Debug, PartialEq)]
enum Operation {
    Reset,
    Ready,
    ReadConfigZeroCommand,
    ReadConfigZeroResult(usize),
    ReadConfigTwoCommand,
    ReadConfigTwoResult(usize),
    GenerateEntropyCommand(usize),
    GenerateEntropyResult(usize),
    SetupConfigOne,
    SetupConfigTwo(usize),
    LockZoneConfig(usize),
    LockResponse(usize),
    CreateKeyPair(usize, u16),
    ReadKeyPair(usize),
    LockDataOtp(usize),
    LockSlot0(usize),
    StartSha(usize),
    ShaLoad(usize),
    ShaLoadResponse(usize),
    ReadySha,
    ShaRun(usize),
    ShaEnd(usize),
    LoadTempKeyNonce(usize),
    LoadTempKeyCheckNonce(usize),
    VerifySubmitData(usize),
    CompleteVerify(usize),
}

pub struct Atecc508a<'a> {
    buffer: TakeCell<'static, [u8]>,
    i2c: &'a dyn I2CDevice,
    op: Cell<Operation>,
    op_len: Cell<usize>,

    entropy_buffer: TakeCell<'static, [u8; 32]>,
    entropy_offset: Cell<usize>,
    entropy_client: OptionalCell<&'a dyn entropy::Client32>,

    digest_client: OptionalCell<&'a dyn digest::ClientDataHash<32>>,
    digest_buffer: TakeCell<'static, [u8; 64]>,
    write_len: Cell<usize>,
    remain_len: Cell<usize>,
    hash_data: MapCell<SubSliceMutImmut<'static, u8>>,
    digest_data: TakeCell<'static, [u8; 32]>,

    secure_client: OptionalCell<&'a dyn ClientVerify<32, 64>>,
    message_data: TakeCell<'static, [u8; 32]>,
    signature_data: TakeCell<'static, [u8; 64]>,
    ext_public_key: TakeCell<'static, [u8; 64]>,
    ext_public_key_temp: TakeCell<'static, [u8; 64]>,
    key_set_client: OptionalCell<&'a dyn keys::SetKeyBySliceClient<64>>,
    deferred_call: kernel::deferred_call::DeferredCall,

    wakeup_device: fn(),

    config_lock: Cell<bool>,
    data_lock: Cell<bool>,
    public_key: OptionalCell<[u8; PUBLIC_KEY_SIZE]>,
}

impl<'a> Atecc508a<'a> {
    pub fn new(
        i2c: &'a dyn I2CDevice,
        buffer: &'static mut [u8],
        entropy_buffer: &'static mut [u8; 32],
        digest_buffer: &'static mut [u8; 64],
        wakeup_device: fn(),
    ) -> Self {
        Atecc508a {
            buffer: TakeCell::new(buffer),
            i2c,
            op: Cell::new(Operation::Ready),
            op_len: Cell::new(0),
            entropy_buffer: TakeCell::new(entropy_buffer),
            entropy_offset: Cell::new(0),
            entropy_client: OptionalCell::empty(),
            digest_client: OptionalCell::empty(),
            digest_buffer: TakeCell::new(digest_buffer),
            write_len: Cell::new(0),
            remain_len: Cell::new(0),
            hash_data: MapCell::empty(),
            digest_data: TakeCell::empty(),
            secure_client: OptionalCell::empty(),
            message_data: TakeCell::empty(),
            signature_data: TakeCell::empty(),
            ext_public_key: TakeCell::empty(),
            ext_public_key_temp: TakeCell::empty(),
            key_set_client: OptionalCell::empty(),
            deferred_call: kernel::deferred_call::DeferredCall::new(),
            wakeup_device,
            config_lock: Cell::new(false),
            data_lock: Cell::new(false),
            public_key: OptionalCell::new([0; PUBLIC_KEY_SIZE]),
        }
    }

    fn calculate_crc(data: &[u8]) -> u16 {
        let mut crc_register: u16 = 0;

        for counter in 0..data.len() {
            let mut shift_register: u8 = 0x01;

            while shift_register > 0x00 {
                let data_val = data[counter] & shift_register;
                let data_bit = u16::from(data_val > 0);

                let crc_bit = crc_register >> 15;
                crc_register <<= 1;

                if data_bit != crc_bit {
                    crc_register ^= 0x8005;
                }

                shift_register <<= 1;
            }
        }

        crc_register
    }

    fn read(&self, zone: u8, address: u16, length: usize) -> Result<(), ErrorCode> {
        let mut zone_calc = zone;

        match length {
            32 => zone_calc |= 1 << 7,
            4 => zone_calc &= !(1 << 7),
            _ => return Err(ErrorCode::SIZE),
        }

        self.op_len.set(length);

        self.send_command(COMMAND_OPCODE_READ, zone_calc, address, 0);

        Ok(())
    }

    fn write(&self, zone: u8, address: u16, length: usize) -> Result<(), ErrorCode> {
        let mut zone_calc = zone;

        match length {
            32 => zone_calc |= 1 << 7,
            4 => zone_calc &= !(1 << 7),
            _ => return Err(ErrorCode::SIZE),
        }

        self.op_len.set(length);

        self.send_command(COMMAND_OPCODE_WRITE, zone_calc, address, length);

        Ok(())
    }

    fn idle(&self) {
        self.buffer.take().map(|buffer| {
            buffer[ATRCC508A_PROTOCOL_FIELD_COMMAND] = WORD_ADDRESS_VALUE_IDLE;

            let _ = self.i2c.write(buffer, 1);
        });
    }

    fn reset(&self) {
        self.buffer.take().map(|buffer| {
            self.op.set(Operation::Reset);

            buffer[ATRCC508A_PROTOCOL_FIELD_COMMAND] = WORD_ADDRESS_VALUE_RESET;

            let _ = self.i2c.write(buffer, 1);
        });
    }

    fn send_command(&self, command_opcode: u8, param1: u8, param2: u16, length: usize) {
        let i2c_length = length + ATRCC508A_PROTOCOL_OVERHEAD;

        self.buffer.take().map(|buffer| {
            buffer[ATRCC508A_PROTOCOL_FIELD_COMMAND] = WORD_ADDRESS_VALUE_COMMAND;
            buffer[ATRCC508A_PROTOCOL_FIELD_LENGTH] =
                (i2c_length - ATRCC508A_PROTOCOL_FIELD_SIZE_LENGTH) as u8;
            buffer[ATRCC508A_PROTOCOL_FIELD_OPCODE] = command_opcode;
            buffer[ATRCC508A_PROTOCOL_FIELD_PARAM1] = param1;
            buffer[ATRCC508A_PROTOCOL_FIELD_PARAM2] = (param2 & 0xFF) as u8;
            buffer[ATRCC508A_PROTOCOL_FIELD_PARAM2 + 1] = ((param2 >> 8) & 0xFF) as u8;

            let data_crc_len = i2c_length
                - (ATRCC508A_PROTOCOL_FIELD_SIZE_COMMAND + ATRCC508A_PROTOCOL_FIELD_SIZE_CRC);
            let crc = Self::calculate_crc(
                &buffer[ATRCC508A_PROTOCOL_FIELD_LENGTH
                    ..(data_crc_len + ATRCC508A_PROTOCOL_FIELD_LENGTH)],
            );

            buffer[i2c_length - ATRCC508A_PROTOCOL_FIELD_SIZE_CRC] = (crc & 0xFF) as u8;
            buffer[i2c_length - ATRCC508A_PROTOCOL_FIELD_SIZE_CRC + 1] = ((crc >> 8) & 0xFF) as u8;

            let _ = self.i2c.write(buffer, i2c_length);
        });
    }

    /// Read information from the configuration zone and print it
    /// This will work while the device is either locked or unlocked
    pub fn read_config_zone(&self) -> Result<(), ErrorCode> {
        assert_eq!(self.op.get(), Operation::Ready);

        self.op.set(Operation::ReadConfigZeroCommand);

        (self.wakeup_device)();

        self.read(
            ZONE_CONFIG,
            ADDRESS_CONFIG_READ_BLOCK_0,
            CONFIG_ZONE_READ_SIZE,
        )
    }

    /// Setup the device keys and config
    ///
    /// This will only work on an unlocked device and will lock the device!
    ///
    /// The slots will be configured as below
    ///
    ///```ignore
    /// Slot 0: 0x2083
    ///     - ReadKey: External signatures of arbitrary messages are enabled
    ///                Internal signatures of messages generated by GenDig or GenKey are enabled
    ///     - IsSecret: The contents of this slot are secret
    ///     - WriteConfig: PubInvalid
    ///
    ///     Key Config: 0x33
    ///        - Private: The key slot contains an ECC private key
    ///        - PubInfo: The public version of this key can always be generated
    ///        - KeyType: P256 NIST ECC key
    ///        - Lockable: Slot can be individually locked using the Lock command
    ///
    /// Slot 1: 0x2083
    ///     - ReadKey: External signatures of arbitrary messages are enabled
    ///                Internal signatures of messages generated by GenDig or GenKey are enabled
    ///     - IsSecret: The contents of this slot are secret
    ///     - WriteConfig: PubInvalid
    ///
    ///     Key Config: 0x33
    ///        - Private: The key slot contains an ECC private key
    ///        - PubInfo: The public version of this key can always be generated
    ///        - KeyType: P256 NIST ECC key
    ///        - Lockable: Slot can be individually locked using the Lock command
    ///
    /// Slot 2:
    ///     - ReadKey: Then this slot can be the source for the CheckMac/Copy operation
    ///     - WriteConfig: Always
    ///```
    pub fn setup_tock_config(&self) -> Result<(), ErrorCode> {
        self.op.set(Operation::SetupConfigOne);

        (self.wakeup_device)();

        // Set keytype on slot 0 and 1 to 0x3300
        self.buffer.take().map(|buffer| {
            buffer[ATRCC508A_PROTOCOL_FIELD_DATA..(ATRCC508A_PROTOCOL_FIELD_DATA + 4)]
                .copy_from_slice(&[0x33, 0x00, 0x33, 0x00]);

            self.buffer.replace(buffer);
        });

        self.write(ZONE_CONFIG, 96 / 4, 4)?;

        Ok(())
    }

    /// Lock the zone config
    pub fn lock_zone_config(&self) -> Result<(), ErrorCode> {
        self.op.set(Operation::LockZoneConfig(0));

        self.send_command(COMMAND_OPCODE_LOCK, LOCK_MODE_ZONE_CONFIG, 0x0000, 0);

        Ok(())
    }

    /// Create a key pair in the `slot`
    pub fn create_key_pair(&self, slot: u16) -> Result<(), ErrorCode> {
        self.op.set(Operation::CreateKeyPair(0, slot));

        (self.wakeup_device)();

        self.send_command(COMMAND_OPCODE_GENKEY, GENKEY_MODE_NEW_PRIVATE, slot, 0);

        Ok(())
    }

    /// Retrieve the public key from the slot.
    ///
    /// This can be called only after `create_key_pair()` has completed
    pub fn get_public_key(
        &'a self,
        _slot: u16,
    ) -> Result<&'a OptionalCell<[u8; PUBLIC_KEY_SIZE]>, ErrorCode> {
        if self.public_key.get().unwrap().iter().all(|&x| x == 0) {
            return Err(ErrorCode::BUSY);
        }

        Ok(&self.public_key)
    }

    /// Set the public key to use for the `verify` command, if using an
    /// external key.
    ///
    /// This will return the previous key if one was stored. Pass in
    /// `None` to retrieve the key without providing a new one.
    pub fn set_public_key(
        &self,
        public_key: Option<&'static mut [u8; 64]>,
    ) -> Option<&'static mut [u8; 64]> {
        let ret = self.ext_public_key.take();

        if let Some(key) = public_key {
            self.ext_public_key.replace(key);
        }

        ret
    }

    /// Lock the data and OTP
    pub fn lock_data_and_otp(&self) -> Result<(), ErrorCode> {
        self.op.set(Operation::LockDataOtp(0));

        (self.wakeup_device)();

        self.send_command(COMMAND_OPCODE_LOCK, LOCK_MODE_ZONE_DATA_AND_OTP, 0x0000, 0);

        Ok(())
    }

    /// Lock the slot0 config
    pub fn lock_slot0(&self) -> Result<(), ErrorCode> {
        self.op.set(Operation::LockSlot0(0));

        (self.wakeup_device)();

        self.send_command(COMMAND_OPCODE_LOCK, LOCK_MODE_SLOT0, 0x0000, 0);

        Ok(())
    }

    /// Check is the device configuration is locked
    pub fn device_locked(&self) -> bool {
        self.config_lock.get() && self.data_lock.get()
    }

    fn sha_update(&self) -> bool {
        let op_len = (self.hash_data.map_or(0, |buf| match buf {
            SubSliceMutImmut::Mutable(b) => {
                let len = b.len();
                self.remain_len.get() + len
            }
            SubSliceMutImmut::Immutable(b) => {
                let len = b.len();
                self.remain_len.get() + len
            }
        }))
        .min(64);

        // The SHA Update step only supports 64-bytes, so if we have less
        // we store the data and write it when we have more data or on the
        // End command
        if op_len < 64 {
            self.op.set(Operation::ReadySha);

            self.hash_data.map(|buf| {
                if op_len > 0 {
                    self.digest_buffer.map(|digest_buffer| {
                        digest_buffer[self.remain_len.get()..(self.remain_len.get() + op_len)]
                            .copy_from_slice(&buf[0..op_len]);

                        self.remain_len.set(self.remain_len.get() + op_len);

                        buf.slice(op_len..);
                    });
                }

                self.idle();
            });

            return false;
        }

        // At this point we have at least 64 bytes to write
        self.buffer.take().map(|buffer| {
            let remain_len = self.remain_len.get();

            if remain_len > 0 {
                self.digest_buffer.map(|digest_buffer| {
                    buffer[ATRCC508A_PROTOCOL_FIELD_DATA
                        ..(ATRCC508A_PROTOCOL_FIELD_DATA + remain_len)]
                        .copy_from_slice(&digest_buffer[0..remain_len]);
                });
            }

            self.hash_data.map(|hash_data| {
                buffer[ATRCC508A_PROTOCOL_FIELD_DATA + remain_len
                    ..(ATRCC508A_PROTOCOL_FIELD_DATA + op_len)]
                    .copy_from_slice(&hash_data[0..(op_len - remain_len)]);

                hash_data.slice((op_len - remain_len)..);
            });

            self.remain_len.set(0);
            self.buffer.replace(buffer);
        });

        self.op.set(Operation::ShaLoadResponse(0));

        true
    }
}

impl I2CClient for Atecc508a<'_> {
    fn command_complete(&self, buffer: &'static mut [u8], status: Result<(), i2c::Error>) {
        match self.op.get() {
            Operation::Ready => unreachable!(),
            Operation::ReadySha => {
                self.buffer.replace(buffer);

                self.hash_data.take().map(|buf| {
                    self.digest_client.map(|client| match buf {
                        SubSliceMutImmut::Mutable(b) => client.add_mut_data_done(Ok(()), b),
                        SubSliceMutImmut::Immutable(b) => client.add_data_done(Ok(()), b),
                    })
                });
            }
            Operation::Reset => {
                self.buffer.replace(buffer);
            }
            Operation::ReadConfigZeroCommand => {
                self.op.set(Operation::ReadConfigZeroResult(0));

                let _ = self
                    .i2c
                    .read(buffer, self.op_len.get() + RESPONSE_COUNT_SIZE + CRC_SIZE);
            }
            Operation::ReadConfigZeroResult(run) => {
                if status == Err(i2c::Error::DataNak) || status == Err(i2c::Error::AddressNak) {
                    // The device isn't ready yet, try again
                    if run == 10 {
                        return;
                    }

                    self.op.set(Operation::ReadConfigZeroResult(run + 1));

                    let _ = self
                        .i2c
                        .read(buffer, self.op_len.get() + RESPONSE_COUNT_SIZE + CRC_SIZE);

                    return;
                }

                self.op.set(Operation::ReadConfigTwoCommand);

                assert_eq!(status, Ok(()));

                let mut serial_num: [u8; 9] = [0; 9];
                serial_num[0..3].copy_from_slice(&buffer[0..3]);
                serial_num[4..8].copy_from_slice(&buffer[8..12]);

                debug!("ATECC508A Serial Number: {serial_num:x?}");
                debug!("ATECC508A Rev Number: {:x?}", &buffer[4..7]);

                self.buffer.replace(buffer);

                let _ = self.read(
                    ZONE_CONFIG,
                    ADDRESS_CONFIG_READ_BLOCK_2,
                    CONFIG_ZONE_READ_SIZE,
                );
            }
            Operation::ReadConfigTwoCommand => {
                self.op.set(Operation::ReadConfigTwoResult(0));

                let _ = self
                    .i2c
                    .read(buffer, self.op_len.get() + RESPONSE_COUNT_SIZE + CRC_SIZE);
            }
            Operation::ReadConfigTwoResult(run) => {
                if status == Err(i2c::Error::DataNak) || status == Err(i2c::Error::AddressNak) {
                    // The device isn't ready yet, try again
                    if run == 10 {
                        return;
                    }

                    self.op.set(Operation::ReadConfigTwoResult(run + 1));

                    let _ = self
                        .i2c
                        .read(buffer, self.op_len.get() + RESPONSE_COUNT_SIZE + CRC_SIZE);

                    return;
                }

                self.op.set(Operation::Ready);

                assert_eq!(status, Ok(()));

                let otp_lock = buffer[CONFIG_ZONE_OTP_LOCK - 63];
                if otp_lock == 0x55 {
                    debug!("ATECC508A Data and OTP UnLocked");
                } else if otp_lock == 0x00 {
                    debug!("ATECC508A Data and OTP Locked");
                    self.data_lock.set(true);
                } else {
                    debug!("ATECC508A Data and OTP Invalid Config");
                }

                let config_lock = buffer[CONFIG_ZONE_LOCK_STATUS - 63];
                if config_lock == 0x55 {
                    debug!("ATECC508A Config Zone UnLocked");
                } else if config_lock == 0x00 {
                    debug!("ATECC508A Config Zone Locked");
                    self.config_lock.set(true);
                } else {
                    debug!("ATECC508A Config Zone Invalid Config");
                }

                debug!(
                    "ATECC508A Slot Lock Status: 0x{:x} and 0x{:x}",
                    &buffer[CONFIG_ZONE_SLOTS_LOCK0 - 63],
                    &buffer[CONFIG_ZONE_SLOTS_LOCK1 - 63]
                );

                self.buffer.replace(buffer);
            }
            Operation::GenerateEntropyCommand(run) => {
                if status == Err(i2c::Error::DataNak) || status == Err(i2c::Error::AddressNak) {
                    self.buffer.replace(buffer);

                    // The device isn't ready yet, try again
                    if run == 10 {
                        self.entropy_client.map(move |client| {
                            client.entropy_available(
                                &mut Atecc508aRngIter(self),
                                Err(ErrorCode::NOACK),
                            );
                        });

                        return;
                    }

                    self.op.set(Operation::GenerateEntropyCommand(run + 1));
                    self.send_command(COMMAND_OPCODE_RANDOM, 0x00, 0x0000, 0);

                    return;
                }

                self.op.set(Operation::GenerateEntropyResult(0));

                let _ = self.i2c.read(
                    buffer,
                    RESPONSE_COUNT_SIZE + RESPONSE_RANDOM_SIZE + CRC_SIZE,
                );
            }
            Operation::GenerateEntropyResult(run) => {
                if status == Err(i2c::Error::DataNak) || status == Err(i2c::Error::AddressNak) {
                    // The device isn't ready yet, try again

                    if run == 1000 {
                        self.entropy_client.map(move |client| {
                            client.entropy_available(
                                &mut Atecc508aRngIter(self),
                                Err(ErrorCode::NOACK),
                            );
                        });

                        return;
                    }

                    self.op.set(Operation::GenerateEntropyResult(run + 1));

                    let _ = self.i2c.read(
                        buffer,
                        RESPONSE_COUNT_SIZE + RESPONSE_RANDOM_SIZE + CRC_SIZE,
                    );

                    return;
                }

                self.op.set(Operation::Ready);

                self.entropy_buffer.take().map(|entropy_buffer| {
                    entropy_buffer.copy_from_slice(
                        &buffer[RESPONSE_COUNT_SIZE..(RESPONSE_COUNT_SIZE + RESPONSE_RANDOM_SIZE)],
                    );

                    self.entropy_buffer.replace(entropy_buffer);
                });

                self.buffer.replace(buffer);

                if self.entropy_client.map(move |client| {
                    client.entropy_available(&mut Atecc508aRngIter(self), Ok(()))
                }) == Some(entropy::Continue::More)
                {
                    // We need more
                    if let Err(e) = self.get() {
                        self.entropy_client.map(move |client| {
                            client.entropy_available(&mut (0..0), Err(e));
                        });
                    }
                }
            }
            Operation::SetupConfigOne => {
                self.op.set(Operation::SetupConfigTwo(0));

                self.buffer.replace(buffer);

                // Set slot config on slot 0 and 1 to 0x8320
                self.buffer.take().map(|buffer| {
                    buffer[ATRCC508A_PROTOCOL_FIELD_DATA..(ATRCC508A_PROTOCOL_FIELD_DATA + 4)]
                        .copy_from_slice(&[0x83, 0x20, 0x83, 0x20]);

                    self.buffer.replace(buffer);
                });

                let _ = self.write(ZONE_CONFIG, 20 / 4, 4);
            }
            Operation::SetupConfigTwo(run) => {
                self.buffer.replace(buffer);

                if status == Err(i2c::Error::DataNak) || status == Err(i2c::Error::AddressNak) {
                    // The device isn't ready yet, try again
                    if run == 10 {
                        self.op.set(Operation::Ready);
                        return;
                    }

                    self.op.set(Operation::SetupConfigTwo(run + 1));
                    let _ = self.write(ZONE_CONFIG, 20 / 4, 4);
                    return;
                }

                self.op.set(Operation::Ready);
            }
            Operation::LockZoneConfig(run) => {
                if status == Err(i2c::Error::DataNak) || status == Err(i2c::Error::AddressNak) {
                    self.buffer.replace(buffer);

                    // The device isn't ready yet, try again
                    if run == 30 {
                        self.op.set(Operation::Ready);
                        return;
                    }

                    self.op.set(Operation::LockZoneConfig(run + 1));
                    self.send_command(COMMAND_OPCODE_LOCK, LOCK_MODE_ZONE_CONFIG, 0x0000, 0);
                    return;
                }

                self.op.set(Operation::LockResponse(0));

                let _ = self.i2c.read(
                    buffer,
                    RESPONSE_COUNT_SIZE + RESPONSE_SIGNAL_SIZE + CRC_SIZE,
                );
            }
            Operation::LockResponse(run) => {
                if status == Err(i2c::Error::DataNak) || status == Err(i2c::Error::AddressNak) {
                    // The device isn't ready yet, try again
                    if run == 100 {
                        self.buffer.replace(buffer);
                        self.op.set(Operation::Ready);
                        return;
                    }

                    self.op.set(Operation::LockResponse(run + 1));
                    let _ = self.i2c.read(
                        buffer,
                        RESPONSE_COUNT_SIZE + RESPONSE_SIGNAL_SIZE + CRC_SIZE,
                    );
                    return;
                }

                self.op.set(Operation::Ready);

                let response = buffer[RESPONSE_SIGNAL_INDEX];

                if response != ATRCC508A_SUCCESSFUL_LOCK {
                    debug!("Failed to lock the device");
                }

                self.buffer.replace(buffer);
            }
            Operation::CreateKeyPair(run, slot) => {
                if status == Err(i2c::Error::DataNak) || status == Err(i2c::Error::AddressNak) {
                    self.buffer.replace(buffer);

                    // The device isn't ready yet, try again
                    if run == 10 {
                        self.op.set(Operation::Ready);
                        return;
                    }

                    self.op.set(Operation::CreateKeyPair(run + 1, slot));
                    self.send_command(COMMAND_OPCODE_GENKEY, GENKEY_MODE_NEW_PRIVATE, slot, 0);
                    return;
                }

                self.op.set(Operation::ReadKeyPair(0));

                let _ = self
                    .i2c
                    .read(buffer, RESPONSE_COUNT_SIZE + PUBLIC_KEY_SIZE + CRC_SIZE);
            }
            Operation::ReadKeyPair(run) => {
                if status == Err(i2c::Error::DataNak) || status == Err(i2c::Error::AddressNak) {
                    // The device isn't ready yet, try again
                    // This can take awhile to generate
                    if run == 5000 {
                        self.buffer.replace(buffer);
                        self.op.set(Operation::Ready);
                        return;
                    }

                    self.op.set(Operation::ReadKeyPair(run + 1));
                    let _ = self
                        .i2c
                        .read(buffer, RESPONSE_COUNT_SIZE + PUBLIC_KEY_SIZE + CRC_SIZE);
                    return;
                }

                self.public_key.take().map(|mut pub_key| {
                    pub_key.copy_from_slice(
                        &buffer[RESPONSE_COUNT_SIZE..(RESPONSE_COUNT_SIZE + PUBLIC_KEY_SIZE)],
                    );
                    self.public_key.set(pub_key);
                });

                self.op.set(Operation::Ready);
                self.buffer.replace(buffer);
            }
            Operation::LockDataOtp(run) => {
                if status == Err(i2c::Error::DataNak) || status == Err(i2c::Error::AddressNak) {
                    self.buffer.replace(buffer);

                    // The device isn't ready yet, try again
                    if run == 100 {
                        self.op.set(Operation::Ready);
                        return;
                    }

                    self.op.set(Operation::LockDataOtp(run + 1));
                    self.send_command(COMMAND_OPCODE_LOCK, LOCK_MODE_ZONE_DATA_AND_OTP, 0x0000, 0);
                    return;
                }

                self.op.set(Operation::LockResponse(0));

                let _ = self.i2c.read(
                    buffer,
                    RESPONSE_COUNT_SIZE + RESPONSE_SIGNAL_SIZE + CRC_SIZE,
                );
            }
            Operation::LockSlot0(run) => {
                if status == Err(i2c::Error::DataNak) || status == Err(i2c::Error::AddressNak) {
                    self.buffer.replace(buffer);

                    // The device isn't ready yet, try again
                    if run == 100 {
                        self.op.set(Operation::Ready);
                        return;
                    }

                    self.op.set(Operation::LockSlot0(run + 1));
                    self.send_command(COMMAND_OPCODE_LOCK, LOCK_MODE_SLOT0, 0x0000, 0);
                    return;
                }

                self.op.set(Operation::LockResponse(0));

                let _ = self.i2c.read(
                    buffer,
                    RESPONSE_COUNT_SIZE + RESPONSE_SIGNAL_SIZE + CRC_SIZE,
                );
            }
            Operation::StartSha(run) => {
                if status == Err(i2c::Error::DataNak) || status == Err(i2c::Error::AddressNak) {
                    self.buffer.replace(buffer);

                    // The device isn't ready yet, try again
                    if run == 10 {
                        self.op.set(Operation::Ready);
                        return;
                    }

                    self.op.set(Operation::StartSha(run + 1));
                    self.send_command(COMMAND_OPCODE_SHA, SHA_START, 0x0000, 0);
                    return;
                }

                self.op.set(Operation::ShaLoad(0));

                let _ = self.i2c.read(
                    buffer,
                    RESPONSE_COUNT_SIZE + RESPONSE_SIGNAL_SIZE + CRC_SIZE,
                );
            }
            Operation::ShaLoad(run) => {
                if status == Err(i2c::Error::DataNak) || status == Err(i2c::Error::AddressNak) {
                    // The device isn't ready yet, try again
                    if run == 50 {
                        self.op.set(Operation::Ready);
                        return;
                    }

                    self.op.set(Operation::ShaLoad(run + 1));
                    let _ = self.i2c.read(
                        buffer,
                        RESPONSE_COUNT_SIZE + RESPONSE_SIGNAL_SIZE + CRC_SIZE,
                    );
                    return;
                }

                self.buffer.replace(buffer);

                if self.sha_update() {
                    self.send_command(COMMAND_OPCODE_SHA, SHA_UPDATE, 64, 64);
                }
            }
            Operation::ShaLoadResponse(run) => {
                if status == Err(i2c::Error::DataNak) || status == Err(i2c::Error::AddressNak) {
                    // The device isn't ready yet, try again
                    if run == 10 {
                        self.op.set(Operation::Ready);
                        return;
                    }

                    self.op.set(Operation::ShaLoadResponse(run + 1));
                    self.send_command(COMMAND_OPCODE_SHA, SHA_UPDATE, 64, 64);

                    return;
                }

                self.op.set(Operation::ShaLoad(0));
                let _ = self.i2c.read(
                    buffer,
                    RESPONSE_COUNT_SIZE + RESPONSE_SIGNAL_SIZE + CRC_SIZE,
                );
            }
            Operation::ShaRun(run) => {
                if status == Err(i2c::Error::DataNak) || status == Err(i2c::Error::AddressNak) {
                    self.buffer.replace(buffer);

                    // The device isn't ready yet, try again
                    if run == 10 {
                        self.op.set(Operation::Ready);
                        return;
                    }

                    self.op.set(Operation::ShaRun(run + 1));
                    self.send_command(
                        COMMAND_OPCODE_SHA,
                        SHA_END,
                        self.remain_len.get() as u16,
                        self.remain_len.get(),
                    );
                    return;
                }

                self.op.set(Operation::ShaEnd(0));
                self.remain_len.set(0);

                let _ = self
                    .i2c
                    .read(buffer, RESPONSE_COUNT_SIZE + RESPONSE_SHA_SIZE + CRC_SIZE);
            }
            Operation::ShaEnd(run) => {
                if status == Err(i2c::Error::DataNak) || status == Err(i2c::Error::AddressNak) {
                    // The device isn't ready yet, try again
                    if run == 500 {
                        self.op.set(Operation::Ready);
                        return;
                    }

                    self.op.set(Operation::ShaEnd(run + 1));
                    let _ = self
                        .i2c
                        .read(buffer, RESPONSE_COUNT_SIZE + RESPONSE_SHA_SIZE + CRC_SIZE);
                    return;
                }

                self.digest_data.take().map(|digest_data| {
                    digest_data[0..32]
                        .copy_from_slice(&buffer[RESPONSE_COUNT_SIZE..(RESPONSE_COUNT_SIZE + 32)]);

                    self.buffer.replace(buffer);

                    self.op.set(Operation::Ready);

                    self.digest_client.map(|client| {
                        client.hash_done(Ok(()), digest_data);
                    })
                });
            }
            Operation::LoadTempKeyNonce(run) => {
                if status == Err(i2c::Error::DataNak) || status == Err(i2c::Error::AddressNak) {
                    self.buffer.replace(buffer);

                    // The device isn't ready yet, try again
                    if run == 10 {
                        self.op.set(Operation::Ready);
                        return;
                    }

                    self.op.set(Operation::LoadTempKeyNonce(run + 1));
                    self.send_command(COMMAND_OPCODE_NONCE, NONCE_MODE_PASSTHROUGH, 0x0000, 32);
                    return;
                }

                self.op.set(Operation::LoadTempKeyCheckNonce(0));

                let _ = self.i2c.read(
                    buffer,
                    RESPONSE_COUNT_SIZE + RESPONSE_SIGNAL_SIZE + CRC_SIZE,
                );
            }
            Operation::LoadTempKeyCheckNonce(run) => {
                if status == Err(i2c::Error::DataNak) || status == Err(i2c::Error::AddressNak) {
                    // The device isn't ready yet, try again
                    if run == 10 {
                        self.op.set(Operation::Ready);
                        return;
                    }

                    self.op.set(Operation::LoadTempKeyCheckNonce(run + 1));
                    let _ = self.i2c.read(
                        buffer,
                        RESPONSE_COUNT_SIZE + RESPONSE_SIGNAL_SIZE + CRC_SIZE,
                    );
                    return;
                }

                if buffer[RESPONSE_SIGNAL_INDEX] != ATRCC508A_SUCCESSFUL_TEMPKEY {
                    self.buffer.replace(buffer);

                    self.secure_client.map(|client| {
                        client.verification_done(
                            Err(ErrorCode::FAIL),
                            self.message_data.take().unwrap(),
                            self.signature_data.take().unwrap(),
                        );
                    });

                    return;
                }

                // Append Signature
                self.signature_data.map(|signature_data| {
                    buffer[ATRCC508A_PROTOCOL_FIELD_DATA..(ATRCC508A_PROTOCOL_FIELD_DATA + 64)]
                        .copy_from_slice(signature_data);
                });

                // Append Public Key
                self.ext_public_key.map(|ext_public_key| {
                    buffer[(ATRCC508A_PROTOCOL_FIELD_DATA + 64)
                        ..(ATRCC508A_PROTOCOL_FIELD_DATA + 128)]
                        .copy_from_slice(ext_public_key);
                });

                self.buffer.replace(buffer);
                self.op.set(Operation::VerifySubmitData(0));

                self.send_command(
                    COMMAND_OPCODE_VERIFY,
                    VERIFY_MODE_EXTERNAL,
                    VERIFY_PARAM2_KEYTYPE_ECC as u16,
                    128,
                );
            }
            Operation::VerifySubmitData(run) => {
                if status == Err(i2c::Error::DataNak) || status == Err(i2c::Error::AddressNak) {
                    self.buffer.replace(buffer);

                    // The device isn't ready yet, try again
                    if run == 10 {
                        self.op.set(Operation::Ready);
                        return;
                    }

                    self.op.set(Operation::VerifySubmitData(run + 1));
                    self.send_command(
                        COMMAND_OPCODE_VERIFY,
                        VERIFY_MODE_EXTERNAL,
                        VERIFY_PARAM2_KEYTYPE_ECC as u16,
                        128,
                    );
                    return;
                }

                self.op.set(Operation::CompleteVerify(0));
                let _ = self.i2c.read(
                    buffer,
                    RESPONSE_COUNT_SIZE + RESPONSE_SIGNAL_SIZE + CRC_SIZE,
                );
            }
            Operation::CompleteVerify(run) => {
                if status == Err(i2c::Error::DataNak) || status == Err(i2c::Error::AddressNak) {
                    // The device isn't ready yet, try again
                    if run == 100 {
                        self.op.set(Operation::Ready);
                        return;
                    }

                    self.op.set(Operation::CompleteVerify(run + 1));
                    let _ = self.i2c.read(
                        buffer,
                        RESPONSE_COUNT_SIZE + RESPONSE_SIGNAL_SIZE + CRC_SIZE,
                    );
                    return;
                }

                let ret = buffer[RESPONSE_SIGNAL_INDEX];

                self.op.set(Operation::Ready);
                self.buffer.replace(buffer);

                self.secure_client.map(|client| {
                    if ret == ATRCC508A_SUCCESSFUL_VERIFY {
                        client.verification_done(
                            Ok(true),
                            self.message_data.take().unwrap(),
                            self.signature_data.take().unwrap(),
                        );
                    } else if ret == 1 {
                        client.verification_done(
                            Ok(false),
                            self.message_data.take().unwrap(),
                            self.signature_data.take().unwrap(),
                        );
                    } else {
                        client.verification_done(
                            Err(ErrorCode::FAIL),
                            self.message_data.take().unwrap(),
                            self.signature_data.take().unwrap(),
                        );
                    }
                });
            }
        }
    }
}

struct Atecc508aRngIter<'a, 'b: 'a>(&'a Atecc508a<'b>);

impl Iterator for Atecc508aRngIter<'_, '_> {
    type Item = u32;

    fn next(&mut self) -> Option<u32> {
        self.0.entropy_buffer.take().map(|entropy_buffer| {
            let offset = self.0.entropy_offset.get();
            let entropy_bytes =
                <[u8; 4]>::try_from(&entropy_buffer[(offset + 0)..(offset + 4)]).unwrap();
            let entropy: u32 = u32::from_be_bytes(entropy_bytes);

            if offset >= 28 {
                self.0.entropy_offset.set(0);
            } else {
                self.0.entropy_offset.set(offset + 4);
            }
            self.0.entropy_buffer.replace(entropy_buffer);

            entropy
        })
    }
}

impl<'a> entropy::Entropy32<'a> for Atecc508a<'a> {
    fn set_client(&'a self, client: &'a dyn entropy::Client32) {
        self.entropy_client.set(client);
    }

    fn get(&self) -> Result<(), ErrorCode> {
        self.op.set(Operation::GenerateEntropyCommand(0));

        (self.wakeup_device)();

        self.send_command(COMMAND_OPCODE_RANDOM, 0x00, 0x0000, 0);

        Ok(())
    }

    fn cancel(&self) -> Result<(), ErrorCode> {
        Ok(())
    }
}

impl<'a> digest::DigestData<'a, 32> for Atecc508a<'a> {
    fn set_data_client(&'a self, _client: &'a dyn digest::ClientData<32>) {
        unimplemented!()
    }

    fn add_data(
        &self,
        data: SubSlice<'static, u8>,
    ) -> Result<(), (ErrorCode, SubSlice<'static, u8>)> {
        if !(self.op.get() == Operation::Ready || self.op.get() == Operation::ReadySha) {
            return Err((ErrorCode::BUSY, data));
        }

        (self.wakeup_device)();

        self.write_len.set(data.len());
        self.hash_data.replace(SubSliceMutImmut::Immutable(data));

        if self.op.get() == Operation::Ready {
            self.op.set(Operation::StartSha(0));

            self.send_command(COMMAND_OPCODE_SHA, SHA_START, 0x0000, 0);
        } else {
            self.op.set(Operation::ShaLoad(0));

            if self.sha_update() {
                self.send_command(COMMAND_OPCODE_SHA, SHA_UPDATE, 64, 64);
            }
        }

        Ok(())
    }

    fn add_mut_data(
        &self,
        data: SubSliceMut<'static, u8>,
    ) -> Result<(), (ErrorCode, SubSliceMut<'static, u8>)> {
        if !(self.op.get() == Operation::Ready || self.op.get() == Operation::ReadySha) {
            return Err((ErrorCode::BUSY, data));
        }

        self.write_len.set(data.len());
        self.hash_data.replace(SubSliceMutImmut::Mutable(data));

        if self.op.get() == Operation::Ready {
            self.op.set(Operation::StartSha(0));

            (self.wakeup_device)();

            self.send_command(COMMAND_OPCODE_SHA, SHA_START, 0x0000, 0);
        } else {
            self.op.set(Operation::ShaLoad(0));

            if self.sha_update() {
                (self.wakeup_device)();

                self.send_command(COMMAND_OPCODE_SHA, SHA_UPDATE, 64, 64);
            }
        }

        Ok(())
    }

    /// This will reset the device to clear the data
    ///
    /// This is an async operation though, as it requires the I2C operation
    /// to complete and the I2C callback to occur, but the `clear_data()`
    /// definition is syncronous, so this can race.
    fn clear_data(&self) {
        (self.wakeup_device)();

        self.reset();
    }
}

impl<'a> digest::DigestHash<'a, 32> for Atecc508a<'a> {
    fn set_hash_client(&'a self, _client: &'a dyn digest::ClientHash<32>) {
        unimplemented!()
    }

    fn run(
        &'a self,
        digest: &'static mut [u8; 32],
    ) -> Result<(), (ErrorCode, &'static mut [u8; 32])> {
        let remain_len = self.remain_len.get();

        if self.op.get() != Operation::ReadySha {
            return Err((ErrorCode::BUSY, digest));
        }

        (self.wakeup_device)();

        self.op.set(Operation::ShaRun(0));
        self.digest_data.replace(digest);

        if remain_len > 0 {
            self.buffer.take().map(|buffer| {
                self.digest_buffer.map(|digest_buffer| {
                    buffer[ATRCC508A_PROTOCOL_FIELD_DATA
                        ..(ATRCC508A_PROTOCOL_FIELD_DATA + remain_len)]
                        .copy_from_slice(&digest_buffer[0..remain_len]);
                });

                self.buffer.replace(buffer);
            });
        }

        self.send_command(
            COMMAND_OPCODE_SHA,
            SHA_END,
            self.remain_len.get() as u16,
            remain_len,
        );

        Ok(())
    }
}

impl<'a> digest::DigestDataHash<'a, 32> for Atecc508a<'a> {
    fn set_client(&'a self, client: &'a dyn digest::ClientDataHash<32>) {
        self.digest_client.set(client);
    }
}

impl<'a> SignatureVerify<'a, 32, 64> for Atecc508a<'a> {
    fn set_verify_client(&self, client: &'a dyn ClientVerify<32, 64>) {
        self.secure_client.set(client);
    }

    /// Check the signature against the external public key loaded via
    /// `set_public_key()`.
    ///
    /// Verifying that a message was signed by the device is not support
    /// yet.
    fn verify(
        &self,
        hash: &'static mut [u8; 32],
        signature: &'static mut [u8; 64],
    ) -> Result<(), (ErrorCode, &'static mut [u8; 32], &'static mut [u8; 64])> {
        if self.ext_public_key.is_none() {
            return Err((ErrorCode::OFF, hash, signature));
        }

        (self.wakeup_device)();

        self.op.set(Operation::LoadTempKeyNonce(0));

        self.buffer.map(|buffer| {
            buffer[ATRCC508A_PROTOCOL_FIELD_DATA..(ATRCC508A_PROTOCOL_FIELD_DATA + 32)]
                .copy_from_slice(hash);
        });

        self.message_data.replace(hash);
        self.signature_data.replace(signature);

        self.send_command(COMMAND_OPCODE_NONCE, NONCE_MODE_PASSTHROUGH, 0x0000, 32);

        Ok(())
    }
}

impl<'a> keys::SetKeyBySlice<'a, 64> for Atecc508a<'a> {
    fn set_key(
        &self,
        key: &'static mut [u8; 64],
    ) -> Result<(), (ErrorCode, &'static mut [u8; 64])> {
        // Copy the key into our public key buffer.
        self.ext_public_key.map(|epkey| {
            epkey.copy_from_slice(key);
        });

        // Save the key so we can return the buffer in `set_key_done()`.
        self.ext_public_key_temp.replace(key);
        self.deferred_call.set();
        Ok(())
    }

    fn set_client(&self, client: &'a dyn keys::SetKeyBySliceClient<64>) {
        self.key_set_client.replace(client);
    }
}

impl kernel::deferred_call::DeferredCallClient for Atecc508a<'_> {
    fn handle_deferred_call(&self) {
        self.key_set_client.map(|client| {
            self.ext_public_key_temp
                .take()
                .map(|key| client.set_key_done(key, Ok(())))
        });
    }

    fn register(&'static self) {
        self.deferred_call.register(self);
    }
}
