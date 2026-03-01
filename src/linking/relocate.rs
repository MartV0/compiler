/// Corresponds to an address somewhere that needs to be fixed during linking
pub struct RelocationEntrie {
    /// Index of address to relocate in the byte vec
    pub index: usize,
    /// How much bytes, the address contains that need to be relocated
    /// Usually 8 (64 bits)
    pub bytes: u8,
}

/// Relocate address listed in entries
pub fn relocate(file: &mut Vec<u8>, entries: Vec<RelocationEntrie>, base_address: u64) {
    for RelocationEntrie { index, bytes } in entries {
        let bytes = bytes.into();
        // Convert the address to u64
        let mut address = [0; 8];
        address[0..bytes].clone_from_slice(&file[index..index + bytes]);
        let address: u64 = u64::from_le_bytes(address);

        let new_address = base_address + address;

        // Write the new address back as little endian bytes
        file[index..index + bytes].clone_from_slice(&new_address.to_le_bytes()[0..bytes]);
    }
}

