// portable pty-preparser for wide range of compilers (gcc,HPaCC,SunCC,IBMxlC)
#include <unistd.h>
#include <stdarg.h>
#include <string.h>
#include <stdio.h>
#include <sys/types.h>
#include <sys/uio.h>
#include <stdlib.h>
#if defined(arch_hpux10) || defined(arch_sparcOS5)
#include <alloca.h>
#endif

#include "output.h"

int output_args(int fd, char * const * args, int n)
{
    if ( !n )
        return 0;
    else if ( n < 0 )
    {
        n = 0;
        for ( char * const * p = args; *p; ++p )
            ++n;
    }
    struct iovec * vec = (struct iovec *)alloca(sizeof(struct iovec) * n);
    char * const * p = args;
    for ( int i = 0; i < n; ++p, ++i )
    {
        vec[i].iov_base = (caddr_t)*p;
        vec[i].iov_len  = strlen(*p);
    }
    return writev(fd, vec, n);
}

int outputs(int fd, const char * arg0, ...)
{
    if ( !arg0 )
        return 0;
    
    va_list ap;
    va_start(ap, arg0);
    int n = 1;
    while ( va_arg(ap, caddr_t) )
        ++n;
    va_end(ap);
    va_start(ap, arg0);
    struct iovec * vec = (struct iovec *)alloca(sizeof(struct iovec) * n);
    vec[0].iov_base = (caddr_t)arg0;
    vec[0].iov_len  = strlen(arg0);
    for ( int i = 1; i < n; ++i )
    {
        vec[i].iov_base = va_arg(ap, caddr_t);
        vec[i].iov_len  = strlen((const char*)(vec[i].iov_base));
    }
    va_end(ap);
    return writev(fd, vec, n);
}

int output(int fd, const char * arg0, int len0, ...)
{
    if ( !arg0 )
        return 0;
    
    va_list ap;
    va_start(ap, len0);
    int n = 1, al;
    for ( ;; )
    {
        if ( !va_arg(ap, caddr_t) )
            break;
        al = va_arg(ap, int);
        ++n;
    }
    
    va_end(ap);
    va_start(ap, len0);
    struct iovec * vec = (struct iovec *)alloca(sizeof(struct iovec) * n);
    vec[0].iov_base = (caddr_t)arg0;
    vec[0].iov_len  = len0 < 0 ? strlen(arg0): (size_t)len0;
    for ( int i = 1; i < n; ++i )
    {
        vec[i].iov_base = va_arg(ap, caddr_t);
        al = va_arg(ap, int);
        vec[i].iov_len  = al < 0 ? strlen((const char*)vec[i].iov_base): (size_t)al;
    }
    va_end(ap);
    return writev(fd, vec, n);
}

int outputf(int fd, const char * format, ...)
{
    va_list ap;
    va_start(ap, format);
    
    FILE * fp = 1 == fd ? stdout: (2 == fd ? stderr: fdopen(fd, "w"));
    if ( fp )
    {
        vfprintf(fp, format, ap);
        if ( !(1 == fd || 2 == fd) )
            fclose(fp);
    }
    va_end(ap);
    return 0;
}
