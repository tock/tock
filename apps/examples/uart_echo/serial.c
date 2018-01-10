#include <tock.h>

int serial_read_short(char* buf, size_t len);
int serial_read(char* buf, size_t len);
int serial_write(char* buf, size_t len);

int serial_read_short(char* buf, size_t len) {
  int read_len = 0;
  void read_callback(int rlen,
                     __attribute__ ((unused)) int unused1,
                     __attribute__ ((unused)) int unused2,
                     void* ud) {
    *((bool*)ud) = true;
    read_len = rlen;
  }

  int ret = allow(1, 0, buf, len);
  bool done = false;
  if (ret < 0)  return ret;
  ret = subscribe(1, 0, read_callback, &done);
  if (ret < 0)  return ret;
  ret = command(1, 2, len, 0);
  if (ret < 0)  return ret;
  yield_for(&done);
  return read_len;
}

int serial_read(char* buf, size_t len) {
  size_t index;
  for (index = 0; index < len;) {
    size_t left = len - index;
    size_t count = serial_read_short(buf + index, left);
    index += count;
  }
  return (int)index;
}

int serial_write(char* buf, size_t len) {
  int write_len = 0;
  void write_callback(int wlen,
                     __attribute__ ((unused)) int unused1,
                     __attribute__ ((unused)) int unused2,
                     void* ud) {
    *((bool*)ud) = true;
    write_len = wlen;
  }
  bool done = false;

  int ret = allow(1, 1, buf, len);
  if (ret < 0) return ret;

  ret = subscribe(1, 1, write_callback, &done);
  if (ret < 0) return ret;

  ret = command(1, 1, len, 0);
  yield_for(&done);
  return write_len;
}

