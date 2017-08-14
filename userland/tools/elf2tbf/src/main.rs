extern crate elf;
extern crate getopts;

use getopts::Options;
use std::cmp;
use std::env;
use std::fmt;
use std::fs::File;
use std::io;
use std::io::{Cursor, Read, Seek, SeekFrom, Write};
use std::mem;
use std::path::Path;
use std::slice;

/// Takes a value and rounds it up to be aligned % 8
macro_rules! align8 {
    ( $e:expr ) => ( ($e) + ((8 - (($e) % 8)) % 8 ) );
}

/// Takes a value and rounds it up to be aligned % 4
macro_rules! align4 {
    ( $e:expr ) => ( ($e) + ((4 - (($e) % 4)) % 4 ) );
}

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
struct TbfHeaderPicOption1Fields {
    base: TbfHeaderTlv,
    text_offset: u32,
    data_offset: u32,
    data_size: u32,
    bss_memory_offset: u32,
    bss_size: u32,
    relocation_data_offset: u32,
    relocation_data_size: u32,
    got_offset: u32,
    got_size: u32,
    minimum_stack_length: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
struct TbfHeaderWriteableFlashRegion {
    offset: u32,
    size: u32,
}

impl fmt::Display for TbfHeaderBase {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "
               version: {:>8} {:>#10X}
           header_size: {:>8} {:>#10X}
            total_size: {:>8} {:>#10X}
                 flags: {:>8} {:>#10X}
",
        self.version, self.version,
        self.header_size, self.header_size,
        self.total_size, self.total_size,
        self.flags, self.flags,
        )
    }
}

impl fmt::Display for TbfHeaderMain {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "
        init_fn_offset: {:>8} {:>#10X}
        protected_size: {:>8} {:>#10X}
      minimum_ram_size: {:>8} {:>#10X}
",
        self.init_fn_offset, self.init_fn_offset,
        self.protected_size, self.protected_size,
        self.minimum_ram_size, self.minimum_ram_size,
        )
    }
}

impl fmt::Display for TbfHeaderPicOption1Fields {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "
           text_offset: {:>8} {:>#10X}
           data_offset: {:>8} {:>#10X}
             data_size: {:>8} {:>#10X}
     bss_memory_offset: {:>8} {:>#10X}
              bss_size: {:>8} {:>#10X}
relocation_data_offset: {:>8} {:>#10X}
  relocation_data_size: {:>8} {:>#10X}
            got_offset: {:>8} {:>#10X}
              got_size: {:>8} {:>#10X}
  minimum_stack_length: {:>8} {:>#10X}
",
        self.text_offset, self.text_offset,
        self.data_offset, self.data_offset,
        self.data_size, self.data_size,
        self.bss_memory_offset, self.bss_memory_offset,
        self.bss_size, self.bss_size,
        self.relocation_data_offset, self.relocation_data_offset,
        self.relocation_data_size, self.relocation_data_size,
        self.got_offset, self.got_offset,
        self.got_size, self.got_size,
        self.minimum_stack_length, self.minimum_stack_length,
        )
    }
}

impl fmt::Display for TbfHeaderWriteableFlashRegion {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "
    flash region:
                offset: {:>8} {:>#10X}
                  size: {:>8} {:>#10X}
",
        self.offset, self.offset,
        self.size, self.size,
        )
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();
    let mut opts = Options::new();
    opts.optopt("o", "", "set output file name", "OUTFILE");
    opts.optopt("n", "", "set package name", "PACKAGE_NAME");
    opts.optflag("v", "verbose", "be verbose");
    opts.optflag("p", "include-pic-info", "include PIC information");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => panic!(f.to_string()),
    };
    let output = matches.opt_str("o");
    let package_name = matches.opt_str("n");
    let verbose = matches.opt_present("v");
    let pic = matches.opt_present("p");
    let input = if !matches.free.is_empty() {
        matches.free[0].clone()
    } else {
        print_usage(&program, opts);
        return;
    };


    let path = Path::new(&input);
    let file = match elf::File::open_path(&path) {
        Ok(f) => f,
        Err(e) => panic!("Error: {:?}", e),
    };

    match output {
            None => {
                let mut out = io::stdout();
                do_work(&file, &mut out, package_name, verbose, pic)
            }
            Some(name) => {
                match File::create(Path::new(&name)) {
                    Ok(mut f) => do_work(&file, &mut f, package_name, verbose, pic),
                    Err(e) => panic!("Error: {:?}", e),
                }
            }
        }
        .expect("Failed to write output");
}

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} [-o OUTFILE] FILE", program);
    print!("{}", opts.usage(&brief));
}

fn get_section<'a>(input: &'a elf::File, name: &str) -> elf::Section {
    match input.get_section(name) {
        Some(section) => {
            elf::Section {
                data: section.data.clone(),
                shdr: section.shdr.clone(),
            }
        }
        None => {
            elf::Section {
                data: Vec::new(),
                shdr: elf::types::SectionHeader {
                    name: String::from(name),
                    shtype: elf::types::SHT_NULL,
                    flags: elf::types::SHF_NONE,
                    addr: 0,
                    offset: 0,
                    size: 0,
                    link: 0,
                    info: 0,
                    addralign: 0,
                    entsize: 0,
                },
            }
        }
    }
}

unsafe fn as_byte_slice<'a, T: Copy>(input: &'a T) -> &'a [u8] {
    slice::from_raw_parts(input as *const T as *const u8, mem::size_of::<T>())
}

fn do_work(input: &elf::File,
           output: &mut Write,
           package_name: Option<String>,
           verbose: bool,
           pic: bool)
           -> io::Result<()> {
    let package_name = package_name.unwrap_or(String::new());
    let (relocation_data_size, rel_data) = match input.sections
        .iter()
        .find(|section| section.shdr.name == ".rel.data".as_ref()) {
        Some(section) => (section.shdr.size, section.data.as_ref()),
        None => (0 as u64, &[] as &[u8]),

    };
    let text = get_section(input, ".text");
    let got = get_section(input, ".got");
    let data = get_section(input, ".data");
    let bss = get_section(input, ".bss");
    let appstate = get_section(input, ".app_state");

    // For these, we only care about the length
    let stack_len = get_section(input, ".stack").data.len() as u32;
    let app_heap_len = get_section(input, ".app_heap").data.len() as u32;
    let kernel_heap_len = get_section(input, ".kernel_heap").data.len() as u32;

    // Need to calculate lengths ahead of time.
    // Need the base and the main section.
    let mut header_length = mem::size_of::<TbfHeaderBase>() + mem::size_of::<TbfHeaderMain>();

    // If we have a package name, add that section.
    let mut post_name_pad = 0;
    if package_name.len() > 0 {
        let name_total_size = align4!(mem::size_of::<TbfHeaderTlv>() + package_name.len());
        header_length += name_total_size;

        // Calculate the padding required after the package name TLV. All blocks
        // must be four byte aligned, so we may need to add some padding.
        post_name_pad = name_total_size - (mem::size_of::<TbfHeaderTlv>() + package_name.len());
    }

    // If we need the kernel to do PIC fixup for us add the space for that.
    if pic {
        header_length += mem::size_of::<TbfHeaderPicOption1Fields>();
    }

    // We have one app flash region, add that.
    if appstate.data.len() > 0 {
        header_length += mem::size_of::<TbfHeaderTlv>() +
                         mem::size_of::<TbfHeaderWriteableFlashRegion>();
    }

    // Now we can calculate the entire size of the app in flash.
    let mut total_size = (header_length + rel_data.len() + text.data.len() + got.data.len() +
                          data.data.len() +
                          appstate.data.len()) as u32;

    let ending_pad = if total_size.count_ones() > 1 {
        let power2len = cmp::max(1 << (32 - total_size.leading_zeros()), 512);
        power2len - total_size
    } else {
        0
    };
    total_size += ending_pad;

    // Calculate the offset between the start of the flash region and the actual
    // app code. Also need to get the padding size.
    let app_start_offset = align8!(header_length);
    let post_header_pad = app_start_offset as usize - header_length;

    // To start we just restrict the app from writing all of the space before
    // its actual code and whatnot.
    let protected_size = app_start_offset;

    // First up is the app writeable app_state section. If this is not used or
    // non-existent, it will just be zero and won't matter. But we put it first
    // so that changes to the app won't move it.
    let appstate_offset = app_start_offset as u32;
    let appstate_size = appstate.shdr.size as u32;
    let relocation_data_offset = align8!(appstate_offset + appstate_size);
    // Make sure we pad back to a multiple of 8.
    let post_appstate_pad = relocation_data_offset - (appstate_offset + appstate_size);
    let text_offset = relocation_data_offset + (relocation_data_size as u32);
    let text_size = text.shdr.size as u32;
    let init_fn_offset = (input.ehdr.entry - text.shdr.addr) as u32 + text_offset;
    let got_offset = text_offset + text_size;
    let got_size = got.shdr.size as u32;
    let data_offset = got_offset + got_size;
    let data_size = data.shdr.size as u32;
    let bss_size = bss.shdr.size as u32;
    let bss_memory_offset = bss.shdr.addr as u32;
    let minimum_ram_size = stack_len + app_heap_len + kernel_heap_len + got_size + data_size +
                           bss_size;

    // Flags default to app is enabled.
    let flags = 0x00000001;

    let tbf_header_version = 2;

    let tbf_header = TbfHeaderBase {
        version: tbf_header_version,
        header_size: header_length as u16,
        total_size: total_size,
        flags: flags,
        checksum: 0,
    };

    let tbf_main = TbfHeaderMain {
        base: TbfHeaderTlv {
            tipe: TbfHeaderTypes::TbfHeaderMain,
            length: (mem::size_of::<TbfHeaderMain>() - mem::size_of::<TbfHeaderTlv>()) as u16,
        },
        init_fn_offset: init_fn_offset,
        protected_size: protected_size as u32,
        minimum_ram_size: minimum_ram_size,
    };

    let tbf_pic = TbfHeaderPicOption1Fields {
        base: TbfHeaderTlv {
            tipe: TbfHeaderTypes::TbfHeaderPicOption1,
            length: (mem::size_of::<TbfHeaderPicOption1Fields>() -
                     mem::size_of::<TbfHeaderTlv>()) as u16,
        },
        text_offset: text_offset,
        data_offset: data_offset,
        data_size: data_size,
        bss_memory_offset: bss_memory_offset,
        bss_size: bss_size,
        relocation_data_offset: relocation_data_offset,
        relocation_data_size: relocation_data_size as u32,
        got_offset: got_offset,
        got_size: got_size,
        minimum_stack_length: stack_len,
    };

    let tbf_package_name_tlv = TbfHeaderTlv {
        tipe: TbfHeaderTypes::TbfHeaderPackageName,
        length: package_name.len() as u16,
    };

    let tbf_flash_regions_tlv = TbfHeaderTlv {
        tipe: TbfHeaderTypes::TbfHeaderWriteableFlashRegions,
        length: 8,
    };

    let tbf_flash_region = TbfHeaderWriteableFlashRegion {
        offset: appstate_offset,
        size: appstate_size,
    };

    if verbose {
        print!("{}", tbf_header);
        print!("{}", tbf_main);
        if pic {
            print!("{}", tbf_pic);
        }
        print!("{}", tbf_flash_region);
    }

    // Calculate the header checksum.
    let mut header_buf = Cursor::new(Vec::new());

    // Write all bytes to an in-memory file for the header.
    try!(header_buf.write_all(unsafe { as_byte_slice(&tbf_header) }));
    try!(header_buf.write_all(unsafe { as_byte_slice(&tbf_main) }));
    try!(header_buf.write_all(unsafe { as_byte_slice(&tbf_package_name_tlv) }));
    try!(header_buf.write_all(package_name.as_ref()));
    try!(do_pad(&mut header_buf, post_name_pad));
    if pic {
        try!(header_buf.write_all(unsafe { as_byte_slice(&tbf_pic) }));
    }

    // Only put these in the header if the app_state section is nonzero.
    if appstate.data.len() > 0 {
        try!(header_buf.write_all(unsafe { as_byte_slice(&tbf_flash_regions_tlv) }));
        try!(header_buf.write_all(unsafe { as_byte_slice(&tbf_flash_region) }));
    }

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
    try!(header_buf.seek(SeekFrom::Start(12)));
    wordbuf[0] = ((checksum >> 0) & 0xFF) as u8;
    wordbuf[1] = ((checksum >> 8) & 0xFF) as u8;
    wordbuf[2] = ((checksum >> 16) & 0xFF) as u8;
    wordbuf[3] = ((checksum >> 24) & 0xFF) as u8;
    try!(header_buf.write(&wordbuf));
    try!(header_buf.seek(SeekFrom::Start(0)));

    fn do_pad(output: &mut Write, length: usize) -> io::Result<()> {
        let mut pad = length;
        let zero_buf = [0u8; 512];
        while pad > 0 {
            let amount_to_write = cmp::min(zero_buf.len(), pad);
            pad -= try!(output.write(&zero_buf[..amount_to_write]));
        }
        Ok(())
    }

    // Write the header and actual app to a binary file.
    try!(output.write_all(header_buf.get_ref()));
    try!(do_pad(output, post_header_pad as usize));
    try!(output.write_all(appstate.data.as_ref()));
    try!(do_pad(output, post_appstate_pad as usize));
    try!(output.write_all(rel_data.as_ref()));
    try!(output.write_all(text.data.as_ref()));
    try!(output.write_all(got.data.as_ref()));
    try!(output.write_all(data.data.as_ref()));

    // Pad to get a power of 2 sized flash app.
    try!(do_pad(output, ending_pad as usize));

    Ok(())
}
