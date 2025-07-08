// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Types and Data Structures for TBFs.

use core::fmt;
use core::mem::size_of;

/// We only support up to a fixed number of storage permissions for each of read
/// and modify. This simplification enables us to use fixed sized buffers.
const NUM_STORAGE_PERMISSIONS: usize = 8;

/// Error when parsing just the beginning of the TBF header. This is only used
/// when establishing the linked list structure of apps installed in flash.
pub enum InitialTbfParseError {
    /// We were unable to parse the beginning of the header. This either means
    /// we ran out of flash, or the trusted values are invalid meaning this is
    /// just empty flash after the end of the last app. This error is fine, as
    /// it just means we must have hit the end of the linked list of apps.
    UnableToParse,

    /// Some length or value in the header is invalid. The header parsing has
    /// failed at this point. However, the total app length value is a trusted
    /// field, so we return that value with this error so that we can skip over
    /// this invalid app and continue to check for additional apps.
    InvalidHeader(u32),
}

impl From<core::array::TryFromSliceError> for InitialTbfParseError {
    // Convert a slice to a parsed type. Since we control how long we make our
    // slices, this conversion should never fail. If it does, then this is a bug
    // in this library that must be fixed.
    fn from(_error: core::array::TryFromSliceError) -> Self {
        InitialTbfParseError::UnableToParse
    }
}

/// Error when parsing an app's TBF header.
pub enum TbfParseError {
    /// Not enough bytes in the buffer to parse the expected field.
    NotEnoughFlash,

    /// Unknown version of the TBF header.
    UnsupportedVersion(u16),

    /// Checksum calculation did not match what is stored in the TBF header.
    /// First value is the checksum provided, second value is the checksum we
    /// calculated.
    ChecksumMismatch(u32, u32),

    /// One of the TLV entries did not parse correctly. This could happen if the
    /// TLV.length does not match the size of a fixed-length entry. The `usize`
    /// is the value of the "tipe" field.
    BadTlvEntry(usize),

    /// The app name in the TBF header could not be successfully parsed as a
    /// UTF-8 string.
    BadProcessName,

    /// Internal kernel error. This is a bug inside of this library. Likely this
    /// means that for some reason a slice was not sized properly for parsing a
    /// certain type, which is something completely controlled by this library.
    /// If the slice passed in is not long enough, then a `get()` call will
    /// fail and that will trigger a different error.
    InternalError,

    /// The number of variable length entries (for example the number of
    /// `TbfHeaderDriverPermission` entries in `TbfHeaderV2Permissions`) is
    /// too long for Tock to parse.
    /// This can be fixed by increasing the number in `TbfHeaderV2`.
    TooManyEntries(usize),
}

impl From<core::array::TryFromSliceError> for TbfParseError {
    // Convert a slice to a parsed type. Since we control how long we make our
    // slices, this conversion should never fail. If it does, then this is a bug
    // in this library that must be fixed.
    fn from(_error: core::array::TryFromSliceError) -> Self {
        TbfParseError::InternalError
    }
}

impl fmt::Debug for TbfParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TbfParseError::NotEnoughFlash => write!(f, "Buffer too short to parse TBF header"),
            TbfParseError::UnsupportedVersion(version) => {
                write!(f, "TBF version {} unsupported", version)
            }
            TbfParseError::ChecksumMismatch(app, calc) => write!(
                f,
                "Checksum verification failed: app:{:#x}, calc:{:#x}",
                app, calc
            ),
            TbfParseError::BadTlvEntry(tipe) => write!(f, "TLV entry type {} is invalid", tipe),
            TbfParseError::BadProcessName => write!(f, "Process name not UTF-8"),
            TbfParseError::InternalError => write!(f, "Internal kernel error. This is a bug."),
            TbfParseError::TooManyEntries(tipe) => {
                write!(
                    f,
                    "There are too many variable entries of {} for Tock to parse",
                    tipe
                )
            }
        }
    }
}

// TBF structure

/// TBF fields that must be present in all v2 headers.
#[derive(Clone, Copy, Debug)]
pub struct TbfHeaderV2Base {
    pub(crate) version: u16,
    pub(crate) header_size: u16,
    pub(crate) total_size: u32,
    pub(crate) flags: u32,
    pub(crate) checksum: u32,
}

