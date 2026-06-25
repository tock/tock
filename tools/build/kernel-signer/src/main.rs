// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2025.

//! Kernel signing tool for Tock secure boot
//! Computes kernel size from actual PT_LOAD segments

use anyhow::{anyhow, Context, Result};
use clap::Parser;
use goblin::elf::{program_header::PT_LOAD, Elf};
use p256::ecdsa::{signature::hazmat::PrehashSigner, Signature, SigningKey};
use p256::pkcs8::DecodePrivateKey;
use sha2::{Digest, Sha256};
use std::{fs, path::PathBuf};

const TLV_TYPE_SIGNATURE: u16 = 0x0104;
const TLV_TYPE_KERNEL_FLASH: u16 = 0x0102;

const SIG_RS_LEN: usize = 64;
const SIG_ALGO_LEN: usize = 4;
const SIG_VALUE_LEN: usize = SIG_RS_LEN + SIG_ALGO_LEN;

const ALGO_ECDSA_P256_SHA256: u32 = 1;

// Typical flash memory range for embedded systems
const FLASH_START: usize = 0x00000000;
const FLASH_END: usize = 0x00100000; // 1MB max

#[derive(Parser, Debug)]
#[command(author, version, about = "Sign a Tock kernel ELF for secure boot")]
struct Args {
    /// Path to kernel ELF
    kernel: PathBuf,

    /// PEM file containing the P-256 private key (overrides built-in)
    #[arg(long)]
    key: Option<PathBuf>,

    /// Output path; defaults to in-place overwrite of input ELF
    #[arg(long)]
    out: Option<PathBuf>,
}

const PRIVATE_KEY_PEM: &str = r#"
-----BEGIN PRIVATE KEY-----
MIGHAgEAMBMGByqGSM49AgEGCCqGSM49AwEHBG0wawIBAQQg9kwjBrAc65xuZSsE
x31rkSDpTl68NRjLbG/ioUPqbaahRANCAATKUz70xgjmgxHR+dTVUB19r8vwFRZO
jSkAzRxjMKf8Ih+c69XR6R9rgQOu4DIi/7zSdghcAShr/8okxhZp1NEd
-----END PRIVATE KEY-----
"#;

fn main() -> Result<()> {
    let args = Args::parse();
    let out_path = args.out.clone().unwrap_or_else(|| args.kernel.clone());

    let mut elf_bytes = fs::read(&args.kernel).context("read ELF")?;
    let elf = Elf::parse(&elf_bytes).context("parse ELF")?;

    // Get physical base of attributes
    let (attr_off, attr_size) = locate_attributes(&elf, &elf_bytes)?;
    let attr_seg = segment_containing_offset(&elf, attr_off, attr_size)
        .context("PT_LOAD containing .attributes not found")?;
    let attr_paddr = (attr_seg.p_paddr as usize) + (attr_off - attr_seg.p_offset as usize);
    let attr_slice = &elf_bytes[attr_off..attr_off + attr_size];

    // Parse TLVs
    let (sig_val_off_in_attr, _sig_hdr_off_in_attr) =
        find_tlv_value(attr_slice, TLV_TYPE_SIGNATURE)
            .context("Signature TLV not found")?;
    let (kern_flash_start, _kern_flash_len) =
        parse_kernel_flash_tlv(attr_slice)
            .context("Kernel Flash TLV (0x0102) not found")?;

    // Compute actual kernel size from PT_LOAD segments
    let mut loads: Vec<_> = elf.program_headers.iter().cloned()
        .filter(|ph| ph.p_type == PT_LOAD)
        .collect();
    loads.sort_by_key(|ph| ph.p_paddr);

    // Find actual kernel end by looking at flash segments
    let kernel_start = kern_flash_start as usize;
    let mut kernel_end = kernel_start;
    
    for ph in &loads {
        let seg_start = ph.p_paddr as usize;
        let seg_end = seg_start + ph.p_filesz as usize;
        
        // Only consider segments in flash range, before attributes
        if seg_start >= FLASH_START && seg_start < FLASH_END && seg_end <= attr_paddr {
            if ph.p_filesz > 0 {
                kernel_end = kernel_end.max(seg_end);
            }
        }
    }

    let sig_value_paddr = attr_paddr + sig_val_off_in_attr;
    let attr_start_paddr = attr_paddr;
    let attr_end_paddr = attr_paddr + attr_size;

    // Compute digest
    let digest = hash_kernel_and_attributes(
        &elf_bytes,
        &loads,
        kernel_start,
        kernel_end,
        attr_start_paddr,
        attr_end_paddr,
        sig_value_paddr,
        SIG_RS_LEN,
    )?;

    // Sign and write back
    let signing_key_pem = if let Some(path) = args.key {
        fs::read_to_string(path).context("read private key PEM")?
    } else {
        PRIVATE_KEY_PEM.to_string()
    };
    let sk = SigningKey::from_pkcs8_pem(signing_key_pem.trim()).context("load private key")?;
    let sig: Signature = sk.sign_prehash(&digest).context("sign_prehash")?;
    let rs = sig.to_bytes();

    let sig_value_off_in_elf = attr_off + sig_val_off_in_attr;
    elf_bytes[sig_value_off_in_elf .. sig_value_off_in_elf + SIG_RS_LEN].copy_from_slice(&rs);
    elf_bytes[sig_value_off_in_elf + SIG_RS_LEN .. sig_value_off_in_elf + SIG_VALUE_LEN]
        .copy_from_slice(&ALGO_ECDSA_P256_SHA256.to_le_bytes());

    fs::write(&out_path, &elf_bytes).context("write signed ELF")?;
    println!("\nSigned kernel saved to {}", out_path.display());
    Ok(())
}

