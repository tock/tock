#![feature(raw)]
extern crate elf;
extern crate getopts;

use getopts::Options;
use std::env;
use std::path::Path;
use std::fs::File;
use std::io;
use std::io::Write;


#[repr(C,packed)]
#[derive(Debug)]
struct LoadInfo {
    rel_data_size: u32,
    entry_loc: u32,        /* Entry point for user application */
    init_data_loc: u32,    /* Data initialization information in flash */
    init_data_size: u32,   /* Size of initialization information */
    got_start_offset: u32, /* Offset in memory to start of GOT */
    got_end_offset: u32,   /* Offset in memory to end of GOT */
    bss_start_offset: u32, /* Offset in memory to start of BSS */
    bss_end_offset: u32    /* Offset in memory to end of BSS */
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();
    let mut opts = Options::new();
    opts.optopt("o", "", "set output file name", "OUTFILE");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => { m }
        Err(f) => { panic!(f.to_string()) }
    };
    let output = matches.opt_str("o");
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
            do_work(&file, &mut out)
        }
        Some(name) => match File::create(Path::new(&name)) {
            Ok(mut f) => do_work(&file, &mut f),
            Err(e) => panic!("Error: {:?}", e),
        }
    };


}

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} [-o OUTFILE] FILE", program);
    print!("{}", opts.usage(&brief));
}

fn get_section<'a>(input: &'a elf::File, name: &str) -> &'a elf::Section {
    match input.get_section(name) {
        Some(s) => {
            s
        },
        None => panic!("Failed to look up {} section", name),
    }
}

fn do_work(input: &elf::File, output: &mut Write) {
    let (rel_data_size, rel_data) = match input.sections.iter()
            .find(|section| section.shdr.name == ".rel.data".as_ref()) {
        Some(section) => {
            (section.shdr.size, section.data.as_ref())
        },
        None => (0 as u64, &[] as &[u8])

    };
    let text = get_section(input, ".text");
    let got = get_section(input, ".got");
    let data = get_section(input, ".data");
    let bss = get_section(input, ".bss");

    let load_info = LoadInfo {
        rel_data_size: rel_data_size as u32,
        entry_loc: (input.ehdr.entry ^ 0x80000000) as u32,
        init_data_loc: text.shdr.size as u32,
        init_data_size: (data.shdr.size + got.shdr.size) as u32,
        got_start_offset: 0,
        got_end_offset: got.shdr.size as u32,
        bss_start_offset: bss.shdr.addr as u32,
        bss_end_offset: (bss.shdr.addr + bss.shdr.size) as u32
    };

    let load_info_bytes : &[u8] = unsafe {
        std::mem::transmute(std::raw::Slice {
            data: &load_info,
            len: std::mem::size_of::<LoadInfo>()
        })
    };

    let mut total_len : u32 =
        (std::mem::size_of::<u32>() +
        load_info_bytes.as_ref().len() +
        rel_data.len() +
        text.data.len() +
        got.data.len() +
        data.data.len()) as u32;

    let pad = if total_len.count_ones() > 1 {
        let power2len = 1 << (32 - total_len.leading_zeros());
        power2len - total_len
    } else {
        0
    };
    total_len = total_len + pad;

    let total_len_buf : &[u8; 4] = unsafe {
        std::mem::transmute(&total_len)
    };

    let _ = output.write(total_len_buf);

    let _ = output.write(load_info_bytes);

    let _ = output.write(rel_data.as_ref());

    let _ = output.write(text.data.as_ref());
    let _ = output.write(got.data.as_ref());
    let _ = output.write(data.data.as_ref());

    for _ in 0..pad {
        let _ = output.write(&[0]);
    }
}

