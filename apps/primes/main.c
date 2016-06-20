#include <stdbool.h>
#include <stdio.h>

bool naive_is_prime(int prime) {
  for (int i = 2; i <= prime / 2; ++i) {
    if (prime % i == 0) {
      return false;
    }
  }
  return true;
}

int main() {
  for (int prime = 10000000; true; ++prime) {
    if (naive_is_prime(prime)) {
      printf("%d\n", prime);
    }
  }
  return 0;
}