// Helper functions

fn find_tlv_value(attr: &[u8], tlv_type: u16) -> Result<(usize, usize)> {
    if attr.len() < 8 { return Err(anyhow!(".attributes too small")); }
    if &attr[attr.len()-4..] != b"TOCK" { return Err(anyhow!("TOCK sentinel not found")); }
    let mut pos = attr.len() - 8;
    for _ in 0..128 {
        if pos < 4 { break; }
        let t  = u16::from_le_bytes([attr[pos-4], attr[pos-3]]);
        let ln = u16::from_le_bytes([attr[pos-2], attr[pos-1]]) as usize;
        if pos < 4 + ln { return Err(anyhow!("malformed TLV chain")); }
        let value_start = pos - 4 - ln;
        let header_off  = pos - 4;
        if t == tlv_type { return Ok((value_start, header_off)); }
        pos = value_start;
    }
    Err(anyhow!(format!("TLV 0x{tlv_type:04x} not found")))
}   

fn parse_kernel_flash_tlv(attr: &[u8]) -> Result<(u32, u32)> {
    let (value_off, _hdr) = find_tlv_value(attr, TLV_TYPE_KERNEL_FLASH)?;
    let v = &attr[value_off .. value_off + 8];
    let start = u32::from_le_bytes([v[0],v[1],v[2],v[3]]);
    let len   = u32::from_le_bytes([v[4],v[5],v[6],v[7]]);
    Ok((start, len))
}

fn locate_attributes<'a>(elf: &Elf<'a>, bytes: &'a [u8]) -> Result<(usize, usize)> {
    let (off, size) = elf
        .section_headers
        .iter()
        .find_map(|sh| {
            let name = elf.shdr_strtab.get_at(sh.sh_name)?;
            (name == ".attributes").then_some((sh.sh_offset as usize, sh.sh_size as usize))
        })
        .context(".attributes section not found")?;

    if off + size > bytes.len() {
        return Err(anyhow!(".attributes extends beyond file"));
    }
    Ok((off, size))
}

fn segment_containing_offset<'a>(
    elf: &'a Elf<'_>,
    off: usize,
    size: usize,
) -> Option<&'a goblin::elf::ProgramHeader> {
    elf.program_headers.iter().find(|ph| {
        ph.p_type == PT_LOAD
            && (off as u64) >= ph.p_offset
            && ((off + size) as u64) <= ph.p_offset + ph.p_filesz
    })
}

