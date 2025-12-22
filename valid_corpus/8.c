#include <stdio.h>

struct P { int x, y; };

struct P combine(struct P a, struct P b) {
    struct P r = { a.x && b.x, a.y || b.y };
    return r;
}

int main(void) {
    struct P a = {1,0}, b = {0,1};
    struct P r = combine(a, b);
    printf("%d %d\n", r.x, r.y);
    return 0;
}