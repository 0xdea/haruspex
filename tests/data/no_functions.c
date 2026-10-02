// Data-only object file with type definitions but no functions, used to test that haruspex
// rejects such binaries and cleans up its output directory.
// Built with: clang -target x86_64-linux-gnu -O0 -g -c -o no_functions no_functions.c
struct point {
    int x;
    int y;
};

struct point origin = {1, 2};
