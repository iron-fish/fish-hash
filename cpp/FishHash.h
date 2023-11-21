#pragma once

#include <stdint.h>
#include <stdbool.h>

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
	union hash512* const light_cache;
	const int full_dataset_num_items;
	union hash1024* full_dataset;
};


struct fishhash_context* get_context(bool full);
void prebuild_dataset(struct fishhash_context*, uint32_t numThreads);
void hash(uint8_t * output, const struct fishhash_context * ctx, const uint8_t * header, uint64_t header_size);
