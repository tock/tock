#ifndef _FIRESTORM_H
#define _FIRESTORM_H

#include <unistd.h>
#include "tock.h"
#include "gpio.h"

// Pin definitions
#define LED_0 PC10

#ifdef __cplusplus
extern "C" {
#endif

enum firestorm_cb_type {
  PUTSTR,
  READTMP,
  ASYNC,
  SPIBUF,
  GPIO,
};

void putstr(const char* str);
void putnstr(const char* str, size_t len);
void putnstr_async(const char* str, size_t len, subscribe_cb cb, void* userdata);

int timer_oneshot_subscribe(subscribe_cb cb, void *userdata);
int timer_repeating_subscribe(subscribe_cb cb, void *userdata);

int spi_read_write(const char* write, char* read, size_t  len, subscribe_cb cb);

// Output pins on Firestorm
// From https://github.com/SoftwareDefinedBuildings/storm/blob/master/docs/_posts/2014-10-02-pins.md
//  combined with the eagle files for Firestorm https://github.com/helena-project/firestorm
enum GPIO_Pin_enum{
  PC10=0,
  PA16,
  PA12,
  PC09,
  PA10,
  PA11,
  PA19,
  PA13,
};
#define LED_0   PC10
#define P2      PA16
#define P3      PA12
#define P5      PA10
#define P6      PA11
#define P7      PA19
#define P8      PA13

#ifdef __cplusplus
}
#endif

#endif // _FIRESTORM_H
