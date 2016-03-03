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

// what about directly setting a pointer?
int* my_ptr = 0xFEEDBEEF;

// what about a pointer in a struct
struct test_struct_t {
  uint32_t  data_1;
  int*      ptr_1;
  uint32_t  data_2;
};
struct test_struct_t my_struct = {
  .data_1 = 0x00000001,
  .ptr_1  = "String in a struct worked\n",
  .data_1 = 0x80000001,
};

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

  // directly setting a pointer
  {
    char buf[64];
    snprintf(buf, 64, "Directly set pointer (should be 0xFEEDBEEF) = 0x%X\n", my_ptr);
    putstr(buf);
  }

  // structures
  {
    char buf[64];
    snprintf(buf, 64, "Structure data1 (should be 0x00000001) = 0x%X\n", my_struct.data_1);
    putstr(buf);
  }
  putstr(my_struct.ptr_1);
  {
    char buf[64];
    snprintf(buf, 64, "Structure data2 (should be 0x80000001) = 0x%X\n", my_struct.data_2);
    putstr(buf);
  }


  return 0;
}

