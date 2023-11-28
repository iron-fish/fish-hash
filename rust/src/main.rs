use fish_hash_bindings::keccak2;

use crate::rust_hash::Hash512;

mod fish_hash_bindings;
mod keccak;
mod rust_hash;

fn main() {
    unsafe {
        // compare_hash();

        // compare_keccak();

        compare_get_context_light();
    }
}

unsafe fn compare_get_context_light() {
    let context_c = fish_hash_bindings::get_context(false);
    println!("{:?}", context_c.read().light_cache.read().bytes);

    let context_r = rust_hash::get_context(false);
    println!("{:?}", context_r.light_cache[0].0);

    for i in 0..context_r.light_cache.len() {
        assert_eq!(
            context_c.read().light_cache.add(i).read().bytes,
            context_r.light_cache[i].0
        );
    }
}

unsafe fn compare_keccak() {
    let input = [3u8; 64];

    let mut out_c: [u64; 8] = [0; 8];
    keccak2(out_c.as_mut_ptr(), 512, input.as_ptr(), 64);

    println!("{:?}", out_c);

    let mut out_r: [u64; 8] = [0; 8];
    keccak::keccak(&mut out_r, 512, input.as_ptr(), 64);

    println!("{:?}", out_r);

    assert_eq!(out_c, out_r);
}

unsafe fn compare_hash() {
    let input = "dsfdsfsdgdaafsd";
    let context = fish_hash_bindings::get_context(true);
    let output = hash(context, input);

    // Print the hash as a hex string
    println!("{:02X?}", output);
}

unsafe fn hash(context: *mut fish_hash_bindings::fishhash_context, input: &str) -> [u8; 32] {
    let input_bytes = input.as_bytes();
    let mut output: [u8; 32] = [0; 32];

    fish_hash_bindings::hash(
        output.as_mut_ptr(),
        context,
        input_bytes.as_ptr(),
        input_bytes.len() as u64,
    );

    output
}
