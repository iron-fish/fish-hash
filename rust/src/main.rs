use fish_hash_bindings::keccak2;

use crate::rust_hash::Hash512;

mod fish_hash_bindings;
mod keccak;
mod rust_hash;

fn main() {
    let input = "dsfdsfsdgdaafsd";

    unsafe {
        // let context = fish_hash_bindings::get_context(true);
        // let output = hash(context, input);

        // // Print the hash as a hex string
        // println!("{:02X?}", output);

        let mut out2: [u64; 8] = [0; 8];
        let item2 = [3u8; 64];
        keccak2(out2.as_mut_ptr(), 512, item2.as_ptr(), 64);

        println!("{:?}", out2);

        let mut out3: [u64; 8] = [0; 8];
        let mut item3 = Hash512([3u8; 64]);
        keccak::keccak(&mut out3, 512, &mut item3, 64);

        println!("{:?}", out3);

        assert_eq!(out2, out3);
    }
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
