use blake3::Hasher;

use crate::keccak::keccak;

// TODO: These are static in the c++ version
const FNV_PRIME: u32 = 0x01000193;
const FULL_DATASET_ITEM_PARENTS: i32 = 512;
const NUM_DATASET_ACCESSES: i32 = 32;
const LIGHT_CACHE_ROUND: i32 = 3;

const LIGHT_CACHE_NUM_ITEMS: usize = 1179641;
const FULL_DATASET_NUM_ITEMS: usize = 37748717;
const SEED: Hash256 = Hash256([
    0xeb, 0x01, 0x63, 0xae, 0xf2, 0xab, 0x1c, 0x5a, 0x66, 0x31, 0x0c, 0x1c, 0x14, 0xd6, 0x0f, 0x42,
    0x55, 0xa9, 0xb3, 0x9b, 0x0e, 0xdf, 0x26, 0x53, 0x98, 0x44, 0xf1, 0x17, 0xad, 0x67, 0x21, 0x19,
]);

pub trait HashData<const U64_SIZE: usize, const U32_SIZE: usize> {
    unsafe fn as_64s(&mut self) -> &mut [u64; U64_SIZE];
    unsafe fn as_32s(&mut self) -> &mut [u32; U32_SIZE];
    fn as_mut_ptr(&mut self) -> *mut u8;
    fn as_ptr(&self) -> *const u8;
}

#[derive(Clone, Copy)]
pub struct Hash256([u8; 32]);
impl HashData<4, 8> for Hash256 {
    unsafe fn as_64s(&mut self) -> &mut [u64; 4] {
        std::mem::transmute::<&mut [u8; 32], &mut [u64; 4]>(&mut self.0)
    }

    unsafe fn as_32s(&mut self) -> &mut [u32; 8] {
        std::mem::transmute::<&mut [u8; 32], &mut [u32; 8]>(&mut self.0)
    }

    fn as_mut_ptr(&mut self) -> *mut u8 {
        self.0.as_mut_ptr()
    }

    fn as_ptr(&self) -> *const u8 {
        self.0.as_ptr()
    }
}

#[derive(Clone, Copy)]
pub struct Hash512(pub [u8; 64]);
impl HashData<8, 16> for Hash512 {
    unsafe fn as_64s(&mut self) -> &mut [u64; 8] {
        std::mem::transmute::<&mut [u8; 64], &mut [u64; 8]>(&mut self.0)
    }

    unsafe fn as_32s(&mut self) -> &mut [u32; 16] {
        std::mem::transmute::<&mut [u8; 64], &mut [u32; 16]>(&mut self.0)
    }

    fn as_mut_ptr(&mut self) -> *mut u8 {
        self.0.as_mut_ptr()
    }

    fn as_ptr(&self) -> *const u8 {
        self.0.as_ptr()
    }
}
pub struct Hash1024([u8; 128]);
impl HashData<16, 32> for Hash1024 {
    unsafe fn as_64s(&mut self) -> &mut [u64; 16] {
        std::mem::transmute::<&mut [u8; 128], &mut [u64; 16]>(&mut self.0)
    }

    unsafe fn as_32s(&mut self) -> &mut [u32; 32] {
        std::mem::transmute::<&mut [u8; 128], &mut [u32; 32]>(&mut self.0)
    }

    fn as_mut_ptr(&mut self) -> *mut u8 {
        self.0.as_mut_ptr()
    }

    fn as_ptr(&self) -> *const u8 {
        self.0.as_ptr()
    }
}

pub struct Context {
    light_cache_num_items: i64, // TODO: Don't need these probably
    light_cache: [Hash512; LIGHT_CACHE_NUM_ITEMS], // TODO: This'll need to be a union type if we need that
}

pub fn get_context(full: bool) -> Context {
    // TODO: mutex

    let mut light_cache = [[0; 64]; LIGHT_CACHE_NUM_ITEMS];
    // build_light_cache(&mut light_cache);

    todo!()
}

pub fn prebuild_dataset(context: &Context, num_threads: u32) -> () {
    todo!()
}

// TODO: Probably want to return instead of using an out-variable
pub fn hash(mut output: &[u8], context: &Context, header: &[u8]) -> () {
    let mut seed: [u8; 64] = [0; 64];

    let mut hasher = Hasher::new();
    hasher.update(header);
    let mut output_reader = hasher.finalize_xof();
    output_reader.fill(&mut seed);

    let mix_hash = fishhash_kernel(context, &seed);
}

fn fishhash_kernel(context: &Context, seed: &[u8; 64]) -> [u8; 32] {
    todo!()
}

unsafe fn build_light_cache(cache: &mut [Hash512; LIGHT_CACHE_NUM_ITEMS]) -> () {
    // let mut item: Hash512 = Hash512([0; 64]);
    // let mut item = [0u64; 8];
    // keccak(item, 512, &SEED, std::mem::size_of_val(&SEED));
    // cache[0] = Hash512(item);

    // for i in 0..LIGHT_CACHE_NUM_ITEMS {
    //     let size = std::mem::size_of_val(&item);
    //     keccak(item, 512, &item, size);
    // }

    todo!()
}
