#include <stdint.h>
#include <stdio.h>

#include <internal/nonvolatile_storage.h>

uint8_t readbuf[256];
uint8_t writebuf[256];

bool done = false;

static void read_done(int length,
                      __attribute__ ((unused)) int arg1,
                      __attribute__ ((unused)) int arg2,
                      __attribute__ ((unused)) void* ud) {
  printf("Finished read! %i\n", length);
  done = true;
}

static void write_done(int length,
                       __attribute__ ((unused)) int arg1,
                       __attribute__ ((unused)) int arg2,
                       __attribute__ ((unused)) void* ud) {
  printf("Finished write! %i\n", length);
  done = true;
}

int main (void) {
  int ret;

  printf("[Nonvolatile Storage] Test App\n");

  ret = nonvolatile_storage_internal_read_buffer(readbuf, 256);
  if (ret != 0) printf("ERROR setting read buffer\n");

  ret = nonvolatile_storage_internal_write_buffer(writebuf, 256);
  if (ret != 0) printf("ERROR setting write buffer\n");

  // Setup callbacks
  ret = nonvolatile_storage_internal_read_done_subscribe(read_done, NULL);
  if (ret != 0) printf("ERROR setting read done callback\n");

  ret = nonvolatile_storage_internal_write_done_subscribe(write_done, NULL);
  if (ret != 0) printf("ERROR setting write done callback\n");

  int num_bytes = nonvolatile_storage_internal_get_number_bytes();
  printf("Have %i bytes of nonvolatile storage\n", num_bytes);

  writebuf[0] = 5;
  writebuf[1] = 10;
  writebuf[2] = 20;
  writebuf[3] = 200;
  writebuf[4] = 123;
  writebuf[5] = 88;

  done = false;
  ret  = nonvolatile_storage_internal_write(0, 6);
  if (ret != 0) printf("ERROR calling write\n");
  yield_for(&done);

  writebuf[0] = 33;
  writebuf[1] = 3;
  writebuf[2] = 66;
  writebuf[3] = 6;
  writebuf[4] = 99;
  writebuf[5] = 9;
  writebuf[6] = 100;
  writebuf[7] = 101;

  done = false;
  ret  = nonvolatile_storage_internal_write(6, 8);
  if (ret != 0) printf("ERROR calling write\n");
  yield_for(&done);

  done = false;
  ret  = nonvolatile_storage_internal_read(0, 14);
  if (ret != 0) printf("ERROR calling read\n");
  yield_for(&done);

  for (int i = 0; i < 14; i++) {
    printf("got[%i]: %i\n", i, readbuf[i]);
  }

  return 0;
}
