/* vim: set sw=2 expandtab tw=80: */

#include <string>

#include <firestorm.h>
#include <tock.h>

using namespace std;

CB_TYPE nop(int, int, int, void*) {
  return 0;
}

int main() {
  gpio_enable(0);
  gpio_set(0);
  string *hello = new string("Hello\r\n");
  putnstr_async(hello->c_str(), hello->length(), nop, NULL);
}

