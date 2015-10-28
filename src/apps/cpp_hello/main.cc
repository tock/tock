/* vim: set sw=2 expandtab tw=80: */

#include <string>

#include <firestorm.h>
#include <tock.h>

using namespace std;

int main() {
  gpio_enable(0);
  gpio_set(0);

  string *hello = new string("Hello from C++\r\n");
  putnstr(hello->c_str(), hello->length());
  delete hello;
}

