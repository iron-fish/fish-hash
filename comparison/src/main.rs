use std::time::Instant;

use fish_hash::HashData;

mod fish_hash_bindings;

fn main() {
    unsafe {
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
    let context_r = fish_hash::Context::new(false);
    let elapsed_r = start_r.elapsed();

    println!("{:?}", context_r.light_cache[0].as_bytes());

    for i in 0..context_r.light_cache.len() {
        assert_eq!(
            context_c.read().light_cache.add(i).read().bytes,
            context_r.light_cache[i].as_bytes()
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

#[allow(dead_code)]
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

    let mut context_r = fish_hash::Context::new(true);

    let start_r = Instant::now();
    context_r.prebuild_dataset(num_threads as usize);
    let elapsed_r = start_r.elapsed();

    println!(
        "prebuild_dataset: Rust took {:?} milliseconds",
        elapsed_r.as_millis()
    );

    for (i, hash1024) in context_r.full_dataset.as_ref().unwrap().iter().enumerate() {
        assert_eq!(
            context_c.read().full_dataset.add(i).read().bytes,
            hash1024.as_bytes(),
            "index {}",
            i
        );
    }
}

unsafe fn compare_validation() {
    let inputs = vec![
        "dsfdsfsdgdaafsd",
        "the quick brown fox jumps over the lazy dog",
        "zxbcnmv,ahjsdklfeiwuopqr78309241-turhgeiwaov89b76zxcajhsdklfb423qkjlr",
        "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat.",
    ];

    let context_c = fish_hash_bindings::get_context(false);
    let mut context_r = fish_hash::Context::new(false);

    for input in inputs {
        println!("Validating {:?}", input);

        let start_c = Instant::now();
        let output_c = hash_c(context_c, input);
        let elapsed_c = start_c.elapsed();

        // Print the hash as a hex string
        println!("C++ : {:02X?}", output_c);

        let start_r = Instant::now();
        let mut output_r = [0u8; 32];
        fish_hash::hash(&mut output_r, &mut context_r, input.as_bytes());
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
    let mut context_r = fish_hash::Context::new(true);

    if prebuild {
        fish_hash_bindings::prebuild_dataset(context_c, num_threads);
        context_r.prebuild_dataset(num_threads as usize);
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
        fish_hash::hash(&mut output_r, &mut context_r, input.as_bytes());
        let elapsed_r = start_r.elapsed();

        println!("Rust: {:02X?}", output_r);

        println!("hash: C++  took {:?} microseconds", elapsed_c.as_micros());
        println!("hash: Rust took {:?} microseconds", elapsed_r.as_micros());

        assert_eq!(output_c, output_r);

        for (i, hash1024) in context_r.full_dataset.as_ref().unwrap().iter().enumerate() {
            assert_eq!(
                context_c.read().full_dataset.add(i).read().bytes,
                hash1024.as_bytes(),
                "index {}",
                i
            );
        }
    }
}

unsafe fn hash_c(context: *mut fish_hash_bindings::FishhashContext, input: &str) -> [u8; 32] {
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
