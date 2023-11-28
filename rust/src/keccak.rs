use keccak::f1600;
use std::mem::size_of;

use crate::rust_hash::HashData;

// hash512 item;
// keccak(item.word64s, 512, seed.bytes, sizeof(seed))
// inline void keccak(uint64_t* out, size_t bits, const uint8_t* data, size_t size)
pub unsafe fn keccak<const T: usize, const N: usize>(
    out: &[u64],
    bits: usize,
    data: &mut impl HashData<T, N>,
    mut size: usize,
) {
    // pub unsafe fn keccak(out: &[u64], bits: usize, data: &[u8], size: usize) {
    const WORD_SIZE: usize = size_of::<u64>();
    let hash_size = bits / 8;
    let block_size = (1600 - bits * 2) / 8;

    // let mut i: usize;
    // let mut state_iter: &u64;
    // let mut last_word: u64 = 0;

    let data_64s = data.as_64s();

    let mut state: [u64; 25] = [0; 25];

    while size >= block_size {
        for i in 0..(block_size / WORD_SIZE) {
            state[i] ^= data_64s[i].to_le();
        }

        f1600(&mut state);

        size -= block_size;
    }

    todo!()
}
