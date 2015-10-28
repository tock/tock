#include <stdint.h>
#include <sys/stat.h>
#include <stdio.h>
#include <string.h>
#include <stdlib.h>
#include <inttypes.h>

#include <firestorm.h>

//------------------------------
// LIBC SUPPORT STUBS
//------------------------------

void* __dso_handle = 0;

int _isatty(int fd)
{
    return 1;
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
int _fstat(int fd, struct stat *buf)
{
    return -1;
}
int _lseek(int fd, uint32_t offset, int whence)
{
    return -1;
}
int _read(int fd, void *buf, uint32_t count)
{
    return 0; //k_read(fd, (uint8_t*) buf, count);
}
void _exit(int status)
{
  while(666);
}
void abort()
{
  while(666);
}
int _getpid()
{
  return 0;
}
int _kill(pid_t pid, int sig)
{
  return -1;
}

