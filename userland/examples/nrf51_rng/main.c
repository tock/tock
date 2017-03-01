#include <stdio.h>
#include <timer.h>
#include <rng.h>

#define SIZE 16

uint8_t data[SIZE];

static void callback(
    __attribute__ ((unused)) int x, 
    __attribute__ ((unused)) int not_used2,
    __attribute__ ((unused)) int arg2,
    __attribute__ ((unused)) void *ud){
  printf("RANDOMNESS: \r\n");
  for(int i = 0; i < SIZE; i++) {
    printf("%d ", data[i]);
  }
  printf("\r\n");
}

int main(void)
{
  printf("\rTRNG Demo App\r\n");
  
  for(int i = 0; i < SIZE; i++) { data[i] = i;}

  rng_set_buffer(data, SIZE);
  rng_set_callback(callback, NULL);

  for(int i = 0; i < 6; i++) {
    rng_get_random(SIZE);
    delay_ms(1000);
  }
  return 0;
}