/// Types in TLV structures for each optional block of the header.
#[derive(Clone, Copy, Debug)]
pub enum TbfHeaderTypes {
    TbfHeaderMain = 1,
    TbfHeaderWriteableFlashRegions = 2,
    TbfHeaderPackageName = 3,
    TbfHeaderFixedAddresses = 5,
    TbfHeaderPermissions = 6,
    TbfHeaderStoragePermissions = 7,
    TbfHeaderKernelVersion = 8,
    TbfHeaderProgram = 9,
    TbfHeaderShortId = 10,
    TbfFooterCredentials = 128,

    /// Some field in the header that we do not understand. Since the TLV format
    /// specifies the length of each section, if we get a field we do not
    /// understand we just skip it, rather than throwing an error.
    Unknown,
}

/// The TLV header (T and L).
#[derive(Clone, Copy, Debug)]
pub struct TbfTlv {
    pub(crate) tipe: TbfHeaderTypes,
    pub(crate) length: u16,
}

/// The v2 Main Header for apps.
///
/// All apps must have either a Main Header or a Program Header. Without
/// either, the TBF object is considered padding. Main and Program Headers
/// differ in whether they specify the endpoint of the process binary; Main
/// Headers do not, while Program Headers do. A TBF with a Main Header cannot
/// have any Credentials Footers, while a TBF with a Program Header can.
#[derive(Clone, Copy, Debug)]
pub struct TbfHeaderV2Main {
    init_fn_offset: u32,
    protected_trailer_size: u32,
    minimum_ram_size: u32,
}

/// The v2 Program Header for apps.
///
/// All apps must have either a Main Header or a Program Header. Without
/// either, the TBF object is considered padding. Main and Program Headers
/// differ in whether they specify the endpoint of the process binary; Main
/// Headers do not, while Program Headers do. A Program Header includes
/// the binary end offset so that a Verifier knows where Credentials Headers
/// start. The region between the end of the binary and the end of the TBF
/// is reserved for Credentials Footers.
#[derive(Clone, Copy, Debug)]
pub struct TbfHeaderV2Program {
    init_fn_offset: u32,
    protected_trailer_size: u32,
    minimum_ram_size: u32,
    binary_end_offset: u32,
    version: u32,
}

/// Writeable flash regions only need an offset and size.
///
/// There can be multiple (or zero) flash regions defined, so this is its own
/// struct.
#[derive(Clone, Copy, Debug, Default)]
pub struct TbfHeaderV2WriteableFlashRegion {
    writeable_flash_region_offset: u32,
    writeable_flash_region_size: u32,
}

/// Optional fixed addresses for flash and RAM for this process.
///
/// If a process is compiled for a specific address this header entry lets the
/// kernel know what those addresses are.
///
/// If this header is omitted the kernel will assume that the process is
/// position-independent and can be loaded at any (reasonably aligned) flash
/// address and can be given any (reasonable aligned) memory segment.
///
/// If this header is included, the kernel will check these values when setting
/// up the process. If a process wants to set one fixed address but not the other, the unused one
/// can be set to 0xFFFFFFFF.
#[derive(Clone, Copy, Debug, Default)]
pub struct TbfHeaderV2FixedAddresses {
    /// The absolute address of the start of RAM that the process expects. For
    /// example, if the process was linked with a RAM region starting at
    /// address `0x00023000`, then this would be set to `0x00023000`.
    start_process_ram: u32,
    /// The absolute address of the start of the process binary. This does _not_
    /// include the TBF header. This is the address the process used for the
    /// start of flash with the linker.
    start_process_flash: u32,
}

#[derive(Clone, Copy, Debug, Default)]
struct TbfHeaderDriverPermission {
    driver_number: u32,
    offset: u32,
    allowed_commands: u64,
}

/// A list of permissions for this app
#[derive(Clone, Copy, Debug)]
pub struct TbfHeaderV2Permissions<const L: usize> {
    length: u16,
    perms: [TbfHeaderDriverPermission; L],
}

/// A list of storage (read/write/modify) permissions for this app.
#[derive(Clone, Copy, Debug)]
pub struct TbfHeaderV2StoragePermissions<const L: usize> {
    write_id: Option<core::num::NonZeroU32>,
    read_length: u16,
    read_ids: [u32; L],
    modify_length: u16,
    modify_ids: [u32; L],
}

#[derive(Clone, Copy, Debug)]
pub struct TbfHeaderV2KernelVersion {
    major: u16,
    minor: u16,
}

