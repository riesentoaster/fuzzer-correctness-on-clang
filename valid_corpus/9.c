#include <stdio.h>

static int hidden(int x) { return x ^ 0x55; }

int main(void) {
    int v = 12345;
    printf("%d\n", hidden(v));
    return 0;
}