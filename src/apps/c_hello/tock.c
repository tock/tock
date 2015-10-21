#include <inttypes.h>
#include <unistd.h>
#include "tock.h"

extern int __wait();
extern int __command(uint32_t, uint32_t, int);
extern int __allow();
extern int __subscribe();


int wait() {
  return __wait();
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
