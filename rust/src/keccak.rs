use tiny_keccak::{self, Hasher};

pub fn keccak_in_place(data: &mut [u8]) {
    let mut hasher = tiny_keccak::Keccak::v512();
    hasher.update(data);
    hasher.finalize(data);
}

pub fn keccak(out: &mut [u8], data: &[u8]) {
    let mut hasher = tiny_keccak::Keccak::v512();
    hasher.update(data);
    hasher.finalize(out);
}
