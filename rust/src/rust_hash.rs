use blake3::Hasher;

use crate::keccak::{keccak, keccak_in_place};

// TODO: These are static in the c++ version
const FNV_PRIME: u32 = 0x01000193;
const FULL_DATASET_ITEM_PARENTS: u32 = 512;
const NUM_DATASET_ACCESSES: i32 = 32;
const LIGHT_CACHE_ROUNDS: i32 = 3;

const LIGHT_CACHE_NUM_ITEMS: u32 = 1179641;
const FULL_DATASET_NUM_ITEMS: u32 = 37748717;
const SEED: Hash256 = Hash256([
    0xeb, 0x01, 0x63, 0xae, 0xf2, 0xab, 0x1c, 0x5a, 0x66, 0x31, 0x0c, 0x1c, 0x14, 0xd6, 0x0f, 0x42,
    0x55, 0xa9, 0xb3, 0x9b, 0x0e, 0xdf, 0x26, 0x53, 0x98, 0x44, 0xf1, 0x17, 0xad, 0x67, 0x21, 0x19,
]);

const SIZE_U32: usize = std::mem::size_of::<u32>();
const SIZE_U64: usize = std::mem::size_of::<u64>();

pub trait HashData {
    fn new() -> Self;
    fn get_as_u32(&self, index: usize) -> u32;
    fn set_as_u32(&mut self, index: usize, value: u32);
    fn get_as_u64(&self, index: usize) -> u64;
    fn set_as_u64(&mut self, index: usize, value: u64);
}

#[derive(Debug)]
pub struct Hash256([u8; 32]);

impl HashData for Hash256 {
    fn new() -> Self {
        Self([0; 32])
    }

    fn get_as_u32(&self, index: usize) -> u32 {
        u32::from_le_bytes(
            self.0[index * SIZE_U32..index * SIZE_U32 + SIZE_U32]
                .try_into()
                .unwrap(),
        )
    }

    fn set_as_u32(&mut self, index: usize, value: u32) {
        self.0[index * SIZE_U32..index * SIZE_U32 + SIZE_U32].copy_from_slice(&value.to_le_bytes())
    }

    fn get_as_u64(&self, index: usize) -> u64 {
        u64::from_le_bytes(
            self.0[index * SIZE_U64..index * SIZE_U64 + SIZE_U64]
                .try_into()
                .unwrap(),
        )
    }

