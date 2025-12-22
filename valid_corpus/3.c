#include <stdio.h>

#define USE_ADD 1
#define X 10
#define Y 20

inline int add(int a, int b) { return a + b; }
inline int mul(int a, int b) { return a * b; }

int main(void) {
#if USE_ADD
    printf("%d\n", add(X, Y));
#else
    printf("%d\n", mul(X, Y));
#endif
    return 0;
}