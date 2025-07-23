#include <stdint.h>
#include <stdio.h>
#include <sys/auxv.h>
#include <sys/syscall.h>
#include <sys/time.h>
#include <time.h>
#include <unistd.h>

#define PRINT(tag, ret, arg1, arg2)                                            \
  do {                                                                         \
    if ((ret) == 0) {                                                          \
      printf("%s: %ld.%09ld\n", (tag), arg1, arg2);                            \
    } else {                                                                   \
      printf("%s failed, ret = %d\n", (tag), (int)(ret));                      \
    }                                                                          \
  } while (0)

void test_clock_gettime(clockid_t clock_id) {
  printf("clock_gettime test => clock_id = %d\n", clock_id);
  int ret;

  struct timespec ts1;
  ret = syscall(SYS_clock_gettime, clock_id, &ts1);
  PRINT("sys", ret, ts1.tv_sec, ts1.tv_nsec);

  struct timespec ts2;
  ret = clock_gettime(clock_id, &ts2);
  PRINT("vdso", ret, ts2.tv_sec, ts2.tv_nsec);

  struct timespec ts3;
  ret = syscall(SYS_clock_gettime, clock_id, &ts3);
  PRINT("sys", ret, ts3.tv_sec, ts3.tv_nsec);
}

void test_gettimeofday() {
  printf("gettimeofday test\n");
  int ret;

  struct timeval tv1;
  ret = syscall(SYS_gettimeofday, &tv1, NULL);
  PRINT("sys", ret, tv1.tv_sec, tv1.tv_usec);

  struct timeval tv2;
  ret = gettimeofday(&tv1, NULL);
  PRINT("vdso", ret, tv2.tv_sec, tv2.tv_usec);
  struct timeval tv3;
  ret = syscall(SYS_gettimeofday, &tv2, NULL);
  PRINT("sys", ret, tv3.tv_sec, tv3.tv_usec);
}

int main() {
  // unsigned long addr = getauxval(AT_SYSINFO_EHDR);
  // printf("vdso elf header address = %#lx\n", addr);
  //
  test_clock_gettime(CLOCK_MONOTONIC);
  test_clock_gettime(CLOCK_REALTIME);

  test_gettimeofday();
}
