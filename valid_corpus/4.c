#include <stdio.h>

typedef struct {
    int a;
    struct {
        float x, y;
    } inner;
} T;

int main(void) {
    T val = { .a = 3, .inner = { .x = 1.5f, .y = 2.5f } };
    printf("%d %.2f %.2f\n", val.a, val.inner.x, val.inner.y);
    return 0;
}