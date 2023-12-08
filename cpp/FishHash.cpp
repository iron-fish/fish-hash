#include "FishHash.h"
#include "3rdParty/keccak.h"
#include "3rdParty/blake3.h"

#include "3rdParty/keccak.c"
#include "3rdParty/blake3.c"

#include <cstdlib>
#include <cstring>
#include <memory>
#include <mutex>
#include <thread>
#include <vector>

namespace FishHash {

	/*****************************
	
	     Constants and Helpers

	******************************/
	
	static const uint32_t fnv_prime = 0x01000193;
	static int full_dataset_item_parents = 512;
	static int num_dataset_accesses = 32;
	static int light_cache_rounds = 3;
	
	const int light_cache_num_items = 1179641;
	const int full_dataset_num_items = 37748717;
	hash256 seed = {.bytes={0xeb,0x01,0x63,0xae,0xf2,0xab,0x1c,0x5a,
				0x66,0x31,0x0c,0x1c,0x14,0xd6,0x0f,0x42,
				0x55,0xa9,0xb3,0x9b,0x0e,0xdf,0x26,0x53,
				0x98,0x44,0xf1,0x17,0xad,0x67,0x21,0x19}};
				
	std::shared_ptr<fishhash_context> shared_context;
	std::mutex shared_context_mutex;
	
	/*****************************
	
		Utility Functions
	
	******************************/
	
	#if __clang__
	__attribute__((no_sanitize("unsigned-integer-overflow")))
	#endif
	static inline uint32_t fnv1(uint32_t u, uint32_t v) noexcept {
		return (u * fnv_prime) ^ v;
	}
	
	inline hash512 fnv1(const hash512& u, const hash512& v) noexcept {
		hash512 r;
		for (size_t i = 0; i < sizeof(r) / sizeof(r.word32s[0]); ++i)
			r.word32s[i] = fnv1(u.word32s[i], v.word32s[i]);
		return r;
	}
	

	/*****************************
	
	    Data Set Item Calculation
		
	******************************/
	
	
	
	struct item_state
	{
	    const hash512* const cache;
	    const int64_t num_cache_items;
	    const uint32_t seed;

	    hash512 mix;

	    inline item_state(const fishhash_context& ctx, int64_t index) noexcept
	      : cache{ctx.light_cache},
		num_cache_items{ctx.light_cache_num_items},
		seed{static_cast<uint32_t>(index)}
	    {
		mix = cache[index % num_cache_items];
		mix.word32s[0] ^= seed;
				
		keccak(mix.word64s, 512, mix.bytes, 64);
		
	    }

	    inline void update(uint32_t round) noexcept
	    {
		static constexpr size_t num_words = sizeof(mix) / sizeof(uint32_t);
		const uint32_t t = fnv1(seed ^ round, mix.word32s[round % num_words]);
		const int64_t parent_index = t % num_cache_items;
		mix = fnv1(mix, cache[parent_index]);
	    }

	    inline hash512 final() noexcept { 
	    	keccak(mix.word64s, 512, mix.bytes, 64);
	    	return mix; 
	    }
	};


	hash1024 calculate_dataset_item_1024(const fishhash_context& ctx, uint32_t index) noexcept
	{
	    item_state item0{ctx, int64_t(index) * 2};
	    item_state item1{ctx, int64_t(index) * 2 + 1};

	    for (uint32_t j = 0; j < full_dataset_item_parents; ++j)
	    {
		item0.update(j);
		item1.update(j);
	    }

	    return hash1024{{item0.final(), item1.final()}};
	}


	/*****************************
	
	        Hashing function
		
	******************************/
	
	inline hash1024 lookup(const fishhash_context& ctx, uint32_t index) {
		if (ctx.full_dataset != NULL) {
			hash1024 * item = &ctx.full_dataset[index];
			
			// Ability to handle lazy lookup
			if (item->word64s[0] == 0) {
				*item = calculate_dataset_item_1024(ctx, index);
			}
			
			return *item;
		} else {
			return calculate_dataset_item_1024(ctx, index);
		}
	}

	inline hash256 fishhash_kernel( const fishhash_context& ctx, const hash512& seed) noexcept {
		const uint32_t index_limit = static_cast<uint32_t>(ctx.full_dataset_num_items);
		const uint32_t seed_init = seed.word32s[0];
	    
		hash1024 mix{seed, seed};

		for (uint32_t i = 0; i < num_dataset_accesses; ++i) {
					
			// Calculate new fetching indexes
			const uint32_t p0 = mix.word32s[0] % index_limit;
			const uint32_t p1 = mix.word32s[4] % index_limit;
			const uint32_t p2 = mix.word32s[8] % index_limit;
					       
			hash1024 fetch0 = lookup(ctx, p0);
			hash1024 fetch1 = lookup(ctx, p1);
			hash1024 fetch2 = lookup(ctx, p2);
						
			// Modify fetch1 and fetch2
			for (size_t j = 0; j < 32; ++j) {
				fetch1.word32s[j] = fnv1(mix.word32s[j], fetch1.word32s[j]);
				fetch2.word32s[j] = mix.word32s[j] ^ fetch2.word32s[j];
			}
						
		     	// Final computation of new mix
			for (size_t j = 0; j < 16; ++j)
				mix.word64s[j] = fetch0.word64s[j] * fetch1.word64s[j] + fetch2.word64s[j];
		}

		// Collapse the result into 32 bytes
		hash256 mix_hash;
		static constexpr size_t num_words = sizeof(mix) / sizeof(uint32_t);
		for (size_t i = 0; i < num_words; i += 4) {
			const uint32_t h1 = fnv1(mix.word32s[i], mix.word32s[i + 1]);
			const uint32_t h2 = fnv1(h1, mix.word32s[i + 2]);
			const uint32_t h3 = fnv1(h2, mix.word32s[i + 3]);
			mix_hash.word32s[i / 4] = h3;
		}

		return mix_hash;
	}

