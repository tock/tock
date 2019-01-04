//! Tock Binary Format Header definitions and parsing code.

use core::{mem, slice, str};

/// Takes a value and rounds it up to be aligned % 4
macro_rules! align4 {
    ($e:expr) => {
        ($e) + ((4 - (($e) % 4)) % 4)
    };
}

/// TBF fields that must be present in all v2 headers.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
crate struct TbfHeaderV2Base {
    version: u16,
    header_size: u16,
    total_size: u32,
    flags: u32,
    checksum: u32,
}

/// Types in TLV structures for each optional block of the header.
#[repr(u16)]
#[derive(Clone, Copy, Debug)]
#[allow(dead_code)]
crate enum TbfHeaderTypes {
    TbfHeaderMain = 1,
    TbfHeaderWriteableFlashRegions = 2,
    TbfHeaderPackageName = 3,
    Unused = 5,
}

/// The TLV header (T and L).
#[repr(C)]
#[derive(Clone, Copy, Debug)]
crate struct TbfHeaderTlv {
    tipe: TbfHeaderTypes,
    length: u16,
}

/// The v2 main section for apps.
///
/// All apps must have a main section. Without it, the header is considered as
/// only padding.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
crate struct TbfHeaderV2Main {
    init_fn_offset: u32,
    protected_size: u32,
    minimum_ram_size: u32,
}

/// Writeable flash regions only need an offset and size.
///
/// There can be multiple (or zero) flash regions defined, so this is its own
/// struct.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
crate struct TbfHeaderV2WriteableFlashRegion {
    writeable_flash_region_offset: u32,
    writeable_flash_region_size: u32,
}

/// Single header that can contain all parts of a v2 header.
#[derive(Clone, Copy, Debug)]
crate struct TbfHeaderV2 {
    base: &'static TbfHeaderV2Base,
    main: Option<&'static TbfHeaderV2Main>,
    package_name: Option<&'static str>,
    writeable_regions: Option<&'static [TbfHeaderV2WriteableFlashRegion]>,
}

/// Type that represents the fields of the Tock Binary Format header.
///
/// This specifies the locations of the different code and memory sections
/// in the tock binary, as well as other information about the application.
/// The kernel can also use this header to keep persistent state about
/// the application.
#[derive(Debug)]
crate enum TbfHeader {
    TbfHeaderV2(TbfHeaderV2),
    Padding(&'static TbfHeaderV2Base),
}

impl TbfHeader {
    /// Return whether this is an app or just padding between apps.
    crate fn is_app(&self) -> bool {
        match *self {
            TbfHeader::TbfHeaderV2(_) => true,
            TbfHeader::Padding(_) => false,
        }
    }

    /// Return whether the application is enabled or not.
    /// Disabled applications are not started by the kernel.
    crate fn enabled(&self) -> bool {
        match *self {
            TbfHeader::TbfHeaderV2(hd) => {
                // Bit 1 of flags is the enable/disable bit.
                hd.base.flags & 0x00000001 == 1
            }
            TbfHeader::Padding(_) => false,
        }
    }

    /// Get the total size in flash of this app or padding.
    crate fn get_total_size(&self) -> u32 {
        match *self {
            TbfHeader::TbfHeaderV2(hd) => hd.base.total_size,
            TbfHeader::Padding(hd) => hd.total_size,
        }
    }

    /// Add up all of the relevant fields in header version 1, or just used the
    /// app provided value in version 2 to get the total amount of RAM that is
    /// needed for this app.
    crate fn get_minimum_app_ram_size(&self) -> u32 {
        match *self {
            TbfHeader::TbfHeaderV2(hd) => hd.main.map_or(0, |m| m.minimum_ram_size),
            _ => 0,
        }
    }

    /// Get the number of bytes from the start of the app's region in flash that
    /// is for kernel use only. The app cannot write this region.
    crate fn get_protected_size(&self) -> u32 {
        match *self {
            TbfHeader::TbfHeaderV2(hd) => {
                hd.main.map_or(0, |m| m.protected_size) + (hd.base.header_size as u32)
            }
            _ => 0,
        }
    }

    /// Get the offset from the beginning of the app's flash region where the
    /// app should start executing.
    crate fn get_init_function_offset(&self) -> u32 {
        match *self {
            TbfHeader::TbfHeaderV2(hd) => {
                hd.main.map_or(0, |m| m.init_fn_offset) + (hd.base.header_size as u32)
            }
            _ => 0,
        }
    }

    /// Get the name of the app.
    crate fn get_package_name(&self) -> &'static str {
        match *self {
            TbfHeader::TbfHeaderV2(hd) => hd.package_name.unwrap_or(""),
            _ => "",
        }
    }

    /// Get the number of flash regions this app has specified in its header.
    crate fn number_writeable_flash_regions(&self) -> usize {
        match *self {
            TbfHeader::TbfHeaderV2(hd) => hd.writeable_regions.map_or(0, |wr| wr.len()),
            _ => 0,
        }
    }

    /// Get the offset and size of a given flash region.
    crate fn get_writeable_flash_region(&self, index: usize) -> (u32, u32) {
        match *self {
            TbfHeader::TbfHeaderV2(hd) => hd.writeable_regions.map_or((0, 0), |wr| {
                if wr.len() > index {
                    (
                        wr[index].writeable_flash_region_offset,
                        wr[index].writeable_flash_region_size,
                    )
                } else {
                    (0, 0)
                }
            }),
            _ => (0, 0),
        }
    }
}

