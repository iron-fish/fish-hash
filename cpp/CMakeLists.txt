set(CMAKE_CXX_STANDARD 11)

add_library(
    FishHash
    FishHash.cpp
    3rdParty/blake3.c
    3rdParty/blake3_portable.c
    3rdParty/blake3_dispatch.c
    3rdParty/keccak.c
)



