// With a heavy emphasis on security, Tock prefers to avoid several functions
// that commonly introduce bugs in (embedded) code. This header is injected as
// part of the tock build, where we add warning attributes to functions.

#include <stdarg.h>
#include <stdio.h>
#include <string.h>

#pragma GCC diagnostic push
#pragma GCC diagnostic ignored "-Wredundant-decls"


// C++ doesn't have the `restrict` keyword so copy whole decl's in this section
#ifdef __cplusplus
extern "C" {

__attribute__((warning ("prefer snprintf over sprintf")))
int sprintf(char * str, const char * format, ...);

__attribute__((warning ("prefer vsnprintf over vsprintf")))
int vsprintf(char * str, const char * format, va_list ap);

}
#else // !defined __cplusplus

__attribute__((warning ("prefer snprintf over sprintf")))
int sprintf(char * restrict str, const char * restrict format, ...);

__attribute__((warning ("prefer vsnprintf over vsprintf")))
int vsprintf(char * restrict str, const char * restrict format, va_list ap);

#endif // __cplusplus


#pragma GCC diagnostic pop
