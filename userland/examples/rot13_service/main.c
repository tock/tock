#include "../../libtock/tock.h"
#include <stdio.h>

#define IPC_DRIVER 0x4c

static void rot13_callback(int pid, int len, int arg2, void* ud) {
  char* buf = (char*)arg2;
  int length = buf[0];
  if (length > len - 1) {
    length = len - 1;
  }
  buf++;
  for (int i = 0; i < len; ++i) {
    if (buf[i] >= 'a' && buf[i] <= 'z') {
      buf[i] = (((buf[i] - 'a') + 13) % 26) + 'a';
    } else if (buf[i] >= 'A' && buf[i] <= 'Z') {
      buf[i] = (((buf[i] - 'A') + 13) % 26) + 'A';
    }
  }
  command(IPC_DRIVER, pid, 0);
}

int main() {
  subscribe(IPC_DRIVER, 0, rot13_callback, 0);
  while (1) {
    yield();
  }
  return 0;
}

