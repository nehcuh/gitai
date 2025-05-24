// Utility functions
pub fn calculate_hash(content: &str) -> String {
    // Simple hash function to avoid dependency on sha2
    let mut hash: u64 = 0;
    for byte in content.bytes() {
        hash = hash.wrapping_mul(31).wrapping_add(byte as u64);
    }
    format!("{:x}", hash)
}
