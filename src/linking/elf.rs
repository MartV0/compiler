#![allow(non_camel_case_types)]

const EI_NIDENT: usize = 16;

type Elf64_Half = u16; // TODO: alignment 2
type Elf64_Word = u32; // TODO: alignment 4
type Elf64_Xword = u64; // TODO: alignment 4
type Elf64_Addr = u64; // TODO: alignment 8
type Elf64_Off = u64; // TODO: alignment 8 

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

pub fn create_elf() -> Vec<u8> {
    // let mut data: Vec<u8> = "Hello world!".as_bytes().to_vec();
    // let data_len: u64 = data.len().try_into().expect("Failed to converst usize to u64");
    // let data_len_bytes = data_len.to_le_bytes();
    // let mut code: Vec<u8> = vec![
    //     //	mov    eax,0x1
    //     0xb8, 0x01, 0x00, 0x00, 0x00,       
    //     //	mov    edi,0x1
    //     0xbf, 0x01, 0x00, 0x00, 0x00,       
    //     //	movabs rsi, pointer to string, placeholder
    //     0x48, 0xbe, 0xAA, 0xAA, 0xAA, 0xAA, 0x00, 
    //     0x00, 0x00, 0x00,
    //     //	mov    edx, length of string
    //     0xba, data_len_bytes[0], data_len_bytes[1], data_len_bytes[2], data_len_bytes[3], 
    //     //	syscall
    //     0x0f, 0x05,                
    //     //	mov    eax,0x3c
    //     0xb8, 0x3c, 0x00, 0x00, 0x00,       
    //     //	mov    edi, exit code: 0x0
    //     0xbf, 0x00, 0x00, 0x00, 0x00,       
    //     //	syscall
    //     0x0f, 0x05                
    // ];
    let mut code: Vec<u8> = vec![
        0x48, 0xC7, 0xC0, 0x3C, 0x00, 0x00, 0x00, // mov rax, 60
        0x48, 0xC7, 0xC7, 0x2A, 0x00, 0x00, 0x00, // mov rdi, 42
        0x0F, 0x05, // syscall (the newer syscall instruction for x86-64 int 0x80 on x86)
    ];
    let text_header_offset = (ELF_HEADER_SIZE + PROGRAM_HEADER_SIZE).into();
    let code_virtual_address = text_header_offset + 0x400000;
    let code_len: u64 = code.len().try_into().expect("Failed to converst usize to u64");
    let text_program_header = create_program_header(
        ProgramHeaderType::Text, 
        text_header_offset,
        code_virtual_address,
        code_len
    );
    // let data_virtual_address: u64 = code_virtual_address + code_len;
    // let data_program_header = create_program_header(
    //     ProgramHeaderType::Rodata, 
    //     text_header_offset + code_len,
    //     data_virtual_address, 
    //     data_len
    // );
    // let str_ptr_bytes = data_virtual_address.to_le_bytes();
    // code[12..16].clone_from_slice(&str_ptr_bytes[0..4]);
    let header = create_elf_header(1, code_virtual_address);
    let mut res = elf_header_to_bytes(header).to_vec();
    res.append(&mut program_header_to_bytes(text_program_header).to_vec());
    // res.append(&mut program_header_to_bytes(data_program_header).to_vec());
    res.append(&mut code);
    // res.append(&mut data);
    res
}

enum ProgramHeaderType {
    Rodata, // Strings
    Text // Bytecode
}

fn create_program_header(ph_type: ProgramHeaderType, offset: Elf64_Off, virtual_adress: Elf64_Addr, size: Elf64_Xword) -> Elf64_Phdr {
    // flags:
    // executable
    let PF_X = 1 << 0;
    // writable
    let PF_W = 1 << 1;
    // readible
    let PF_R = 1 << 2;
    let mut flags = PF_R;
    if let ProgramHeaderType::Text = ph_type {
        flags |= PF_X;
    }

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
        // TODO: 4096?
        p_align: 0x1000
    }
}

fn create_elf_header(num_pheaders: Elf64_Half, entry_point: Elf64_Addr) -> Elf64_Ehdr {
    let magic_number: u8 = 0x7f;
    // 64 bit objects
    let class: u8 = 2;
    // Least significant byte first
    // Seems to be only relevant for how the elf stuff gets interpreted
    let data: u8 = 1;
    // Always 1
    let version: u8 = 1;
    // 3: linux, TODO: usually system v -> 0, even on linux
    let os_abi: u8 = 0;
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
        e_version: 1,
        // Entry point of address
        e_entry: entry_point,
        // Program headers offset
        // TODO: All directly in a list after this offset
        e_phoff: 64,
        // TODO: Section headers offset
        e_shoff: 0,
        // Processor specific flags
        e_flags: 0,
        // Size of this elf header, 52 for 32 bit
        e_ehsize: ELF_HEADER_SIZE,
        // size per program header
        e_phentsize: PROGRAM_HEADER_SIZE,
        // number of program headers
        e_phnum: num_pheaders,
        // TODO: size per section header
        e_shentsize: 0,
        // TODO: number of section headers
        e_shnum: 0,
        // TODO: Section header string table index
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