/// The v2 ShortId for apps.
///
/// Header to specify a fixed ShortID for an app.
#[derive(Clone, Copy, Debug)]
pub struct TbfHeaderV2ShortId {
    short_id: Option<core::num::NonZeroU32>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TbfFooterV2CredentialsType {
    Reserved = 0,
    Rsa3072Key = 1,
    Rsa4096Key = 2,
    SHA256 = 3,
    SHA384 = 4,
    SHA512 = 5,
    EcdsaNistP256 = 6,
}

#[derive(Clone, Copy, Debug)]
pub struct TbfFooterV2Credentials {
    format: TbfFooterV2CredentialsType,
    data: &'static [u8],
}

impl TbfFooterV2Credentials {
    pub fn format(&self) -> TbfFooterV2CredentialsType {
        self.format
    }

    pub fn data(&self) -> &'static [u8] {
        self.data
    }
}

// Conversion functions from slices to the various TBF fields.

impl core::convert::TryFrom<&[u8]> for TbfHeaderV2Base {
    type Error = TbfParseError;

    fn try_from(b: &[u8]) -> Result<TbfHeaderV2Base, Self::Error> {
        if b.len() < 16 {
            return Err(TbfParseError::InternalError);
        }
        Ok(TbfHeaderV2Base {
            version: u16::from_le_bytes(
                b.get(0..2)
                    .ok_or(TbfParseError::InternalError)?
                    .try_into()?,
            ),
            header_size: u16::from_le_bytes(
                b.get(2..4)
                    .ok_or(TbfParseError::InternalError)?
                    .try_into()?,
            ),
            total_size: u32::from_le_bytes(
                b.get(4..8)
                    .ok_or(TbfParseError::InternalError)?
                    .try_into()?,
            ),
            flags: u32::from_le_bytes(
                b.get(8..12)
                    .ok_or(TbfParseError::InternalError)?
                    .try_into()?,
            ),
            checksum: u32::from_le_bytes(
                b.get(12..16)
                    .ok_or(TbfParseError::InternalError)?
                    .try_into()?,
            ),
        })
    }
}

impl core::convert::TryFrom<u16> for TbfHeaderTypes {
    type Error = TbfParseError;

    fn try_from(h: u16) -> Result<TbfHeaderTypes, Self::Error> {
        match h {
            1 => Ok(TbfHeaderTypes::TbfHeaderMain),
            2 => Ok(TbfHeaderTypes::TbfHeaderWriteableFlashRegions),
            3 => Ok(TbfHeaderTypes::TbfHeaderPackageName),
            5 => Ok(TbfHeaderTypes::TbfHeaderFixedAddresses),
            6 => Ok(TbfHeaderTypes::TbfHeaderPermissions),
            7 => Ok(TbfHeaderTypes::TbfHeaderStoragePermissions),
            8 => Ok(TbfHeaderTypes::TbfHeaderKernelVersion),
            9 => Ok(TbfHeaderTypes::TbfHeaderProgram),
            10 => Ok(TbfHeaderTypes::TbfHeaderShortId),
            128 => Ok(TbfHeaderTypes::TbfFooterCredentials),
            _ => Ok(TbfHeaderTypes::Unknown),
        }
    }
}

impl core::convert::TryFrom<&[u8]> for TbfTlv {
    type Error = TbfParseError;

    fn try_from(b: &[u8]) -> Result<TbfTlv, Self::Error> {
        Ok(TbfTlv {
            tipe: u16::from_le_bytes(
                b.get(0..2)
                    .ok_or(TbfParseError::InternalError)?
                    .try_into()?,
            )
            .try_into()?,
            length: u16::from_le_bytes(
                b.get(2..4)
                    .ok_or(TbfParseError::InternalError)?
                    .try_into()?,
            ),
        })
    }
}

impl core::convert::TryFrom<&[u8]> for TbfHeaderV2Main {
    type Error = TbfParseError;

    fn try_from(b: &[u8]) -> Result<TbfHeaderV2Main, Self::Error> {
        // For 3 or more fields, this shortcut check reduces code size
        if b.len() < 12 {
            return Err(TbfParseError::InternalError);
        }
        Ok(TbfHeaderV2Main {
            init_fn_offset: u32::from_le_bytes(
                b.get(0..4)
                    .ok_or(TbfParseError::InternalError)?
                    .try_into()?,
            ),
            protected_trailer_size: u32::from_le_bytes(
                b.get(4..8)
                    .ok_or(TbfParseError::InternalError)?
                    .try_into()?,
            ),
            minimum_ram_size: u32::from_le_bytes(
                b.get(8..12)
                    .ok_or(TbfParseError::InternalError)?
                    .try_into()?,
            ),
        })
    }
}

