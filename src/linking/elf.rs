#![allow(non_camel_case_types)]

const EI_NIDENT: usize = 16;

type Elf64_Half = u16; // TODO: alignment 2
type Elf64_Word = u32; // TODO: alignment 4
type Elf64_Xword = u64; // TODO: alignment 4
type Elf64_Addr = u64; // TODO: alignment 8
type Elf64_Off = u64; // TODO: alignment 8 

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
    let header = create_header();
    header_to_bytes(header).to_vec()
}

fn create_header() -> Elf64_Ehdr {
    let magic_number: u8 = 0x7f;
    // 64 bit objects
    let class: u8 = 2;
    // Most significatn byte first TODO: weet niet of dit goed is
    let data: u8 = 2;
    // Always 1
    let version: u8 = 1;
    // 3: linux, TODO: usually system v -> 0, even on linux
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
        e_version: 1,
        // TODO: Entry point of address
        e_entry: 0,
        // TODO: Program headers offset
        // All directly in a list after this offset
        e_phoff: 0,
        // TODO: Section headers offset
        e_shoff: 0,
        // Processor specific flags
        e_flags: 0,
        // Size of this elf header, 52 for 32 bit
        // TODO: alignment might change size here?
        e_ehsize: 64,
        // TODO: size per program header
        e_phentsize: 0,
        // TODO: number of program headers
        e_phnum: 0,
        // TODO: size per section header
        e_shentsize: 0,
        // TODO: number of section headers
        e_shnum: 0,
        // TODO: Section header string table index
        // Used to resolve names of sections in the file
        e_shstrndx: 0,
    }
}

fn header_to_bytes(header: Elf64_Ehdr) -> [u8; 64] {
    copy_into(vec![
        &header.e_ident,
        &header.e_type.to_be_bytes(),
        &header.e_machine.to_be_bytes(),
        &header.e_version.to_be_bytes(),
        &header.e_entry.to_be_bytes(),
        &header.e_phoff.to_be_bytes(),
        &header.e_shoff.to_be_bytes(),
        &header.e_flags.to_be_bytes(),
        &header.e_ehsize.to_be_bytes(),
        &header.e_phentsize.to_be_bytes(),
        &header.e_phnum.to_be_bytes(),
        &header.e_shentsize.to_be_bytes(),
        &header.e_shnum.to_be_bytes(),
        &header.e_shstrndx.to_be_bytes(),
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
