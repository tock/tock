#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <timer.h>
#include <crc.h>
#include <rng.h>

struct test_case {
  enum crc_alg alg;
  uint32_t output;
  char *input;
};

#define CASE(alg, output, input) char input_ ## alg ## _ ## output [] = input;
#include "test_cases.h"
#undef CASE

static struct test_case test_cases[] = {
#define CASE(alg, output, input) \
  { alg, output, input_ ## alg ## _ ## output },
#include "test_cases.h"
#undef CASE
};

int n_test_cases = sizeof(test_cases) / sizeof(struct test_case);

int test_index;

void receive_result(int, int, int, void *);

bool completed;

uint32_t procid;

int main(void) {
  int r;

  // Get a random number to distinguish this app instance
  if ((r = rng_sync((uint8_t *) &procid, 4, 4)) != 4) {
    printf("RNG failure\n");
    exit(1);
  }

  if (!crc_exists()) {
    printf("CRC driver does not exist\n");
    exit(1);
  }

  uint32_t v = crc_version();
  if (v != 0x00000202) {
    printf("CRC version unexpected: %lu\n", v);
    exit(1);
  }

  if (crc_subscribe(receive_result, 0) !=0) {
    printf("CRC subscribe failed\n");
    exit(1);
  }

  while (1) {
    for (test_index = 0; test_index < n_test_cases; test_index++) {
      struct test_case *t = &test_cases[test_index];

      printf("Requesting test case %d (length %d) ...\n",
             test_index, strlen(t->input));

      if ((r = crc_set_buffer(t->input, strlen(t->input))) != 0) {
        printf("CRC set-buffer failed: %d\n", r);
        exit(1);
      }

      completed = false;
      if ((r = crc_request(t->alg)) != 0) {
        printf("CRC request failed: %d\n", r);
        exit(1);
      }

      printf("Waiting for CRC results ...\n");
      yield_for(&completed);
    }

    printf("\n\n");
    delay_ms(1000);
  }
}

void receive_result(int status, int v1,
                    __attribute__((unused)) int v2,
                    __attribute__((unused)) void *data)
{
  uint32_t result = v1;

  struct test_case *t = &test_cases[test_index];

  printf("[%8lx] Case %d: ", procid, test_index);
  if (status == SUCCESS) {
    printf("result=%8lx ", result);
    if (result == t->output)
      printf("(OK)");
    else
      printf("(Expected %8lx)", t->output);
  }
  else {
    printf("failed with status %d\n", status);
  }
  printf("\n");

  completed = true;
}