	void hash(uint8_t * output, const fishhash_context * ctx, const uint8_t * header, uint64_t header_size) noexcept {
		hash512 seed; 
	   
		blake3_hasher hasher;
		blake3_hasher_init(&hasher);
		blake3_hasher_update(&hasher, header, header_size);
		blake3_hasher_finalize(&hasher, seed.bytes, 64);
				
		const hash256 mix_hash = fishhash_kernel(*ctx, seed);
	    
		uint8_t final_data[sizeof(seed) + sizeof(mix_hash)];
		std::memcpy(&final_data[0], seed.bytes, sizeof(seed));
		std::memcpy(&final_data[sizeof(seed)], mix_hash.bytes, sizeof(mix_hash));
	    
		hash256 finValue;
	    
		if (!output) output = static_cast<uint8_t*>(std::calloc(1, 32));
		
		uint32_t * data = (uint32_t *) final_data;
			    
		blake3_hasher_init(&hasher);
		blake3_hasher_update(&hasher, final_data, 64 + 32);
		blake3_hasher_finalize(&hasher, output, 32);
	}
	
	inline hash512 bitwise_xor(const hash512& x, const hash512& y) noexcept {
		hash512 z;
		for (size_t i = 0; i < sizeof(z) / sizeof(z.word64s[0]); ++i)
			z.word64s[i] = x.word64s[i] ^ y.word64s[i];
		return z;
	}
	
	void build_light_cache( hash512 cache[], int num_items, const hash256& seed) noexcept {
		hash512 item;
		keccak(item.word64s, 512, seed.bytes, sizeof(seed));
		cache[0] = item;
		
		for (int i = 1; i < num_items; ++i) {
			keccak(item.word64s, 512, item.bytes, sizeof(item));
			cache[i] = item;
		}

		for (int q = 0; q < light_cache_rounds; ++q) {
			for (int i = 0; i < num_items; ++i) {
			    const uint32_t index_limit = static_cast<uint32_t>(num_items);

			    // First index: 4 first bytes of the item as little-endian integer.
			    const uint32_t t = cache[i].word32s[0];
			    const uint32_t v = t % index_limit;

			    // Second index.
			    const uint32_t w = static_cast<uint32_t>(num_items + (i - 1)) % index_limit;

			    const hash512 x = bitwise_xor(cache[v], cache[w]);			    
			    keccak(cache[i].word64s, 512, x.bytes, sizeof(x));
			    
			}
		}
	}
	
	void build_dataset_segment(fishhash_context * ctx, uint32_t start, uint32_t end) {
		for (uint32_t i=start; i<end; ++i) {
			ctx -> full_dataset[i] = calculate_dataset_item_1024(*ctx, i);
		}
	}	
	
	/*****************************
	
	        Context functions
		
	******************************/
		
	fishhash_context* get_context(bool full) noexcept {
		std::lock_guard<std::mutex> lock{shared_context_mutex};
	
		if (shared_context) {
			// If a context is present and either no full dataset is requested or its present, return the context
			if ( (!full) || (shared_context->full_dataset) ) return shared_context.get();
		}
	
		shared_context.reset();
					
		size_t context_alloc_size = sizeof(hash512);
		size_t light_cache_size = light_cache_num_items * sizeof(hash512);
		size_t full_dataset_size = full ? full_dataset_num_items * sizeof(hash1024) : 0;
		
		size_t alloc_size = context_alloc_size + light_cache_size + full_dataset_size;

		char* const alloc_data = static_cast<char*>(std::calloc(1, alloc_size));
		if (!alloc_data) return nullptr;  // Signal out-of-memory by returning null
		
		hash512* const light_cache = reinterpret_cast<hash512*>(alloc_data + context_alloc_size);
		build_light_cache(light_cache, light_cache_num_items, seed);
		
		hash1024* full_dataset = full ? reinterpret_cast<hash1024*>(alloc_data + context_alloc_size + light_cache_size): nullptr;
				
		shared_context.reset( new (alloc_data) fishhash_context{
     			light_cache_num_items,
			light_cache,
       			full_dataset_num_items,
        		full_dataset} );
        		
        	return shared_context.get();
			
	}
		
	void prebuild_dataset(fishhash_context * ctx, uint32_t numThreads) noexcept {
		// If the context is not initialized as full context, return to avoid segmentation faults
		if (ctx->full_dataset == NULL) return;
	
		if (numThreads > 1) {
    			uint32_t batch_size = ctx->full_dataset_num_items / numThreads;
    			
    			// Launch worker threads
    			std::vector< std::thread > threads(numThreads);
    			for(unsigned i = 0; i < numThreads; ++i) {
            			int start = i * batch_size;
            			int end = i == (numThreads-1) ? ctx->full_dataset_num_items  : (i+1) * batch_size;
            			 
            			threads[i] = std::thread(build_dataset_segment, ctx, start, end);
        		}
    			
    			// Join them in for completion
    			for(unsigned i = 0; i < numThreads; ++i) {
    				threads[i].join();
    			}
		} else {
			build_dataset_segment(ctx, 0, ctx->full_dataset_num_items);
		}
	}
}
