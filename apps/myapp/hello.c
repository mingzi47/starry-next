#include <stdio.h>
#include <sys/auxv.h>
#include <time.h>

void test_sys() {
  struct timespec ts;
  int ret;
  int clk_id = CLOCK_REALTIME; // 0

  asm volatile("movq $228, %%rax\n\t"   // SYS_clock_gettime
               "movq %[clk], %%rdi\n\t" // clock ID
               "movq %[tsp], %%rsi\n\t" // pointer to timespec
               "syscall\n\t"
               "movl %%eax, %[ret]\n\t" // store return value
               : [ret] "=r"(ret)
               : [clk] "r"((long)clk_id), [tsp] "r"(&ts)
               : "rax", "rdi", "rsi", "rcx", "r11", "memory");

  if (ret == 0) {
    printf("syscall CLOCK_MONOTONIC: %ld.%09ld\n", ts.tv_sec, ts.tv_nsec);
  } else {
    printf("syscall clock_gettime failed, res = %d\n", ret);
  }
}

int main() {
  unsigned long addr = getauxval(AT_SYSINFO_EHDR);
  printf("vdso elf header address = %#lx\n", addr);

  test_sys();
  struct timespec ts;
  int res = clock_gettime(CLOCK_MONOTONIC, &ts);

  if (res == 0) {
    printf("vdso CLOCK_MONOTONIC: %ld.%09ld\n", ts.tv_sec, ts.tv_nsec);
  } else {
    printf("vdso clock_gettime failed, res = %d\n", res);
  }
  test_sys();
}
