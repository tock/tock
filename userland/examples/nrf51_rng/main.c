#include <stdio.h>
#include <timer.h>
#include <rng.h>

#define SIZE 16

char data[SIZE];

static void callback(int x, int not_used2,
    __attribute__ ((unused)) int arg2,
    __attribute__ ((unused)) void *ud){
  printf("CALLBACK RNG: \r\n");
  for(int i = 0; i < SIZE; i++) {
    printf("%d ");
  }
  printf("\r\n");
}

int main(void)
{
  printf("TRNG Demo App\r\n");
  
  for(int i = 0; i < SIZE; i++) { data[i] = i;}

  rng_set_buffer(data, SIZE);
  rng_set_callback(callback, NULL);

  for(int i = 0; i < 3; i++) {
    rng_get_random(16);
    delay_ms(1000);
  }
  return 0;
}
