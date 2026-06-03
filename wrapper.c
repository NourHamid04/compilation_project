#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>

extern int64_t func(int64_t);

int main(void) {
    int64_t inp;
    int64_t out;

    scanf("%ld", &inp);
    out = func(inp);
    printf("%ld\n", out);

    return EXIT_SUCCESS;
}