    fn set_as_u64(&mut self, index: usize, value: u64) {
        self.0[index * SIZE_U64..index * SIZE_U64 + SIZE_U64].copy_from_slice(&value.to_le_bytes())
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Hash512(pub [u8; 64]);

impl HashData for Hash512 {
    fn new() -> Self {
        Self([0; 64])
    }

    fn get_as_u32(&self, index: usize) -> u32 {
        u32::from_le_bytes(
            self.0[index * SIZE_U32..index * SIZE_U32 + SIZE_U32]
                .try_into()
                .unwrap(),
        )
    }

    fn set_as_u32(&mut self, index: usize, value: u32) {
        self.0[index * SIZE_U32..index * SIZE_U32 + SIZE_U32].copy_from_slice(&value.to_le_bytes())
    }

    fn get_as_u64(&self, index: usize) -> u64 {
        u64::from_le_bytes(
            self.0[index * SIZE_U64..index * SIZE_U64 + SIZE_U64]
                .try_into()
                .unwrap(),
        )
    }

    fn set_as_u64(&mut self, index: usize, value: u64) {
        self.0[index * SIZE_U64..index * SIZE_U64 + SIZE_U64].copy_from_slice(&value.to_le_bytes())
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Hash1024(pub [u8; 128]);

impl HashData for Hash1024 {
    fn new() -> Self {
        Self([0; 128])
    }

    fn get_as_u32(&self, index: usize) -> u32 {
        u32::from_le_bytes(
            self.0[index * SIZE_U32..index * SIZE_U32 + SIZE_U32]
                .try_into()
                .unwrap(),
        )
    }

    fn set_as_u32(&mut self, index: usize, value: u32) {
        self.0[index * SIZE_U32..index * SIZE_U32 + SIZE_U32].copy_from_slice(&value.to_le_bytes())
    }

    fn get_as_u64(&self, index: usize) -> u64 {
        u64::from_le_bytes(
            self.0[index * SIZE_U64..index * SIZE_U64 + SIZE_U64]
                .try_into()
                .unwrap(),
        )
    }

    fn set_as_u64(&mut self, index: usize, value: u64) {
        self.0[index * SIZE_U64..index * SIZE_U64 + SIZE_U64].copy_from_slice(&value.to_le_bytes())
    }
}

impl Hash1024 {
    fn from_512s(first: &Hash512, second: &Hash512) -> Self {
        let mut hash = Self::new();
        let (first_half, second_half) = hash.0.split_at_mut(first.0.len());
        first_half.copy_from_slice(&first.0);
        second_half.copy_from_slice(&second.0);

        hash
    }
}

pub struct Context {
    pub light_cache: Box<[Hash512]>,
    pub full_dataset: Option<Box<[Hash1024]>>,
}

impl Context {
    pub fn new(full: bool) -> Self {
        // Vec into boxed sliced, because you can't allocate an array directly on
        // the heap in rust
        // https://stackoverflow.com/questions/25805174/creating-a-fixed-size-array-on-heap-in-rust/68122278#68122278
        let mut light_cache =
            vec![Hash512::new(); LIGHT_CACHE_NUM_ITEMS as usize].into_boxed_slice();
        build_light_cache(&mut light_cache);

        let full_dataset = if full {
            Some(vec![Hash1024::new(); FULL_DATASET_NUM_ITEMS as usize].into_boxed_slice())
        } else {
            None
        };

        Context {
            light_cache,
            full_dataset,
        }
    }

    pub fn prebuild_dataset(&mut self, num_threads: usize) {
        if self.full_dataset.is_none() {
            return;
        }

        let full_dataset = self.full_dataset.as_mut().unwrap();

        if num_threads > 1 {
            std::thread::scope(|scope| {
                let batch_size = full_dataset.len() / num_threads;

                let mut threads = Vec::with_capacity(num_threads);

                let chunks = full_dataset.chunks_mut(batch_size);

                let light_cache_slice = &self.light_cache[0..];

                for (index, chunk) in chunks.enumerate() {
                    let start = index * batch_size;

                    let thread_handle =
                        scope.spawn(move || build_dataset_segment(chunk, light_cache_slice, start));
                    threads.push(thread_handle);
                }

                for handle in threads {
                    handle.join().unwrap();
                }
            });
        } else {
            build_dataset_segment(&mut full_dataset[0..], &self.light_cache, 0);
        }
    }
}

fn build_dataset_segment(dataset_slice: &mut [Hash1024], light_cache: &[Hash512], offset: usize) {
    for (index, item) in dataset_slice.iter_mut().enumerate() {
        *item = calculate_dataset_item_1024(light_cache, offset + index);
    }
}

fn fnv1(u: u32, v: u32) -> u32 {
    (u * FNV_PRIME) ^ v
}

fn fnv1_512(u: Hash512, v: Hash512) -> Hash512 {
    let mut r = Hash512::new();

    for i in 0..r.0.len() / SIZE_U32 {
        r.set_as_u32(i, fnv1(u.get_as_u32(i), v.get_as_u32(i)));
    }

    r
}

fn calculate_dataset_item_1024(light_cache: &[Hash512], index: usize) -> Hash1024 {
    let seed0 = (index * 2) as u32;
    let seed1 = seed0 + 1;

    let mut mix0 = light_cache[(seed0 % LIGHT_CACHE_NUM_ITEMS) as usize];
    let mut mix1 = light_cache[(seed1 % LIGHT_CACHE_NUM_ITEMS) as usize];

    let mix0_seed = mix0.get_as_u32(0) ^ seed0;
    let mix1_seed = mix1.get_as_u32(0) ^ seed1;

    mix0.set_as_u32(0, mix0_seed);
    mix1.set_as_u32(0, mix1_seed);

    keccak_in_place(&mut mix0.0);
    keccak_in_place(&mut mix1.0);

    const NUM_WORDS: u32 = 16; // TODO: not sure why this was calculated dynamically in C++
    for j in 0..FULL_DATASET_ITEM_PARENTS {
        let t0 = fnv1(seed0 ^ j, mix0.get_as_u32((j % NUM_WORDS) as usize));
        let t1 = fnv1(seed1 ^ j, mix1.get_as_u32((j % NUM_WORDS) as usize));
        mix0 = fnv1_512(mix0, light_cache[(t0 % LIGHT_CACHE_NUM_ITEMS) as usize]);
        mix1 = fnv1_512(mix1, light_cache[(t1 % LIGHT_CACHE_NUM_ITEMS) as usize]);
    }

    keccak_in_place(&mut mix0.0);
    keccak_in_place(&mut mix1.0);

    Hash1024::from_512s(&mix0, &mix1)
}

pub fn hash(output: &mut [u8], context: &mut Context, header: &[u8]) {
    let mut seed: Hash512 = Hash512::new();

    let mut hasher = Hasher::new();
    hasher.update(header);
    let mut output_reader = hasher.finalize_xof();
    output_reader.fill(&mut seed.0);

    let mix_hash = fishhash_kernel(context, &seed);

    let mut final_data: [u8; 96] = [0; 96];
    final_data[0..64].copy_from_slice(&seed.0);
    final_data[64..].copy_from_slice(&mix_hash.0);

    let hash = blake3::hash(&final_data);
    output.copy_from_slice(hash.as_bytes());
}

fn fishhash_kernel(context: &mut Context, seed: &Hash512) -> Hash256 {
    let mut mix = Hash1024::from_512s(seed, seed);

    for _ in 0..NUM_DATASET_ACCESSES as usize {
        // Calculate new fetching indexes
        let p0 = mix.get_as_u32(0) % FULL_DATASET_NUM_ITEMS;
        let p1 = mix.get_as_u32(4) % FULL_DATASET_NUM_ITEMS;
        let p2 = mix.get_as_u32(8) % FULL_DATASET_NUM_ITEMS;

        let fetch0 = lookup(context, p0 as usize);
        let mut fetch1 = lookup(context, p1 as usize);
        let mut fetch2 = lookup(context, p2 as usize);

        // Modify fetch1 and fetch2
        for j in 0..32 {
            fetch1.set_as_u32(j, fnv1(mix.get_as_u32(j), fetch1.get_as_u32(j)));
            fetch2.set_as_u32(j, mix.get_as_u32(j) ^ fetch2.get_as_u32(j));
        }

        // Final computation of new mix
        for j in 0..16 {
            mix.set_as_u64(
                j,
                fetch0.get_as_u64(j) * fetch1.get_as_u64(j) + fetch2.get_as_u64(j),
            );
        }
    }

    // Collapse the result into 32 bytes
    let mut mix_hash = Hash256::new();
    let num_words = std::mem::size_of_val(&mix) / std::mem::size_of::<u32>();

    for i in (0..num_words).step_by(4) {
        let h1 = fnv1(mix.get_as_u32(i), mix.get_as_u32(i + 1));
        let h2 = fnv1(h1, mix.get_as_u32(i + 2));
        let h3 = fnv1(h2, mix.get_as_u32(i + 3));
        mix_hash.set_as_u32(i / 4, h3);
    }

    mix_hash
}

fn lookup(context: &mut Context, index: usize) -> Hash1024 {
    match &mut context.full_dataset {
        Some(dataset) => {
            let item = &mut dataset[index];
            if item.get_as_u64(0) == 0 {
                *item = calculate_dataset_item_1024(&context.light_cache, index);
            }

            *item
        }
        None => calculate_dataset_item_1024(&context.light_cache, index),
    }
}

fn build_light_cache(cache: &mut [Hash512]) {
    let mut item: Hash512 = Hash512::new();
    keccak(&mut item.0, &SEED.0);
    cache[0] = item;

    for cache_item in cache
        .iter_mut()
        .take(LIGHT_CACHE_NUM_ITEMS as usize)
        .skip(1)
    {
        keccak_in_place(&mut item.0);
        *cache_item = item;
    }

    for _ in 0..LIGHT_CACHE_ROUNDS {
        for i in 0..LIGHT_CACHE_NUM_ITEMS {
            // First index: 4 first bytes of the item as little-endian integer
            let t: u32 = u32::from_le_bytes(cache[i as usize].0[0..4].try_into().unwrap());
            let v: u32 = t % LIGHT_CACHE_NUM_ITEMS;

            // Second index
            let w: u32 =
                (LIGHT_CACHE_NUM_ITEMS.wrapping_add(i.wrapping_sub(1))) % LIGHT_CACHE_NUM_ITEMS;

            let x: Hash512 = bitwise_xor(&cache[v as usize], &cache[w as usize]);
            keccak(&mut cache[i as usize].0, &x.0);
        }
    }
}

// TODO: Pretty sure this will work for both big and little endian
// but we should test it
fn bitwise_xor(x: &Hash512, y: &Hash512) -> Hash512 {
    let mut z: Hash512 = Hash512::new();

    for i in 0..64 {
        z.0[i] = x.0[i] ^ y.0[i];
    }

    z
}
