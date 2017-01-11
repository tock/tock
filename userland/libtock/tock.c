#include <inttypes.h>
#include <stdlib.h>
#include <unistd.h>
#include <stdbool.h>
#include "tock.h"

void yield_for(bool *cond) {
  while(!*cond) {
    yield();
  }
}

void yield() {
  asm volatile(
      "svc 0"
      :
      :
      : "memory", "r0"
      );
}

int subscribe(uint32_t driver, uint32_t subscribe,
              subscribe_cb cb, void* userdata) {
  register uint32_t r0 asm("r0") = driver;
  register uint32_t r1 asm("r1") = subscribe;
  register void*    r2 asm("r2") = cb;
  register void*    r3 asm("r3") = userdata;
  register int ret asm ("r0");
  asm volatile(
      "svc 1"
      : "=r" (ret)
      : "r" (r0), "r" (r1), "r" (r2), "r" (r3)
      : "memory");
  return ret;
}


int command(uint32_t driver, uint32_t command, int data) {
  register uint32_t r0 asm("r0") = driver;
  register uint32_t r1 asm("r1") = command;
  register uint32_t r2 asm("r2") = data;
  register int ret asm ("r0");
  asm volatile(
      "svc 2"
      : "=r" (ret)
      : "r" (r0), "r" (r1), "r" (r2)
      : "memory"
      );
  return ret;
}

int allow(uint32_t driver, uint32_t allow, void* ptr, size_t size) {
  register uint32_t r0 asm("r0") = driver;
  register uint32_t r1 asm("r1") = allow;
  register void*    r2 asm("r2") = ptr;
  register size_t   r3 asm("r3") = size;
  register int ret asm ("r0");
  asm volatile(
      "svc 3"
      : "=r" (ret)
      : "r" (r0), "r" (r1), "r" (r2), "r" (r3)
      : "memory"
      );
  return ret;
}

int memop(uint32_t op_type, int arg1) {
  register uint32_t r0 asm("r0") = op_type;
  register int      r1 asm("r1") = arg1;
  register int ret asm ("r0");
  asm volatile(
      "svc 4"
      : "=r" (ret)
      : "r" (r0), "r" (r1)
      : "memory"
      );
  return ret;
}

bool driver_exists(uint32_t driver) {
  int ret = command(driver, 0, 0);
  return ret >= 0;
}
