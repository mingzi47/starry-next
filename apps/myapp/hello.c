#include "unistd.h"

int main()
{
    char msg[] = "Hello, World!\n";
    write(1, msg, sizeof(msg));
    return 0;
}

