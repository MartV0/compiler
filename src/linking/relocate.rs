use crate::assembling::AssemblingResult;
/// For this module I took some inspiration from here: 
/// https://refspecs.linuxbase.org/elf/gabi4+/ch4.reloc.html
use crate::linking::elf::SegmentType;

/// Corresponds to an address somewhere that needs to be fixed during linking
#[derive(Debug)]
pub struct RelocationEntrie {
    /// Index of address to relocate in the segment, measured from the beginning
    pub offset: usize,
    /// How much bytes, the address contains that needs to be relocated
    /// Usually 8 (64 bits)
    pub bytes: u8,
    /// Which segment the relocation should be based on
    /// This is the segment the address refers to
    pub segment: SegmentType
}

/// Relocate address listed in entries
/// Changes addresses that are currently relative to the beginning of a segment
/// to their proper location in the virtual memory
pub fn relocate(input: &mut AssemblingResult, code_base_address: u64, data_base_address: u64) {
    let AssemblingResult { code, data: _data, code_relocate } = input;
    relocate_segment(code, code_relocate, code_base_address, data_base_address);
}

fn relocate_segment(file: &mut Vec<u8>, entries: &Vec<RelocationEntrie>, code_base_address: u64, data_base_address: u64) {
    for RelocationEntrie { offset, bytes, segment } in entries {
        let bytes = (*bytes).into();
        let offset = *offset;
        // Convert the address to u64
        let mut address = [0; 8];
        address[0..bytes].clone_from_slice(&file[offset..offset + bytes]);
        let address: u64 = u64::from_le_bytes(address);

        let new_address = match segment {
            SegmentType::Data => data_base_address,
            SegmentType::Text => code_base_address
        } + address;

        // Write the new address back as little endian bytes
        file[offset..offset + bytes].clone_from_slice(&new_address.to_le_bytes()[0..bytes]);
    }
}

