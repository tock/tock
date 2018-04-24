extern crate chrono;
extern crate elf;
extern crate getopts;
extern crate tar;

use getopts::Options;
use std::{fs, path};
use std::cmp;
use std::env;
use std::fmt::Write as fmtwrite;
use std::io;
use std::io::{Seek, Write};
use std::mem;

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
    opts.reqopt(
        "",
        "app-heap",
        "set app heap size in bytes",
        "APP_HEAP_SIZE",
    );
    opts.reqopt(
        "",
        "kernel-heap",
        "set kernel heap size in bytes",
        "KERNEL_HEAP_SIZE",
    );
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
    let stack_len = matches
        .opt_str("stack")
        .unwrap()
        .parse::<u32>()
        .expect("Stack size must be an integer.");
    let app_heap_len = matches
        .opt_str("app-heap")
        .unwrap()
        .parse::<u32>()
        .expect("App heap size must be an integer.");
    let kernel_heap_len = matches
        .opt_str("kernel-heap")
        .unwrap()
        .parse::<u32>()
        .expect("Kernel heap size must be an integer.");

    // Check that we have at least one input file elf to process.
    if matches.free.is_empty() {
        print_usage(&program, opts);
        return;
    };

    // Create the metadata.toml file needed for the TAB file.
    let mut metadata_toml = String::new();
    write!(
        &mut metadata_toml,
        "tab-version = 1
name = \"{}\"
only-for-boards = \"\"
build-date = {}",
        package_name.clone().unwrap(),
        chrono::prelude::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
    ).unwrap();

    // Start creating a tar archive which will be the .tab file.
    let tab_name = fs::File::create(path::Path::new(&output.unwrap()))
        .expect("Could not create the output file.");
    let mut tab = tar::Builder::new(tab_name);

    // Add the metadata file without creating a real file on the filesystem.
    let mut header = tar::Header::new_gnu();
    header.set_size(metadata_toml.as_bytes().len() as u64);
    header.set_mode(0o644);
    header.set_cksum();
    tab.append_data(&mut header, "metadata.toml", metadata_toml.as_bytes())
        .unwrap();

    // Iterate all input elfs. Convert them to Tock friendly binaries and then
    // add them to the TAB file.
    for input_elf in matches.free {
        let elf_path = path::Path::new(&input_elf);
        let bin_path = path::Path::new(&input_elf).with_extension("bin");

        let elffile = elf::File::open_path(&elf_path).expect("Could not open the .elf file.");
        // Get output file as both read/write for creating the binary and
        // adding it to the TAB tar file.
        let mut outfile: fs::File = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open(bin_path.clone())
            .unwrap();

        // Do the conversion to a tock binary.
        elf_to_tbf(
            &elffile,
            &mut outfile,
            package_name.clone(),
            verbose,
            stack_len,
            app_heap_len,
            kernel_heap_len,
        ).unwrap();

        // Add the file to the TAB tar file.
        outfile.seek(io::SeekFrom::Start(0)).unwrap();
        tab.append_file(bin_path.file_name().unwrap(), &mut outfile)
            .unwrap();
    }
}

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} [-o OUTFILE] FILE [FILE...]", program);
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

fn elf_to_tbf(
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
    let got_size = align4!(got.shdr.size) as u32;
    let data_size = align4!(data.shdr.size) as u32;
    let bss_size = align4!(bss.shdr.size) as u32;
    let minimum_ram_size = align8!(stack_len) + align4!(app_heap_len) + align4!(kernel_heap_len)
        + got_size + data_size + bss_size;

    // Keep track of an index of where we are in creating the app binary.
    let mut binary_index = 0;

    // Now we can create the first pass TBF header. This is mostly to get the
    // size of the header since we have to fill in some of the offsets later.
    let mut tbfheader = header::TbfHeader::new();
    let header_length = tbfheader.create(minimum_ram_size, appstate.shdr.size > 0, package_name);
    binary_index += header_length;

    // `app_start` is the address that is passed to the app.
    let app_start = binary_index;

    // Next up is the .text section.
    let section_start_text = binary_index;
    binary_index += text.data.len();

    // Next up is the app writeable app_state section. If this is not used or
    // non-existent, it will just be zero and won't matter.
    let appstate_offset = binary_index;
    let appstate_size = appstate.shdr.size as usize;
    binary_index += appstate_size;

    // Next up is the .got section.
    binary_index += got.data.len();

    // Next up is the .data section.
    binary_index += data.data.len();

    // Next up is the rel_data. We also include a u32 length to begin the
    // rel_data.
    binary_index += rel_data.len() + mem::size_of::<u32>();

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
    let init_fn_offset =
        (input.ehdr.entry - text.shdr.addr) as u32 + (section_start_text - app_start) as u32;

    // Now we can update the header with key values that we have now calculated.
    tbfheader.set_total_size(total_size as u32);
    tbfheader.set_init_fn_offset(init_fn_offset as u32);
    tbfheader.set_appstate_values(appstate_offset as u32, appstate_size as u32);

    if verbose {
        print!("{}", tbfheader);
    }

    // Write the header and actual app to a binary file.
    output.write_all(tbfheader.generate().unwrap().get_ref())?;

    output.write_all(text.data.as_ref())?;
    output.write_all(appstate.data.as_ref())?;
    output.write_all(got.data.as_ref())?;
    output.write_all(data.data.as_ref())?;

    let rel_data_len: [u8; 4] = [
        (rel_data.len() & 0xff) as u8,
        (rel_data.len() >> 8 & 0xff) as u8,
        (rel_data.len() >> 16 & 0xff) as u8,
        (rel_data.len() >> 24 & 0xff) as u8,
    ];
    output.write_all(&rel_data_len)?;
    output.write_all(rel_data.as_ref())?;

    // Pad to get a power of 2 sized flash app.
    util::do_pad(output, post_content_pad as usize)?;

    Ok(())
}
