#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <firestorm.h>
#include <tock.h>

int gpio_enable(unsigned int pin) {
  return command(1, 0, pin);
}

int gpio_set(unsigned int pin) {
  return command(1, 2, pin);
}

int gpio_clear(unsigned int pin) {
  return command(1, 3, pin);
}

int gpio_toggle(unsigned int pin) {
  return command(1, 4, pin);
}

static CB_TYPE putstr_cb(int _x, int _y, int _z, void* str) {
  free(str);
  return PUTSTR;
}

void putnstr(const char *str, size_t len) {
  char* buf = (char*)malloc(len * sizeof(char));
  strncpy(buf, str, len);
  putnstr_async(buf, len, putstr_cb, buf);
  wait_for(PUTSTR);
}

void putnstr_async(const char *str, size_t len, subscribe_cb cb, void* userdata) {
  allow(0, 1, (void*)str, len);
  subscribe(0, 1, cb, userdata);
}

void putstr(const char *str) {
  putnstr(str, strlen(str));
}

static CB_TYPE read_tmp006_cb(int r0, int r1, int r2, void* ud) {
  int16_t *res = (int16_t*)ud;
  *res = (int16_t)r0 / 32;

  return READTMP;
}


int tmp006_read(int16_t *temperature) {
  int error = tmp006_read_async(read_tmp006_cb, (void*)temperature);
  if (error < 0) {
    return error;
  }
  wait_for(READTMP);
  return 0;
}

int tmp006_read_async(subscribe_cb cb, void* userdata) {
  return subscribe(2, 0, cb, userdata);
}

int tmp006_enable() {
  return command(2, 0, 0);
}

int timer_oneshot_subscribe(subscribe_cb cb, void *userdata) {
  return subscribe(3, 0, cb, userdata);
}

int timer_repeating_subscribe(subscribe_cb cb, void *userdata) {
  return subscribe(4, 1, cb, userdata);
}

int spi_write(unsigned char byte) {
  return command(4, 0, byte);
}

int spi_read_buf(const char* str, size_t len) {
  allow(4, 0, (void*)str, len);
}

static CB_TYPE spi_cb(int r0, int r1, int r2, void* ud) {
  return SPIBUF;
}

int spi_write_buf(const char* str, 
		  size_t len, 
		  subscribe_cb cb, 
		  void* userdata) {
  allow(4, 1, (void*)str, len);
  subscribe(4, 0, cb, userdata);
  command(4, 1, len);
}

int spi_block_write(const char* str, 
		    size_t len, 
		    void* userdata) {
    spi_write_buf(str, len, spi_cb, userdata);
//    wait_for(SPIBUF);
}
