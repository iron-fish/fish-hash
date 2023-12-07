use std::time::Instant;

use fish_hash_bindings::keccak2;

mod fish_hash_bindings;
mod keccak;
mod rust_hash;

fn main() {
    unsafe {
        compare_keccak();
        compare_get_context_light();
        compare_validation();
        compare_hash(false);
        // compare_hash(true);
        // compare_prebuild_dataset();
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
    rust_hash::prebuild_dataset(&mut dataset, &context_r.light_cache, num_threads as usize);
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

    let mut out_c_bytes = [0u8; 64];

    let start_c = Instant::now();
    let mut out_c: [u64; 8] = [0; 8];
    keccak2(
        out_c.as_mut_ptr(),
        512,
        input.as_ptr(),
        input.len() as isize,
    );
    let elapsed_c = start_c.elapsed();

    for (index, i) in out_c.iter().enumerate() {
        out_c_bytes[index * 8..index * 8 + 8].copy_from_slice(&i.to_le_bytes());
    }

    println!("{:?}", out_c_bytes);

    let start_r = Instant::now();
    let mut out_r: [u8; 64] = [0; 64];
    keccak::keccak(&mut out_r, &input);
    let elapsed_r = start_r.elapsed();

    println!("{:?}", out_r);

    assert_eq!(out_c_bytes, out_r);

    println!("keccak: C++  took {:?} nanoseconds", elapsed_c.as_nanos());
    println!("keccak: Rust took {:?} nanoseconds", elapsed_r.as_nanos());
}

unsafe fn compare_validation() {
    let inputs = vec![
        "dsfdsfsdgdaafsd",
        "the quick brown fox jumps over the lazy dog",
        "zxbcnmv,ahjsdklfeiwuopqr78309241-turhgeiwaov89b76zxcajhsdklfb423qkjlr",
        "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat.",
    ];

    let context_c = fish_hash_bindings::get_context(false);
    let mut context_r = rust_hash::get_context(false);

    for input in inputs {
        println!("Validating {:?}", input);

        let start_c = Instant::now();
        let output_c = hash_c(context_c, input);
        let elapsed_c = start_c.elapsed();

        // Print the hash as a hex string
        println!("C++ : {:02X?}", output_c);

        let start_r = Instant::now();
        let mut output_r = [0u8; 32];
        rust_hash::hash(&mut output_r, &mut context_r, input.as_bytes());
        let elapsed_r = start_r.elapsed();

        println!("Rust: {:02X?}", output_r);

        println!(
            "hash(light): C++  took {:?} microseconds",
            elapsed_c.as_micros()
        );
        println!(
            "hash(light): Rust took {:?} microseconds",
            elapsed_r.as_micros()
        );

        assert_eq!(output_c, output_r);
    }
}

unsafe fn compare_hash(prebuild: bool) {
    let inputs = vec![
        "dsfdsfsdgdaafsd",
        "the quick brown fox jumps over the lazy dog",
        "zxbcnmv,ahjsdklfeiwuopqr78309241-turhgeiwaov89b76zxcajhsdklfb423qkjlr",
        "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat.",
    ];

    let num_threads = 8;

    let context_c = fish_hash_bindings::get_context(true);
    let mut context_r = rust_hash::get_context(true);
    let dataset = context_r.full_dataset.as_mut().unwrap();

    if prebuild {
        fish_hash_bindings::prebuild_dataset(context_c, num_threads);
        rust_hash::prebuild_dataset(dataset, &context_r.light_cache, num_threads as usize);
    }

    for input in inputs {
        println!("Hashing {:?}", input);

        let start_c = Instant::now();
        let output_c = hash_c(context_c, input);
        let elapsed_c = start_c.elapsed();

        // Print the hash as a hex string
        println!("C++ : {:02X?}", output_c);

        let start_r = Instant::now();
        let mut output_r = [0u8; 32];
        rust_hash::hash(&mut output_r, &mut context_r, input.as_bytes());
        let elapsed_r = start_r.elapsed();

        println!("Rust: {:02X?}", output_r);

        println!("hash: C++  took {:?} microseconds", elapsed_c.as_micros());
        println!("hash: Rust took {:?} microseconds", elapsed_r.as_micros());

        assert_eq!(output_c, output_r);

        for (i, hash1024) in context_r.full_dataset.as_ref().unwrap().iter().enumerate() {
            assert_eq!(
                context_c.read().full_dataset.add(i).read().bytes,
                hash1024.0,
                "index {}",
                i
            );
        }
    }
}

unsafe fn hash_c(context: *mut fish_hash_bindings::fishhash_context, input: &str) -> [u8; 32] {
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
