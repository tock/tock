extern crate elf;
extern crate getopts;

use getopts::Options;
use std::cmp;
use std::env;
use std::fmt;
use std::fs::File;
use std::io;
use std::io::Write;
use std::mem;
use std::path::Path;
use std::slice;


#[repr(C)]
#[derive(Clone, Copy, Debug)]
struct LoadInfo {
    version: u32,
    total_size: u32,
    entry_offset: u32,
    rel_data_offset: u32,
    rel_data_size: u32,
    text_offset: u32,
    text_size: u32,
    got_offset: u32,
    got_size: u32,
    data_offset: u32,
    data_size: u32,
    bss_mem_offset: u32,
    bss_size: u32,
    min_stack_len: u32,
    min_app_heap_len: u32,
    min_kernel_heap_len: u32,
    package_name_offset: u32,
    package_name_size: u32,
    checksum: u32,
}

impl fmt::Display for LoadInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "
            version: {:>8} {:>#10X}
         total_size: {:>8} {:>#10X}
       entry_offset: {:>8} {:>#10X}
    rel_data_offset: {:>8} {:>#10X}
      rel_data_size: {:>8} {:>#10X}
        text_offset: {:>8} {:>#10X}
          text_size: {:>8} {:>#10X}
         got_offset: {:>8} {:>#10X}
           got_size: {:>8} {:>#10X}
        data_offset: {:>8} {:>#10X}
          data_size: {:>8} {:>#10X}
     bss_mem_offset: {:>8} {:>#10X}
           bss_size: {:>8} {:>#10X}
      min_stack_len: {:>8} {:>#10X}
   min_app_heap_len: {:>8} {:>#10X}
min_kernel_heap_len: {:>8} {:>#10X}
package_name_offset: {:>8} {:>#10X}
  package_name_size: {:>8} {:>#10X}
           checksum: {:>8} {:>#10X}
",
        self.version, self.version,
        self.total_size, self.total_size,
        self.entry_offset, self.entry_offset,
        self.rel_data_offset, self.rel_data_offset,
        self.rel_data_size, self.rel_data_size,
        self.text_offset, self.text_offset,
        self.text_size, self.text_size,
        self.got_offset, self.got_offset,
        self.got_size, self.got_size,
        self.data_offset, self.data_offset,
        self.data_size, self.data_size,
        self.bss_mem_offset, self.bss_mem_offset,
        self.bss_size, self.bss_size,
        self.min_stack_len, self.min_stack_len,
        self.min_app_heap_len, self.min_app_heap_len,
        self.min_kernel_heap_len, self.min_kernel_heap_len,
        self.package_name_offset, self.package_name_offset,
        self.package_name_size, self.package_name_size,
        self.checksum, self.checksum,
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

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => panic!(f.to_string()),
    };
    let output = matches.opt_str("o");
    let package_name = matches.opt_str("n");
    let verbose = matches.opt_present("v");
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
                do_work(&file, &mut out, package_name, verbose)
            }
            Some(name) => {
                match File::create(Path::new(&name)) {
                    Ok(mut f) => do_work(&file, &mut f, package_name, verbose),
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
           verbose: bool)
           -> io::Result<()> {
    let package_name = package_name.unwrap_or(String::new());
    let (rel_data_size, rel_data) = match input.sections
        .iter()
        .find(|section| section.shdr.name == ".rel.data".as_ref()) {
        Some(section) => (section.shdr.size, section.data.as_ref()),
        None => (0 as u64, &[] as &[u8]),

    };
    let text = get_section(input, ".text");
    let got = get_section(input, ".got");
    let data = get_section(input, ".data");
    let bss = get_section(input, ".bss");

    // For these, we only care about the length
    let stack_len = get_section(input, ".stack").data.len() as u32;
    let app_heap_len = get_section(input, ".app_heap").data.len() as u32;
    let kernel_heap_len = get_section(input, ".kernel_heap").data.len() as u32;

    let mut total_size = (mem::size_of::<LoadInfo>() + rel_data.len() + text.data.len() +
                          got.data.len() +
                          data.data.len() + package_name.len()) as u32;

    let pad = if total_size.count_ones() > 1 {
        let power2len = 1 << (32 - total_size.leading_zeros());
        power2len - total_size
    } else {
        0
    };
    total_size = total_size + pad;

    let rel_data_offset = mem::size_of::<LoadInfo>() as u32;
    let text_offset = rel_data_offset + (rel_data_size as u32);
    let text_size = text.shdr.size as u32;
    let entry_offset = (input.ehdr.entry ^ 0x80000000) as u32 + text_offset;
    let got_offset = text_offset + text_size;
    let got_size = got.shdr.size as u32;
    let data_offset = got_offset + got_size;
    let data_size = data.shdr.size as u32;
    let package_name_offset = data_offset + data_size;
    let package_name_size = package_name.len() as u32;

    let load_info_version = 1;

    let load_info = LoadInfo {
        version: load_info_version,
        total_size: total_size,
        entry_offset: entry_offset,
        rel_data_offset: rel_data_offset,
        rel_data_size: rel_data_size as u32,
        text_offset: text_offset,
        text_size: text_size,
        got_offset: got_offset,
        got_size: got_size,
        data_offset: data_offset,
        data_size: data_size,
        bss_mem_offset: bss.shdr.addr as u32,
        bss_size: bss.shdr.size as u32,
        min_stack_len: stack_len,
        min_app_heap_len: app_heap_len,
        min_kernel_heap_len: kernel_heap_len,
        package_name_offset: package_name_offset,
        package_name_size: package_name_size,
        checksum: load_info_version ^ total_size ^ entry_offset ^ rel_data_offset ^
                  rel_data_size as u32 ^ text_offset ^ text_size ^ got_offset ^
                  got_size ^
                  data_offset ^ data_size ^ bss.shdr.addr as u32 ^
                  bss.shdr.size as u32 ^
                  stack_len ^ app_heap_len ^
                  kernel_heap_len ^ package_name_offset ^ package_name_size,
    };

    if verbose {
        print!("{}", load_info);
    }

    try!(output.write_all(unsafe { as_byte_slice(&load_info) }));
    try!(output.write_all(rel_data.as_ref()));
    try!(output.write_all(text.data.as_ref()));
    try!(output.write_all(got.data.as_ref()));
    try!(output.write_all(data.data.as_ref()));
    try!(output.write_all(package_name.as_ref()));

    let mut pad = pad as usize;
    let zero_buf = [0u8; 512];
    while pad > 0 {
        let amount_to_write = cmp::min(zero_buf.len(), pad);
        pad -= try!(output.write(&zero_buf[..amount_to_write]));
    }

    Ok(())
}
