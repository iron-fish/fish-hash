mod fish_hash_bindings;

fn main() {
    let input = "dsfdsfsdgdaafsd";

    unsafe {
        let context = fish_hash_bindings::get_context(true);
        let output =  hash(context, input);

        // Print the hash as a hex string
        print!("{:02X?}", output);
    }
}

unsafe fn hash(context: *mut fish_hash_bindings::fishhash_context, input: &str) -> [u8; 32] {
        let input_bytes = input.as_bytes();
        let mut output: [u8; 32] = [0; 32];

        fish_hash_bindings::hash(
            output.as_mut_ptr(),
            context,
            input_bytes.as_ptr(),
            input_bytes.len() as u64
        );

        output
}
