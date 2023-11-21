#ifndef KECCAK_H
#define KECCAK_H

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

void keccak(uint64_t* out, size_t bits, const uint8_t* data, size_t size);

#ifdef __cplusplus
}
#endif

#endif /* KECCAK_H */