/// Converts a pointer to memory to a TbfHeader struct
///
/// This function takes a pointer to arbitrary memory and optionally returns a
/// TBF header struct. This function will validate the header checksum, but does
/// not perform sanity or security checking on the structure.
crate unsafe fn parse_and_validate_tbf_header(address: *const u8) -> Option<TbfHeader> {
    let version = *(address as *const u16);

    match version {
        2 => {
            let tbf_header_base = &*(address as *const TbfHeaderV2Base);

            // Some sanity checking. Make sure the header isn't longer than the
            // total app. Make sure the total app fits inside a reasonable size
            // of flash.
            if tbf_header_base.header_size as u32 >= tbf_header_base.total_size
                || tbf_header_base.total_size > 0x010000000
            {
                return None;
            }

            // Calculate checksum. The checksum is the XOR of each 4 byte word
            // in the header.
            let mut chunks = tbf_header_base.header_size as usize / 4;
            let leftover_bytes = tbf_header_base.header_size as usize % 4;
            if leftover_bytes != 0 {
                chunks += 1;
            }
            let mut checksum: u32 = 0;
            let header = slice::from_raw_parts(address as *const u32, chunks);
            for (i, chunk) in header.iter().enumerate() {
                if i == 3 {
                    // Skip the checksum field.
                } else if i == chunks - 1 && leftover_bytes != 0 {
                    // In this case, we don't want to use the entire word.
                    checksum ^= *chunk & (0xFFFFFFFF >> (4 - leftover_bytes));
                } else {
                    checksum ^= *chunk;
                }
            }

            if checksum != tbf_header_base.checksum {
                return None;
            }

            // Skip the base of the header.
            let mut offset = mem::size_of::<TbfHeaderV2Base>() as isize;
            let mut remaining_length = tbf_header_base.header_size as usize - offset as usize;

            // Check if this is a real app or just padding. Padding apps are
            // identified by not having any options.
            if remaining_length == 0 {
                // Just padding.
                if checksum == tbf_header_base.checksum {
                    Some(TbfHeader::Padding(tbf_header_base))
                } else {
                    None
                }
            } else {
                // This is an actual app.

                // Places to save fields that we parse out of the header
                // options.
                let mut main_pointer: Option<&TbfHeaderV2Main> = None;
                let mut wfr_pointer: Option<&'static [TbfHeaderV2WriteableFlashRegion]> = None;
                let mut app_name_str = "";

                // Loop through the header looking for known options.
                while remaining_length > mem::size_of::<TbfHeaderTlv>() {
                    let tbf_tlv_header = &*(address.offset(offset) as *const TbfHeaderTlv);

                    remaining_length -= mem::size_of::<TbfHeaderTlv>();
                    offset += mem::size_of::<TbfHeaderTlv>() as isize;

                    // Only parse known TLV blocks. There is no type 0.
                    if (tbf_tlv_header.tipe as u16) < TbfHeaderTypes::Unused as u16
                        && (tbf_tlv_header.tipe as u16) > 0
                    {
                        // This lets us skip unknown header types.

                        match tbf_tlv_header.tipe {
                            TbfHeaderTypes::TbfHeaderMain =>
                            /* Main */
                            {
                                if remaining_length >= mem::size_of::<TbfHeaderV2Main>()
                                    && tbf_tlv_header.length as usize
                                        == mem::size_of::<TbfHeaderV2Main>()
                                {
                                    let tbf_main =
                                        &*(address.offset(offset) as *const TbfHeaderV2Main);
                                    main_pointer = Some(tbf_main);
                                }
                            }
                            TbfHeaderTypes::TbfHeaderWriteableFlashRegions =>
                            /* Writeable Flash Regions */
                            {
                                // Length must be a multiple of the size of a region definition.
                                if tbf_tlv_header.length as usize
                                    % mem::size_of::<TbfHeaderV2WriteableFlashRegion>()
                                    == 0
                                {
                                    let number_regions = tbf_tlv_header.length as usize
                                        / mem::size_of::<TbfHeaderV2WriteableFlashRegion>();
                                    let region_start = &*(address.offset(offset)
                                        as *const TbfHeaderV2WriteableFlashRegion);
                                    let regions =
                                        slice::from_raw_parts(region_start, number_regions);
                                    wfr_pointer = Some(regions);
                                }
                            }
                            TbfHeaderTypes::TbfHeaderPackageName =>
                            /* Package Name */
                            {
                                if remaining_length >= tbf_tlv_header.length as usize {
                                    let package_name_byte_array = slice::from_raw_parts(
                                        address.offset(offset),
                                        tbf_tlv_header.length as usize,
                                    );
                                    let _ =
                                        str::from_utf8(package_name_byte_array).map(|name_str| {
                                            app_name_str = name_str;
                                        });
                                }
                            }
                            TbfHeaderTypes::Unused => {}
                        }
                    }

                    // All TLV blocks are padded to 4 bytes, so we need to skip
                    // more if the length is not a multiple of 4.
                    remaining_length -= align4!(tbf_tlv_header.length) as usize;
                    offset += align4!(tbf_tlv_header.length) as isize;
                }

                let tbf_header = TbfHeaderV2 {
                    base: tbf_header_base,
                    main: main_pointer,
                    package_name: Some(app_name_str),
                    writeable_regions: wfr_pointer,
                };

                Some(TbfHeader::TbfHeaderV2(tbf_header))
            }
        }

        // If we don't recognize the version number, we assume this is not a
        // valid app.
        _ => None,
    }
}
