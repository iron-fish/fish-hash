#pragma once

#include <cstdint>

#ifdef __cplusplus
extern "C" {
#endif

namespace FishHash {

	union hash256 {
	    uint64_t word64s[4];
	    uint32_t word32s[8];
	    uint8_t bytes[32];
	    char str[32];
	};

	union hash512 {
	    uint64_t word64s[8];
	    uint32_t word32s[16];
	    uint8_t bytes[64];
	    char str[64];
	};

	union hash1024 {
	    union hash512 hash512s[2];
	    uint64_t word64s[16];
	    uint32_t word32s[32];
	    uint8_t bytes[128];
	    char str[128];
	};


	struct fishhash_context	{
	    const int light_cache_num_items;
	    hash512* const light_cache;
	    const int full_dataset_num_items;
	    hash1024* full_dataset;
	};
	
	
	fishhash_context* get_context(bool full = false) noexcept;
	void prebuild_dataset(fishhash_context*, uint32_t numThreads = 1) noexcept;
	void hash(uint8_t * output, const fishhash_context * ctx, const uint8_t * header, uint64_t header_size) noexcept;
}

#ifdef __cplusplus
}
#endif

