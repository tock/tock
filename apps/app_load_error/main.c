/* vim: set sw=2 expandtab tw=80: */

#include <firestorm.h>

char* dev_name = "Read-only data works\n";

void testFn () {
  putstr("Function pointers work\n");
}

void (*testFnPtr)() = testFn;

void main() {
  putstr("Testing for app loading errors\n");

  putstr(dev_name);
  (*testFnPtr)();
}

