mod fish_hash_bindings;

fn main() {
    unsafe {
        let c = fish_hash_bindings::get_context(false);
        let lcache = (*c).light_cache_num_items;
        dbg!(lcache);
    }
}
