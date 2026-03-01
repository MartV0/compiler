#![allow(non_camel_case_types, non_snake_case)]

use crate::linking::relocate;
use crate::compiling::CompilationResult;
/// In this module the creation of the ELF binary is done, from the already assembled byte code.
/// Created this file by following these resources:
/// - Amazing introduction, but stil pretty detailed: https://www.youtube.com/watch?v=nC1U1LJQL8o
/// - Creating a elf binary by hand (first comment contains x86_64): https://www.youtube.com/watch?v=XH6jDiKxod8
/// - Linux elf man page: https://www.man7.org/linux/man-pages/man5/elf.5.html


const EI_NIDENT: usize = 16;

type Elf64_Half = u16;
type Elf64_Word = u32;
type Elf64_Xword = u64;
type Elf64_Addr = u64;
type Elf64_Off = u64;

const ELF_HEADER_SIZE: Elf64_Half = 64;
const ELF_HEADER_SIZE_U: usize = 64;
const PROGRAM_HEADER_SIZE: Elf64_Half = 56;
const PROGRAM_HEADER_SIZE_U: usize = 56;

/// Elf file header
struct Elf64_Ehdr {
    e_ident: [u8; EI_NIDENT],
    e_type: Elf64_Half,
    e_machine: Elf64_Half,
    e_version: Elf64_Word,
    e_entry: Elf64_Addr,
    e_phoff: Elf64_Off,
    e_shoff: Elf64_Off,
    e_flags: Elf64_Word,
    e_ehsize: Elf64_Half,
    e_phentsize: Elf64_Half,
    e_phnum: Elf64_Half,
    e_shentsize: Elf64_Half,
    e_shnum: Elf64_Half,
    e_shstrndx: Elf64_Half,
}

/// Program header
/// Weird: order of these elements seems to be dependent on if it is 64 or 32 bit, maybe due to alignment?
struct Elf64_Phdr {
    p_type: Elf64_Word,
    p_flags: Elf64_Word,
    p_offset: Elf64_Off,
    p_vaddr: Elf64_Addr,
    p_paddr: Elf64_Addr,
    p_filesz: Elf64_Xword,
    p_memsz: Elf64_Xword,
    p_align: Elf64_Xword,
}

#[allow(dead_code)]
/// Section header
struct Elf64_Shdr {
    sh_name: Elf64_Word,
    sh_type: Elf64_Word,
    sh_flags: Elf64_Xword,
    sh_addr: Elf64_Addr,
    sh_offset: Elf64_Off,
    sh_size: Elf64_Xword,
    sh_link: Elf64_Word,
    sh_info: Elf64_Word,
    sh_addralign: Elf64_Xword,
    sh_entsize: Elf64_Xword,
}

pub fn create_elf(input: CompilationResult) -> Vec<u8> {
    let mut code = input.code;
    let code_relocate = input.code_relocate;
    let mut data = input.data;

    let header_count = 2;

    // Create code/text header
    let text_file_offset = (ELF_HEADER_SIZE + PROGRAM_HEADER_SIZE * header_count).into();
    // 0x400000 is default offset for text/code segment
    let code_virtual_address = text_file_offset + 0x400000;
    let code_len: u64 = code.len().try_into().expect("Failed to convert usize to u64");
    let text_program_header = create_program_header(
        ProgramHeaderType::Text, 
        text_file_offset,
        code_virtual_address,
        code_len
    );

    // Create data header
    let data_len: u64 = data.len().try_into().expect("Failed to convert usize to u64");
    let data_offset = text_file_offset + code_len;
    // Not sure why the + 0x1000 was necessary, but it fixed a segfault so im keeping it hehe
    // Maybe because alignment?
    let data_virtual_address: u64 = code_virtual_address + code_len + 0x1000;
    let data_program_header = create_program_header(
        ProgramHeaderType::Data, 
        data_offset,
        data_virtual_address, 
        data_len
    );

    // Relocate all references to data in the code to the new virtual address
    relocate::relocate(&mut code, code_relocate, data_virtual_address);

    // Add all sections of the elf file together
    let header = create_elf_header(header_count, code_virtual_address);
    let mut res = elf_header_to_bytes(header).to_vec();
    res.append(&mut program_header_to_bytes(text_program_header).to_vec());
    res.append(&mut program_header_to_bytes(data_program_header).to_vec());
    res.append(&mut code);
    res.append(&mut data);
    res
}

enum ProgramHeaderType {
    Data, // Global data/variables
    Text // Bytecode
}

