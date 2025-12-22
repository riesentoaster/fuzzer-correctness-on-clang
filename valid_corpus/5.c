#include <stdio.h>

int main(void) {
    double d = 3.14;
    int i = (int)d * 3;
    switch (i) {
        case 9: printf("nine\n"); break;
        default: printf("%d\n", i);
    }
    return 0;
}