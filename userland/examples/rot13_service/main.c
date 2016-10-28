#include <tock.h>

#define IPC_DRIVER 0x4c

struct rot13_buf {
  int8_t length;
  char buf[31];
};

static void rot13_callback(int pid, int len, int buf, void* ud) {
  struct rot13_buf *rb = (struct rot13_buf*)buf;
  int length = rb->length;
  if (length > len - 1) {
    length = len - 1;
  }
  for (int i = 0; i < length; ++i) {
    if (rb->buf[i] >= 'a' && rb->buf[i] <= 'z') {
      rb->buf[i] = (((rb->buf[i] - 'a') + 13) % 26) + 'a';
    } else if (rb->buf[i] >= 'A' && rb->buf[i] <= 'Z') {
      rb->buf[i] = (((rb->buf[i] - 'A') + 13) % 26) + 'A';
    }
  }
  command(IPC_DRIVER, pid, 0);
}

int main() {
  subscribe(IPC_DRIVER, 0, rot13_callback, NULL);
  return 0;
}

