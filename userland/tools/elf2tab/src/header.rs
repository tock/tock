use std::fmt;
use std::io;
use std::io::Read;
use std::io::Seek;
use std::io::SeekFrom;
use std::io::Write;
use std::mem;
use std::vec;
use util;

#[repr(u16)]
#[derive(Clone, Copy, Debug)]
#[allow(dead_code)]
enum TbfHeaderTypes {
    TbfHeaderMain = 1,
    TbfHeaderWriteableFlashRegions = 2,
    TbfHeaderPackageName = 3,
    TbfHeaderPicOption1 = 4,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
struct TbfHeaderTlv {
    tipe: TbfHeaderTypes,
    length: u16,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
struct TbfHeaderBase {
    version: u16,
    header_size: u16,
    total_size: u32,
    flags: u32,
    checksum: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
struct TbfHeaderMain {
    base: TbfHeaderTlv,
    init_fn_offset: u32,
    protected_size: u32,
    minimum_ram_size: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
struct TbfHeaderWriteableFlashRegion {
    base: TbfHeaderTlv,
    offset: u32,
    size: u32,
}

impl fmt::Display for TbfHeaderBase {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "
               version: {:>8} {:>#10X}
           header_size: {:>8} {:>#10X}
            total_size: {:>8} {:>#10X}
                 flags: {:>8} {:>#10X}
",
            self.version,
            self.version,
            self.header_size,
            self.header_size,
            self.total_size,
            self.total_size,
            self.flags,
            self.flags,
        )
    }
}

impl fmt::Display for TbfHeaderMain {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "
        init_fn_offset: {:>8} {:>#10X}
        protected_size: {:>8} {:>#10X}
      minimum_ram_size: {:>8} {:>#10X}
",
            self.init_fn_offset,
            self.init_fn_offset,
            self.protected_size,
            self.protected_size,
            self.minimum_ram_size,
            self.minimum_ram_size,
        )
    }
}

impl fmt::Display for TbfHeaderWriteableFlashRegion {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "
    flash region:
                offset: {:>8} {:>#10X}
                  size: {:>8} {:>#10X}
",
            self.offset, self.offset, self.size, self.size,
        )
    }
}

pub struct TbfHeader {
    hdr_base: TbfHeaderBase,
    hdr_main: TbfHeaderMain,
    hdr_pkg_name_tlv: Option<TbfHeaderTlv>,
    hdr_wfr: Option<TbfHeaderWriteableFlashRegion>,
    package_name: String,
    package_name_pad: usize,
}

impl TbfHeader {
    pub fn new() -> TbfHeader {
        TbfHeader {
            hdr_base: TbfHeaderBase {
                version: 2, // Current version is 2.
                header_size: 0,
                total_size: 0,
                flags: 0,
                checksum: 0,
            },
            hdr_main: TbfHeaderMain {
                base: TbfHeaderTlv {
                    tipe: TbfHeaderTypes::TbfHeaderMain,
                    length: (mem::size_of::<TbfHeaderMain>() - mem::size_of::<TbfHeaderTlv>())
                        as u16,
                },
                init_fn_offset: 0,
                protected_size: 0,
                minimum_ram_size: 0,
            },
            hdr_pkg_name_tlv: None,
            hdr_wfr: None,
            package_name: String::new(),
            package_name_pad: 0,
        }
    }

    /// Start creating the Tock Binary Format Header. This function expects
    /// a few parameters that should be known very easily. Other values that
    /// we need to create the header (like the location of things in the flash
    /// binary) can be passed in later after we know the size of the header.
    ///
    /// Returns: The length of the header in bytes. The length is guaranteed
    ///          to be a multiple of 4.
    pub fn create(&mut self, minimum_ram_size: u32, appstate: bool, package_name: String) -> usize {
        // Need to calculate lengths ahead of time.
        // Need the base and the main section.
        let mut header_length = mem::size_of::<TbfHeaderBase>() + mem::size_of::<TbfHeaderMain>();

        // If we have a package name, add that section.
        self.package_name_pad = if package_name.len() > 0 {
            // Header increases by the TLV and name length.
            header_length += mem::size_of::<TbfHeaderTlv>() + package_name.len();
            // How much padding is needed to ensure we are aligned to 4?
            let pad = align4needed!(header_length);
            // Header length increases by that padding
            header_length += pad;
            pad
        } else {
            0
        };

        // We have one app flash region, add that.
        if appstate {
            header_length += mem::size_of::<TbfHeaderWriteableFlashRegion>();
        }

        // Flags default to app is enabled.
        let flags = 0x00000001;

        // Fill in the fields that we can at this point.
        self.hdr_base.header_size = header_length as u16;
        self.hdr_base.flags = flags;
        self.hdr_main.minimum_ram_size = minimum_ram_size;

        // If a package name exists, keep track of it and add it to the header.
        self.package_name = package_name;
        if self.package_name.len() > 0 {
            self.hdr_pkg_name_tlv = Some(TbfHeaderTlv {
                tipe: TbfHeaderTypes::TbfHeaderPackageName,
                length: self.package_name.len() as u16,
            });
        }

        // If there is an app state region, start setting up that header.
        if appstate {
            self.hdr_wfr = Some(TbfHeaderWriteableFlashRegion {
                base: TbfHeaderTlv {
                    tipe: TbfHeaderTypes::TbfHeaderWriteableFlashRegions,
                    length: 8,
                },
                offset: 0,
                size: 0,
            });
        }

        // Return the length by generating the header and seeing how long it is.
        self.generate().unwrap().get_ref().len()
    }

    /// Update the header with correct size for the entire app binary.
    pub fn set_total_size(&mut self, total_size: u32) {
        self.hdr_base.total_size = total_size;
    }

    /// Update the header with the correct offset for the _start function.
    pub fn set_init_fn_offset(&mut self, init_fn_offset: u32) {
        self.hdr_main.init_fn_offset = init_fn_offset;
    }

    /// Update the header with appstate values if appropriate.
    pub fn set_appstate_values(&mut self, appstate_offset: u32, appstate_size: u32) {
        self.hdr_wfr.as_mut().map(|wfr| {
            wfr.offset = appstate_offset;
            wfr.size = appstate_size;
        });
    }

    /// Create the header in binary form.
    pub fn generate(&self) -> io::Result<(io::Cursor<vec::Vec<u8>>)> {
        let mut header_buf = io::Cursor::new(Vec::new());

        // Write all bytes to an in-memory file for the header.
        try!(header_buf.write_all(unsafe { util::as_byte_slice(&self.hdr_base) }));
        try!(header_buf.write_all(unsafe { util::as_byte_slice(&self.hdr_main) }));
        if self.package_name.len() > 0 {
            try!(header_buf.write_all(unsafe { util::as_byte_slice(&self.hdr_pkg_name_tlv) }));
            try!(header_buf.write_all(self.package_name.as_ref()));
            try!(util::do_pad(&mut header_buf, self.package_name_pad));
        }

        // Only put these in the header if the app_state section is nonzero.
        match self.hdr_wfr {
            Some(wfr) => {
                try!(header_buf.write_all(unsafe { util::as_byte_slice(&wfr) }));
            }
            None => {}
        }

        let current_length = header_buf.get_ref().len();
        try!(util::do_pad(&mut header_buf, align4needed!(current_length)));

        self.inject_checksum(header_buf)
    }

    /// Take a TBF header and calculate the checksum. Then insert that checksum
    /// into the actual binary.
    fn inject_checksum(
        &self,
        mut header_buf: io::Cursor<vec::Vec<u8>>,
    ) -> io::Result<(io::Cursor<vec::Vec<u8>>)> {
        // Start from the beginning and iterate through the buffer as words.
        try!(header_buf.seek(SeekFrom::Start(0)));
        let mut wordbuf = [0u8; 4];
        let mut checksum: u32 = 0;
        loop {
            let ret = header_buf.read(&mut wordbuf);
            match ret {
                Ok(count) => {
                    // Combine the bytes back into a word, handling if we don't
                    // get a full word.
                    let mut word = 0;
                    for i in 0..count {
                        word |= (wordbuf[i] as u32) << (8 * i);
                    }
                    checksum ^= word;
                    if count != 4 {
                        break;
                    }
                }
                Err(_) => println!("Error calculating checksum."),
            }
        }

        // Now we need to insert the checksum into the correct position in the
        // header.
        try!(header_buf.seek(io::SeekFrom::Start(12)));
        wordbuf[0] = ((checksum >> 0) & 0xFF) as u8;
        wordbuf[1] = ((checksum >> 8) & 0xFF) as u8;
        wordbuf[2] = ((checksum >> 16) & 0xFF) as u8;
        wordbuf[3] = ((checksum >> 24) & 0xFF) as u8;
        try!(header_buf.write(&wordbuf));
        try!(header_buf.seek(io::SeekFrom::Start(0)));

        Ok(header_buf)
    }
}

impl fmt::Display for TbfHeader {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.hdr_base)?;
        write!(f, "{}", self.hdr_main)?;
        self.hdr_wfr.map_or(Ok(()), |wfr| write!(f, "{}", wfr))?;
        Ok(())
    }
}
