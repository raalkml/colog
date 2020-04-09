// portable pty-preparser for wide range of compilers (gcc,HPaCC,SunCC,IBMxlC)
#ifndef __output_h__
#define __output_h__ 1

// n could be -1 (count args, must end with 0 than)
int output(int fd, const char * const * args, int n);
// arguments must end with 0 (\0-terminated strings only)
int outputs(int fd, const char * arg0, ...);
// see fprintf(3)
int outputf(int fd, const char * format, ...);

// arguments must end with [0, 0]
int output(int fd, const char * arg0, int len0, ...);

#include "escape_seq.h"
static inline
int print2(int fd,
           const char * color1,
           const char * prefix, int preflen,
           const char * color2,
           const char * suffix, int sufflen)
{
    if ( sufflen > 0 && suffix[sufflen - 1] == '\n' )
        --sufflen;
    return output(fd,
                  color1,           -1,  prefix, preflen, 
                  color2,           -1,  suffix, sufflen, 
                  esc_ normal _esc, -1, "\n", 1, 0, 0);
}

static inline
int error(int fd,
          const char * prefix,      int preflen,
          const char * description, int desclen)
{
    return print2(fd,
                  esc_ bold ";" red _esc,         // file
                  prefix, preflen,
                  esc_ bold_off ";" yellow_bg ";" black _esc, // error text
                  description, desclen);
}
static inline
int warning(int fd,
            const char * prefix,      int preflen,
            const char * description, int desclen)
{
    return print2(fd,
                  esc_ bold ";" yellow _esc,      // file
                  prefix, preflen,
                  esc_ bold_off _esc, // warning text
                  description, desclen);
}

#endif // __output_h__
