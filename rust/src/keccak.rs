use keccak::f1600;
use std::mem::size_of;

use crate::rust_hash::HashData;

// hash512 item;
// keccak(item.word64s, 512, seed.bytes, sizeof(seed))
// inline void keccak(uint64_t* out, size_t bits, const uint8_t* data, size_t size)
pub unsafe fn keccak<const T: usize, const N: usize>(
    out: &mut [u64],
    bits: usize, // TODO: This can probably be calculated from somewhere, or hard-coded
    data: &impl HashData<T, N>,
    mut size: usize,
) {
    const WORD_SIZE: usize = size_of::<u64>();
    let hash_size = bits / 8;
    let block_size = (1600 - bits * 2) / 8;

    let mut state_iter: *mut u64;
    let mut last_word: u64 = 0;
    let mut last_word_iter: *mut u8 = (&mut last_word as *mut u64).cast();

    let mut state: [u64; 25] = [0; 25];

    let mut data_ptr = data.as_ptr();

    while size >= block_size {
        for i in 0..(block_size / WORD_SIZE) {
            state[i] ^= load_le(data_ptr);
            data_ptr = data_ptr.add(WORD_SIZE);
        }

        f1600(&mut state);

        size -= block_size;
    }

    state_iter = state.as_mut_ptr();

    while size >= WORD_SIZE {
        *state_iter ^= load_le(data_ptr);
        state_iter = state_iter.add(1);
        data_ptr = data_ptr.add(WORD_SIZE);
        size -= WORD_SIZE;
    }

    while size > 0 {
        *last_word_iter = *data_ptr;
        last_word_iter = last_word_iter.add(1);
        data_ptr = data_ptr.add(1);
        size -= 1;
    }
    *last_word_iter = 0x01;
    *state_iter ^= last_word.to_le();

    state[(block_size / WORD_SIZE) - 1] ^= 0x8000000000000000;

    f1600(&mut state);

    for i in 0..(hash_size / WORD_SIZE) {
        out[i] = state[i].to_le();
    }
}

#[inline]
unsafe fn load_le(data: *const u8) -> u64 {
    let mut word: u64 = 0;
    data.copy_to_nonoverlapping((&mut word as *mut u64).cast(), std::mem::size_of::<u64>());
    word.to_le()
}
