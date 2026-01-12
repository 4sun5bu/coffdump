use std::env;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom, BufReader, Error};
use bincode::{config, Encode, Decode};

#[derive(Encode, Decode, PartialEq, Debug)]
struct Header {
    f_magic: u16,
    f_nscns: u16,
    f_timdat: u32,
    f_symptr: u32,
    f_nsyms: u32,
    f_opthdr: u16,
    f_flags: u16,
}

#[derive(Encode, Decode, PartialEq, Debug)]
struct Section {
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
    n_scnum: i16,
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
    let hdr: Header = bincode::decode_from_reader(&mut reader, config).unwrap();
    println!("[Header]");
    print_filhdr(&hdr);

    let mut sections: Vec<Section> = Vec::new();  
    for _ in 0..hdr.f_nscns {
        let scnhdr: Section = bincode::decode_from_reader(&mut reader, config).unwrap();
        sections.push(scnhdr);
    }

    println!("\n[Sections]");
    let mut n = 1;
    for scn in &sections {
        println!("scnum {} ", n);
        print_scnhdr(scn);
        n += 1;
    }

    let mut relocs: Vec<Vec<Relocation>> = Vec::new();
    for scn in &sections {
        reader.seek(SeekFrom::Start(scn.s_relptr as u64))?;
        let mut scn_relocs: Vec<Relocation> = Vec::new();
        for _ in 0..scn.s_nreloc {
            let rel: Relocation = bincode::decode_from_reader(&mut reader, config).unwrap();
            scn_relocs.push(rel);
        }
        relocs.push(scn_relocs);
    }

//    let mut relocs: Vec<Relocation> = Vec::new();
//    for scn in &sections {
//        reader.seek(SeekFrom::Start(scn.s_relptr as u64))?;
//        for _ in 0..scn.s_nreloc {
//           let rel: Relocation = bincode::decode_from_reader(&mut reader, config).unwrap();
//           relocs.push(rel);
//        }
//    }
    println!("\n[Relocation infomation]");
    print_relocinfo(&relocs);

    reader.seek(SeekFrom::Start(hdr.f_symptr as u64))?;
    let mut symbs: Vec<Symbol> = Vec::new();
    for _ in 0..hdr.f_nsyms {
        let sym: Symbol = bincode::decode_from_reader(&mut reader, config).unwrap();
        symbs.push(sym);
    }
    println!("\n[Symbol Table]");
    print_symtab(&symbs);

    Ok(())
}

fn print_filhdr(hdr: &Header) {
    println!("  magic : 0x{:x}", hdr.f_magic);
    println!("  nscns : {} ", hdr.f_nscns);
    print!("  symoff : 0x{:06x}", hdr.f_symptr);
    println!("  nsyms : {}", hdr.f_nsyms);
    println!("  flags : 0x{:x}", hdr.f_flags);
}

fn print_scnhdr(scnhdr: &Section) {
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

fn print_reloc(reloc: &Relocation) {
    print!("  reloc vaddr : 0x{:06x}", reloc.r_vaddr);
    print!("  symndx : {:2}", reloc.r_symndx);
    println!("  type : 0x{:04x}", reloc.r_type);
}

fn print_relocinfo(relocs: &Vec<Vec<Relocation>>) {
    let mut scnno = 1;
    for scn in relocs{
        for rel in scn {
            print!("  scnum : {}", scnno);
            print_reloc(rel);
        }
        scnno = scnno + 1;
    }
}

fn print_sym(sym: &Symbol) {
    print!("  name : ["); 
    for c in sym.n_name {
        print!("0x{:02x} ", c);
    }
    print!("\x08]");
    if sym.n_name[0] != 0x00 {
        let name = String::from_utf8(sym.n_name.to_vec()).unwrap();
        println!(" {}", name);
    } else {
        let off = ((sym.n_name[4] as u32) << 24)
            + ((sym.n_name[5] as u32) << 16)
            + ((sym.n_name[6] as u32) << 8)
            + (sym.n_name[7] as u32);
        println!(" 0x{:06x}", off);
    }
    print!("  value : 0x{:08x} ", sym.n_value);
    print!("scnum : {:2} ", sym.n_scnum);
    print!("type : 0x{:04x} ", sym.n_type);
    print!("class : 0x{:02x} ", sym.n_sclass);
    println!("numaux : 0x{:02x}", sym.n_numaux);
}

fn print_symtab(syms: &Vec<Symbol>) {
    for sym in syms {
        print_sym(sym);
    }
}
