use blake3::Hasher;

use crate::keccak::{keccak, keccak_in_place};

// TODO: These are static in the c++ version
const FNV_PRIME: u32 = 0x01000193;
const FULL_DATASET_ITEM_PARENTS: u32 = 512;
const NUM_DATASET_ACCESSES: i32 = 32;
const LIGHT_CACHE_ROUNDS: i32 = 3;

const LIGHT_CACHE_NUM_ITEMS: usize = 1179641;
const FULL_DATASET_NUM_ITEMS: usize = 37748717;
const SEED: Hash256 = Hash256([
    0xeb, 0x01, 0x63, 0xae, 0xf2, 0xab, 0x1c, 0x5a, 0x66, 0x31, 0x0c, 0x1c, 0x14, 0xd6, 0x0f, 0x42,
    0x55, 0xa9, 0xb3, 0x9b, 0x0e, 0xdf, 0x26, 0x53, 0x98, 0x44, 0xf1, 0x17, 0xad, 0x67, 0x21, 0x19,
]);

pub trait HashData<const U64_SIZE: usize, const U32_SIZE: usize> {
    unsafe fn as_64s_mut(&mut self) -> &mut [u64; U64_SIZE];
    unsafe fn as_32s_mut(&mut self) -> &mut [u32; U32_SIZE];
    unsafe fn as_64s(&self) -> &[u64; U64_SIZE];
    unsafe fn as_32s(&self) -> &[u32; U32_SIZE];
}

#[derive(Debug)]
pub struct Hash256([u8; 32]);
impl HashData<4, 8> for Hash256 {
    unsafe fn as_64s_mut(&mut self) -> &mut [u64; 4] {
        std::mem::transmute::<&mut [u8; 32], &mut [u64; 4]>(&mut self.0)
    }

    unsafe fn as_32s_mut(&mut self) -> &mut [u32; 8] {
        std::mem::transmute::<&mut [u8; 32], &mut [u32; 8]>(&mut self.0)
    }

    unsafe fn as_64s(&self) -> &[u64; 4] {
        std::mem::transmute::<&[u8; 32], &[u64; 4]>(&self.0)
    }

    unsafe fn as_32s(&self) -> &[u32; 8] {
        std::mem::transmute::<&[u8; 32], &[u32; 8]>(&self.0)
    }
}

// TODO: We really dont want clone/copy here probably
#[derive(Clone, Copy, Debug)]
pub struct Hash512(pub [u8; 64]);
impl HashData<8, 16> for Hash512 {
    unsafe fn as_64s_mut(&mut self) -> &mut [u64; 8] {
        std::mem::transmute::<&mut [u8; 64], &mut [u64; 8]>(&mut self.0)
    }

    unsafe fn as_32s_mut(&mut self) -> &mut [u32; 16] {
        std::mem::transmute::<&mut [u8; 64], &mut [u32; 16]>(&mut self.0)
    }

    unsafe fn as_64s(&self) -> &[u64; 8] {
        std::mem::transmute::<&[u8; 64], &[u64; 8]>(&self.0)
    }

