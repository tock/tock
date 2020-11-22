//! Tock Binary Format Header definitions and parsing code.

// Parsing the headers does not require any unsafe operations.
#![forbid(unsafe_code)]

use core::convert::TryInto;
use core::fmt;
use core::iter::Iterator;
use core::{mem, str};

/// Takes a value and rounds it up to be aligned % 4
macro_rules! align4 {
    ($e:expr $(,)?) => {
        ($e) + ((4 - (($e) % 4)) % 4)
    };
}

/// Error when parsing just the beginning of the TBF header. This is only used
/// when establishing the linked list structure of apps installed in flash.
pub(crate) enum InitialTbfParseError {
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
        }
    }
}

// TBF structure

/// TBF fields that must be present in all v2 headers.
#[derive(Clone, Copy, Debug)]
pub(crate) struct TbfHeaderV2Base {
    version: u16,
    header_size: u16,
    total_size: u32,
    flags: u32,
    checksum: u32,
}

/// Types in TLV structures for each optional block of the header.
#[derive(Clone, Copy, Debug)]
pub(crate) enum TbfHeaderTypes {
    TbfHeaderMain = 1,
    TbfHeaderWriteableFlashRegions = 2,
    TbfHeaderPackageName = 3,
    TbfHeaderFixedAddresses = 5,

    /// Some field in the header that we do not understand. Since the TLV format
    /// specifies the length of each section, if we get a field we do not
    /// understand we just skip it, rather than throwing an error.
    Unknown,
}

/// The TLV header (T and L).
#[derive(Clone, Copy, Debug)]
pub(crate) struct TbfHeaderTlv {
    tipe: TbfHeaderTypes,
    length: u16,
}

/// The v2 main section for apps.
///
/// All apps must have a main section. Without it, the header is considered as
/// only padding.
#[derive(Clone, Copy, Debug)]
pub(crate) struct TbfHeaderV2Main {
    init_fn_offset: u32,
    protected_size: u32,
    minimum_ram_size: u32,
}

/// Writeable flash regions only need an offset and size.
///
/// There can be multiple (or zero) flash regions defined, so this is its own
/// struct.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct TbfHeaderV2WriteableFlashRegion {
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
pub(crate) struct TbfHeaderV2FixedAddresses {
    /// The absolute address of the start of RAM that the process expects. For
    /// example, if the process was linked with a RAM region starting at
    /// address `0x00023000`, then this would be set to `0x00023000`.
    start_process_ram: u32,
    /// The absolute address of the start of the process binary. This does _not_
    /// include the TBF header. This is the address the process used for the
    /// start of flash with the linker.
    start_process_flash: u32,
}

// Conversion functions from slices to the various TBF fields.

impl core::convert::TryFrom<&[u8]> for TbfHeaderV2Base {
    type Error = TbfParseError;

    fn try_from(b: &[u8]) -> Result<TbfHeaderV2Base, Self::Error> {
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
            _ => Ok(TbfHeaderTypes::Unknown),
        }
    }
}

impl core::convert::TryFrom<&[u8]> for TbfHeaderTlv {
    type Error = TbfParseError;