impl core::convert::TryFrom<&[u8]> for TbfHeaderV2Program {
    type Error = TbfParseError;
    fn try_from(b: &[u8]) -> Result<TbfHeaderV2Program, Self::Error> {
        // For 3 or more fields, this shortcut check reduces code size
        if b.len() < 20 {
            return Err(TbfParseError::InternalError);
        }
        Ok(TbfHeaderV2Program {
            init_fn_offset: u32::from_le_bytes(
                b.get(0..4)
                    .ok_or(TbfParseError::InternalError)?
                    .try_into()?,
            ),
            protected_trailer_size: u32::from_le_bytes(
                b.get(4..8)
                    .ok_or(TbfParseError::InternalError)?
                    .try_into()?,
            ),
            minimum_ram_size: u32::from_le_bytes(
                b.get(8..12)
                    .ok_or(TbfParseError::InternalError)?
                    .try_into()?,
            ),
            binary_end_offset: u32::from_le_bytes(
                b.get(12..16)
                    .ok_or(TbfParseError::InternalError)?
                    .try_into()?,
            ),
            version: u32::from_le_bytes(
                b.get(16..20)
                    .ok_or(TbfParseError::InternalError)?
                    .try_into()?,
            ),
        })
    }
}

impl core::convert::TryFrom<&[u8]> for TbfHeaderV2WriteableFlashRegion {
    type Error = TbfParseError;

    fn try_from(b: &[u8]) -> Result<TbfHeaderV2WriteableFlashRegion, Self::Error> {
        Ok(TbfHeaderV2WriteableFlashRegion {
            writeable_flash_region_offset: u32::from_le_bytes(
                b.get(0..4)
                    .ok_or(TbfParseError::InternalError)?
                    .try_into()?,
            ),
            writeable_flash_region_size: u32::from_le_bytes(
                b.get(4..8)
                    .ok_or(TbfParseError::InternalError)?
                    .try_into()?,
            ),
        })
    }
}

impl core::convert::TryFrom<&[u8]> for TbfHeaderV2FixedAddresses {
    type Error = TbfParseError;

    fn try_from(b: &[u8]) -> Result<TbfHeaderV2FixedAddresses, Self::Error> {
        Ok(TbfHeaderV2FixedAddresses {
            start_process_ram: u32::from_le_bytes(
                b.get(0..4)
                    .ok_or(TbfParseError::InternalError)?
                    .try_into()?,
            ),
            start_process_flash: u32::from_le_bytes(
                b.get(4..8)
                    .ok_or(TbfParseError::InternalError)?
                    .try_into()?,
            ),
        })
    }
}

impl core::convert::TryFrom<&[u8]> for TbfHeaderDriverPermission {
    type Error = TbfParseError;

    fn try_from(b: &[u8]) -> Result<TbfHeaderDriverPermission, Self::Error> {
        // For 3 or more fields, this shortcut check reduces code size
        if b.len() < 16 {
            return Err(TbfParseError::InternalError);
        }
        Ok(TbfHeaderDriverPermission {
            driver_number: u32::from_le_bytes(
                b.get(0..4)
                    .ok_or(TbfParseError::InternalError)?
                    .try_into()?,
            ),
            offset: u32::from_le_bytes(
                b.get(4..8)
                    .ok_or(TbfParseError::InternalError)?
                    .try_into()?,
            ),
            allowed_commands: u64::from_le_bytes(
                b.get(8..16)
                    .ok_or(TbfParseError::InternalError)?
                    .try_into()?,
            ),
        })
    }
}

impl<const L: usize> core::convert::TryFrom<&[u8]> for TbfHeaderV2StoragePermissions<L> {
    type Error = TbfParseError;

