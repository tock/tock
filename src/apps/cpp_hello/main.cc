/* vim: set sw=2 expandtab tw=80: */

#include <string>

#include <firestorm.h>
#include <stdio.h>
#include <tock.h>

using namespace std;

int main() {
  gpio_enable(0);
  gpio_set(0);

  string hello = "Hello from C++\r\n";
  write(0, hello.c_str(), hello.length());
}

