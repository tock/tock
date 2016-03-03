/* vim: set sw=2 expandtab tw=80: */

#include <firestorm.h>

// pointer to read-only data
char* dev_name = "Read-only data works\n";

// function pointers
void testFn () {
  putstr("Function pointers work\n");
}
void (*testFnPtr)() = testFn;

// indirection to global variable in .data
char global_string[] = "I should print three times\n";
char* global_string_ptr = global_string;
char** global_string_ptr_ptr = &global_string_ptr;

int main() {
  putstr("Testing for app loading errors\n");

  // read-only data
  putstr(dev_name);

  // function pointers
  (*testFnPtr)();

  // indirection to data section
  putstr(global_string);
  putstr(global_string_ptr);
  putstr(*global_string_ptr_ptr);

  return 0;
}