    fn try_from(b: &[u8]) -> Result<TbfHeaderV2StoragePermissions<L>, Self::Error> {
        let mut read_end = 6;

        let write_id = core::num::NonZeroU32::new(u32::from_le_bytes(
            b.get(0..4)
                .ok_or(TbfParseError::NotEnoughFlash)?
                .try_into()?,
        ));

        let read_length = u16::from_le_bytes(
            b.get(4..6)
                .ok_or(TbfParseError::NotEnoughFlash)?
                .try_into()?,
        );

        let mut read_ids: [u32; L] = [0; L];
        for i in 0..read_length as usize {
            let start = 6 + (i * size_of::<u32>());
            read_end = start + size_of::<u32>();
            if let Some(read_id) = read_ids.get_mut(i) {
                *read_id = u32::from_le_bytes(
                    b.get(start..read_end)
                        .ok_or(TbfParseError::NotEnoughFlash)?
                        .try_into()?,
                );
            } else {
                return Err(TbfParseError::BadTlvEntry(
                    TbfHeaderTypes::TbfHeaderStoragePermissions as usize,
                ));
            }
        }

        let modify_length = u16::from_le_bytes(
            b.get(read_end..(read_end + 2))
                .ok_or(TbfParseError::NotEnoughFlash)?
                .try_into()?,
        );

        let mut modify_ids: [u32; L] = [0; L];
        for i in 0..modify_length as usize {
            let start = read_end + 2 + (i * size_of::<u32>());
            let modify_end = start + size_of::<u32>();
            if let Some(modify_id) = modify_ids.get_mut(i) {
                *modify_id = u32::from_le_bytes(
                    b.get(start..modify_end)
                        .ok_or(TbfParseError::NotEnoughFlash)?
                        .try_into()?,
                );
            } else {
                return Err(TbfParseError::BadTlvEntry(
                    TbfHeaderTypes::TbfHeaderStoragePermissions as usize,
                ));
            }
        }

        Ok(TbfHeaderV2StoragePermissions {
            write_id,
            read_length,
            read_ids,
            modify_length,
            modify_ids,
        })
    }
}

impl core::convert::TryFrom<&[u8]> for TbfHeaderV2KernelVersion {
    type Error = TbfParseError;

    fn try_from(b: &[u8]) -> Result<TbfHeaderV2KernelVersion, Self::Error> {
        Ok(TbfHeaderV2KernelVersion {
            major: u16::from_le_bytes(
                b.get(0..2)
                    .ok_or(TbfParseError::InternalError)?
                    .try_into()?,
            ),
            minor: u16::from_le_bytes(
                b.get(2..4)
                    .ok_or(TbfParseError::InternalError)?
                    .try_into()?,
            ),
        })
    }
}

impl core::convert::TryFrom<&[u8]> for TbfHeaderV2ShortId {
    type Error = TbfParseError;

    fn try_from(b: &[u8]) -> Result<TbfHeaderV2ShortId, Self::Error> {
        Ok(TbfHeaderV2ShortId {
            short_id: core::num::NonZeroU32::new(u32::from_le_bytes(
                b.get(0..4)
                    .ok_or(TbfParseError::InternalError)?
                    .try_into()?,
            )),
        })
    }
}

impl core::convert::TryFrom<&'static [u8]> for TbfFooterV2Credentials {
    type Error = TbfParseError;

    fn try_from(b: &'static [u8]) -> Result<TbfFooterV2Credentials, Self::Error> {
        let format = u32::from_le_bytes(
            b.get(0..4)
                .ok_or(TbfParseError::InternalError)?
                .try_into()?,
        );
        let ftype = match format {
            0 => TbfFooterV2CredentialsType::Reserved,
            1 => TbfFooterV2CredentialsType::Rsa3072Key,
            2 => TbfFooterV2CredentialsType::Rsa4096Key,
            3 => TbfFooterV2CredentialsType::SHA256,
            4 => TbfFooterV2CredentialsType::SHA384,
            5 => TbfFooterV2CredentialsType::SHA512,
            6 => TbfFooterV2CredentialsType::EcdsaNistP256,
            _ => {
                return Err(TbfParseError::BadTlvEntry(
                    TbfHeaderTypes::TbfFooterCredentials as usize,
                ));
            }
        };
        let length = match ftype {
            TbfFooterV2CredentialsType::Reserved => 0,
            TbfFooterV2CredentialsType::Rsa3072Key => 768,
            TbfFooterV2CredentialsType::Rsa4096Key => 1024,
            TbfFooterV2CredentialsType::SHA256 => 32,
            TbfFooterV2CredentialsType::SHA384 => 48,
            TbfFooterV2CredentialsType::SHA512 => 64,
            TbfFooterV2CredentialsType::EcdsaNistP256 => 64,
        };
        let data = &b
            .get(4..(length + 4))
            .ok_or(TbfParseError::NotEnoughFlash)?;
        Ok(TbfFooterV2Credentials {
            format: ftype,
            data,
        })
    }
}

/// The command permissions specified by the TBF header.
///
/// Use the `get_command_permissions()` function to retrieve these.
pub enum CommandPermissions {
    /// The TBF header did not specify any permissions for any driver numbers.
    NoPermsAtAll,
    /// The TBF header did specify permissions for at least one driver number,
    /// but not for the requested driver number.
    NoPermsThisDriver,
    /// The bitmask of allowed command numbers starting from the offset provided
    /// when this enum was created.
    Mask(u64),
}

