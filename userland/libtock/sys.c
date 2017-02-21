#include <sys/stat.h>
#include <sys/types.h>

#include "console.h"
#include "tock.h"

// XXX Suppress unused parameter warnings for this file as the implementations
// are currently all just stubs
#pragma GCC diagnostic ignored "-Wunused-parameter"

// XXX Suppress missing prototype warnings for this file as the headers should
// be in newlib internals, but first stab at including things didn't quite work
// and the warnings are just noise
#pragma GCC diagnostic ignored "-Wmissing-prototypes"
#pragma GCC diagnostic ignored "-Wstrict-prototypes"

// XXX Also suppress attribute suggestions as these are stubs
#pragma GCC diagnostic ignored "-Wsuggest-attribute=const"

//------------------------------
// LIBC SUPPORT STUBS
//------------------------------

void* __dso_handle = 0;

int _isatty(int fd)
{
    if (fd == 0)
    {
        return 1;
    }
    return 0;
}
int _open(const char* path, int flags, ...)
{
  return -1;
}
int _write(int fd, const void *buf, uint32_t count)
{
    putnstr((const char*)buf, count);
    return count;
}
int _close(int fd)
{
    return -1;
}
int _fstat(int fd, struct stat *st)
{
    st->st_mode = S_IFCHR;
    return 0;
}
int _lseek(int fd, uint32_t offset, int whence)
{
    return 0;
}
int _read(int fd, void *buf, uint32_t count)
{
    return 0; //k_read(fd, (uint8_t*) buf, count);
}
void _exit(int __status)
{
  while(666) {}
}
int _getpid(void)
{
  return 0;
}
int _kill(pid_t pid, int sig)
{
  return -1;
}

caddr_t _sbrk(int incr)
{
  return memop(1, incr);
}

