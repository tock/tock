#include <stdio.h>
#include <timer.h>
#include <string.h>

#define IPC_DRIVER 0x4c

static char* rot13_service_name = "org.tockos.examples.rot1";
int rot13_svc_num = 0;

char buf[64] __attribute__((aligned(64)));

struct rot13_buf {
  int8_t length;
  char buf[31];
};

static void rot13_callback(int pid, int len, int arg2, void* ud) {
  struct rot13_buf *rb = (struct rot13_buf*)ud;
  printf("%d: %.*s\n", rb->length, rb->length, rb->buf);
  delay_ms(500);
  command(IPC_DRIVER, rot13_svc_num, 0);
}

int main(void) {
  char svc_name[32];
  memcpy(svc_name, rot13_service_name, strlen(rot13_service_name));
  rot13_svc_num = allow(IPC_DRIVER, 0, svc_name, strlen(rot13_service_name));

  struct rot13_buf *rb = (struct rot13_buf*)buf;
  subscribe(IPC_DRIVER, 0, rot13_callback, rb);
  rb->length = snprintf(rb->buf, sizeof(rb->buf), "Hello World!");
  allow(IPC_DRIVER, rot13_svc_num, rb, 64);

  command(IPC_DRIVER, rot13_svc_num, 0);
  return 0;
}