/// Single header that can contain all parts of a v2 header.
///
/// Note, this struct limits the number of writeable regions an app can have to
/// four since we need to statically know the length of the array to store in
/// this type.
#[derive(Clone, Copy, Debug)]
pub struct TbfHeaderV2<'a> {
    pub(crate) base: TbfHeaderV2Base,
    pub(crate) main: Option<TbfHeaderV2Main>,
    pub(crate) program: Option<TbfHeaderV2Program>,
    pub(crate) package_name: Option<&'a str>,
    pub(crate) writeable_regions: Option<&'a [u8]>,
    pub(crate) fixed_addresses: Option<&'a [u8]>,
    pub(crate) permissions: Option<&'a [u8]>,
    pub(crate) storage_permissions: Option<&'a [u8]>,
    pub(crate) kernel_version: Option<TbfHeaderV2KernelVersion>,
    pub(crate) short_id: Option<TbfHeaderV2ShortId>,
}

/// Type that represents the fields of the Tock Binary Format header.
///
/// This specifies the locations of the different code and memory sections
/// in the tock binary, as well as other information about the application.
/// The kernel can also use this header to keep persistent state about
/// the application.
#[derive(Debug)]
pub enum TbfHeader<'a> {
    TbfHeaderV2(TbfHeaderV2<'a>),
    Padding(TbfHeaderV2Base),
}

impl<'a> TbfHeader<'a> {
    /// Return the length of the header.
    pub fn length(&self) -> u16 {
        match *self {
            TbfHeader::TbfHeaderV2(hd) => hd.base.header_size,
            TbfHeader::Padding(base) => base.header_size,
        }
    }

    /// Return whether this is an app or just padding between apps.
    pub fn is_app(&self) -> bool {
        match *self {
            TbfHeader::TbfHeaderV2(_) => true,
            TbfHeader::Padding(_) => false,
        }
    }

    /// Return whether the application is enabled or not.
    /// Disabled applications are not started by the kernel.
    pub fn enabled(&self) -> bool {
        match *self {
            TbfHeader::TbfHeaderV2(hd) => {
                // Bit 1 of flags is the enable/disable bit.
                hd.base.flags & 0x00000001 == 1
            }
            TbfHeader::Padding(_) => false,
        }
    }

    /// Add up all of the relevant fields in header version 1, or just used the
    /// app provided value in version 2 to get the total amount of RAM that is
    /// needed for this app.
    pub fn get_minimum_app_ram_size(&self) -> u32 {
        match *self {
            TbfHeader::TbfHeaderV2(hd) => {
                if hd.program.is_some() {
                    hd.program.map_or(0, |p| p.minimum_ram_size)
                } else if hd.main.is_some() {
                    hd.main.map_or(0, |m| m.minimum_ram_size)
                } else {
                    0
                }
            }
            _ => 0,
        }
    }

    /// Get the number of bytes from the start of the app's region in flash that
    /// is for kernel use only. The app cannot write this region.
    pub fn get_protected_size(&self) -> u32 {
        match *self {
            TbfHeader::TbfHeaderV2(hd) => {
                if hd.program.is_some() {
                    hd.program.map_or(0, |p| {
                        (hd.base.header_size as u32) + p.protected_trailer_size
                    })
                } else if hd.main.is_some() {
                    hd.main.map_or(0, |m| {
                        (hd.base.header_size as u32) + m.protected_trailer_size
                    })
                } else {
                    0
                }
            }
            _ => 0,
        }
    }

    /// Get the start offset of the application binary from the beginning
    /// of the process binary (start of the TBF header). Only valid if this
    /// is an app.
    pub fn get_app_start_offset(&self) -> u32 {
        // The application binary starts after the header plus any
        // additional protected space.
        self.get_protected_size()
    }

    /// Get the offset from the beginning of the app's flash region where the
    /// app should start executing.
    pub fn get_init_function_offset(&self) -> u32 {
        match *self {
            TbfHeader::TbfHeaderV2(hd) => {
                if hd.program.is_some() {
                    hd.program
                        .map_or(0, |p| p.init_fn_offset + (hd.base.header_size as u32))
                } else if hd.main.is_some() {
                    hd.main
                        .map_or(0, |m| m.init_fn_offset + (hd.base.header_size as u32))
                } else {
                    0
                }
            }
            _ => 0,
        }
    }

