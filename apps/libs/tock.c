#include <inttypes.h>
#include <stdlib.h>
#include <unistd.h>
#include "tock.h"
#include "firestorm.h"

extern int __wait();
extern int __command(uint32_t, uint32_t, int);
extern int __allow();
extern int __subscribe();
extern int __memop(uint32_t, int);

int yield() {
    return command(99, 0, 0); 
}

void wait() {
  __wait();
}

void wait_for(bool *cond) {
  while(!*cond) {
    __wait();
  }
}

int subscribe(uint32_t driver, uint32_t subscribe,
              subscribe_cb cb, void* userdata) {
  return __subscribe(driver, subscribe, cb, userdata);
}


int command(uint32_t driver, uint32_t command, int data) {
  return __command(driver, command, data);
}

int allow(uint32_t driver, uint32_t allow, void* ptr, size_t size) {
  return __allow(driver, allow, ptr, size);
}

int memop(uint32_t op_type, int arg1) {
  return __memop(op_type, arg1);
}

