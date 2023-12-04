use std::time::Instant;

use fish_hash_bindings::keccak2;

mod fish_hash_bindings;
mod keccak;
mod rust_hash;

fn main() {
    unsafe {
        // compare_hash();

        compare_keccak();
        compare_get_context_light();
        compare_prebuild_dataset();
    }
}

unsafe fn compare_get_context_light() {
    let start_c = Instant::now();
    let context_c = fish_hash_bindings::get_context(false);
    let elapsed_c = start_c.elapsed();

    println!("{:?}", context_c.read().light_cache.read().bytes);

    let start_r = Instant::now();
    let context_r = rust_hash::get_context(false);
    let elapsed_r = start_r.elapsed();

    println!("{:?}", context_r.light_cache[0].0);

    for i in 0..context_r.light_cache.len() {
        assert_eq!(
            context_c.read().light_cache.add(i).read().bytes,
            context_r.light_cache[i].0
        );
    }

    println!(
        "get_context(false): C++  took {:?} milliseconds",
        elapsed_c.as_millis()
    );
    println!(
        "get_context(false): Rust took {:?} milliseconds",
        elapsed_r.as_millis()
    );
}

unsafe fn compare_prebuild_dataset() {
    let num_threads = 8;

    let context_c = fish_hash_bindings::get_context(true);

    let start_c = Instant::now();
    fish_hash_bindings::prebuild_dataset(context_c, num_threads);
    let elapsed_c = start_c.elapsed();

    println!(
        "prebuild_dataset: C++  took {:?} milliseconds",
        elapsed_c.as_millis()
    );

    let context_r = rust_hash::get_context(true);
    let mut dataset = context_r.full_dataset.unwrap();

    let start_r = Instant::now();
    rust_hash::prebuild_dataset(&mut dataset, context_r.light_cache, num_threads as usize);
    let elapsed_r = start_r.elapsed();

    println!(
        "prebuild_dataset: Rust took {:?} milliseconds",
        elapsed_r.as_millis()
    );

    for (i, hash1024) in dataset.iter().enumerate() {
        assert_eq!(
            context_c.read().full_dataset.add(i).read().bytes,
            hash1024.0,
            "index {}",
            i
        );
    }
}

unsafe fn compare_keccak() {
    let input = [3u8; 64];

    let start_c = Instant::now();
    let mut out_c: [u64; 8] = [0; 8];
    keccak2(out_c.as_mut_ptr(), 512, input.as_ptr(), 64);
    let elapsed_c = start_c.elapsed();

    println!("{:?}", out_c);

    let start_r = Instant::now();
    let mut out_r: [u64; 8] = [0; 8];
    keccak::keccak(&mut out_r, 512, input.as_ptr(), 64);
    let elapsed_r = start_r.elapsed();

    println!("{:?}", out_r);

    assert_eq!(out_c, out_r);

    println!("keccak: C++  took {:?} nanoseconds", elapsed_c.as_nanos());
    println!("keccak: Rust took {:?} nanoseconds", elapsed_r.as_nanos());
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
