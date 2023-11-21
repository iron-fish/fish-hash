cmake_minimum_required(VERSION 3.9)
project(LibFishHash CXX)
set(CMAKE_CXX_STANDARD 11)
set(CMAKE_OSX_DEPLOYMENT_TARGET 12.0)

add_library(
    FishHash
    STATIC
    FishHash.cpp
    3rdParty/blake3.c
    3rdParty/blake3_portable.c
    3rdParty/blake3_dispatch.c
    3rdParty/keccak.c
)

install(TARGETS FishHash DESTINATION .)