/// Create program header for the supplied type of segment
fn create_program_header(
    ph_type: ProgramHeaderType,
    offset: Elf64_Off,
    virtual_adress: Elf64_Addr,
    size: Elf64_Xword
) -> Elf64_Phdr {
    // flags:
    // executable
    let PF_X = 1 << 0;
    // writable
    let PF_W = 1 << 1;
    // readable
    let PF_R = 1 << 2;

    let flags = match ph_type {
        ProgramHeaderType::Text => PF_X,
        ProgramHeaderType::Data => PF_R | PF_W,
    };

    Elf64_Phdr {
        // load: gets loaded into memory
        p_type: 1,
        p_flags: flags,
        // Where to find segment in the file
        p_offset: offset,
        p_vaddr: virtual_adress,
        // physical memory, only relevant for firmware etc
        p_paddr: 0,
        p_filesz: size,
        // Size in memory
        // equal for both the bytecode and the text, but might be different for other types
        p_memsz: size,
        // 4096 alignment seems to be required in x86_64
        p_align: 0x1000
    }
}

/// Create the global elf header
fn create_elf_header(num_pheaders: Elf64_Half, entry_point: Elf64_Addr) -> Elf64_Ehdr {
    let magic_number: u8 = 0x7f;
    // 64 bit objects
    let class: u8 = 2;
    // Least significant byte first
    // Seems to be only relevant for how the elf stuf gets interpreted
    let data: u8 = 1;
    // Always 1
    let version: u8 = 1;
    // 3: linux, usually system v/0, even on linux
    let os_abi: u8 = 3;
    // Usually never used
    let abi_version: u8 = 0;

    Elf64_Ehdr {
        e_ident: [
            magic_number, b'E', b'L', b'F', class, data, version, os_abi,
            abi_version, 0, 0, 0, 0, 0, 0, 0,
        ],
        // executable file
        // does not support aslr/ position independent code, use 3 if pid
        e_type: 2,
        // amd64
        e_machine: 0x3E,
        // elf version
        e_version: 1,
        // Entry point of address
        e_entry: entry_point,
        // Program headers offset
        // All directly in a list after this header
        e_phoff: ELF_HEADER_SIZE.into(),
        // Section headers offset
        e_shoff: 0,
        // Processor specific flags
        e_flags: 0,
        // Size of this elf header, 52 for 32 bit
        e_ehsize: ELF_HEADER_SIZE,
        // size per program header
        e_phentsize: PROGRAM_HEADER_SIZE,
        // number of program headers
        e_phnum: num_pheaders,
        // size per section header
        e_shentsize: 64,
        // number of section headers
        e_shnum: 0,
        // Section header string table index
        // Used to resolve names of sections in the file
        e_shstrndx: 0,
    }
}

fn elf_header_to_bytes(header: Elf64_Ehdr) -> [u8; ELF_HEADER_SIZE_U] {
    copy_into(vec![
        &header.e_ident,
        &header.e_type.to_le_bytes(),
        &header.e_machine.to_le_bytes(),
        &header.e_version.to_le_bytes(),
        &header.e_entry.to_le_bytes(),
        &header.e_phoff.to_le_bytes(),
        &header.e_shoff.to_le_bytes(),
        &header.e_flags.to_le_bytes(),
        &header.e_ehsize.to_le_bytes(),
        &header.e_phentsize.to_le_bytes(),
        &header.e_phnum.to_le_bytes(),
        &header.e_shentsize.to_le_bytes(),
        &header.e_shnum.to_le_bytes(),
        &header.e_shstrndx.to_le_bytes(),
    ])
}

fn program_header_to_bytes(header: Elf64_Phdr) -> [u8; PROGRAM_HEADER_SIZE_U] {
    copy_into(vec![
        &header.p_type.to_le_bytes(),
        &header.p_flags.to_le_bytes(),
        &header.p_offset.to_le_bytes(),
        &header.p_vaddr.to_le_bytes(),
        &header.p_paddr.to_le_bytes(),
        &header.p_filesz.to_le_bytes(),
        &header.p_memsz.to_le_bytes(),
        &header.p_align.to_le_bytes(),
    ])
}

fn copy_into<const N: usize>(slices: Vec<&[u8]>) -> [u8; N] {
    let mut index = 0;
    let mut array = [0; N];
    for slice in slices {
        let end_index = index + slice.len();
        array[index..end_index].clone_from_slice(slice);
        index = end_index;
    }
    array
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_copy_into() {
        let arr1 = [1, 2, 3];
        let arr2 = [4];

        assert_eq!(copy_into::<4>(vec![&arr1, &arr2]), [1, 2, 3, 4]);
    }
}
