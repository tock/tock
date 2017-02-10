#include <inttypes.h>
#include <stdlib.h>
#include <unistd.h>
#include <stdbool.h>
#include "tock.h"

#pragma GCC diagnostic ignored "-Wunused-parameter"

void yield_for(bool *cond) {
  while(!*cond) {
    yield();
  }
}

void yield(void) {
  asm volatile("push {lr}\nsvc 0\npop {pc}" ::: "memory", "r0");
}

int subscribe(uint32_t driver, uint32_t subscribe,
              subscribe_cb cb, void* userdata) {
  register int ret __asm__ ("r0");
  asm volatile("svc 1" ::: "memory", "r0");
  return ret;
}


int command(uint32_t driver, uint32_t command, int data) {
  register int ret __asm__ ("r0");
  asm volatile("svc 2\nbx lr" ::: "memory", "r0");
  return ret;
}

int allow(uint32_t driver, uint32_t allow, void* ptr, size_t size) {
  register int ret __asm__ ("r0");
  asm volatile("svc 3\nbx lr" ::: "memory", "r0");
  return ret;
}

void* memop(uint32_t op_type, int arg1) {
  register void* ret __asm__ ("r0");
  asm volatile("svc 4\nbx lr" ::: "memory", "r0");
  return ret;
}

bool driver_exists(uint32_t driver) {
  int ret = command(driver, 0, 0);
  return ret >= 0;
}
