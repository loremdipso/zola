// TODO: use good hash
const VERSION: u64 = 1;

pub fn get_stable_hash(s: &str) -> u64 {
    let mut hash = 14695981039346656037_u64; // FNV offset basis
    for byte in s.as_bytes() {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(1099511628211_u64); // FNV prime
    }
    hash + VERSION
}