    unsafe fn as_32s(&self) -> &[u32; 16] {
        std::mem::transmute::<&[u8; 64], &[u32; 16]>(&self.0)
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Hash1024(pub [u8; 128]);
impl HashData<16, 32> for Hash1024 {
    unsafe fn as_64s_mut(&mut self) -> &mut [u64; 16] {
        std::mem::transmute::<&mut [u8; 128], &mut [u64; 16]>(&mut self.0)
    }

    unsafe fn as_32s_mut(&mut self) -> &mut [u32; 32] {
        std::mem::transmute::<&mut [u8; 128], &mut [u32; 32]>(&mut self.0)
    }

    unsafe fn as_64s(&self) -> &[u64; 16] {
        std::mem::transmute::<&[u8; 128], &[u64; 16]>(&self.0)
    }

    unsafe fn as_32s(&self) -> &[u32; 32] {
        std::mem::transmute::<&[u8; 128], &[u32; 32]>(&self.0)
    }
}

pub struct Context {
    pub light_cache: Box<[Hash512]>,
    pub full_dataset: Option<Box<[Hash1024]>>,
}

impl Context {
    fn new(full: bool) -> Self {
        // TODO: mutex

        // Vec into boxed sliced, because you can't allocate an array directly on
        // the heap in rust
        // https://stackoverflow.com/questions/25805174/creating-a-fixed-size-array-on-heap-in-rust/68122278#68122278
        let mut light_cache = vec![Hash512([0; 64]); LIGHT_CACHE_NUM_ITEMS].into_boxed_slice();
        build_light_cache(&mut light_cache);

        let full_dataset = if full {
            Some(vec![Hash1024([0; 128]); FULL_DATASET_NUM_ITEMS].into_boxed_slice())
        } else {
            None
        };

        Context {
            light_cache,
            full_dataset,
        }
    }
}

// TODO: keeping this function around to mirror the C++ API 1:1 for now
pub fn get_context(full: bool) -> Context {
    Context::new(full)
}

pub unsafe fn prebuild_dataset(
    full_dataset: &mut Box<[Hash1024]>,
    light_cache: &Box<[Hash512]>,
    num_threads: usize,
) {
    if num_threads > 1 {
        std::thread::scope(|scope| {
            let batch_size = full_dataset.len() / num_threads;

            let mut threads = Vec::with_capacity(num_threads);

            let mut chunks = full_dataset.chunks_mut(batch_size);

            let light_cache_slice = &light_cache[0..];

            for (index, chunk) in chunks.enumerate() {
                let start = index * batch_size;

                let thread_handle =
                    scope.spawn(move || build_dataset_segment(chunk, &light_cache_slice, start));
                threads.push(thread_handle);
            }

            for handle in threads {
                handle.join();
            }
        });
    } else {
        build_dataset_segment(&mut full_dataset[0..], &light_cache, 0);
    }
}

pub unsafe fn build_dataset_segment(
    dataset_slice: &mut [Hash1024],
    light_cache: &[Hash512],
    offset: usize,
) {
    for (index, item) in dataset_slice.iter_mut().enumerate() {
        *item = calculate_dataset_item_1024(light_cache, offset + index);
    }
}

fn fnv1(u: u32, v: u32) -> u32 {
    (u * FNV_PRIME) ^ v
}

unsafe fn fnv1_512(u: Hash512, v: Hash512) -> Hash512 {
    let mut r = Hash512([0; 64]);
    let r32s = r.as_32s_mut();

    for (i, item) in r32s.iter_mut().enumerate() {
        //TODO: pretty sure we will always have 16 of them
        *item = fnv1(u.as_32s()[i], v.as_32s()[i])
    }

    r
}

pub struct ItemState<'a> {
    pub seed: u32,
    pub mix: Hash512,
    pub light_cache: &'a [Hash512],
}

impl<'a> ItemState<'a> {
    pub unsafe fn new(light_cache: &'a [Hash512], index: usize) -> Self {
        let mut mix = light_cache[index % LIGHT_CACHE_NUM_ITEMS];
        let seed = index as u32; // TODO: Do we need to cast here??
        mix.as_32s_mut()[0] ^= seed; // TODO: Does this actually modify in place??

        keccak_in_place(&mut mix.0);

        ItemState {
            seed,
            mix,
            light_cache,
        }
    }

    pub unsafe fn update(&mut self, round: u32) {
        let num_words: u32 = 16; // TODO: not sure why this was calculated dynamically in C++
        let index: usize = (round % num_words) as usize; // TODO: may need usize here??
        let t: u32 = fnv1(self.seed ^ round, self.mix.as_32s()[index]);
        let parent_index = (t as usize) % LIGHT_CACHE_NUM_ITEMS; // TODO: casting u32 to usize here too
        self.mix = fnv1_512(self.mix, self.light_cache[parent_index]);
    }

