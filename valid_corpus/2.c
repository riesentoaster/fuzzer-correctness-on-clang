#include <stdio.h>

volatile int v = 5;

int main(void) {
    int arr[5] = {1,2,3,4,5};
    int *p = arr;
    for (; p < arr + 5; ++p) {
        v += *p;
    }
    printf("%d\n", v);
    return 0;
}