use crate::abstract_syntax_tree;
use crate::linking::relocate;

/// Struct containing the raw bytecode and data, still needs to be converted to elf/linked
pub struct CompilationResult {
    pub code: Vec<u8>,
    pub data: Vec<u8>,
    pub code_relocate: Vec<relocate::RelocationEntrie>
}

/// Generates bytecode section, and string section from AST
pub fn compile(program: abstract_syntax_tree::Program) -> CompilationResult {
    let data: Vec<u8> = "Hello world\n".as_bytes().to_vec();
    let data_len: u64 = data.len().try_into().expect("Failed to convert usize to u64");
    let data_len_bytes = data_len.to_le_bytes();
    let code: Vec<u8> = vec![
        //	mov    eax,0x1
        0xb8, 0x01, 0x00, 0x00, 0x00,       
        //	mov    edi,0x1
        0xbf, 0x01, 0x00, 0x00, 0x00,       
        //	movabs rsi, pointer to string beginning of data section
        0x48, 0xbe, 0x00, 0x00, 0x00, 0x00, 0x00, 
        0x00, 0x00, 0x00,
        //	mov    edx, length of string
        0xba, data_len_bytes[0], data_len_bytes[1], data_len_bytes[2], data_len_bytes[3], 
        //	syscall
        0x0f, 0x05,                
        //	mov    eax,0x3c
        0xb8, 0x3c, 0x00, 0x00, 0x00,       
        //	mov    edi, exit code: 0x0
        0xbf, 0x00, 0x00, 0x00, 0x00,       
        //	syscall
        0x0f, 0x05                
    ];

    let relocate = vec![
        relocate::RelocationEntrie{
            index: 12,
            bytes: 4
        }
    ];

    CompilationResult {
        code,
        data,
        code_relocate: relocate
    }
}
