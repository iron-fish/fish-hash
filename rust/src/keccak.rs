use keccak::f1600;
use std::mem::size_of;

pub unsafe fn keccak_in_place(data: &mut [u8], bits: usize, mut size: usize) {
    _keccak(data, bits, data.as_ptr(), size)
}

pub unsafe fn keccak(out: &mut [u8], bits: usize, data: &[u8], mut size: usize) {
    _keccak(out, bits, data.as_ptr(), size)
}

unsafe fn _keccak(
    out: &mut [u8],
    bits: usize, // TODO: This can probably be calculated from somewhere, or hard-coded
    data_ptr: *const u8,
    mut size: usize,
) {
    const WORD_SIZE: usize = size_of::<u64>();
    let hash_size = bits / 8;
    let block_size = (1600 - bits * 2) / 8;

    let mut state_iter: *mut u64;
    let mut last_word: u64 = 0;
    let mut last_word_iter: *mut u8 = (&mut last_word as *mut u64).cast();

    let mut state: [u64; 25] = [0; 25];
    let mut data = data_ptr;

    while size >= block_size {
        for i in 0..(block_size / WORD_SIZE) {
            state[i] ^= load_le(data);
            data = data.add(WORD_SIZE);
        }

        f1600(&mut state);

        size -= block_size;
    }

    state_iter = state.as_mut_ptr();

    while size >= WORD_SIZE {
        *state_iter ^= load_le(data);
        state_iter = state_iter.add(1);
        data = data.add(WORD_SIZE);
        size -= WORD_SIZE;
    }

    while size > 0 {
        *last_word_iter = *data;
        last_word_iter = last_word_iter.add(1);
        data = data.add(1);
        size -= 1;
    }
    *last_word_iter = 0x01;
    *state_iter ^= last_word.to_le();

    state[(block_size / WORD_SIZE) - 1] ^= 0x8000000000000000;

    f1600(&mut state);

    for i in 0..(hash_size / WORD_SIZE) {
        let index = i * 8;
        out[index..index + WORD_SIZE].copy_from_slice(&state[i].to_le_bytes());
    }
}

#[inline]
unsafe fn load_le(data: *const u8) -> u64 {
    let mut word: u64 = 0;
    data.copy_to_nonoverlapping((&mut word as *mut u64).cast(), std::mem::size_of::<u64>());
    word.to_le()
}
