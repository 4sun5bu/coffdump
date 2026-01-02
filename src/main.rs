use std::env;
use std::fs::File;
use std::io::{Read, SeekFrom, BufReader, Error};
use bincode::{config, Encode, Decode};

#[derive(Encode, Decode, PartialEq, Debug)]
struct CoffHeader {
    f_magic: u16,
    f_nscns: u16,
    f_timdat: u32,
    f_symptr: u32,
    f_nsyms: u32,
    f_opthdr: u16,
    f_flags: u16,
}

#[derive(Encode, Decode, PartialEq, Debug)]
struct SectionHeader {
    s_name: [u8; 8],
    s_paddr: u32,
    s_vaddr: u32,
    s_size: u32,
    s_scnptr: u32,
    s_relptr: u32,
    s_lnnoptr: u32,
    s_nreloc: u16,
    s_nlnno: u16,
    s_flags: u32,
}

#[derive(Encode, Decode, PartialEq, Debug)]
struct Relocation {
    r_vaddr: u32,
    r_symndx: u32,
    r_offset: u32,
    r_type: u16,
    r_stuff: u16,
}

#[derive(Encode, Decode, PartialEq, Debug)]
struct Symbol {
    n_name: [u8; 8],
    n_value: u32,
    n_scnum: u16,
    n_type: u16,
    n_sclass: u8,
    n_numaux: u8,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        panic!("no file");
    }
    let filename = &args[1];
    let file = File::open(filename)?;
    let mut reader = BufReader::new(file);

    let config = bincode::config::standard().with_fixed_int_encoding().with_big_endian();
    let hdr : CoffHeader = bincode::decode_from_reader(&mut reader, config).unwrap();
    println!("[Header]");
    print_filhdr(&hdr);

    let mut sections: Vec<SectionHeader> = Vec::new();  
    for _ in 0..hdr.f_nscns {
        let scnhdr : SectionHeader = bincode::decode_from_reader(&mut reader, config).unwrap();
        sections.push(scnhdr);
    }

    println!("\n[Sections]");
    let mut n = 0;
    for scn in sections {
        println!("section {} ", n);
        print_scnhdr(&scn);
        n += 1;
    }

    Ok(())
}

fn print_filhdr(hdr: &CoffHeader) {
    println!("  magic : 0x{:x}", hdr.f_magic);
    println!("  nscns : {} ", hdr.f_nscns);
    print!("  symoff : 0x{:06x}", hdr.f_symptr);
    println!("  nsyms : {}", hdr.f_nsyms);
    println!("  flags : 0x{:x}", hdr.f_flags);
}

fn print_scnhdr(scnhdr: &SectionHeader) {
    print!("  name : ["); 
    for c in scnhdr.s_name {
        print!("0x{:02x} ", c);
    }
    print!("\x08]");
    let name = String::from_utf8(scnhdr.s_name.to_vec()).unwrap();
    println!(" {}", name);
    print!("  paddress : 0x{:06x}", scnhdr.s_paddr);
    print!("  vaddr : 0x{:06x}", scnhdr.s_vaddr);
    println!(" size : {}", scnhdr.s_size);
    print!("  scnoff : 0x{:06x}", scnhdr.s_scnptr);
    print!("  reloff : 0x{:06x}", scnhdr.s_relptr);
    print!("  nreloc : {}", scnhdr.s_nreloc);
    println!("  flags : 0x{:06x}", scnhdr.s_flags);
}

fn print_relooc(reloc: &Relocation) {
    print!("  reloc vaddr : 0x{:06x}", reloc.r_vaddr);
    print!("  symndx : {}", reloc.r_symndx);
    println!("  type : 0x{:04x}", reloc.r_type);
}

fn print_sym(sym: &Symbol) {
    print!("  name : ["); 
    for c in sym.n_name {
        print!("0x{:02x} ", c);
    }
    print!("\x08]");
    if sym.n_name[0] == 0x00 {
        let name = String::from_utf8(sym.n_name.to_vec()).unwrap();
        println!(" {}", name);
    } else {
        let off = (sym.n_name[0] as u32) << 24
            + (sym.n_name[1] as u32) << 16
            + (sym.n_name[2] as u32) << 8
            + (sym.n_name[3] as u32);
        println!("symoff : {:06x}", off);
    }
    println!("  value : {:08x}", sym.n_value);
    println!("  section no :  {}", sym.n_scnum);
    println!("  type : {:04x}", sym.n_type);
    println!("  class : {:02x}", sym.n_sclass);
    println!("  numaux : {:02x}", sym.n_numaux);
}

