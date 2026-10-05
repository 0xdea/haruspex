// Data-only object file for a processor without a Hex-Rays decompiler (MSP430),
// used to test that haruspex fails cleanly when no decompiler is available.
// Clang can't target MSP430 everywhere, so build an i386 object and patch its
// ELF `e_machine` field (bytes 18-19) to `EM_MSP430` (105, i.e., 0x69 0x00):
// clang -target i386-linux-gnu -O0 -c -o no_decompiler no_decompiler.c
// printf '\x69' | dd of=no_decompiler bs=1 seek=18 conv=notrunc
struct point {
    int x;
    int y;
};

struct point origin = {1, 2};