    pub fn _final(&mut self) -> Hash512 {
        keccak_in_place(&mut self.mix.0);

        self.mix.clone()
    }
}

unsafe fn calculate_dataset_item_1024(light_cache: &[Hash512], index: usize) -> Hash1024 {
    let mut item0 = ItemState::new(light_cache, index * 2);
    let mut item1 = ItemState::new(light_cache, index * 2 + 1);

    for j in 0..FULL_DATASET_ITEM_PARENTS {
        item0.update(j);
        item1.update(j);
    }

    // TODO: remove unnecessary copies here
    let dataset_item: [u8; 128] = {
        let final0 = item0._final().0;
        let final1 = item1._final().0;

        let mut whole: [u8; 128] = [0; 128];
        let (one, two) = whole.split_at_mut(final0.len());
        one.copy_from_slice(&final0);
        two.copy_from_slice(&final1);
        whole
    };

    Hash1024(dataset_item)
}

// TODO: Probably want to return instead of using an out-variable
pub unsafe fn hash(output: &mut [u8], context: &mut Context, header: &[u8]) {
    let mut seed: Hash512 = Hash512([0; 64]);

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

unsafe fn fishhash_kernel(context: &mut Context, seed: &Hash512) -> Hash256 {
    let index_limit: u32 = FULL_DATASET_NUM_ITEMS as u32;
    let seed_init = seed.as_32s()[0];

    // TODO: From trait for Hash1024?
    let mut mix: Hash1024 = Hash1024([0; 128]);
    mix.0[0..64].copy_from_slice(&seed.0);
    mix.0.copy_within(0..64, 64);

    for i in 0..NUM_DATASET_ACCESSES as usize {
        // Calculate new fetching indexes
        let p0 = mix.as_32s()[0] % index_limit;
        let p1 = mix.as_32s()[4] % index_limit;
        let p2 = mix.as_32s()[8] % index_limit;

        let fetch0 = lookup(context, p0 as usize);
        let mut fetch1 = lookup(context, p1 as usize);
        let mut fetch2 = lookup(context, p2 as usize);

        // Modify fetch1 and fetch2
        for j in 0..32 {
            fetch1.as_32s_mut()[j] = fnv1(mix.as_32s()[j], fetch1.as_32s()[j]);
            fetch2.as_32s_mut()[j] = mix.as_32s()[j] ^ fetch2.as_32s()[j];
        }

        // Final computation of new mix
        for j in 0..16 {
            mix.as_64s_mut()[j] = fetch0.as_64s()[j] * fetch1.as_64s()[j] + fetch2.as_64s()[j];
        }
    }

    // Collapse the result into 32 bytes
    let mut mix_hash = Hash256([0; 32]);
    let num_words = std::mem::size_of_val(&mix) / std::mem::size_of::<u32>();

    // TODO: Not 100% sure this is the same behavior
    for i in (0..num_words).step_by(4) {
        let h1 = fnv1(mix.as_32s()[i], mix.as_32s()[i + 1]);
        let h2 = fnv1(h1, mix.as_32s()[i + 2]);
        let h3 = fnv1(h2, mix.as_32s()[i + 3]);
        mix_hash.as_32s_mut()[i / 4] = h3;
    }

    mix_hash
}

unsafe fn lookup(context: &mut Context, index: usize) -> Hash1024 {
    match &mut context.full_dataset {
        Some(dataset) => {
            let item = &mut dataset[index];
            if item.as_64s()[0] == 0 {
                *item = calculate_dataset_item_1024(&context.light_cache, index);
            }

            return *item;
        }
        None => return calculate_dataset_item_1024(&context.light_cache, index),
    }
}

fn build_light_cache(cache: &mut [Hash512]) {
    let mut item: Hash512 = Hash512([0; 64]);
    keccak(&mut item.0, &SEED.0);
    cache[0] = item;

    for i in 1..LIGHT_CACHE_NUM_ITEMS {
        let size = std::mem::size_of_val(&item);
        keccak_in_place(&mut item.0);
        cache[i] = item;
    }

    for _ in 0..LIGHT_CACHE_ROUNDS {
        for i in 0..LIGHT_CACHE_NUM_ITEMS {
            // First index: 4 first bytes of the item as little-endian integer
            let t: usize = u32::from_le_bytes(cache[i].0[0..4].try_into().unwrap()) as usize;
            let v: usize = t % LIGHT_CACHE_NUM_ITEMS;

            // Second index
            let w: usize =
                (LIGHT_CACHE_NUM_ITEMS.wrapping_add(i.wrapping_sub(1))) % LIGHT_CACHE_NUM_ITEMS;

            let x: Hash512 = bitwise_xor(&cache[v], &cache[w]);
            keccak(&mut cache[i].0, &x.0);
        }
    }
}

// TODO: Pretty sure this will work for both big and little endian
// but we should test it
fn bitwise_xor(x: &Hash512, y: &Hash512) -> Hash512 {
    let mut z: Hash512 = Hash512([0; 64]);

    for i in 0..64 {
        z.0[i] = x.0[i] ^ y.0[i];
    }

    z
}
