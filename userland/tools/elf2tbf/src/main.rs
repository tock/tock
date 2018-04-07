extern crate elf;
extern crate getopts;

use getopts::Options;
use std::cmp;
use std::env;
use std::fs::File;
use std::io;
use std::io::Write;
use std::mem;
use std::path::Path;

#[macro_use]
mod util;
mod header;


fn main() {
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();
    let mut opts = Options::new();
    opts.reqopt("o", "", "set output file name", "OUTFILE");
    opts.optopt("n", "", "set package name", "PACKAGE_NAME");
    opts.reqopt("", "stack", "set stack size in bytes", "STACK_SIZE");
    opts.reqopt("", "app-heap", "set app heap size in bytes", "APP_HEAP_SIZE");
    opts.reqopt("", "kernel-heap", "set kernel heap size in bytes", "KERNEL_HEAP_SIZE");
    opts.optflag("", "crt0-header", "include crt0 header for PIC fixups");
    opts.optflag("v", "verbose", "be verbose");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => panic!(f.to_string()),
    };

    let output = matches.opt_str("o");
    let package_name = matches.opt_str("n");
    let verbose = matches.opt_present("v");

    // Get the memory requirements from the app.
    let stack_len = matches.opt_str("stack").unwrap().parse::<u32>().unwrap();
    let app_heap_len = matches.opt_str("app-heap").unwrap().parse::<u32>().unwrap();
    let kernel_heap_len = matches.opt_str("kernel-heap").unwrap().parse::<u32>().unwrap();

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
        None => panic!("Need to specify an output file"),
        Some(name) => match File::create(Path::new(&name)) {
            Ok(mut f) => do_work(&file, &mut f, package_name, verbose, stack_len, app_heap_len, kernel_heap_len),
            Err(e) => panic!("Error: {:?}", e),
        },
    }.expect("Failed to write output");
}

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} [-o OUTFILE] FILE", program);
    print!("{}", opts.usage(&brief));
}

fn get_section<'a>(input: &'a elf::File, name: &str) -> elf::Section {
    match input.get_section(name) {
        Some(section) => elf::Section {
            data: section.data.clone(),
            shdr: section.shdr.clone(),
        },
        None => elf::Section {
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
        },
    }
}

fn do_work(
    input: &elf::File,
    output: &mut Write,
    package_name: Option<String>,
    verbose: bool,
    stack_len: u32,
    app_heap_len: u32,
    kernel_heap_len: u32,
) -> io::Result<()> {
    let package_name = package_name.unwrap_or(String::new());

    // Pull out the sections from the .elf we need.
    let rel_data = input
        .sections
        .iter()
        .find(|section| section.shdr.name == ".rel.data".as_ref())
        .map(|section| section.data.as_ref())
        .unwrap_or(&[] as &[u8]);
    let text = get_section(input, ".text");
    let got = get_section(input, ".got");
    let data = get_section(input, ".data");
    let bss = get_section(input, ".bss");
    let appstate = get_section(input, ".app_state");

    // Calculate how much RAM this app should ask the kernel for.
    let got_size = got.shdr.size as u32;
    let data_size = data.shdr.size as u32;
    let bss_size = bss.shdr.size as u32;
    let minimum_ram_size =
        stack_len + app_heap_len + kernel_heap_len + got_size + data_size + bss_size;

    // Keep track of an index of where we are in creating the app binary.
    let mut binary_index = 0;

    // Now we can create the first pass TBF header. This is mostly to get the
    // size of the header since we have to fill in some of the offsets later.
    let mut tbfheader = header::TbfHeader::new();
    let header_length = tbfheader.create(minimum_ram_size, appstate.shdr.size > 0, package_name);
    binary_index += header_length;

    // `app_start` is the address that is passed to the app.
    let app_start = binary_index;

    // Next up is the app writeable app_state section. If this is not used or
    // non-existent, it will just be zero and won't matter. But we put it early
    // so that changes to the app won't move it.
    let appstate_offset = binary_index;
    let appstate_size = appstate.shdr.size as usize;
    // Make sure we pad back to a multiple of 4.
    let post_appstate_pad = align4needed!(appstate_size);
    binary_index += appstate_size + post_appstate_pad;

    // Next up is the .text section.
    let section_start_text = binary_index;
    let post_text_pad = align4needed!(text.data.len());
    binary_index += text.data.len() + post_text_pad;

    // Next up is the .got section.
    let post_got_pad = align4needed!(got.data.len());
    binary_index += got.data.len() + post_got_pad;

    // Next up is the .data section.
    let post_data_pad = align4needed!(data.data.len());
    binary_index += data.data.len() + post_data_pad;

    // Next up is the rel_data. We also include a u32 length to begin the
    // rel_data.
    let post_reldata_pad = align4needed!(rel_data.len());
    binary_index += rel_data.len() + post_reldata_pad + mem::size_of::<u32>();

    // That is everything that we are going to include in our app binary. Now
    // we need to pad the binary to a power of 2 in size, and make sure it is
    // at least 512 bytes in size.
    let post_content_pad = if binary_index.count_ones() > 1 {
        let power2len = cmp::max(1 << (32 - (binary_index as u32).leading_zeros()), 512);
        power2len - binary_index
    } else {
        0
    };
    binary_index += post_content_pad;
    let total_size = binary_index;

    // Now we can calculate sizes and offsets that need to go to the header.

    // The init function is where the app will start executing, defined as
    // an offset from the end of protected region at the beginning of the app
    // in flash. Typically the protected region only includes the TBF header.
    // To calculate the offset we need to find the function in the binary
    // and then add the offset to the start of the .text section.
    let init_fn_offset = (input.ehdr.entry - text.shdr.addr) as u32 +
        (section_start_text - app_start) as u32;

    // Now we can update the header with key values that we have now calculated.
    tbfheader.set_total_size(total_size as u32);
    tbfheader.set_init_fn_offset(init_fn_offset as u32);
    tbfheader.set_appstate_values(appstate_offset as u32, appstate_size as u32);

    if verbose {
        print!("{}", tbfheader);
    }


    // Write the header and actual app to a binary file.
    try!(output.write_all(tbfheader.generate().unwrap().get_ref()));

    try!(output.write_all(appstate.data.as_ref()));
    try!(util::do_pad(output, post_appstate_pad as usize));

    try!(output.write_all(text.data.as_ref()));
    try!(util::do_pad(output, post_text_pad as usize));

    try!(output.write_all(got.data.as_ref()));
    try!(util::do_pad(output, post_got_pad as usize));

    try!(output.write_all(data.data.as_ref()));
    try!(util::do_pad(output, post_data_pad as usize));

    let rel_data_len: [u8; 4] = [
        (rel_data.len() & 0xff) as u8,
        (rel_data.len() >> 8 & 0xff) as u8,
        (rel_data.len() >> 16 & 0xff) as u8,
        (rel_data.len() >> 24 & 0xff) as u8,
    ];
    try!(output.write_all(&rel_data_len));
    try!(output.write_all(rel_data.as_ref()));
    try!(util::do_pad(output, post_reldata_pad as usize));

    // Pad to get a power of 2 sized flash app.
    try!(util::do_pad(output, post_content_pad as usize));

    Ok(())
}
