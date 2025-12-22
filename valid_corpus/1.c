#include <stdio.h>

static int counter = 0;

int recurse(int x) {
    counter |= x;
    if (x <= 1) return x;
    return recurse(x - 1) + recurse(x - 2);
}

int main(void) {
    printf("%d %d\n", recurse(6), counter);
    return 0;
}