fn hash_kernel_and_attributes(
    elf_bytes: &[u8],
    loads: &[goblin::elf::ProgramHeader],
    kernel_start: usize,
    kernel_end: usize,
    attr_start: usize,
    attr_end: usize,
    sig_paddr: usize,
    sig_len: usize,
) -> Result<[u8; 32]> {
    let mut hasher = Sha256::new();

    // Hash kernel region
    hash_flash_window(
        &mut hasher,
        elf_bytes,
        loads,
        kernel_start,
        kernel_end,
        sig_paddr,
        sig_len,
    )?;

    // Hash attributes region
    hash_flash_window(
        &mut hasher,
        elf_bytes,
        loads,
        attr_start,
        attr_end,
        sig_paddr,
        sig_len,
    )?;

    let digest = hasher.finalize();
    Ok(digest.into())
}

fn hash_flash_window(
    hasher: &mut Sha256,
    elf_bytes: &[u8],
    loads: &[goblin::elf::ProgramHeader],
    win_start: usize,
    win_end: usize,
    sig_paddr: usize,
    sig_len: usize,
) -> Result<()> {
    let mut cur = win_start;

    for ph in loads {
        let seg_start = ph.p_paddr as usize;
        let seg_end = seg_start + ph.p_memsz as usize;

        // Skip segments outside flash range or with no file data
        if seg_start >= FLASH_END || ph.p_filesz == 0 {
            continue;
        }

        if seg_end <= win_start {
            continue;
        }
        if seg_start >= win_end {
            break;
        }

        // Pad any gaps before this segment
        if seg_start > cur {
            hash_fill(hasher, seg_start - cur, 0x00);
        }

        let h_start = seg_start.max(win_start);
        let h_end = seg_end.min(win_end);
        let h_len = h_end - h_start;

        let seg_file_start = ph.p_offset as usize;
        let seg_file_end = seg_file_start + ph.p_filesz as usize;

        let file_range_start = seg_file_start + (h_start - seg_start);
        let file_range_end = (file_range_start + h_len).min(seg_file_end);

        if file_range_start < file_range_end {
            let data = &elf_bytes[file_range_start..file_range_end];
            hash_data_with_sig_zero(hasher, data, h_start, sig_paddr, sig_paddr + sig_len);

            let hashed_len = file_range_end - file_range_start;
            if hashed_len < h_len {
                hash_fill(hasher, h_len - hashed_len, 0x00);
            }
        } else {
            hash_fill(hasher, h_len, 0x00);
        }

        cur = h_end;
    }

    // Pad rest of the window
    if cur < win_end {
        hash_fill(hasher, win_end - cur, 0x00);
    }

    Ok(())
}

fn hash_fill(hasher: &mut Sha256, size: usize, byte: u8) {
    let mut buf = [0u8; 4096];
    buf.fill(byte);
    let mut rem = size;
    while rem > 0 {
        let chunk = rem.min(buf.len());
        hasher.update(&buf[..chunk]);
        rem -= chunk;
    }
}

fn hash_data_with_sig_zero(
    hasher: &mut Sha256,
    data: &[u8],
    region_start: usize,
    sig_start: usize,
    sig_end: usize,
) {
    let region_end = region_start + data.len();

    if sig_end <= region_start || sig_start >= region_end {
        hasher.update(data);
        return;
    }

    let ovl_start = sig_start.max(region_start) - region_start;
    let ovl_end = sig_end.min(region_end) - region_start;

    if ovl_start > 0 {
        hasher.update(&data[..ovl_start]);
    }

    let zeros_len = ovl_end.saturating_sub(ovl_start);
    if zeros_len > 0 {
        let zeros = [0u8; SIG_VALUE_LEN];
        let mut rem = zeros_len;
        while rem > 0 {
            let chunk = rem.min(zeros.len());
            hasher.update(&zeros[..chunk]);
            rem -= chunk;
        }
    }

    if ovl_end < data.len() {
        hasher.update(&data[ovl_end..]);
    }
}