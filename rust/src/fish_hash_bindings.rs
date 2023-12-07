// mod bindings;
#[repr(C)]
#[derive(Copy, Clone)]
pub union hash256 {
    pub word64s: [u64; 4usize],
    pub word32s: [u32; 8usize],
    pub bytes: [u8; 32usize],
    pub str_: [::std::os::raw::c_char; 32usize],
}

#[repr(C)]
#[derive(Copy, Clone)]
pub union hash512 {
    pub word64s: [u64; 8usize],
    pub word32s: [u32; 16usize],
    pub bytes: [u8; 64usize],
    pub str_: [::std::os::raw::c_char; 64usize],
}

#[repr(C)]
#[derive(Copy, Clone)]
pub union hash1024 {
    pub hash512s: [hash512; 2usize],
    pub word64s: [u64; 16usize],
    pub word32s: [u32; 32usize],
    pub bytes: [u8; 128usize],
    pub str_: [::std::os::raw::c_char; 128usize],
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
// TODO: Can this be re-named?
pub struct fishhash_context {
    pub light_cache_num_items: ::std::os::raw::c_int,
    pub light_cache: *mut hash512,
    pub full_dataset_num_items: ::std::os::raw::c_int,
    pub full_dataset: *mut hash1024,
}

extern "C" {
    pub fn get_context(full: bool) -> *mut fishhash_context;
    pub fn prebuild_dataset(arg1: *mut fishhash_context, numThreads: u32);
    pub fn hash(output: *mut u8, ctx: *const fishhash_context, header: *const u8, header_size: u64);
}
