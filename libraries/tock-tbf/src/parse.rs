//! Tock Binary Format parsing code.

use core::convert::TryInto;
use core::iter::Iterator;
use core::{mem, str};

use crate::types;

/// Takes a value and rounds it up to be aligned % 4
macro_rules! align4 {
    ($e:expr $(,)?) => {
        ($e) + ((4 - (($e) % 4)) % 4)
    };
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
pub fn parse_tbf_header_lengths(
    app: &'static [u8; 8],
) -> Result<(u16, u16, u32), types::InitialTbfParseError> {
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
                Err(types::InitialTbfParseError::InvalidHeader(tbf_size))
            } else {
                Ok((version, tbf_header_size, tbf_size))
            }
        }

        // Since we have to trust the total size, and by extension the version
        // number, if we don't know how to handle the version this must not be
        // an actual app. Likely this is just the end of the app linked list.
        _ => Err(types::InitialTbfParseError::UnableToParse),
    }
}

/// Parse a TBF header stored in flash.
///
/// The `header` must be a slice that only contains the TBF header. The caller
/// should use the `parse_tbf_header_lengths()` function to determine this
/// length to create the correct sized slice.
pub fn parse_tbf_header(
    header: &'static [u8],
    version: u16,
) -> Result<types::TbfHeader, types::TbfParseError> {
    match version {
        2 => {
            // Get the required base. This will succeed because we parsed the
            // first bit of the header already in `parse_tbf_header_lengths()`.
            let tbf_header_base: types::TbfHeaderV2Base = header.try_into()?;

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
                return Err(types::TbfParseError::ChecksumMismatch(
                    tbf_header_base.checksum,
                    checksum,
                ));
            }

            // Get the rest of the header. The `remaining` variable will
            // continue to hold the remainder of the header we have not
            // processed.
            let mut remaining = header
                .get(16..)
                .ok_or(types::TbfParseError::NotEnoughFlash)?;

            // If there is nothing left in the header then this is just a
            // padding "app" between two other apps.
            if remaining.len() == 0 {
                // Just padding.
                Ok(types::TbfHeader::Padding(tbf_header_base))
            } else {
                // This is an actual app.

                // Places to save fields that we parse out of the header
                // options.
                let mut main_pointer: Option<types::TbfHeaderV2Main> = None;
                let mut wfr_pointer: [Option<types::TbfHeaderV2WriteableFlashRegion>; 4] =
                    Default::default();
                let mut app_name_str = "";
                let mut fixed_address_pointer: Option<types::TbfHeaderV2FixedAddresses> = None;
                let mut kernel_version: Option<types::TbfHeaderV2KernelVersion> = None;

                // Iterate the remainder of the header looking for TLV entries.
                while remaining.len() > 0 {
                    // Get the T and L portions of the next header (if it is
                    // there).
                    let tlv_header: types::TbfHeaderTlv = remaining.try_into()?;
                    remaining = remaining
                        .get(4..)
                        .ok_or(types::TbfParseError::NotEnoughFlash)?;

                    match tlv_header.tipe {
                        types::TbfHeaderTypes::TbfHeaderMain => {
                            let entry_len = mem::size_of::<types::TbfHeaderV2Main>();

                            // Check that the size of the TLV entry matches the
                            // size of the Main TLV. If so we can store it.
                            // Otherwise, we fail to parse this TBF header and
                            // throw an error.
                            if tlv_header.length as usize == entry_len {
                                main_pointer = Some(remaining.try_into()?);
                            } else {
                                return Err(types::TbfParseError::BadTlvEntry(
                                    tlv_header.tipe as usize,
                                ));
                            }
                        }

                        types::TbfHeaderTypes::TbfHeaderWriteableFlashRegions => {
                            // Length must be a multiple of the size of a region definition.
                            if tlv_header.length as usize
                                % mem::size_of::<types::TbfHeaderV2WriteableFlashRegion>()
                                == 0
                            {
                                // Calculate how many writeable flash regions
                                // there are specified in this header.
                                let wfr_len =
                                    mem::size_of::<types::TbfHeaderV2WriteableFlashRegion>();
                                let mut number_regions = tlv_header.length as usize / wfr_len;

                                // Capture a slice with just the wfr information.
                                let wfr_slice = remaining
                                    .get(0..tlv_header.length as usize)
                                    .ok_or(types::TbfParseError::NotEnoughFlash)?;

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
                                            .ok_or(types::TbfParseError::NotEnoughFlash)?
                                            .try_into()?,
                                    );
                                }
                            } else {
                                return Err(types::TbfParseError::BadTlvEntry(
                                    tlv_header.tipe as usize,
                                ));
                            }
                        }

                        types::TbfHeaderTypes::TbfHeaderPackageName => {
                            let name_buf = remaining
                                .get(0..tlv_header.length as usize)
                                .ok_or(types::TbfParseError::NotEnoughFlash)?;

                            str::from_utf8(name_buf)
                                .map(|name_str| {
                                    app_name_str = name_str;
                                })
                                .or(Err(types::TbfParseError::BadProcessName))?;
                        }

                        types::TbfHeaderTypes::TbfHeaderFixedAddresses => {
                            let entry_len = 8;
                            if tlv_header.length as usize == entry_len {
                                fixed_address_pointer = Some(remaining.try_into()?);
                            } else {
                                return Err(types::TbfParseError::BadTlvEntry(
                                    tlv_header.tipe as usize,
                                ));
                            }
                        }

                        types::TbfHeaderTypes::TbfHeaderKernelVersion => {
                            let entry_len = 4;
                            if tlv_header.length as usize == entry_len {
                                kernel_version = Some(remaining.try_into()?);
                            } else {
                                return Err(types::TbfParseError::BadTlvEntry(
                                    tlv_header.tipe as usize,
                                ));
                            }
                        }

                        _ => {}
                    }

                    // All TLV blocks are padded to 4 bytes, so we need to skip
                    // more if the length is not a multiple of 4.
                    let skip_len: usize = align4!(tlv_header.length as usize);
                    remaining = remaining
                        .get(skip_len..)
                        .ok_or(types::TbfParseError::NotEnoughFlash)?;
                }

                let tbf_header = types::TbfHeaderV2 {
                    base: tbf_header_base,
                    main: main_pointer,
                    package_name: Some(app_name_str),
                    writeable_regions: Some(wfr_pointer),
                    fixed_addresses: fixed_address_pointer,
                    kernel_version: kernel_version,
                };

                Ok(types::TbfHeader::TbfHeaderV2(tbf_header))
            }
        }
        _ => Err(types::TbfParseError::UnsupportedVersion(version)),
    }
}
