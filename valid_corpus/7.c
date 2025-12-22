#include <stdio.h>

int f1(int x) { return x + 1; }
int f2(int x) { return x * 2; }
int f3(int x) { return x - 3; }

enum Op { ADD1, MUL2, SUB3 };

int main(void) {
    int (*ops[])(int) = { f1, f2, f3 };
    for (int i = 0; i < 3; i++)
        printf("%d\n", ops);
    return 0;
}