    /// Get the name of the app.
    pub fn get_package_name(&self) -> Option<&'a str> {
        match *self {
            TbfHeader::TbfHeaderV2(hd) => hd.package_name,
            _ => None,
        }
    }

    /// Get the number of flash regions this app has specified in its header.
    pub fn number_writeable_flash_regions(&self) -> usize {
        match *self {
            TbfHeader::TbfHeaderV2(hd) => hd.writeable_regions.map_or(0, |wr_slice| {
                let wfr_len = size_of::<TbfHeaderV2WriteableFlashRegion>();
                wr_slice.len() / wfr_len
            }),
            _ => 0,
        }
    }

    /// Get the offset and size of a given flash region.
    pub fn get_writeable_flash_region(&self, index: usize) -> (usize, usize) {
        match *self {
            TbfHeader::TbfHeaderV2(hd) => hd.writeable_regions.map_or((0, 0), |wr_slice| {
                fn get_region(
                    wr_slice: &[u8],
                    index: usize,
                ) -> Result<TbfHeaderV2WriteableFlashRegion, ()> {
                    let wfr_len = size_of::<TbfHeaderV2WriteableFlashRegion>();

                    let wfr = wr_slice
                        .get(index * wfr_len..(index + 1) * wfr_len)
                        .ok_or(())?
                        .try_into()
                        .or(Err(()))?;
                    Ok(wfr)
                }

                match get_region(wr_slice, index) {
                    Ok(wr) => (
                        wr.writeable_flash_region_offset as usize,
                        wr.writeable_flash_region_size as usize,
                    ),
                    Err(()) => (0, 0),
                }
            }),
            _ => (0, 0),
        }
    }

    /// Get the address in RAM this process was specifically compiled for. If
    /// the process is position independent, return `None`.
    pub fn get_fixed_address_ram(&self) -> Option<u32> {
        let hd = match self {
            TbfHeader::TbfHeaderV2(hd) => hd,
            _ => return None,
        };
        let fixed_addresses: TbfHeaderV2FixedAddresses = hd.fixed_addresses?.try_into().ok()?;
        match fixed_addresses.start_process_ram {
            0xFFFFFFFF => None,
            start => Some(start),
        }
    }

    /// Get the address in flash this process was specifically compiled for. If
    /// the process is position independent, return `None`.
    pub fn get_fixed_address_flash(&self) -> Option<u32> {
        let hd = match self {
            TbfHeader::TbfHeaderV2(hd) => hd,
            _ => return None,
        };
        let fixed_addresses: TbfHeaderV2FixedAddresses = hd.fixed_addresses?.try_into().ok()?;
        match fixed_addresses.start_process_flash {
            0xFFFFFFFF => None,
            start => Some(start),
        }
    }

    /// Get the permissions for a specified driver and offset.
    ///
    /// - `driver_num`: The driver to lookup.
    /// - `offset`: The offset for the driver to find. An offset value of 1 will
    ///   find a header with offset 1, so the `allowed_commands` will cover
    ///   command numbers 64 to 127.
    ///
    /// If permissions are found for the driver number, this function will
    /// return `CommandPermissions::Mask`. If there are permissions in the
    /// header but not for this driver the function will return
    /// `CommandPermissions::NoPermsThisDriver`. If the process does not have
    /// any permissions specified, return `CommandPermissions::NoPermsAtAll`.
    pub fn get_command_permissions(&self, driver_num: usize, offset: usize) -> CommandPermissions {
        match self {
            TbfHeader::TbfHeaderV2(hd) => match hd.permissions {
                Some(permissions_tlv_slice) => {
                    // Helper function to wrap the return in a Result.
                    fn get_command_permissions_result(
                        permissions_tlv_slice: &[u8],
                        driver_num: usize,
                        offset: usize,
                    ) -> Result<CommandPermissions, ()> {
                        let mut found_driver_num: bool = false;
                        let perm_len = size_of::<TbfHeaderDriverPermission>();

                        // Read the number of stored permissions.
                        let number_perms = u16::from_le_bytes(
                            permissions_tlv_slice
                                .get(0..2)
                                .ok_or(())?
                                .try_into()
                                .or(Err(()))?,
                        );
                        // Get the remaining slice of just the permissions.
                        let permissions_slice = permissions_tlv_slice.get(2..).ok_or(())?;

                        // Iterate the permissions to find a match.
                        for i in 0..number_perms as usize {
                            let perm: TbfHeaderDriverPermission = permissions_slice
                                .get((i * perm_len)..((i + 1) * perm_len))
                                .ok_or(())?
                                .try_into()
                                .or(Err(()))?;

                            if perm.driver_number == driver_num as u32 {
                                found_driver_num = true;
                                if perm.offset == offset as u32 {
                                    return Ok(CommandPermissions::Mask(perm.allowed_commands));
                                }
                            }
                        }

                        if found_driver_num {
                            // We found this driver number but nothing matched the
                            // requested offset. Since permissions are default off,
                            // we can return a mask of all zeros.
                            Ok(CommandPermissions::Mask(0))
                        } else {
                            Ok(CommandPermissions::NoPermsThisDriver)
                        }
                    }

                    get_command_permissions_result(permissions_tlv_slice, driver_num, offset)
                        .unwrap_or(CommandPermissions::NoPermsAtAll)
                }
                _ => CommandPermissions::NoPermsAtAll,
            },
            _ => CommandPermissions::NoPermsAtAll,
        }
    }

    /// Get the process `write_id`.
    ///
    /// Returns `None` if a `write_id` is not included. This indicates the TBF
    /// does not have the ability to store new items.
    pub fn get_storage_write_id(&self) -> Option<core::num::NonZeroU32> {
        match self {
            TbfHeader::TbfHeaderV2(hd) => match hd.storage_permissions {
                Some(storage_permissions_tlv_slice) => {
                    let write_id = core::num::NonZeroU32::new(u32::from_le_bytes(
                        storage_permissions_tlv_slice.get(0..4)?.try_into().ok()?,
                    ));

                    write_id
                }
                _ => None,
            },
            _ => None,
        }
    }

    /// Get the number of valid `read_ids` and the `read_ids`.
    /// Returns `None` if a `read_ids` is not included.
    pub fn get_storage_read_ids(&self) -> Option<(usize, [u32; NUM_STORAGE_PERMISSIONS])> {
        match self {
            TbfHeader::TbfHeaderV2(hd) => match hd.storage_permissions {
                Some(storage_permissions_tlv_slice) => {
                    let storage_permissions: TbfHeaderV2StoragePermissions<
                        NUM_STORAGE_PERMISSIONS,
                    > = storage_permissions_tlv_slice.try_into().ok()?;

                    Some((
                        storage_permissions.read_length.into(),
                        storage_permissions.read_ids,
                    ))
                }
                _ => None,
            },
            _ => None,
        }
    }

    /// Get the number of valid `access_ids` and the `access_ids`.
    /// Returns `None` if a `access_ids` is not included.
    pub fn get_storage_modify_ids(&self) -> Option<(usize, [u32; NUM_STORAGE_PERMISSIONS])> {
        match self {
            TbfHeader::TbfHeaderV2(hd) => match hd.storage_permissions {
                Some(storage_permissions_tlv_slice) => {
                    let storage_permissions: TbfHeaderV2StoragePermissions<
                        NUM_STORAGE_PERMISSIONS,
                    > = storage_permissions_tlv_slice.try_into().ok()?;

                    Some((
                        storage_permissions.modify_length.into(),
                        storage_permissions.modify_ids,
                    ))
                }
                _ => None,
            },
            _ => None,
        }
    }

    /// Get the minimum compatible kernel version this process requires.
    /// Returns `None` if the kernel compatibility header is not included.
    pub fn get_kernel_version(&self) -> Option<(u16, u16)> {
        match self {
            TbfHeader::TbfHeaderV2(hd) => match hd.kernel_version {
                Some(kernel_version) => Some((kernel_version.major, kernel_version.minor)),
                _ => None,
            },
            _ => None,
        }
    }

    /// Return the offset where the binary ends in the TBF or 0 if there
    /// is no binary. If there is a Main header the end offset is the size
    /// of the TBF, while if there is a Program header it can be smaller.
    pub fn get_binary_end(&self) -> u32 {
        match self {
            TbfHeader::TbfHeaderV2(hd) => hd
                .program
                .map_or(hd.base.total_size, |p| p.binary_end_offset),
            _ => 0,
        }
    }

    /// Return the version number of the Userspace Binary in this TBF
    /// Object, or 0 if there is no binary or no version number.
    pub fn get_binary_version(&self) -> u32 {
        match self {
            TbfHeader::TbfHeaderV2(hd) => hd.program.map_or(0, |p| p.version),
            _ => 0,
        }
    }

    /// Return the fixed ShortId of the application if it was specified in the
    /// TBF header.
    pub fn get_fixed_short_id(&self) -> Option<core::num::NonZeroU32> {
        match self {
            TbfHeader::TbfHeaderV2(hd) => hd.short_id.map_or(None, |si| si.short_id),
            _ => None,
        }
    }
}