    fn try_from(b: &[u8]) -> Result<TbfHeaderTlv, Self::Error> {
        Ok(TbfHeaderTlv {
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
        Ok(TbfHeaderV2Main {
            init_fn_offset: u32::from_le_bytes(
                b.get(0..4)
                    .ok_or(TbfParseError::InternalError)?
                    .try_into()?,
            ),
            protected_size: u32::from_le_bytes(
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

/// Single header that can contain all parts of a v2 header.
///
/// Note, this struct limits the number of writeable regions an app can have to
/// four since we need to statically know the length of the array to store in
/// this type.
#[derive(Clone, Copy, Debug)]
pub(crate) struct TbfHeaderV2 {
    base: TbfHeaderV2Base,
    main: Option<TbfHeaderV2Main>,
    package_name: Option<&'static str>,
    writeable_regions: Option<[Option<TbfHeaderV2WriteableFlashRegion>; 4]>,
    fixed_addresses: Option<TbfHeaderV2FixedAddresses>,
}

/// Type that represents the fields of the Tock Binary Format header.
///
/// This specifies the locations of the different code and memory sections
/// in the tock binary, as well as other information about the application.
/// The kernel can also use this header to keep persistent state about
/// the application.
#[derive(Debug)]
pub(crate) enum TbfHeader {
    TbfHeaderV2(TbfHeaderV2),
    Padding(TbfHeaderV2Base),
}

impl TbfHeader {
    /// Return whether this is an app or just padding between apps.
    pub(crate) fn is_app(&self) -> bool {
        match *self {
            TbfHeader::TbfHeaderV2(_) => true,
            TbfHeader::Padding(_) => false,
        }
    }

    /// Return whether the application is enabled or not.
    /// Disabled applications are not started by the kernel.
    pub(crate) fn enabled(&self) -> bool {
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
    pub(crate) fn get_minimum_app_ram_size(&self) -> u32 {
        match *self {
            TbfHeader::TbfHeaderV2(hd) => hd.main.map_or(0, |m| m.minimum_ram_size),
            _ => 0,
        }
    }

    /// Get the number of bytes from the start of the app's region in flash that
    /// is for kernel use only. The app cannot write this region.
    pub(crate) fn get_protected_size(&self) -> u32 {
        match *self {
            TbfHeader::TbfHeaderV2(hd) => {
                hd.main.map_or(0, |m| m.protected_size) + (hd.base.header_size as u32)
            }
            _ => 0,
        }
    }

    /// Get the offset from the beginning of the app's flash region where the
    /// app should start executing.
    pub(crate) fn get_init_function_offset(&self) -> u32 {
        match *self {
            TbfHeader::TbfHeaderV2(hd) => {
                hd.main.map_or(0, |m| m.init_fn_offset) + (hd.base.header_size as u32)
            }
            _ => 0,
        }
    }

    /// Get the name of the app.
    pub(crate) fn get_package_name(&self) -> Option<&'static str> {
        match *self {
            TbfHeader::TbfHeaderV2(hd) => hd.package_name,
            _ => None,
        }
    }

    /// Get the number of flash regions this app has specified in its header.
    pub(crate) fn number_writeable_flash_regions(&self) -> usize {
        match *self {
            TbfHeader::TbfHeaderV2(hd) => hd.writeable_regions.map_or(0, |wrs| {
                wrs.iter()
                    .fold(0, |acc, wr| if wr.is_some() { acc + 1 } else { acc })
            }),
            _ => 0,
        }
    }

    /// Get the offset and size of a given flash region.
    pub(crate) fn get_writeable_flash_region(&self, index: usize) -> (u32, u32) {
        match *self {
            TbfHeader::TbfHeaderV2(hd) => hd.writeable_regions.map_or((0, 0), |wrs| {
                wrs.get(index).unwrap_or(&None).map_or((0, 0), |wr| {
                    (
                        wr.writeable_flash_region_offset,
                        wr.writeable_flash_region_size,
                    )
                })
            }),
            _ => (0, 0),
        }
    }

    /// Get the address in RAM this process was specifically compiled for. If
    /// the process is position independent, return `None`.
    pub(crate) fn get_fixed_address_ram(&self) -> Option<u32> {
        let hd = match self {
            TbfHeader::TbfHeaderV2(hd) => hd,
            _ => return None,
        };
        match hd.fixed_addresses.as_ref()?.start_process_ram {
            0xFFFFFFFF => None,
            start => Some(start),
        }
    }

    /// Get the address in flash this process was specifically compiled for. If
    /// the process is position independent, return `None`.
    pub(crate) fn get_fixed_address_flash(&self) -> Option<u32> {
        let hd = match self {
            TbfHeader::TbfHeaderV2(hd) => hd,
            _ => return None,
        };
        match hd.fixed_addresses.as_ref()?.start_process_flash {
            0xFFFFFFFF => None,
            start => Some(start),
        }
    }
}

/// Parse the TBF header length and the entire length of the TBF binary.
///
/// ## Return
///
/// If all parsing is successful:
/// - Ok((Version, TBF header length, entire TBF length))
///
/// If we cannot parse the header because we have run out of flash, or the
/// values are entirely wrong we return `UnableToParse`. This means we have hit
/// the end of apps in flash.
/// - Err(InitialTbfParseError::UnableToParse)
///
/// Any other error we return an error and the length of the entire app so that
/// we can skip over it and check for the next app.
/// - Err(InitialTbfParseError::InvalidHeader(app_length))
pub(crate) fn parse_tbf_header_lengths(
    app: &'static [u8; 8],
) -> Result<(u16, u16, u32), InitialTbfParseError> {
    // Version is the first 16 bits of the app TBF contents. We need this to
    // correctly parse the other lengths.
    //
    // ## Safety
    // We trust that the version number has been checked prior to running this
    // parsing code. That is, whatever loaded this application has verified that
    // the version is valid and therefore we can trust it.
    let version = u16::from_le_bytes([app[0], app[1]]);

    match version {
        2 => {
            // In version 2, the next 16 bits after the version represent
            // the size of the TBF header in bytes.
            let tbf_header_size = u16::from_le_bytes([app[2], app[3]]);

            // The next 4 bytes are the size of the entire app's TBF space
            // including the header. This also must be checked before parsing
            // this header and we trust the value in flash.
            let tbf_size = u32::from_le_bytes([app[4], app[5], app[6], app[7]]);

            // Check that the header length isn't greater than the entire app,
            // and is at least as large as the v2 required header (which is 16
            // bytes). If that at least looks good then return the sizes.
            if u32::from(tbf_header_size) > tbf_size || tbf_header_size < 16 {
                Err(InitialTbfParseError::InvalidHeader(tbf_size))
            } else {
                Ok((version, tbf_header_size, tbf_size))
            }
        }

        // Since we have to trust the total size, and by extension the version
        // number, if we don't know how to handle the version this must not be
        // an actual app. Likely this is just the end of the app linked list.
        _ => Err(InitialTbfParseError::UnableToParse),
    }
}

/// Parse a TBF header stored in flash.
///
/// The `header` must be a slice that only contains the TBF header. The caller
/// should use the `parse_tbf_header_lengths()` function to determine this
/// length to create the correct sized slice.
pub(crate) fn parse_tbf_header(
    header: &'static [u8],
    version: u16,
) -> Result<TbfHeader, TbfParseError> {
    match version {
        2 => {
            // Get the required base. This will succeed because we parsed the
            // first bit of the header already in `parse_tbf_header_lengths()`.
            let tbf_header_base: TbfHeaderV2Base = header.try_into()?;

            // Calculate checksum. The checksum is the XOR of each 4 byte word
            // in the header.
            let mut checksum: u32 = 0;

            // Get an iterator across 4 byte fields in the header.
            let header_iter = header.chunks_exact(4);

            // Iterate all chunks and XOR the chunks to compute the checksum.
            for (i, chunk) in header_iter.enumerate() {
                let word = u32::from_le_bytes(chunk.try_into()?);
                if i == 3 {
                    // Skip the checksum field.
                } else {
                    checksum ^= word;
                }
            }

            // Verify the header matches.
            if checksum != tbf_header_base.checksum {
                return Err(TbfParseError::ChecksumMismatch(
                    tbf_header_base.checksum,
                    checksum,
                ));
            }

            // Get the rest of the header. The `remaining` variable will
            // continue to hold the remainder of the header we have not
            // processed.
            let mut remaining = header.get(16..).ok_or(TbfParseError::NotEnoughFlash)?;

            // If there is nothing left in the header then this is just a
            // padding "app" between two other apps.
            if remaining.len() == 0 {
                // Just padding.
                Ok(TbfHeader::Padding(tbf_header_base))
            } else {
                // This is an actual app.

                // Places to save fields that we parse out of the header
                // options.
                let mut main_pointer: Option<TbfHeaderV2Main> = None;
                let mut wfr_pointer: [Option<TbfHeaderV2WriteableFlashRegion>; 4] =
                    Default::default();
                let mut app_name_str = "";
                let mut fixed_address_pointer: Option<TbfHeaderV2FixedAddresses> = None;

                // Iterate the remainder of the header looking for TLV entries.
                while remaining.len() > 0 {
                    // Get the T and L portions of the next header (if it is
                    // there).
                    let tlv_header: TbfHeaderTlv = remaining.try_into()?;
                    remaining = remaining.get(4..).ok_or(TbfParseError::NotEnoughFlash)?;

                    match tlv_header.tipe {
                        TbfHeaderTypes::TbfHeaderMain => {
                            let entry_len = mem::size_of::<TbfHeaderV2Main>();

                            // Check that the size of the TLV entry matches the
                            // size of the Main TLV. If so we can store it.
                            // Otherwise, we fail to parse this TBF header and
                            // throw an error.
                            if tlv_header.length as usize == entry_len {
                                main_pointer = Some(remaining.try_into()?);
                            } else {
                                return Err(TbfParseError::BadTlvEntry(tlv_header.tipe as usize));
                            }
                        }

                        TbfHeaderTypes::TbfHeaderWriteableFlashRegions => {
                            // Length must be a multiple of the size of a region definition.
                            if tlv_header.length as usize
                                % mem::size_of::<TbfHeaderV2WriteableFlashRegion>()
                                == 0
                            {
                                // Calculate how many writeable flash regions
                                // there are specified in this header.
                                let wfr_len = mem::size_of::<TbfHeaderV2WriteableFlashRegion>();
                                let mut number_regions = tlv_header.length as usize / wfr_len;

                                // Capture a slice with just the wfr information.
                                let wfr_slice = remaining
                                    .get(0..tlv_header.length as usize)
                                    .ok_or(TbfParseError::NotEnoughFlash)?;

                                // To enable a static buffer, we only support up
                                // to four writeable flash regions.
                                if number_regions > 4 {
                                    number_regions = 4;
                                }

                                // Convert and store each wfr.
                                for i in 0..number_regions {
                                    wfr_pointer[i] = Some(
                                        wfr_slice
                                            .get(i * wfr_len..(i + 1) * wfr_len)
                                            .ok_or(TbfParseError::NotEnoughFlash)?
                                            .try_into()?,
                                    );
                                }
                            } else {
                                return Err(TbfParseError::BadTlvEntry(tlv_header.tipe as usize));
                            }
                        }

                        TbfHeaderTypes::TbfHeaderPackageName => {
                            let name_buf = remaining
                                .get(0..tlv_header.length as usize)
                                .ok_or(TbfParseError::NotEnoughFlash)?;

                            str::from_utf8(name_buf)
                                .map(|name_str| {
                                    app_name_str = name_str;
                                })
                                .or(Err(TbfParseError::BadProcessName))?;
                        }

                        TbfHeaderTypes::TbfHeaderFixedAddresses => {
                            let entry_len = 8;
                            if tlv_header.length as usize == entry_len {
                                fixed_address_pointer = Some(remaining.try_into()?);
                            } else {
                                return Err(TbfParseError::BadTlvEntry(tlv_header.tipe as usize));
                            }
                        }

                        _ => {}
                    }

                    // All TLV blocks are padded to 4 bytes, so we need to skip
                    // more if the length is not a multiple of 4.
                    let skip_len: usize = align4!(tlv_header.length as usize);
                    remaining = remaining
                        .get(skip_len..)
                        .ok_or(TbfParseError::NotEnoughFlash)?;
                }

                let tbf_header = TbfHeaderV2 {
                    base: tbf_header_base,
                    main: main_pointer,
                    package_name: Some(app_name_str),
                    writeable_regions: Some(wfr_pointer),
                    fixed_addresses: fixed_address_pointer,
                };

                Ok(TbfHeader::TbfHeaderV2(tbf_header))
            }
        }
        _ => Err(TbfParseError::UnsupportedVersion(version)),
    }
}
