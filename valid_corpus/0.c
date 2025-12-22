#include <stdio.h>

enum Kind { K_INT, K_FLOAT };

union Value {
    int i;
    float f;
};

struct Node {
    enum Kind k;
    union Value v;
    int (*fn)(int);
};

int square(int x) { return x * x; }

int main(void) {
    struct Node n = { K_INT, { .i = 7 }, square };
    printf("%d\n", n.fn(n.v.i));
    return 0;
}