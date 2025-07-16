#include <dlfcn.h>
#include <elf.h>
#include <stdio.h>
#include <sys/auxv.h>
#include <time.h>
// #define N 1000000


int main() {
  unsigned long addr = getauxval(AT_SYSINFO_EHDR);
  printf("vdso elf header address = %#lx\n", addr);

  struct timespec ts;
  // for (int i = 0; i < N; i++) {
  int res = clock_gettime(CLOCK_MONOTONIC, &ts);
  // }
}
